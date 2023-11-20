use crate::class::constant::{
    Constant, ConstantClass, ConstantDouble, ConstantFloat, ConstantInteger, ConstantLong,
    ConstantString,
};
use crate::jvm::call::{FlowControl, StackFrame};
use crate::jvm::mem::{FieldDescriptor, JavaValue, ObjectReference};
use crate::jvm::thread::SynchronousMonitor;
use crate::jvm::JavaEnv;
use parking_lot::RwLock;
use std::cmp::Ordering;
use std::sync::Arc;

pub fn jsr(_frame: &mut StackFrame, _offset: i16) -> Result<(), FlowControl> {
    unimplemented!("Jump to subroutine is unsupported")
}
pub fn jsr_w(_frame: &mut StackFrame, _offset: i32) -> Result<(), FlowControl> {
    unimplemented!("Jump to subroutine is unsupported")
}

pub fn ret(_frame: &mut StackFrame, _index: u16) -> Result<(), FlowControl> {
    unimplemented!("Returning by return address is unsupported")
}

pub fn lookupswitch(
    frame: &mut StackFrame,
    default: i32,
    match_offsets: &[(i32, i32)],
) -> Result<(), FlowControl> {
    if let Some(JavaValue::Int(key)) = FieldDescriptor::Int.assign_from(frame.stack.pop().unwrap())
    {
        // debug!("{} -> {:?}", key, self);
        for (match_val, offset) in match_offsets {
            match key.cmp(match_val) {
                Ordering::Greater => {}
                Ordering::Equal => return Err(FlowControl::Branch(*offset as _)),
                Ordering::Less => break,
            }
        }

        return Err(FlowControl::Branch(default as _));
    }
    panic!("Expected int to use in lookup table")
}

pub fn tableswitch(
    frame: &mut StackFrame,
    default: i32,
    low: i32,
    jump_offsets: &[i32],
) -> Result<(), FlowControl> {
    if let Some(JavaValue::Int(key)) = FieldDescriptor::Int.assign_from(frame.stack.pop().unwrap())
    {
        if key < low || key >= low + jump_offsets.len() as i32 {
            return Err(FlowControl::Branch(default as _));
        }

        return Err(FlowControl::Branch(jump_offsets[(key - low) as usize] as _));
    }
    panic!("Expected int to use in lookup table")
}

// TODO: I just guessed on how this one works so check if this is actually right
pub fn athrow(frame: &mut StackFrame) -> Result<(), FlowControl> {
    match frame.stack.pop() {
        Some(JavaValue::Reference(x)) => Err(FlowControl::Throws(x)),
        _ => panic!("Expected reference!"),
    }
}

pub fn checkcast(
    frame: &mut StackFrame,
    jvm: &mut Arc<RwLock<JavaEnv>>,
    index: u16,
) -> Result<(), FlowControl> {
    let class_name = frame.constants.class_name(index);

    if let JavaValue::Reference(Some(v)) = &frame.stack[frame.stack.len() - 1] {
        if matches!(
            jvm.read().instanceof(&v.get_class(), class_name),
            Some(false) | None
        ) {
            // TODO: Check if this is the correct exception
            return Err(FlowControl::throw("java/lang/ClassCastException"));
        }
    } else if !matches!(&frame.stack[frame.stack.len() - 1], JavaValue::Reference(_)) {
        panic!("Expected Reference for castcheck")
    }
    Ok(())
}

pub fn bipush(frame: &mut StackFrame, value: i8) {
    frame.stack.push(JavaValue::Byte(value));
}

pub fn sipush(frame: &mut StackFrame, value: i16) {
    frame.stack.push(JavaValue::Short(value));
}

pub fn ldc(frame: &mut StackFrame, jvm: &mut Arc<RwLock<JavaEnv>>, index: u8) {
    frame.stack.push(match &frame.constants[index as u16] {
        Constant::Int(ConstantInteger { value }) => JavaValue::Int(*value),
        Constant::Float(ConstantFloat { value }) => JavaValue::Float(*value),
        Constant::String(ConstantString { string_index }) => {
            // let text = frame.constants[*string_index as usize - 1]
            //     .expect_utf8()
            //     .unwrap();
            // jvm.write().build_string(&text)
            jvm.write()
                .build_string(frame.constants.text(*string_index))
        }
        Constant::Class(ConstantClass { name_index }) => {
            // let name = frame.constants[*name_index as usize - 1]
            //     .expect_utf8()
            //     .unwrap();
            // JavaValue::Reference(Some(jvm.write().class_instance(&name)))
            JavaValue::Reference(Some(
                jvm.write()
                    .class_instance(frame.constants.text(*name_index)),
            ))
            // JavaValue::Reference(Some(Rc::new(RefCell::new(Object::Class(name)))))
        }
        x => panic!("Attempted to push {:?} to the stack", x),
    });
}

pub fn ldc_w(frame: &mut StackFrame, jvm: &mut Arc<RwLock<JavaEnv>>, index: u16) {
    frame.stack.push(match &frame.constants[index] {
        Constant::Int(ConstantInteger { value }) => JavaValue::Int(*value),
        Constant::Float(ConstantFloat { value }) => JavaValue::Float(*value),
        Constant::String(ConstantString { string_index }) => {
            // let text = frame.constants[*string_index as usize - 1]
            //     .expect_utf8()
            //     .unwrap();
            // jvm.write().build_string(&text)
            jvm.write()
                .build_string(frame.constants.text(*string_index))
        }
        Constant::Class(ConstantClass { name_index }) => {
            // let name = frame.constants[*name_index as usize - 1]
            //     .expect_utf8()
            //     .unwrap();
            // JavaValue::Reference(Some(jvm.write().class_instance(&name)))
            JavaValue::Reference(Some(
                jvm.write()
                    .class_instance(frame.constants.text(*name_index)),
            ))
            // JavaValue::Reference(Some(Rc::new(RefCell::new(Object::Class(name)))))
        }
        x => panic!("Attempted to push {:?} to the stack", x),
    });
}

pub fn ldc2_w(frame: &mut StackFrame, index: u16) {
    let value = match &frame.constants[index] {
        Constant::Double(ConstantDouble { value }) => JavaValue::Double(*value),
        Constant::Long(ConstantLong { value }) => JavaValue::Long(*value),
        x => panic!("Attempted to push {:?} to the stack", x),
    };

    frame.stack.push(value);
    frame.stack.push(value);
}

pub fn goto(offset: i16) -> Result<(), FlowControl> {
    Err(FlowControl::Branch(offset as i64))
}

pub fn goto_w(offset: i32) -> Result<(), FlowControl> {
    Err(FlowControl::Branch(offset as i64))
}

pub fn ireturn(frame: &mut StackFrame) -> Result<(), FlowControl> {
    Err(FlowControl::Return(frame.stack.pop()))
}

pub fn lreturn(frame: &mut StackFrame) -> Result<(), FlowControl> {
    Err(FlowControl::Return(frame.stack.pop()))
}

pub fn freturn(frame: &mut StackFrame) -> Result<(), FlowControl> {
    Err(FlowControl::Return(frame.stack.pop()))
}

pub fn dreturn(frame: &mut StackFrame) -> Result<(), FlowControl> {
    Err(FlowControl::Return(frame.stack.pop()))
}

pub fn areturn(frame: &mut StackFrame) -> Result<(), FlowControl> {
    Err(FlowControl::Return(frame.stack.pop()))
}

pub fn r#return() -> Result<(), FlowControl> {
    Err(FlowControl::Return(None))
}

pub fn iinc(frame: &mut StackFrame, index: u16, val: i16) {
    if let Some(JavaValue::Int(x)) = FieldDescriptor::Int.assign_from(frame.locals[index as usize])
    {
        frame.locals[index as usize] = JavaValue::Int(x + val as i32);
    } else {
        panic!("iinc only operates on type 1 computational type integers");
    }
}

pub fn monitorenter(
    frame: &mut StackFrame,
    jvm: &mut Arc<RwLock<JavaEnv>>,
) -> Result<(), FlowControl> {
    jvm.lock(frame.pop_reference()?);
    Ok(())
}

pub fn monitorexit(
    frame: &mut StackFrame,
    jvm: &mut Arc<RwLock<JavaEnv>>,
) -> Result<(), FlowControl> {
    jvm.unlock(frame.pop_reference()?);
    Ok(())
}
