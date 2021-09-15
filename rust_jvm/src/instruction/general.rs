//! Instructions I have yet to implement, but can still be parsed

use std::io;
use std::io::Cursor;

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

use crate::constant_pool::{
    Constant, ConstantClass, ConstantDouble, ConstantFloat, ConstantInteger, ConstantLong,
    ConstantString,
};
use crate::instruction::{Instruction, InstructionAction, StaticInstruct};
use crate::jvm::call::{FlowControl, StackFrame};
use crate::jvm::mem::{FieldDescriptor, JavaValue};
use crate::jvm::thread::SynchronousMonitor;
use crate::jvm::JavaEnv;
use parking_lot::RwLock;
use std::cmp::Ordering;
use std::sync::Arc;

// TODO: goto_w
// TODO: invokedynamic
instruction! {jsr, 0xa8, u16}
// TODO: jsr_w
instruction! {lcmp, 0x94}
// TODO: lookupswitch
instruction! {ret, 0xa9, u8}
// TODO: multianewarray
// TODO: tableswitch
// TODO: wide

#[derive(Debug, Clone)]
pub struct lookupswitch {
    default: i32,
    match_offset: Vec<(i32, i32)>,
}

impl StaticInstruct for lookupswitch {
    const FORM: u8 = 0xab;

    fn read(_: u8, buffer: &mut Cursor<Vec<u8>>) -> io::Result<Box<dyn Instruction>> {
        // 0-3 bytes padding to get proper alignment
        while buffer.position() % 4 != 0 {
            buffer.read_u8()?;
        }

        let default = buffer.read_i32::<BigEndian>()?;
        let num_pairs = buffer.read_i32::<BigEndian>()? as usize;
        let mut match_offset = Vec::with_capacity(num_pairs);

        for _ in 0..num_pairs {
            match_offset.push((
                buffer.read_i32::<BigEndian>()?,
                buffer.read_i32::<BigEndian>()?,
            ));
        }

        Ok(Box::new(lookupswitch {
            default,
            match_offset,
        }))
    }
}

impl Instruction for lookupswitch {
    fn write(&self, buffer: &mut Cursor<Vec<u8>>) -> io::Result<()> {
        buffer.write_u8(<Self as StaticInstruct>::FORM)?;

        while buffer.position() % 4 != 0 {
            buffer.write_u8(0)?;
        }

        buffer.write_i32::<BigEndian>(self.default)?;
        buffer.write_i32::<BigEndian>(self.match_offset.len() as i32)?;

        for (match_val, offset) in &self.match_offset {
            buffer.write_i32::<BigEndian>(*match_val)?;
            buffer.write_i32::<BigEndian>(*offset)?;
        }

        Ok(())
    }
    fn exec(
        &self,
        stack: &mut crate::jvm::call::StackFrame,
        jvm: &mut std::sync::Arc<parking_lot::RwLock<crate::jvm::JavaEnv>>,
    ) -> Result<(), crate::jvm::call::FlowControl> {
        <Self as crate::instruction::InstructionAction>::exec(self, stack, jvm)
    }
}

impl InstructionAction for lookupswitch {
    fn exec(
        &self,
        frame: &mut StackFrame,
        _: &mut Arc<RwLock<JavaEnv>>,
    ) -> Result<(), FlowControl> {
        if let Some(JavaValue::Int(key)) =
            FieldDescriptor::Int.assign_from(frame.stack.pop().unwrap())
        {
            // debug!("{} -> {:?}", key, self);
            for (match_val, offset) in &self.match_offset {
                match key.cmp(match_val) {
                    Ordering::Greater => {}
                    Ordering::Equal => return Err(FlowControl::Branch(*offset as _)),
                    Ordering::Less => break,
                }
            }

            return Err(FlowControl::Branch(self.default as _));
        }
        panic!("Expected int to use in lookup table")
    }
}

instruction! {@partial athrow, 0xbf}

// TODO: I just guessed on how this one works so check if this is actually right
impl InstructionAction for athrow {
    fn exec(
        &self,
        frame: &mut StackFrame,
        _jvm: &mut Arc<RwLock<JavaEnv>>,
    ) -> Result<(), FlowControl> {
        match frame.stack.pop() {
            Some(JavaValue::Reference(x)) => Err(FlowControl::Throws(x)),
            _ => panic!("Expected reference!"),
        }
    }
}

instruction! {@partial checkcast, 0xc0, u16}

impl InstructionAction for checkcast {
    fn exec(
        &self,
        frame: &mut StackFrame,
        _jvm: &mut Arc<RwLock<JavaEnv>>,
    ) -> Result<(), FlowControl> {
        // TODO: Implement cast checking
        warn!(
            "Skipped cast check as exceptions are not implemented yet: {:?}",
            frame.stack[frame.stack.len() - 1]
        );
        Ok(())
    }
}

instruction! {@partial bipush, 0x10, u8}

impl InstructionAction for bipush {
    fn exec(
        &self,
        frame: &mut StackFrame,
        _jvm: &mut Arc<RwLock<JavaEnv>>,
    ) -> Result<(), FlowControl> {
        let bipush(value) = *self;
        // Be lazy and transmute the byte from unsigned to signed to avoid implementing another
        // pattern in the instruction macro
        let signed = unsafe { ::std::mem::transmute::<_, i8>(value) };
        // Sign extend byte to int as specified in specification
        frame.stack.push(JavaValue::Byte(signed as _));
        Ok(())
    }
}

instruction! {@partial sipush, 0x11, i16}

impl InstructionAction for sipush {
    fn exec(
        &self,
        frame: &mut StackFrame,
        _jvm: &mut Arc<RwLock<JavaEnv>>,
    ) -> Result<(), FlowControl> {
        let sipush(value) = *self;
        // Sign extend short to int as specified in specification
        frame.stack.push(JavaValue::Int(value as _));
        Ok(())
    }
}

instruction! {@partial ldc, 0x12, u8}

impl InstructionAction for ldc {
    fn exec(
        &self,
        frame: &mut StackFrame,
        jvm: &mut Arc<RwLock<JavaEnv>>,
    ) -> Result<(), FlowControl> {
        let ldc(index) = *self;

        frame
            .stack
            .push(match &frame.constants[index as usize - 1] {
                Constant::Int(ConstantInteger { value }) => JavaValue::Int(*value),
                Constant::Float(ConstantFloat { value }) => JavaValue::Float(*value),
                Constant::String(ConstantString { string_index }) => {
                    let text = frame.constants[*string_index as usize - 1]
                        .expect_utf8()
                        .unwrap();
                    jvm.write().build_string(&text)
                }
                Constant::Class(ConstantClass { name_index }) => {
                    let name = frame.constants[*name_index as usize - 1]
                        .expect_utf8()
                        .unwrap();
                    JavaValue::Reference(Some(jvm.write().class_instance(&name)))
                    // JavaValue::Reference(Some(Rc::new(RefCell::new(Object::Class(name)))))
                }
                x => panic!("Attempted to push {:?} to the stack", x),
            });
        Ok(())
    }
}

instruction! {@partial ldc_w, 0x13, u16}

impl InstructionAction for ldc_w {
    fn exec(
        &self,
        frame: &mut StackFrame,
        jvm: &mut Arc<RwLock<JavaEnv>>,
    ) -> Result<(), FlowControl> {
        let ldc_w(index) = *self;

        frame
            .stack
            .push(match &frame.constants[index as usize - 1] {
                Constant::Int(ConstantInteger { value }) => JavaValue::Int(*value),
                Constant::Float(ConstantFloat { value }) => JavaValue::Float(*value),
                Constant::String(ConstantString { string_index }) => {
                    let text = frame.constants[*string_index as usize - 1]
                        .expect_utf8()
                        .unwrap();
                    jvm.write().build_string(&text)
                }
                Constant::Class(ConstantClass { name_index }) => {
                    let name = frame.constants[*name_index as usize - 1]
                        .expect_utf8()
                        .unwrap();
                    JavaValue::Reference(Some(jvm.write().class_instance(&name)))
                    // JavaValue::Reference(Some(Rc::new(RefCell::new(Object::Class(name)))))
                }
                x => panic!("Attempted to push {:?} to the stack", x),
            });
        Ok(())
    }
}

instruction! {@partial ldc2_w, 0x14, u16}

impl InstructionAction for ldc2_w {
    fn exec(
        &self,
        frame: &mut StackFrame,
        _jvm: &mut Arc<RwLock<JavaEnv>>,
    ) -> Result<(), FlowControl> {
        let ldc2_w(index) = *self;

        let value = match &frame.constants[index as usize - 1] {
            Constant::Double(ConstantDouble { value }) => JavaValue::Double(*value),
            Constant::Long(ConstantLong { value }) => JavaValue::Long(*value),
            x => panic!("Attempted to push {:?} to the stack", x),
        };

        frame.stack.push(value);
        frame.stack.push(value);
        Ok(())
    }
}

instruction! {@partial goto, 0xa7, i16}

impl InstructionAction for goto {
    fn exec(
        &self,
        _frame: &mut StackFrame,
        _jvm: &mut Arc<RwLock<JavaEnv>>,
    ) -> Result<(), FlowControl> {
        let goto(offset) = *self;
        // frame.branch_offset += offset as i64;
        Err(FlowControl::Branch(offset as i64))
    }
}

instruction! {@partial ireturn, 0xac}
instruction! {@partial lreturn, 0xad}
instruction! {@partial freturn, 0xae}
instruction! {@partial dreturn, 0xaf}
instruction! {@partial areturn, 0xb0}

impl InstructionAction for ireturn {
    fn exec(
        &self,
        frame: &mut StackFrame,
        _jvm: &mut Arc<RwLock<JavaEnv>>,
    ) -> Result<(), FlowControl> {
        Err(FlowControl::Return(frame.stack.pop()))
    }
}

impl InstructionAction for lreturn {
    fn exec(
        &self,
        frame: &mut StackFrame,
        _jvm: &mut Arc<RwLock<JavaEnv>>,
    ) -> Result<(), FlowControl> {
        Err(FlowControl::Return(frame.stack.pop()))
    }
}

impl InstructionAction for freturn {
    fn exec(
        &self,
        frame: &mut StackFrame,
        _jvm: &mut Arc<RwLock<JavaEnv>>,
    ) -> Result<(), FlowControl> {
        Err(FlowControl::Return(frame.stack.pop()))
    }
}

impl InstructionAction for dreturn {
    fn exec(
        &self,
        frame: &mut StackFrame,
        _jvm: &mut Arc<RwLock<JavaEnv>>,
    ) -> Result<(), FlowControl> {
        Err(FlowControl::Return(frame.stack.pop()))
    }
}

impl InstructionAction for areturn {
    fn exec(
        &self,
        frame: &mut StackFrame,
        _jvm: &mut Arc<RwLock<JavaEnv>>,
    ) -> Result<(), FlowControl> {
        Err(FlowControl::Return(frame.stack.pop()))
    }
}

instruction! {@partial r#return, 0xb1}

impl InstructionAction for r#return {
    fn exec(
        &self,
        _frame: &mut StackFrame,
        _jvm: &mut Arc<RwLock<JavaEnv>>,
    ) -> Result<(), FlowControl> {
        Err(FlowControl::Return(None))
    }
}

// TODO: iinc, 0x84, u8, u8

#[derive(Copy, Clone, Debug)]
pub struct iinc(u8, i8);

impl Instruction for iinc {
    fn write(&self, buffer: &mut Cursor<Vec<u8>>) -> io::Result<()> {
        buffer.write_u8(Self::FORM)?;
        buffer.write_u8(self.0)?;
        buffer.write_i8(self.1)
    }

    fn exec(
        &self,
        frame: &mut StackFrame,
        jvm: &mut Arc<RwLock<JavaEnv>>,
    ) -> Result<(), FlowControl> {
        <Self as InstructionAction>::exec(self, frame, jvm)
    }
}

impl StaticInstruct for iinc {
    const FORM: u8 = 0x84;

    fn read(_form: u8, buffer: &mut Cursor<Vec<u8>>) -> io::Result<Box<dyn Instruction>> {
        Ok(Box::new(iinc(buffer.read_u8()?, buffer.read_i8()?)))
    }
}

impl InstructionAction for iinc {
    fn exec(
        &self,
        frame: &mut StackFrame,
        _jvm: &mut Arc<RwLock<JavaEnv>>,
    ) -> Result<(), FlowControl> {
        let iinc(index, val) = *self;

        match &mut frame.locals[index as usize] {
            JavaValue::Byte(x) => *x += val,
            JavaValue::Char(x) => {
                if val > 0 {
                    *x += val as u16;
                } else {
                    *x -= val.abs() as u16;
                }
            }
            JavaValue::Short(x) => *x += val as i16,
            JavaValue::Int(x) => *x += val as i32,
            JavaValue::Float(x) => *x += val as f32,
            JavaValue::Long(x) => *x += val as i64,
            JavaValue::Double(x) => *x += val as f64,
            x => panic!("can not call iinc on {:?}", x),
        }
        Ok(())
    }
}

instruction! {@partial monitorenter, 0xc2}
instruction! {@partial monitorexit, 0xc3}

impl InstructionAction for monitorenter {
    fn exec(
        &self,
        frame: &mut StackFrame,
        jvm: &mut Arc<RwLock<JavaEnv>>,
    ) -> Result<(), FlowControl> {
        jvm.lock(frame.pop_reference()?);
        Ok(())
    }
}

impl InstructionAction for monitorexit {
    fn exec(
        &self,
        frame: &mut StackFrame,
        jvm: &mut Arc<RwLock<JavaEnv>>,
    ) -> Result<(), FlowControl> {
        jvm.unlock(frame.pop_reference()?);
        Ok(())
    }
}
