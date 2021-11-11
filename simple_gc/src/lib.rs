mod jdk_fork;

use parking_lot::{Condvar, Mutex};
use slice_dst::SliceWithHeader;
use std::any::Any;
use std::cell::Cell;
use std::mem::ManuallyDrop;
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;
use std::sync::atomic::AtomicU32;
use std::thread::{current, ThreadId};
use std::time::Duration;

#[derive(Copy, Clone, Debug)]
pub enum GcDesc {
    /// Objects that are on the stack
    Stack,
    /// Objects owned by the system that do not require references from other java Objects. This
    /// includes objects created by JNI interfaces and it takes priority over stack items.
    System,
    Marked(u8),
}

#[repr(transparent)]
pub struct Object<T> {
    ptr: NonNull<SliceWithHeader<GcHeader, T>>,
}

impl<T> ObjectUnknown<T> {
    fn get_mark(&self) -> GcDesc {
        unsafe { self.ptr.as_ref().header.mark_desc }
    }

    fn set_mark(&mut self, mark: GcDesc) {
        unsafe {
            *(&mut *self.ptr.as_ptr()).header.mark_desc = mark;
        }
    }

    pub fn trace(&self) {}

    pub fn obj_type(&self) -> &ObjectType {
        unsafe { &self.ptr.as_ref().header.obj_type }
    }
}

pub struct ObjectTable {
    mark_state: u8,
    table: Vec<NonNull<SliceWithHeader<GcHeader, ()>>>,
}

impl ObjectTable {
    pub fn mark(&mut self) {
        for obj in &mut self.table {}
    }

    pub fn sweep(&mut self) {}
}

pub enum ObjectType {
    Instance {
        // Schema
    },
    BoolArray,
    ByteArray,
    CharArray,
    ShortArray,
    IntArray,
    LongArray,
    FloatArray,
    DoubleArray,
    ObjectArray(String),
}

struct GcHeader {
    lock: Condvar,
    owner: Mutex<BiasedLockState>,
    obj_type: ObjectType,
    mark_desc: GcDesc,
}

#[derive(Eq, PartialEq, Debug)]
enum BiasedLockState {
    Unclaimed,
    Claimed {
        bias: ThreadId,
        explicit: u32,
        implicit: u32,
    },
}

pub struct BiasedMutex<T: ?Sized> {
    lock: Condvar,
    owner: Mutex<BiasedLockState>,
    data: Cell<T>,
}

unsafe impl<T> Sync for BiasedMutex<T> {}

pub trait StickyLock {
    type Contents;
    type Guard: Deref<Target = Self::Contents> + DerefMut;

    /// Blocks on the current thread until the it can be explicitly biased towards the current
    /// thread.
    fn claim(&self);

    /// Same as clain but may fail
    fn try_claim(&self) -> bool;

    /// Releases an explicit lock held by the current thread
    fn release(&self);

    /// Implicitly claims the biased mutex and returns a guard which can be used to access contained
    /// data.
    fn lock(&self) -> Self::Guard;

    /// Get a raw pointer to the contained data. This is only for implementing volatile access to
    /// enclosed members.
    fn as_ptr(&self) -> *mut Self::Contents;
}

impl<T: ?Sized> StickyLock for NonNull<SliceWithHeader<GcHeader, T>> {
    type Contents = SliceWithHeader<GcHeader, T>;
    type Guard = ();

    fn claim(&self) {
        unsafe {
            let inner = self.as_ref();
            let mut guard = inner.header.owner.lock();
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
                    _ => inner
                        .header
                        .lock
                        .wait_for(&mut guard, Duration::from_millis(50)),
                };
            }
        }
    }

    fn try_claim(&self) -> bool {
        todo!()
    }

    fn release(&self) {
        todo!()
    }

    fn lock(&self) -> Self::Guard {
        todo!()
    }

    fn as_ptr(&self) -> *mut Self::Contents {
        self.as_ptr()
    }
}

impl<T> BiasedMutex<T> {
    pub fn new(value: T) -> Self {
        BiasedMutex {
            lock: Condvar::new(),
            owner: Mutex::new(BiasedLockState::Unclaimed),
            data: Cell::new(value),
        }
    }

    /// Blocks on the current thread until the it can be explicitly biased towards the current
    /// thread.
    pub fn claim(&self) {
        let mut guard = self.owner.lock();
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
                _ => self.lock.wait_for(&mut guard, Duration::from_millis(50)),
            };
        }
    }

    /// Releases an explicit lock held by the current thread
    pub fn release(&self) {
        let mut guard = self.owner.lock();
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
                }
            }
        };
    }

    /// Implicitly claims the biased mutex and returns a guard which can be used to access contained
    /// data.
    pub fn lock(&self) -> BiasedMutexGuard<T> {
        let mut guard = self.owner.lock();
        let id = current().id();

        loop {
            match &mut *guard {
                BiasedLockState::Unclaimed => {
                    *guard = BiasedLockState::Claimed {
                        bias: id,
                        explicit: 0,
                        implicit: 1,
                    };
                    return BiasedMutexGuard { parent: self };
                }
                BiasedLockState::Claimed { bias, implicit, .. } if *bias == id => {
                    *implicit += 1;
                    return BiasedMutexGuard { parent: self };
                }
                _ => self.lock.wait_for(&mut guard, Duration::from_millis(50)),
            };
        }
    }

    /// Get a raw pointer to the contained data. This is only for implementing volatile access to
    /// enclosed members.
    pub fn as_ptr(&self) -> *mut T {
        self.data.as_ptr()
    }
}

pub struct BiasedMutexGuard<'a, T> {
    parent: &'a BiasedMutex<T>,
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
                    *guard = BiasedLockState::Unclaimed;
                }
            }
        };
    }
}

impl<'a, T> Deref for BiasedMutexGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.parent.data.as_ptr() }
    }
}

impl<'a, T> DerefMut for BiasedMutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.parent.data.as_ptr() }
    }
}

pub trait Trace: Any {
    fn trace(&self);
}

// pub struct GcHeader {
//     mark: bool,
// }
//
// pub struct GcBox<T> {
//     header: Cell<GcHeader>,
//     inner: T,
// }
