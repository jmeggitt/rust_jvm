#![allow(dead_code, unused_variables)] // TODO: Remove when committed

use crate::class::constant::ConstantPool;
use jni::sys::{jboolean, jbyte, jchar, jint, jshort};
use std::ops::Deref;

mod checked;
mod unchecked;

pub enum StackError {
    /// Triggered when the maximum size of the stack is exceeded
    Overflow,
    /// Triggered when attempting to pop the stack frame despite no frame being present
    Underflow,
    /// A value would fit within the stack, but exceeds the requested size of the frame
    FrameOverflow,
    /// Attempted to pop a value, but the frame is empty
    FrameUnderflow,
    /// The type requested does not match what was stored on the stack
    TypeViolation,
    /// The value of a local is out of bounds
    LocalIndexOutOfBounds,
    /// A program attempted to load a compute type 2 value which was partially overwritten
    ClobberedType2Load,
}

pub type ReturnAddress = usize;

pub trait OperandStack:
    OperandStackValue<Self::Category1>
    + OperandStackValue<Self::Category2>
    + Deref<Target = ConstantPool>
{
    type Category1;
    type Category2;

    fn pop_stack_frame(&mut self) -> Result<(), StackError>;

    fn push_stack_frame(
        &mut self,
        class_constants: ConstantPool,
        locals: usize,
        size: Option<usize>,
    ) -> Result<(), StackError>;

    // Convenience wrapper functions
    #[inline]
    fn push<T>(&mut self, x: T) -> Result<(), StackError>
    where
        Self: OperandStackValue<T>,
    {
        self.push_value(x)
    }

    #[inline]
    fn pop<T>(&mut self) -> Result<T, StackError>
    where
        Self: OperandStackValue<T>,
    {
        self.pop_value()
    }

    #[inline]
    fn store<T>(&mut self, index: u16, x: T) -> Result<(), StackError>
    where
        Self: OperandStackValue<T>,
    {
        self.store_local(index, x)
    }

    #[inline]
    fn load<T>(&self, index: u16) -> Result<T, StackError>
    where
        Self: OperandStackValue<T>,
    {
        self.load_local(index)
    }
}

pub trait OperandStackValue<T> {
    fn push_value(&mut self, x: T) -> Result<(), StackError>;
    fn pop_value(&mut self) -> Result<T, StackError>;

    fn store_local(&mut self, index: u16, x: T) -> Result<(), StackError>;
    fn load_local(&self, index: u16) -> Result<T, StackError>;
}

impl<S: OperandStackValue<jint>> OperandStackValue<jshort> for S {
    fn push_value(&mut self, x: jshort) -> Result<(), StackError> {
        self.push_value(x as jint)
    }

    fn pop_value(&mut self) -> Result<jshort, StackError> {
        <Self as OperandStackValue<jint>>::pop_value(self).map(|x| x as jshort)
    }

    fn store_local(&mut self, index: u16, x: jshort) -> Result<(), StackError> {
        <Self as OperandStackValue<jint>>::store_local(self, index, x as jint)
    }

    fn load_local(&self, index: u16) -> Result<jshort, StackError> {
        <Self as OperandStackValue<jint>>::load_local(self, index).map(|x| x as jshort)
    }
}

impl<S: OperandStackValue<jint>> OperandStackValue<jchar> for S {
    fn push_value(&mut self, x: jchar) -> Result<(), StackError> {
        self.push_value(x as jshort as jint)
    }

    fn pop_value(&mut self) -> Result<jchar, StackError> {
        <Self as OperandStackValue<jint>>::pop_value(self).map(|x| x as jshort as jchar)
    }

    fn store_local(&mut self, index: u16, x: jchar) -> Result<(), StackError> {
        <Self as OperandStackValue<jint>>::store_local(self, index, x as jshort as jint)
    }

    fn load_local(&self, index: u16) -> Result<jchar, StackError> {
        <Self as OperandStackValue<jint>>::load_local(self, index).map(|x| x as jshort as jchar)
    }
}

impl<S: OperandStackValue<jint>> OperandStackValue<jbyte> for S {
    fn push_value(&mut self, x: jbyte) -> Result<(), StackError> {
        self.push_value(x as jint)
    }

    fn pop_value(&mut self) -> Result<jbyte, StackError> {
        <Self as OperandStackValue<jint>>::pop_value(self).map(|x| x as jbyte)
    }

    fn store_local(&mut self, index: u16, x: jbyte) -> Result<(), StackError> {
        <Self as OperandStackValue<jint>>::store_local(self, index, x as jint)
    }

    fn load_local(&self, index: u16) -> Result<jbyte, StackError> {
        <Self as OperandStackValue<jint>>::load_local(self, index).map(|x| x as jbyte)
    }
}

impl<S: OperandStackValue<jint>> OperandStackValue<jboolean> for S {
    fn push_value(&mut self, x: jboolean) -> Result<(), StackError> {
        self.push_value((x & 1) as jint)
    }

    fn pop_value(&mut self) -> Result<jboolean, StackError> {
        <Self as OperandStackValue<jint>>::pop_value(self).map(|x| (x & 1) as jboolean)
    }

    fn store_local(&mut self, index: u16, x: jboolean) -> Result<(), StackError> {
        <Self as OperandStackValue<jint>>::store_local(self, index, (x & 1) as jint)
    }

    fn load_local(&self, index: u16) -> Result<jboolean, StackError> {
        <Self as OperandStackValue<jint>>::load_local(self, index).map(|x| (x & 1) as jboolean)
    }
}
