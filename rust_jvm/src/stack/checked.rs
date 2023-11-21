use crate::jvm::mem::ObjectHandle;
use crate::stack::{OperandStack, OperandStackValue, StackError};
use jni::sys::{jdouble, jfloat, jint, jlong};
use std::ops::Deref;

#[derive(Debug, Copy, Clone)]
enum StackValue {
    Uninit,
    Int(jint),
    Float(jfloat),
    Long(jlong),
    Double(jdouble),
    Reference(Option<ObjectHandle>),
    ReturnAddress(super::ReturnAddress),
}

use crate::class::constant::ConstantPool;
use StackValue::*;

#[derive(Debug)]
pub struct CheckedCategory1(StackValue);

#[derive(Debug)]
pub struct CheckedCategory2(StackValue);

#[derive(Debug)]
pub struct CheckedCategory1Or2(StackValue, Option<StackValue>);

#[derive(Default, Debug)]
pub struct CheckedOperandStack {
    frames: Vec<CheckedFrame>,
    limit: usize,
    usage: usize,
    base_constant_pool: ConstantPool,
}

impl CheckedOperandStack {
    fn last_frame(&self) -> Result<&CheckedFrame, StackError> {
        match self.frames.last() {
            Some(v) => Ok(v),
            None => Err(StackError::FrameUnderflow),
        }
    }

    fn last_frame_mut(&mut self) -> Result<&mut CheckedFrame, StackError> {
        match self.frames.last_mut() {
            Some(v) => Ok(v),
            None => Err(StackError::FrameUnderflow),
        }
    }

    fn consume_space(&mut self, space: usize) -> Result<(), StackError> {
        if self.usage + space > self.limit {
            return Err(StackError::Overflow);
        }

        let last_frame = self.last_frame_mut()?;
        if last_frame.usage + space > last_frame.limit {
            return Err(StackError::FrameOverflow);
        }

        last_frame.usage += space;
        self.usage += space;
        Ok(())
    }

    fn free_space(&mut self, space: usize) -> Result<(), StackError> {
        if self.usage < space {
            return Err(StackError::Underflow);
        }

        let last_frame = self.last_frame_mut()?;
        if last_frame.usage < space {
            return Err(StackError::FrameUnderflow);
        }

        last_frame.usage -= space;
        self.usage -= space;
        Ok(())
    }

    fn local_entry(&self, index: u16) -> Result<&StackValue, StackError> {
        match self.last_frame()?.locals.get(index as usize) {
            None => Err(StackError::LocalIndexOutOfBounds),
            Some(x) => Ok(x),
        }
    }

    fn local_entry_mut(&mut self, index: u16) -> Result<&mut StackValue, StackError> {
        match self.last_frame_mut()?.locals.get_mut(index as usize) {
            None => Err(StackError::LocalIndexOutOfBounds),
            Some(x) => Ok(x),
        }
    }
}

impl Deref for CheckedOperandStack {
    type Target = ConstantPool;

    fn deref(&self) -> &Self::Target {
        self.last_frame()
            .map(|frame| &frame.class_constants)
            .unwrap_or(&self.base_constant_pool)
    }
}

impl OperandStack for CheckedOperandStack {
    type Category1 = CheckedCategory1;
    type Category2 = CheckedCategory2;

    fn pop_stack_frame(&mut self) -> Result<(), StackError> {
        match self.frames.pop() {
            Some(frame) => {
                self.usage -= frame.usage + 8;
                Ok(())
            }
            None => Err(StackError::FrameUnderflow),
        }
    }

    fn push_stack_frame(
        &mut self,
        class_constants: ConstantPool,
        locals: usize,
        limit: Option<usize>,
    ) -> Result<(), StackError> {
        // Treat each frame as if it requires 8 bytes to simulate the pushing of 2 references. Since
        // references are category 1 types, we treat them as taking 4 bytes each. These 2 imaginary
        // references represent the stack base pointer and the return instruction pointer.
        if self.limit - self.usage < 8 {
            return Err(StackError::Overflow);
        }
        self.usage += 8;

        self.frames.push(CheckedFrame {
            class_constants,
            locals: vec![Uninit; locals],
            items: Vec::new(),
            usage: 0,
            limit: limit.unwrap_or(self.limit - self.usage),
        });

        Ok(())
    }
}

#[derive(Debug)]
struct CheckedFrame {
    class_constants: ConstantPool,
    locals: Vec<StackValue>,
    items: Vec<StackValue>,
    usage: usize,
    limit: usize,
}

impl OperandStackValue<CheckedCategory1> for CheckedOperandStack {
    fn push_value(&mut self, CheckedCategory1(x): CheckedCategory1) -> Result<(), StackError> {
        self.consume_space(4)?;
        self.last_frame_mut()?.items.push(x);
        Ok(())
    }

    fn pop_value(&mut self) -> Result<CheckedCategory1, StackError> {
        self.free_space(4)?;
        let popped_value = match self.last_frame_mut()?.items.pop() {
            Some(x) => x,
            None => unreachable!("we were able to free space"),
        };

        if !matches!(
            popped_value,
            Int(_) | Float(_) | Reference(_) | ReturnAddress(_)
        ) {
            return Err(StackError::TypeViolation);
        }

        Ok(CheckedCategory1(popped_value))
    }

    // It is a type violation to treat generalized category 1 or 2 values as locals
    fn store_local(&mut self, index: u16, x: CheckedCategory1) -> Result<(), StackError> {
        Err(StackError::TypeViolation)
    }

    fn load_local(&self, index: u16) -> Result<CheckedCategory1, StackError> {
        Err(StackError::TypeViolation)
    }
}

impl OperandStackValue<CheckedCategory2> for CheckedOperandStack {
    fn push_value(&mut self, CheckedCategory2(x): CheckedCategory2) -> Result<(), StackError> {
        self.consume_space(8)?;
        self.last_frame_mut()?.items.push(x);
        Ok(())
    }

    fn pop_value(&mut self) -> Result<CheckedCategory2, StackError> {
        self.free_space(8)?;
        let popped_value = match self.last_frame_mut()?.items.pop() {
            Some(x) => x,
            None => unreachable!("we were able to free space"),
        };

        if !matches!(popped_value, Long(_) | Double(_)) {
            return Err(StackError::TypeViolation);
        }

        Ok(CheckedCategory2(popped_value))
    }

    // It is a type violation to treat generalized category 1 or 2 values as locals
    fn store_local(&mut self, index: u16, x: CheckedCategory2) -> Result<(), StackError> {
        Err(StackError::TypeViolation)
    }

    fn load_local(&self, index: u16) -> Result<CheckedCategory2, StackError> {
        Err(StackError::TypeViolation)
    }
}

impl OperandStackValue<jint> for CheckedOperandStack {
    fn push_value(&mut self, x: jint) -> Result<(), StackError> {
        self.push_value(CheckedCategory1(Int(x)))
    }

    fn pop_value(&mut self) -> Result<jint, StackError> {
        match self.pop_value()? {
            CheckedCategory1(Int(x)) => Ok(x),
            CheckedCategory1(_) => Err(StackError::TypeViolation),
        }
    }

    fn store_local(&mut self, index: u16, x: jint) -> Result<(), StackError> {
        *self.local_entry_mut(index)? = Int(x);
        Ok(())
    }

    fn load_local(&self, index: u16) -> Result<jint, StackError> {
        match self.local_entry(index)? {
            Uninit => Ok(0),
            Int(x) => Ok(*x),
            _ => Err(StackError::TypeViolation),
        }
    }
}

impl OperandStackValue<jlong> for CheckedOperandStack {
    fn push_value(&mut self, x: jlong) -> Result<(), StackError> {
        self.push_value(CheckedCategory2(Long(x)))
    }

    fn pop_value(&mut self) -> Result<jlong, StackError> {
        match self.pop_value()? {
            CheckedCategory2(Long(x)) => Ok(x),
            CheckedCategory2(_) => Err(StackError::TypeViolation),
        }
    }

    fn store_local(&mut self, index: u16, x: jlong) -> Result<(), StackError> {
        *self.local_entry_mut(index)? = Long(x);
        *self.local_entry_mut(index + 1)? = Long(x);
        Ok(())
    }

    fn load_local(&self, index: u16) -> Result<jlong, StackError> {
        match (self.local_entry(index)?, self.local_entry(index + 1)?) {
            (Uninit, Uninit) => Ok(0),
            (Long(x), Long(y)) if x == y => Ok(*x),
            (Long(_), _) | (_, Long(_)) => Err(StackError::ClobberedType2Load),
            _ => Err(StackError::TypeViolation),
        }
    }
}

impl OperandStackValue<jfloat> for CheckedOperandStack {
    fn push_value(&mut self, x: jfloat) -> Result<(), StackError> {
        self.push_value(CheckedCategory1(Float(x)))
    }

    fn pop_value(&mut self) -> Result<jfloat, StackError> {
        match self.pop_value()? {
            CheckedCategory1(Float(x)) => Ok(x),
            CheckedCategory1(_) => Err(StackError::TypeViolation),
        }
    }

    fn store_local(&mut self, index: u16, x: jfloat) -> Result<(), StackError> {
        *self.local_entry_mut(index)? = Float(x);
        Ok(())
    }

    fn load_local(&self, index: u16) -> Result<jfloat, StackError> {
        match self.local_entry(index)? {
            Uninit => Ok(0.0),
            Float(x) => Ok(*x),
            _ => Err(StackError::TypeViolation),
        }
    }
}

impl OperandStackValue<jdouble> for CheckedOperandStack {
    fn push_value(&mut self, x: jdouble) -> Result<(), StackError> {
        self.push_value(CheckedCategory2(Double(x)))
    }

    fn pop_value(&mut self) -> Result<jdouble, StackError> {
        match self.pop_value()? {
            CheckedCategory2(Double(x)) => Ok(x),
            CheckedCategory2(_) => Err(StackError::TypeViolation),
        }
    }

    fn store_local(&mut self, index: u16, x: jdouble) -> Result<(), StackError> {
        *self.local_entry_mut(index)? = Double(x);
        *self.local_entry_mut(index + 1)? = Double(x);
        Ok(())
    }

    fn load_local(&self, index: u16) -> Result<jdouble, StackError> {
        match (self.local_entry(index)?, self.local_entry(index + 1)?) {
            (Uninit, Uninit) => Ok(0.0),
            (Double(x), Double(y)) if x == y => Ok(*x),
            (Double(_), _) | (_, Double(_)) => Err(StackError::ClobberedType2Load),
            _ => Err(StackError::TypeViolation),
        }
    }
}

impl OperandStackValue<Option<ObjectHandle>> for CheckedOperandStack {
    fn push_value(&mut self, x: Option<ObjectHandle>) -> Result<(), StackError> {
        self.push_value(CheckedCategory1(Reference(x)))
    }

    fn pop_value(&mut self) -> Result<Option<ObjectHandle>, StackError> {
        match self.pop_value()? {
            CheckedCategory1(Reference(x)) => Ok(x),
            CheckedCategory1(_) => Err(StackError::TypeViolation),
        }
    }

    fn store_local(&mut self, index: u16, x: Option<ObjectHandle>) -> Result<(), StackError> {
        *self.local_entry_mut(index)? = Reference(x);
        Ok(())
    }

    fn load_local(&self, index: u16) -> Result<Option<ObjectHandle>, StackError> {
        match self.local_entry(index)? {
            Uninit => Ok(None),
            Reference(x) => Ok(*x),
            _ => Err(StackError::TypeViolation),
        }
    }
}

impl OperandStackValue<super::ReturnAddress> for CheckedOperandStack {
    fn push_value(&mut self, x: super::ReturnAddress) -> Result<(), StackError> {
        self.push_value(CheckedCategory1(ReturnAddress(x)))
    }

    fn pop_value(&mut self) -> Result<super::ReturnAddress, StackError> {
        match self.pop_value()? {
            CheckedCategory1(ReturnAddress(x)) => Ok(x),
            CheckedCategory1(_) => Err(StackError::TypeViolation),
        }
    }

    fn store_local(&mut self, index: u16, x: super::ReturnAddress) -> Result<(), StackError> {
        *self.local_entry_mut(index)? = ReturnAddress(x);
        Ok(())
    }

    fn load_local(&self, index: u16) -> Result<super::ReturnAddress, StackError> {
        match self.local_entry(index)? {
            ReturnAddress(x) => Ok(*x),
            _ => Err(StackError::TypeViolation),
        }
    }
}
