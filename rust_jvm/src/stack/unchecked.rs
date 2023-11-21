use crate::class::constant::{Constant, ConstantPool};
use crate::jvm::mem::ObjectHandle;
use crate::stack::{OperandStack, OperandStackValue, ReturnAddress, StackError};
use jni::sys::{jdouble, jfloat, jint, jlong};
use std::alloc::Layout;
use std::mem::{align_of, size_of, ManuallyDrop};
use std::ops::Deref;

#[derive(Copy, Clone)]
pub union Category1 {
    // Java values
    int: jint,
    float: jfloat,
    reference: Option<ObjectHandle>,
    return_address: ReturnAddress,
    // Helper values for implementing stack frames
    stack_ptr: *mut Category1,
    class_constants: *const Constant,
}

#[derive(Copy, Clone)]
pub union Category2 {
    long: jlong,
    double: jdouble,
    // In many cases category 2 is treated as being twice as large as category 1. Enforce that this
    // remains the case.
    category1: ManuallyDrop<[Category1; 2]>,
}

#[repr(C)]
pub struct UncheckedOperandStack {
    /// Top of current frame (rsp equivalent)
    stack_ptr: *mut Category1,
    /// Base of current frame (rbp equivalent)
    base_ptr: *mut Category1,
    /// Base of stack allocation
    stack_base: *mut Category1,
    /// Top of stack allocation
    stack_top: *mut Category1,
}

impl UncheckedOperandStack {
    pub fn new(size: usize) -> Self {
        // Check that we won't violate any sizing or alignment constraints
        assert_eq!(2 * size_of::<Category1>(), size_of::<Category2>());
        assert!(align_of::<Category2>() <= size_of::<Category1>());

        // Check that there is no unexpected padding or bloat
        assert_eq!(size_of::<Category1>(), size_of::<*mut ()>().max(4));
        assert_eq!(size_of::<Category2>(), (2 * size_of::<*mut ()>()).max(8));

        // Make sure we have enough space to p the base constant pool
        let slot_count = (size / size_of::<Category2>()).max(1);

        let layout = Layout::array::<Category2>(slot_count).expect("size is not too large");

        unsafe {
            let stack_base = std::alloc::alloc(layout) as *mut Category1;
            let stack_top = stack_base.add(layout.size() / size_of::<Category1>());

            // Add a base constant pool in case the pool is accessed before pushing the first frame
            stack_base.write(Category1 {
                class_constants: ConstantPool::default().into_raw(),
            });
            let base_ptr = stack_base.add(1);

            UncheckedOperandStack {
                stack_ptr: base_ptr,
                base_ptr,
                stack_base,
                stack_top,
            }
        }
    }
}

impl Drop for UncheckedOperandStack {
    fn drop(&mut self) {
        let size = self.stack_top as usize - self.stack_base as usize;
        let layout =
            Layout::array::<Category2>(size / size_of::<Category2>()).expect("valid layout");

        unsafe { std::alloc::dealloc(self.stack_base as *mut u8, layout) }
    }
}

impl Deref for UncheckedOperandStack {
    type Target = ConstantPool;

    fn deref(&self) -> &Self::Target {
        unsafe {
            let ptr = self.base_ptr.sub(1);
            // &*raw_field!(ptr, Category1, class_constants)
            todo!()
        }
    }
}

impl OperandStack for UncheckedOperandStack {
    type Category1 = Category1;
    type Category2 = Category2;

    fn pop_stack_frame(&mut self) -> Result<(), StackError> {
        unsafe {
            if self.base_ptr.sub(1) <= self.stack_base {
                return Err(StackError::Underflow);
            }

            let class_constants = self.stack_base.sub(1).read().class_constants;
            drop(ConstantPool::from_raw(class_constants));

            let next_stack_ptr = self.stack_base.sub(2);
            self.stack_base = next_stack_ptr.read().stack_ptr;
            self.stack_ptr = next_stack_ptr;
            Ok(())
        }
    }

    fn push_stack_frame(
        &mut self,
        class_constants: ConstantPool,
        locals: usize,
        _size: Option<usize>,
    ) -> Result<(), StackError> {
        unsafe {
            // Add enough space for the new base pointer, class constants, and the locals
            let next_stack_ptr = self.stack_ptr.add(2 + locals);

            // We still need to do a check for the stack base pointer case
            if next_stack_ptr <= self.stack_top {
                self.stack_ptr.write(Category1 {
                    stack_ptr: self.base_ptr,
                });
                self.stack_ptr.add(1).write(Category1 {
                    class_constants: class_constants.into_raw(),
                });
                self.base_ptr = self.stack_ptr.add(2);
                self.stack_ptr = next_stack_ptr;
                Ok(())
            } else {
                Err(StackError::Overflow)
            }
        }
    }
}

impl OperandStackValue<Category1> for UncheckedOperandStack {
    #[inline]
    fn push_value(&mut self, x: Category1) -> Result<(), StackError> {
        unsafe {
            let next_stack_ptr = self.stack_ptr.add(1);
            // We still need to do a check for the stack base pointer case
            if next_stack_ptr < self.stack_top {
                self.stack_ptr.write(x);
                self.stack_ptr = next_stack_ptr;
                return Ok(());
            }
        }

        Err(StackError::Overflow)
    }

    #[inline]
    fn pop_value(&mut self) -> Result<Category1, StackError> {
        debug_assert!(self.stack_ptr > self.base_ptr);
        unsafe {
            self.stack_ptr = self.stack_ptr.sub(1);
            Ok(self.stack_ptr.read())
        }
    }

    #[inline]
    fn store_local(&mut self, index: u16, x: Category1) -> Result<(), StackError> {
        unsafe {
            self.base_ptr.add(index as usize).write(x);
        }
        Ok(())
    }

    #[inline]
    fn load_local(&self, index: u16) -> Result<Category1, StackError> {
        unsafe { Ok(self.base_ptr.add(index as usize).read()) }
    }
}

impl OperandStackValue<Category2> for UncheckedOperandStack {
    #[inline]
    fn push_value(&mut self, x: Category2) -> Result<(), StackError> {
        unsafe {
            let next_stack_ptr = self.stack_ptr.offset(2);
            if next_stack_ptr < self.stack_top {
                self.stack_ptr.cast::<Category2>().write(x);
                self.stack_ptr = next_stack_ptr;
                return Ok(());
            }

            Err(StackError::Overflow)
        }
    }

    #[inline]
    fn pop_value(&mut self) -> Result<Category2, StackError> {
        unsafe {
            self.stack_ptr = self.stack_ptr.offset(-2);
            Ok(self.stack_ptr.cast::<Category2>().read())
        }
    }

    #[inline]
    fn store_local(&mut self, index: u16, x: Category2) -> Result<(), StackError> {
        unsafe {
            self.base_ptr
                .add(index as usize)
                .cast::<Category2>()
                .write(x);
        }
        Ok(())
    }

    #[inline]
    fn load_local(&self, index: u16) -> Result<Category2, StackError> {
        unsafe { Ok(self.base_ptr.add(index as usize).cast::<Category2>().read()) }
    }
}

impl OperandStackValue<jint> for UncheckedOperandStack {
    #[inline]
    fn push_value(&mut self, x: jint) -> Result<(), StackError> {
        <Self as OperandStackValue<Category1>>::push_value(self, Category1 { int: x })
    }

    #[inline]
    fn pop_value(&mut self) -> Result<jint, StackError> {
        unsafe { Ok(<Self as OperandStackValue<Category1>>::pop_value(self)?.int) }
    }

    #[inline]
    fn store_local(&mut self, index: u16, x: jint) -> Result<(), StackError> {
        <Self as OperandStackValue<Category1>>::store_local(self, index, Category1 { int: x })
    }

    #[inline]
    fn load_local(&self, index: u16) -> Result<jint, StackError> {
        unsafe { Ok(<Self as OperandStackValue<Category1>>::load_local(self, index)?.int) }
    }
}

impl OperandStackValue<jfloat> for UncheckedOperandStack {
    #[inline]
    fn push_value(&mut self, x: jfloat) -> Result<(), StackError> {
        <Self as OperandStackValue<Category1>>::push_value(self, Category1 { float: x })
    }

    #[inline]
    fn pop_value(&mut self) -> Result<jfloat, StackError> {
        unsafe { Ok(<Self as OperandStackValue<Category1>>::pop_value(self)?.float) }
    }

    #[inline]
    fn store_local(&mut self, index: u16, x: jfloat) -> Result<(), StackError> {
        <Self as OperandStackValue<Category1>>::store_local(self, index, Category1 { float: x })
    }

    #[inline]
    fn load_local(&self, index: u16) -> Result<jfloat, StackError> {
        unsafe { Ok(<Self as OperandStackValue<Category1>>::load_local(self, index)?.float) }
    }
}

impl OperandStackValue<Option<ObjectHandle>> for UncheckedOperandStack {
    #[inline]
    fn push_value(&mut self, x: Option<ObjectHandle>) -> Result<(), StackError> {
        <Self as OperandStackValue<Category1>>::push_value(self, Category1 { reference: x })
    }

    #[inline]
    fn pop_value(&mut self) -> Result<Option<ObjectHandle>, StackError> {
        unsafe { Ok(<Self as OperandStackValue<Category1>>::pop_value(self)?.reference) }
    }

    #[inline]
    fn store_local(&mut self, index: u16, x: Option<ObjectHandle>) -> Result<(), StackError> {
        <Self as OperandStackValue<Category1>>::store_local(self, index, Category1 { reference: x })
    }

    #[inline]
    fn load_local(&self, index: u16) -> Result<Option<ObjectHandle>, StackError> {
        unsafe { Ok(<Self as OperandStackValue<Category1>>::load_local(self, index)?.reference) }
    }
}

impl OperandStackValue<ReturnAddress> for UncheckedOperandStack {
    #[inline]
    fn push_value(&mut self, x: ReturnAddress) -> Result<(), StackError> {
        <Self as OperandStackValue<Category1>>::push_value(self, Category1 { return_address: x })
    }

    #[inline]
    fn pop_value(&mut self) -> Result<ReturnAddress, StackError> {
        unsafe { Ok(<Self as OperandStackValue<Category1>>::pop_value(self)?.return_address) }
    }

    #[inline]
    fn store_local(&mut self, index: u16, x: ReturnAddress) -> Result<(), StackError> {
        <Self as OperandStackValue<Category1>>::store_local(
            self,
            index,
            Category1 { return_address: x },
        )
    }

    #[inline]
    fn load_local(&self, index: u16) -> Result<ReturnAddress, StackError> {
        unsafe {
            Ok(<Self as OperandStackValue<Category1>>::load_local(self, index)?.return_address)
        }
    }
}

impl OperandStackValue<jlong> for UncheckedOperandStack {
    #[inline]
    fn push_value(&mut self, x: jlong) -> Result<(), StackError> {
        <Self as OperandStackValue<Category2>>::push_value(self, Category2 { long: x })
    }

    #[inline]
    fn pop_value(&mut self) -> Result<jlong, StackError> {
        unsafe { Ok(<Self as OperandStackValue<Category2>>::pop_value(self)?.long) }
    }

    #[inline]
    fn store_local(&mut self, index: u16, x: jlong) -> Result<(), StackError> {
        <Self as OperandStackValue<Category2>>::store_local(self, index, Category2 { long: x })
    }

    #[inline]
    fn load_local(&self, index: u16) -> Result<jlong, StackError> {
        unsafe { Ok(<Self as OperandStackValue<Category2>>::load_local(self, index)?.long) }
    }
}

impl OperandStackValue<jdouble> for UncheckedOperandStack {
    #[inline]
    fn push_value(&mut self, x: jdouble) -> Result<(), StackError> {
        <Self as OperandStackValue<Category2>>::push_value(self, Category2 { double: x })
    }

    #[inline]
    fn pop_value(&mut self) -> Result<jdouble, StackError> {
        unsafe { Ok(<Self as OperandStackValue<Category2>>::pop_value(self)?.double) }
    }

    #[inline]
    fn store_local(&mut self, index: u16, x: jdouble) -> Result<(), StackError> {
        <Self as OperandStackValue<Category2>>::store_local(self, index, Category2 { double: x })
    }

    #[inline]
    fn load_local(&self, index: u16) -> Result<jdouble, StackError> {
        unsafe { Ok(<Self as OperandStackValue<Category2>>::load_local(self, index)?.double) }
    }
}
