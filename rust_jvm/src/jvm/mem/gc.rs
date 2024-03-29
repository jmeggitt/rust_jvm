use crate::jvm::mem::RawObject;
use jni::sys::jobject;
use parking_lot::{Condvar, Mutex};
use std::alloc::{dealloc, Layout};
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::ptr::{drop_in_place, NonNull};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::thread::{current, ThreadId};
use std::time::Duration;

pub unsafe trait Trace {
    unsafe fn trace(&self);
}

pub type Gc<T> = Box<T>;

#[derive(Default)]
pub struct ReferenceTable {
    stored: HashMap<NonNull<GcBoxInner<()>>, u32>,
}

thread_local! {
    static LOCAL_REFS: ReferenceTable = ReferenceTable::default();
}

pub static MARK_VER: AtomicBool = AtomicBool::new(false);

bitflags! {
    pub struct MarkDesc: u32 {
        const MARK = 0x4000_0000;
        const NEW_GEN = 0x8000_0000;
        const SYSTEM = 0x2000_0000;
        const GLOBAL_REF = 0x1000_0000;
        const LOCAL_REF = !(Self::MARK.bits | Self::NEW_GEN.bits | Self::SYSTEM.bits | Self::GLOBAL_REF.bits);

        const ROOTS = Self::SYSTEM.bits | Self::GLOBAL_REF.bits | Self::LOCAL_REF.bits;
        // const PERSISTANT = !(Self::MARK | Self::NEW_GEN);
    }
}

#[repr(transparent)]
pub struct GcMark {
    mark: AtomicU32,
}

impl GcMark {
    pub fn new() -> Self {
        GcMark {
            mark: AtomicU32::new(MarkDesc::NEW_GEN.bits + 1),
        }
    }

    unsafe fn increment_local_refs(&mut self) {
        let prior = self.mark.fetch_add(1, Ordering::SeqCst);

        if prior & !MarkDesc::LOCAL_REF.bits != (prior + 1) & !MarkDesc::LOCAL_REF.bits {
            panic!("Local reference count on object exceeded limit! Reduce the number of threads using this object (limit: {})",
                   MarkDesc::LOCAL_REF.bits);
        }
    }

    unsafe fn decrement_local_refs(&mut self) {
        let prior = self.mark.fetch_sub(1, Ordering::SeqCst);

        if prior & !MarkDesc::LOCAL_REF.bits != (prior - 1) & !MarkDesc::LOCAL_REF.bits {
            panic!("Local reference count decremented when no local references existed");
        }
    }

    fn set_global_ref(&mut self) {
        self.mark
            .fetch_and(!MarkDesc::GLOBAL_REF.bits, Ordering::SeqCst);
    }

    fn unset_global_ref(&mut self) {
        self.mark
            .fetch_or(MarkDesc::GLOBAL_REF.bits, Ordering::SeqCst);
    }

    fn set_system_owned(&mut self) {
        self.mark
            .fetch_and(!MarkDesc::SYSTEM.bits, Ordering::SeqCst);
    }

    fn unset_system_owned(&mut self) {
        self.mark.fetch_and(MarkDesc::SYSTEM.bits, Ordering::SeqCst);
    }

    fn is_rooted(&self) -> bool {
        self.mark.load(Ordering::Acquire) & MarkDesc::ROOTS.bits != 0
    }
}

#[derive(Debug)]
#[repr(transparent)]
pub struct GcBox<T: 'static> {
    raw: NonNull<GcBoxInner<T>>,
}

unsafe impl<T> Send for GcBox<T> {}
unsafe impl<T> Sync for GcBox<T> {}

impl<T> PartialEq for GcBox<T> {
    fn eq(&self, other: &Self) -> bool {
        self.raw == other.raw
    }
}

impl<T: 'static> Clone for GcBox<T> {
    fn clone(&self) -> Self {
        *self
    }
}

// Explicitly declare copy instead of deriving due to cases where T does not implement Copy
impl<T: 'static> Copy for GcBox<T> {}

impl<T> GcBox<T> {
    pub fn new(val: T) -> Self {
        let raw = GcBoxInner::new(val);
        // warn!("Allocated box {:p}", raw.as_ptr());

        GcBox { raw }
    }

    // pub fn into_raw(x: Self) -> *mut T {
    //     memoffset::raw_field!(x.raw.as_ptr(), GcBoxInner<T>, data) as *const T as *mut T
    // }
    //
    // pub unsafe fn from_raw(ptr: *mut T) -> Self {
    //     let base =
    //         (ptr as usize - memoffset::offset_of!(GcBoxInner<T>, data)) as *mut GcBoxInner<T>;
    //     GcBox {
    //         raw: NonNull::new_unchecked(base),
    //     }
    // }

    pub fn add_local_ref(&self) {}

    pub fn as_ptr(&self) -> jobject {
        self.raw.as_ptr() as jobject
    }
}

/// Allow direct conversion from a pointer to ObjectUnknown types
impl GcBox<RawObject<()>> {
    pub fn from_ptr(ptr: jobject) -> Option<Self> {
        // warn!("Converting ptr {:p}", ptr);
        Some(Self {
            raw: NonNull::new(ptr as _)?,
        })
    }
}

// impl<T> From<GcBox<RawObject<T>>> for GcBox<RawObject<()>> {
//     fn from(x: GcBox<RawObject<T>>) -> Self {
//         unsafe { transmute(x) }
//     }
// }

unsafe impl<T: Trace> Trace for GcBox<T> {
    unsafe fn trace(&self) {
        todo!()
    }
}

/// Enforce member ordering with repr(C) so mark and locks can be manipulated freely on half-types
#[repr(C)]
pub struct GcBoxInner<T: 'static> {
    lock: Condvar,
    owner: Mutex<BiasedLockState>,
    mark: GcMark,
    data: T,
}

impl<T> GcBoxInner<T> {
    fn new(data: T) -> NonNull<Self> {
        Self::alloc(GcBoxInner {
            lock: Default::default(),
            owner: Mutex::new(BiasedLockState::Unclaimed),
            mark: GcMark::new(),
            data,
        })
    }

    fn alloc(self) -> NonNull<Self> {
        NonNull::new(Box::into_raw(Box::new(self))).unwrap()
    }

    unsafe fn manual_drop(ptr: *mut Self) {
        drop_in_place(ptr);
        dealloc(ptr as *mut u8, Layout::new::<Self>());
    }
}

impl<T: Trace> GcBoxInner<T> {
    fn mark(&mut self) {}
}

impl<T> GcBox<T> {
    pub fn claim_lock(&self) {
        unsafe {
            let inner = self.raw.as_ref();
            let mut guard = inner.owner.lock();
            let id = current().id();

            loop {
                match &mut *guard {
                    BiasedLockState::Unclaimed => {
                        *guard = BiasedLockState::Claimed {
                            bias: id,
                            explicit: 1,
                            implicit: 0,
                        };
                        return;
                    }
                    BiasedLockState::Claimed { bias, explicit, .. } if *bias == id => {
                        *explicit += 1;
                        return;
                    }
                    _ => inner.lock.wait_for(&mut guard, Duration::from_millis(50)),
                };
            }
        }
    }

    pub fn release_lock(&self) {
        unsafe {
            let inner = self.raw.as_ref();
            let mut guard = inner.owner.lock();
            let id = current().id();

            match &mut *guard {
                BiasedLockState::Unclaimed => panic!("Attempted to release unclaimed biased mutex"),
                BiasedLockState::Claimed {
                    bias,
                    implicit,
                    explicit,
                } => {
                    if *bias != id {
                        panic!("Attempted to release biased mutex claimed by another thread!");
                    }

                    if *explicit == 0 {
                        panic!("Attempted to release implicitly claimed biased mutex");
                    }

                    *explicit -= 1;
                    if *implicit == 0 && *explicit == 0 {
                        *guard = BiasedLockState::Unclaimed;
                        inner.lock.notify_one();
                    }
                }
            };
        }
    }

    pub fn lock(&self) -> BiasedMutexGuard<T> {
        unsafe {
            let inner = self.raw.as_ref();

            // warn!("Claiming lock: {:p}", self.raw.as_ptr());
            let mut guard = inner.owner.lock();
            // let mut guard = match inner.owner.try_lock() {
            //     Some(v) => v,
            //     None => panic!("Failed to get lock!"),
            // };
            // warn!("Claimed lock {:p}", self.raw);

            let id = current().id();

            let mut timeout = 0;

            loop {
                match &mut *guard {
                    BiasedLockState::Unclaimed => {
                        *guard = BiasedLockState::Claimed {
                            bias: id,
                            explicit: 0,
                            implicit: 1,
                        };
                        // warn!("Obtained lock {:p}", self.raw);

                        // drop(guard);
                        return BiasedMutexGuard {
                            parent: &mut *self.raw.as_ptr(),
                        };
                    }
                    BiasedLockState::Claimed { bias, implicit, .. } if *bias == id => {
                        *implicit += 1;
                        // warn!("Obtained lock {:p}", self.raw);
                        // drop(guard);
                        return BiasedMutexGuard {
                            parent: &mut *self.raw.as_ptr(),
                        };
                    }
                    _ => {
                        timeout += 1;
                        if timeout == 10 {
                            panic!(
                                "Timed out while waiting for lock. Possible double lock in use!"
                            );
                        }

                        // warn!("Lock rejected: {:?} (for {:?})", &*guard, id);
                        // inner.lock.wait_for(&mut guard, Duration::from_millis(50))
                        inner.lock.wait(&mut guard);
                    }
                };
            }
        }
    }
}

#[derive(Eq, PartialEq, Debug)]
pub enum BiasedLockState {
    Unclaimed,
    Claimed {
        bias: ThreadId,
        explicit: u32,
        implicit: u32,
    },
}

pub struct BiasedMutexGuard<'a, T: 'static> {
    parent: &'a mut GcBoxInner<T>,
}

impl<'a, T> Drop for BiasedMutexGuard<'a, T> {
    fn drop(&mut self) {
        let mut guard = self.parent.owner.lock();
        let id = current().id();

        match &mut *guard {
            BiasedLockState::Unclaimed => {
                unreachable!("Attempted to release unclaimed biased mutex")
            }
            BiasedLockState::Claimed {
                bias,
                implicit,
                explicit,
            } => {
                if *bias != id {
                    unreachable!(
                        "Attempted implicit release of biased mutex claimed by another thread!"
                    );
                }

                if *implicit == 0 {
                    unreachable!(
                        "Attempted to implicitly release biased mutex with no implicit references"
                    );
                }

                *implicit -= 1;
                if *implicit == 0 && *explicit == 0 {
                    // warn!("Released lock {:p}", self.parent);
                    *guard = BiasedLockState::Unclaimed;
                    self.parent.lock.notify_one();
                }
            }
        };

        // drop(guard);
    }
}

impl<'a, T> Deref for BiasedMutexGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.parent.data
    }
}

impl<'a, T> DerefMut for BiasedMutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.parent.data
    }
}
