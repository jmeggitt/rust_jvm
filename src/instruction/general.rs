//! Instructions I have yet to implement, but can still be parsed

use std::io;
use std::io::Cursor;

use byteorder::{ReadBytesExt, WriteBytesExt};

use crate::constant_pool::{
    Constant, ConstantClass, ConstantDouble, ConstantFloat, ConstantInteger, ConstantLong,
    ConstantString,
};
use crate::instruction::{Instruction, InstructionAction, StaticInstruct};
use crate::jvm::{LocalVariable, StackFrame, JVM};

instruction! {athrow, 0xbf}
instruction! {dcmpg, 0x98}
instruction! {dcmpl, 0x97}
instruction! {fcmpg, 0x96}
instruction! {fcmpl, 0x95}
// TODO: goto_w
// TODO: invokedynamic
instruction! {jsr, 0xa8, u16}
// TODO: jsr_w
instruction! {lcmp, 0x94}
// TODO: lookupswitch
instruction! {monitorenter, 0xc2}
instruction! {monitorexit, 0xc3}
instruction! {ret, 0xa9, u8}
// TODO: multianewarray
// TODO: tableswitch
// TODO: wide

instruction! {@partial checkcast, 0xc0, u16}

impl InstructionAction for checkcast {
    fn exec(&self, _frame: &mut StackFrame, _jvm: &mut JVM) {
        warn!("Skipped cast check as exceptions are not implemented yet!");
    }
}

instruction! {@partial bipush, 0x10, u8}

impl InstructionAction for bipush {
    fn exec(&self, frame: &mut StackFrame, _jvm: &mut JVM) {
        let bipush(value) = *self;
        // Be lazy and transmute the byte from unsigned to signed to avoid implementing another
        // pattern in the instruction macro
        let signed = unsafe { ::std::mem::transmute::<_, i8>(value) };
        // Sign extend byte to int as specified in specification
        frame.stack.push(LocalVariable::Int(signed as _));
    }
}

instruction! {@partial sipush, 0x11, i16}

impl InstructionAction for sipush {
    fn exec(&self, frame: &mut StackFrame, _jvm: &mut JVM) {
        let sipush(value) = *self;
        // Sign extend short to int as specified in specification
        frame.stack.push(LocalVariable::Int(value as _));
    }
}

instruction! {@partial ldc, 0x12, u8}

impl InstructionAction for ldc {
    fn exec(&self, frame: &mut StackFrame, jvm: &mut JVM) {
        let ldc(index) = *self;

        frame
            .stack
            .push(match &frame.constants[index as usize - 1] {
                Constant::Int(ConstantInteger { value }) => LocalVariable::Int(*value),
                Constant::Float(ConstantFloat { value }) => LocalVariable::Float(*value),
                Constant::String(ConstantString { string_index }) => {
                    let text = frame.constants[*string_index as usize - 1]
                        .expect_utf8()
                        .unwrap();
                    jvm.build_string(&text)
                }
                Constant::Class(ConstantClass { name_index }) => {
                    let name = frame.constants[*name_index as usize - 1]
                        .expect_utf8()
                        .unwrap();
                    LocalVariable::Reference(Some(jvm.get_class_instance(&name)))
                    // LocalVariable::Reference(Some(Rc::new(RefCell::new(Object::Class(name)))))
                }
                x => panic!("Attempted to push {:?} to the stack", x),
            });
    }
}

instruction! {@partial ldc_w, 0x13, u16}

impl InstructionAction for ldc_w {
    fn exec(&self, frame: &mut StackFrame, jvm: &mut JVM) {
        let ldc_w(index) = *self;

        frame
            .stack
            .push(match &frame.constants[index as usize - 1] {
                Constant::Int(ConstantInteger { value }) => LocalVariable::Int(*value),
                Constant::Float(ConstantFloat { value }) => LocalVariable::Float(*value),
                Constant::String(ConstantString { string_index }) => {
                    let text = frame.constants[*string_index as usize - 1]
                        .expect_utf8()
                        .unwrap();
                    jvm.build_string(&text)
                }
                Constant::Class(ConstantClass { name_index }) => {
                    let name = frame.constants[*name_index as usize - 1]
                        .expect_utf8()
                        .unwrap();
                    LocalVariable::Reference(Some(jvm.get_class_instance(&name)))
                    // LocalVariable::Reference(Some(Rc::new(RefCell::new(Object::Class(name)))))
                }
                x => panic!("Attempted to push {:?} to the stack", x),
            });
    }
}

instruction! {@partial ldc2_w, 0x14, u16}

impl InstructionAction for ldc2_w {
    fn exec(&self, frame: &mut StackFrame, _jvm: &mut JVM) {
        let ldc2_w(index) = *self;

        frame
            .stack
            .push(match &frame.constants[index as usize - 1] {
                Constant::Double(ConstantDouble { value }) => LocalVariable::Double(*value),
                Constant::Long(ConstantLong { value }) => LocalVariable::Long(*value),
                x => panic!("Attempted to push {:?} to the stack", x),
            });
    }
}

instruction! {@partial goto, 0xa7, i16}

impl InstructionAction for goto {
    fn exec(&self, frame: &mut StackFrame, _jvm: &mut JVM) {
        let goto(offset) = *self;
        frame.branch_offset += offset as i64;
    }
}

instruction! {@partial ireturn, 0xac}
instruction! {@partial lreturn, 0xad}
instruction! {@partial freturn, 0xae}
instruction! {@partial dreturn, 0xaf}
instruction! {@partial areturn, 0xb0}

impl InstructionAction for ireturn {
    fn exec(&self, frame: &mut StackFrame, _jvm: &mut JVM) {
        frame.returns = Some(frame.stack.pop());
    }
}

impl InstructionAction for lreturn {
    fn exec(&self, frame: &mut StackFrame, _jvm: &mut JVM) {
        frame.returns = Some(frame.stack.pop());
    }
}

impl InstructionAction for freturn {
    fn exec(&self, frame: &mut StackFrame, _jvm: &mut JVM) {
        frame.returns = Some(frame.stack.pop());
    }
}

impl InstructionAction for dreturn {
    fn exec(&self, frame: &mut StackFrame, _jvm: &mut JVM) {
        frame.returns = Some(frame.stack.pop());
    }
}

impl InstructionAction for areturn {
    fn exec(&self, frame: &mut StackFrame, _jvm: &mut JVM) {
        frame.returns = Some(frame.stack.pop());
    }
}

instruction! {@partial r#return, 0xb1}

impl InstructionAction for r#return {
    fn exec(&self, frame: &mut StackFrame, _jvm: &mut JVM) {
        frame.returns = Some(None);
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

    fn exec(&self, frame: &mut StackFrame, jvm: &mut JVM) {
        <Self as InstructionAction>::exec(self, frame, jvm);
    }
}

impl StaticInstruct for iinc {
    const FORM: u8 = 0x84;

    fn read(_form: u8, buffer: &mut Cursor<Vec<u8>>) -> io::Result<Box<dyn Instruction>> {
        Ok(Box::new(iinc(buffer.read_u8()?, buffer.read_i8()?)))
    }
}

impl InstructionAction for iinc {
    fn exec(&self, frame: &mut StackFrame, _jvm: &mut JVM) {
        let iinc(index, val) = *self;

        match &mut frame.locals[index as usize] {
            LocalVariable::Byte(x) => *x += val,
            LocalVariable::Char(x) => {
                if val > 0 {
                    *x += val as u16;
                } else {
                    *x -= val.abs() as u16;
                }
            }
            LocalVariable::Short(x) => *x += val as i16,
            LocalVariable::Int(x) => *x += val as i32,
            LocalVariable::Float(x) => *x += val as f32,
            LocalVariable::Long(x) => *x += val as i64,
            LocalVariable::Double(x) => *x += val as f64,
            x => panic!("can not call iinc on {:?}", x),
        }
    }
}
