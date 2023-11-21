use std::alloc::{alloc, dealloc, Layout};
use std::marker::PhantomData;
use std::mem::align_of;
use std::ops::Deref;
use std::process::abort;
use std::ptr::{addr_of, addr_of_mut, drop_in_place, slice_from_raw_parts_mut, NonNull};
use std::slice::from_raw_parts;
use std::sync::atomic;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::{Acquire, Relaxed, Release};
use std::{mem, ptr};
use std::fmt::{Debug, Formatter};

/// Roughly equivalent to an `Arc<[T]>`, but with some slight changes. Mainly, it uses a thin
/// pointer, by prefixing the data with the length.
pub struct ThinArcSlice<T> {
    ptr: NonNull<ThinArcSliceInner<()>>,
    phantom: PhantomData<ThinArcSliceInner<[T]>>,
}

impl<T> ThinArcSlice<T> {
    #[inline]
    pub fn len(&self) -> usize {
        unsafe { ptr::read(addr_of!((*self.ptr.as_ptr()).length)) }
    }

    #[inline]
    pub fn as_ptr(&self) -> *const T {
        unsafe { addr_of!((*self.ptr.as_ptr()).data) as *const T }
    }

    #[inline]
    pub fn into_raw(self) -> *const T {
        let result = self.as_ptr();
        mem::forget(self);
        result
    }

    #[inline]
    pub unsafe fn from_raw(ptr: *const T) -> Self {
        unsafe {
            let offset = data_offset::<T>();
            let inner = ptr.cast::<u8>().sub(offset) as *mut ThinArcSliceInner<()>;

            ThinArcSlice {
                ptr: NonNull::new_unchecked(inner),
                phantom: PhantomData,
            }
        }
    }
}

impl<T> Deref for ThinArcSlice<T> {
    type Target = [T];

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe { from_raw_parts(self.as_ptr(), self.len()) }
    }
}

impl<T> From<Vec<T>> for ThinArcSlice<T> {
    fn from(mut value: Vec<T>) -> Self {
        unsafe {
            let result = ThinArcSlice::copy_from_slice(&value);
            value.set_len(0);
            result
        }
    }
}

impl<T: Debug> Debug for ThinArcSlice<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        <[T] as Debug>::fmt(self, f)
    }
}

unsafe impl<T: Sync + Send> Send for ThinArcSlice<T> {}
unsafe impl<T: Sync + Send> Sync for ThinArcSlice<T> {}

impl<T> ThinArcSlice<T> {
    unsafe fn copy_from_slice(v: &[T]) -> Self {
        unsafe {
            let ptr = Self::allocate_for_slice(v.len());
            ptr::copy_nonoverlapping(v.as_ptr(), &mut (*ptr).data as *mut [T] as *mut T, v.len());
            Self::from_ptr(ptr)
        }
    }

    unsafe fn from_inner(fat_ptr: NonNull<ThinArcSliceInner<[T]>>) -> Self {
        let ptr = fat_ptr.cast::<ThinArcSliceInner<()>>();
        Self {
            ptr,
            phantom: PhantomData,
        }
    }

    unsafe fn from_ptr(ptr: *mut ThinArcSliceInner<[T]>) -> Self {
        unsafe { Self::from_inner(NonNull::new_unchecked(ptr)) }
    }

    #[inline]
    fn inner_thin(&self) -> &ThinArcSliceInner<()> {
        unsafe { self.ptr.as_ref() }
    }

    #[cold]
    #[inline(never)]
    unsafe fn drop_slow(&mut self) {
        unsafe {
            // Drop elements of slice
            let length = self.len();
            let data_ptr = addr_of_mut!((*self.ptr.as_ptr()).data) as *mut T;
            let data = slice_from_raw_parts_mut(data_ptr, length);
            drop_in_place(data);

            // Remove the parent allocation
            let slice_layout = Layout::array::<T>(length).unwrap();
            let layout = arc_slice_inner_layout_for_value_layout(slice_layout);

            dealloc(self.ptr.as_ptr().cast(), layout);
        }
    }
}

impl<T> Clone for ThinArcSlice<T> {
    fn clone(&self) -> Self {
        // Read [Arc::clone] for reasoning on atomic ordering
        let ref_count = self.inner_thin().strong_count.fetch_add(1, Relaxed);

        if ref_count > isize::MAX as usize {
            eprintln!("ArcSlice counter overflow");
            abort();
        }

        ThinArcSlice {
            ptr: self.ptr,
            phantom: self.phantom,
        }
    }
}

impl<T> Drop for ThinArcSlice<T> {
    fn drop(&mut self) {
        if self.inner_thin().strong_count.fetch_sub(1, Release) != 1 {
            return;
        }

        atomic::fence(Acquire);

        unsafe {
            self.drop_slow();
        }
    }
}

/// Copy of standard library's [alloc::sync::arcinner_layout_for_value_layout] for [ThinArcSliceInner].
fn arc_slice_inner_layout_for_value_layout(layout: Layout) -> Layout {
    Layout::new::<ThinArcSliceInner<()>>()
        .extend(layout)
        .unwrap()
        .0
        .pad_to_align()
}

fn data_offset<T>() -> usize {
    let base_layout = Layout::new::<ThinArcSliceInner<()>>();
    let with_padding = base_layout
        .align_to(align_of::<T>())
        .unwrap()
        .pad_to_align();

    with_padding.size() - base_layout.size()
}

#[repr(C)]
struct ThinArcSliceInner<T: ?Sized> {
    strong_count: AtomicUsize,
    length: usize,
    data: T,
}

unsafe impl<T: ?Sized + Sync + Send> Send for ThinArcSliceInner<T> {}
unsafe impl<T: ?Sized + Sync + Send> Sync for ThinArcSliceInner<T> {}

impl<T> ThinArcSlice<T> {
    unsafe fn allocate_for_slice(length: usize) -> *mut ThinArcSliceInner<[T]> {
        let slice_layout = Layout::array::<T>(length).unwrap();
        let layout = arc_slice_inner_layout_for_value_layout(slice_layout);

        let ptr = unsafe { alloc(layout) };

        // This looks strange, but this is what the standard library attaches the pointer metadata.
        // Additionally, the pointer metadata is not used by callers. See [Arc::allocate_for_slice]
        let fat_ptr =
            slice_from_raw_parts_mut(ptr as *mut T, length) as *mut ThinArcSliceInner<[T]>;
        unsafe { Self::initialize_inner(fat_ptr, length) }
    }

    unsafe fn initialize_inner(
        inner: *mut ThinArcSliceInner<[T]>,
        length: usize,
    ) -> *mut ThinArcSliceInner<[T]> {
        unsafe {
            ptr::write(&mut (*inner).strong_count, AtomicUsize::new(1));
            ptr::write(&mut (*inner).length, length);
        }

        inner
    }
}
