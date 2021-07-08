use std::cell::UnsafeCell;
use std::io;
use std::io::Cursor;
use std::rc::Rc;

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

use crate::class::BufferedRead;
use crate::constant_pool::Constant;
use crate::instruction::{Instruction, InstructionAction, StaticInstruct};
use crate::jvm::{clean_str, LocalVariable, Object, StackFrame, JVM};
use crate::types::FieldDescriptor;

instruction! {@partial getstatic, 0xb2, u16}

impl InstructionAction for getstatic {
    fn exec(&self, frame: &mut StackFrame, jvm: &mut JVM) {
        let getstatic(field) = *self;

        if let Constant::FieldRef(reference) = &frame.constants[field as usize - 1] {
            let class = frame.constants[reference.class_index as usize - 1]
                .expect_class()
                .unwrap();
            let class_name = frame.constants[class as usize - 1].expect_utf8().unwrap();
            jvm.init_class(&class_name);

            let field = frame.constants[reference.name_and_type_index as usize - 1]
                .expect_name_and_type()
                .unwrap();
            let field_name = frame.constants[field.name_index as usize - 1]
                .expect_utf8()
                .unwrap();
            let descriptor = frame.constants[field.descriptor_index as usize - 1]
                .expect_utf8()
                .unwrap();

            let field_reference = format!("{}_{}", clean_str(&class_name), clean_str(&field_name));
            let value = jvm
                .static_fields
                .get(&field_reference)
                .expect("Static value not found")
                .clone();
            debug!(
                "Got value {:?} from {}::{} {}",
                &value, &class_name, &field_name, descriptor
            );
            frame.stack.push(value);
        } else {
            panic!("Error in getstatic");
        }
    }
}

instruction! {@partial invokestatic, 0xb8, u16}

impl InstructionAction for invokestatic {
    fn exec(&self, frame: &mut StackFrame, jvm: &mut JVM) {
        let invokestatic(field) = *self;

        let (class_index, desc_index) = match &frame.constants[field as usize - 1] {
            Constant::MethodRef(v) => (v.class_index, v.name_and_type_index),
            Constant::InterfaceMethodRef(v) => (v.class_index, v.name_and_type_index),
            _ => panic!(),
        };

        let class = frame.constants[class_index as usize - 1]
            .expect_class()
            .unwrap();
        let class_name = frame.constants[class as usize - 1].expect_utf8().unwrap();
        // jvm.init_class(&class_name);
        jvm.class_loader.attempt_load(&class_name).unwrap();

        let field = frame.constants[desc_index as usize - 1]
            .expect_name_and_type()
            .unwrap();
        let field_name = frame.constants[field.name_index as usize - 1]
            .expect_utf8()
            .unwrap();
        let descriptor = frame.constants[field.descriptor_index as usize - 1]
            .expect_utf8()
            .unwrap();

        if let Ok(FieldDescriptor::Method { args, .. }) = FieldDescriptor::read_str(&descriptor) {
            let mut stack_args = frame.stack[frame.stack.len() - args.len()..].to_vec();

            for _ in 0..args.len() {
                frame.stack.pop();
            }

            match jvm.exec_static(&class_name, &field_name, &descriptor, stack_args) {
                Ok(Some(v)) => frame.stack.push(v),
                Err(e) => frame.throws = Some(e),
                _ => {}
            }
        } else {
            panic!(
                "Unable to execute {}::{} {}",
                &class_name, &field_name, &descriptor
            );
        }
    }
}

instruction! {@partial putstatic, 0xb3, u16}

impl InstructionAction for putstatic {
    fn exec(&self, frame: &mut StackFrame, jvm: &mut JVM) {
        let putstatic(field) = *self;

        let (class_index, desc_index) = match &frame.constants[field as usize - 1] {
            Constant::FieldRef(v) => (v.class_index, v.name_and_type_index),
            x => panic!("Unexpected constant in putstatic: {:?}", x),
        };

        let class = frame.constants[class_index as usize - 1]
            .expect_class()
            .unwrap();
        let class_name = frame.constants[class as usize - 1].expect_utf8().unwrap();
        jvm.init_class(&class_name);

        let field = frame.constants[desc_index as usize - 1]
            .expect_name_and_type()
            .unwrap();
        let field_name = frame.constants[field.name_index as usize - 1]
            .expect_utf8()
            .unwrap();
        let descriptor = frame.constants[field.descriptor_index as usize - 1]
            .expect_utf8()
            .unwrap();

        let value = frame.stack.pop().expect("Unable to pop stack");
        debug!(
            "Put value {:?} into {}::{} {}",
            &value, &class_name, &field_name, descriptor
        );
        let field_reference = format!("{}_{}", clean_str(&class_name), clean_str(&field_name));
        jvm.static_fields.insert(field_reference, value);
    }
}

instruction! {@partial invokevirtual, 0xb6, u16}

impl InstructionAction for invokevirtual {
    fn exec(&self, frame: &mut StackFrame, jvm: &mut JVM) {
        let invokevirtual(field) = *self;

        let (class_index, desc_index) = match &frame.constants[field as usize - 1] {
            Constant::MethodRef(v) => (v.class_index, v.name_and_type_index),
            Constant::InterfaceMethodRef(v) => (v.class_index, v.name_and_type_index),
            x => panic!("Unexpected constant in putstatic: {:?}", x),
        };

        let class = frame.constants[class_index as usize - 1]
            .expect_class()
            .unwrap();
        let class_name = frame.constants[class as usize - 1].expect_utf8().unwrap();
        // jvm.init_class(&class_name);

        let field = frame.constants[desc_index as usize - 1]
            .expect_name_and_type()
            .unwrap();
        let field_name = frame.constants[field.name_index as usize - 1]
            .expect_utf8()
            .unwrap();
        let descriptor = frame.constants[field.descriptor_index as usize - 1]
            .expect_utf8()
            .unwrap();

        if let Ok(FieldDescriptor::Method { args, .. }) = FieldDescriptor::read_str(&descriptor) {
            let mut stack_args = frame.stack[frame.stack.len() - args.len()..].to_vec();

            for _ in 0..args.len() {
                frame.stack.pop();
            }

            let target = match frame.stack.pop() {
                Some(LocalVariable::Reference(Some(v))) => v.clone(),
                _ => {
                    raise_null_pointer_exception(frame, jvm);
                    warn!(
                        "Raised NullPointerException while trying to call {}::{} {}",
                        &class_name, &field_name, &descriptor
                    );
                    return;
                } // x => panic!("Attempted to run invokevirtual, but did not find target object: {:?}", x),
            };

            // stack_args.insert(0, LocalVariable::Reference(Some(target.clone())));
            match jvm.exec_method(target, &field_name, &descriptor, stack_args) {
                Ok(Some(v)) => frame.stack.push(v),
                Err(e) => frame.throws = Some(e),
                _ => {}
            }
        } else {
            panic!(
                "Unable to execute {}::{} {}",
                &class_name, &field_name, &descriptor
            );
        }
    }
}

instruction! {@partial new, 0xbb, u16}

impl InstructionAction for new {
    fn exec(&self, frame: &mut StackFrame, jvm: &mut JVM) {
        let new(field) = *self;
        let class = frame.constants[field as usize - 1]
            .expect_class()
            .expect("Expected class from constant pool");
        let class_name = frame.constants[class as usize - 1].expect_utf8().unwrap();

        // jvm.init_class(&class_name);
        jvm.class_loader.attempt_load(&class_name).unwrap();
        let object = jvm.class_loader.class(&class_name).unwrap().build_object();
        frame
            .stack
            .push(LocalVariable::Reference(Some(Rc::new(UnsafeCell::new(
                object,
            )))));
        debug!("Pushed new instance of {} to the stack", class_name);
    }
}

instruction! {@partial invokespecial, 0xb7, u16}

impl InstructionAction for invokespecial {
    fn exec(&self, frame: &mut StackFrame, jvm: &mut JVM) {
        let invokespecial(field) = *self;

        let (class_index, desc_index) = match &frame.constants[field as usize - 1] {
            Constant::MethodRef(v) => (v.class_index, v.name_and_type_index),
            Constant::InterfaceMethodRef(v) => (v.class_index, v.name_and_type_index),
            x => panic!("Unexpected constant in putstatic: {:?}", x),
        };

        let class = frame.constants[class_index as usize - 1]
            .expect_class()
            .unwrap();
        let class_name = frame.constants[class as usize - 1].expect_utf8().unwrap();
        // jvm.init_class(&class_name);

        let field = frame.constants[desc_index as usize - 1]
            .expect_name_and_type()
            .unwrap();
        let field_name = frame.constants[field.name_index as usize - 1]
            .expect_utf8()
            .unwrap();
        let descriptor = frame.constants[field.descriptor_index as usize - 1]
            .expect_utf8()
            .unwrap();

        let (method_class, main_method, constants) =
            match jvm.find_instance_method(&class_name, &field_name, &descriptor) {
                Some(v) => v,
                _ => panic!(
                    "Unable to find {}::{} {}",
                    &class_name, &field_name, &descriptor
                ),
            };

        // debug!("Attempting to run invokespecial {}::{} {}", &method_class, &field_name, &descriptor);
        // frame.debug_print();

        if let Ok(FieldDescriptor::Method { args, .. }) = FieldDescriptor::read_str(&descriptor) {
            let mut stack_args = frame.stack[frame.stack.len() - args.len()..].to_vec();

            for _ in 0..args.len() {
                frame.stack.pop();
            }

            let target = match frame.stack.pop() {
                Some(LocalVariable::Reference(Some(v))) => v.clone(),
                _ => {
                    raise_null_pointer_exception(frame, jvm);
                    warn!(
                        "Raised NullPointerException while trying to call {}::{} {}",
                        &class_name, &field_name, &descriptor
                    );
                    return;
                }
            };

            // stack_args.insert(0, LocalVariable::Reference(Some(target.clone())));
            match jvm.exec(target, &method_class, main_method, constants, stack_args) {
                Ok(Some(v)) => frame.stack.push(v),
                Err(e) => frame.throws = Some(e),
                _ => {}
            }
        } else {
            panic!(
                "Unable to execute {}::{} {}",
                &class_name, &field_name, &descriptor
            );
        }
    }
}

instruction! {@partial getfield, 0xb4, u16}

impl InstructionAction for getfield {
    fn exec(&self, frame: &mut StackFrame, jvm: &mut JVM) {
        let getfield(field) = *self;

        let (class_index, desc_index) = match &frame.constants[field as usize - 1] {
            Constant::FieldRef(v) => (v.class_index, v.name_and_type_index),
            x => panic!("Unexpected constant in getfield: {:?}", x),
        };

        // let class = frame.constants[class_index as usize - 1]
        //     .expect_class()
        //     .unwrap();
        // let class_name = frame.constants[class as usize - 1].expect_utf8().unwrap();
        // jvm.init_class(&class_name);

        let field = frame.constants[desc_index as usize - 1]
            .expect_name_and_type()
            .unwrap();
        let field_name = frame.constants[field.name_index as usize - 1]
            .expect_utf8()
            .unwrap();

        if let Some(LocalVariable::Reference(Some(mut obj))) = frame.stack.pop() {
            if let Object::Instance { fields, .. } = unsafe { &*obj.get() } {
                if let Some(v) = fields.get(&field_name) {
                    frame.stack.push(v.clone());
                } else {
                    panic!(
                        "Attempted to get field that does not exist: {}",
                        &field_name
                    );
                }
            } else {
                panic!("Attempted to get field from non-instance");
            }
        } else {
            raise_null_pointer_exception(frame, jvm);
        }
    }
}

instruction! {@partial putfield, 0xb5, u16}

impl InstructionAction for putfield {
    fn exec(&self, frame: &mut StackFrame, jvm: &mut JVM) {
        let putfield(field) = *self;

        let (class_index, desc_index) = match &frame.constants[field as usize - 1] {
            Constant::FieldRef(v) => (v.class_index, v.name_and_type_index),
            x => panic!("Unexpected constant in putfield: {:?}", x),
        };

        // let class = frame.constants[class_index as usize - 1]
        //     .expect_class()
        //     .unwrap();
        // let class_name = frame.constants[class as usize - 1].expect_utf8().unwrap();
        // jvm.init_class(&class_name);

        let field = frame.constants[desc_index as usize - 1]
            .expect_name_and_type()
            .unwrap();
        let field_name = frame.constants[field.name_index as usize - 1]
            .expect_utf8()
            .unwrap();

        let value = frame.stack.pop().unwrap();

        if let Some(LocalVariable::Reference(Some(mut obj))) = frame.stack.pop() {
            if let Object::Instance { fields, .. } = unsafe { &mut *obj.get() } {
                fields.insert(field_name, value);
            } else {
                panic!("Attempted to get field from non-instance");
            }
        } else {
            raise_null_pointer_exception(frame, jvm);
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct invokeinterface {
    index: u16,
    count: u8,
}

impl Instruction for invokeinterface {
    fn write(&self, buffer: &mut Cursor<Vec<u8>>) -> io::Result<()> {
        buffer.write_u8(Self::FORM)?;
        buffer.write_u16::<BigEndian>(self.index)?;
        buffer.write_u8(self.count)?;
        buffer.write_u8(0)
    }
    fn exec(&self, frame: &mut StackFrame, jvm: &mut JVM) {
        <Self as InstructionAction>::exec(self, frame, jvm);
    }
}

impl StaticInstruct for invokeinterface {
    const FORM: u8 = 0xb9;

    fn read(form: u8, buffer: &mut Cursor<Vec<u8>>) -> io::Result<Box<dyn Instruction>> {
        let ret = invokeinterface {
            index: buffer.read_u16::<BigEndian>()?,
            count: buffer.read_u8()?,
        };
        assert_eq!(buffer.read_u8()?, 0);
        Ok(Box::new(ret))
    }
}

impl InstructionAction for invokeinterface {
    fn exec(&self, frame: &mut StackFrame, jvm: &mut JVM) {
        let invokeinterface { index, count } = *self;

        let (class_index, desc_index) = match &frame.constants[index as usize - 1] {
            Constant::MethodRef(v) => (v.class_index, v.name_and_type_index),
            Constant::InterfaceMethodRef(v) => (v.class_index, v.name_and_type_index),
            x => panic!("Unexpected constant in putstatic: {:?}", x),
        };

        let class = frame.constants[class_index as usize - 1]
            .expect_class()
            .unwrap();
        let class_name = frame.constants[class as usize - 1].expect_utf8().unwrap();
        // jvm.init_class(&class_name);

        let field = frame.constants[desc_index as usize - 1]
            .expect_name_and_type()
            .unwrap();
        let field_name = frame.constants[field.name_index as usize - 1]
            .expect_utf8()
            .unwrap();
        let descriptor = frame.constants[field.descriptor_index as usize - 1]
            .expect_utf8()
            .unwrap();

        if let Ok(FieldDescriptor::Method { args, .. }) = FieldDescriptor::read_str(&descriptor) {
            let mut stack_args = frame.stack[frame.stack.len() - args.len()..].to_vec();

            for _ in 0..args.len() {
                frame.stack.pop();
            }

            // let target = match frame.stack.pop() {
            //     Some(LocalVariable::Reference(Some(v))) => v.clone(),
            //     _ => panic!("Attempted to run invokevirtual, but did not find target object!"),
            // };

            let target = match frame.stack.pop() {
                Some(LocalVariable::Reference(Some(v))) => v.clone(),
                _ => {
                    raise_null_pointer_exception(frame, jvm);
                    warn!(
                        "Raised NullPointerException while trying to call {}::{} {}",
                        &class_name, &field_name, &descriptor
                    );
                    return;
                }
            };

            // stack_args.insert(0, LocalVariable::Reference(Some(target.clone())));
            match jvm.exec_method(target, &field_name, &descriptor, stack_args) {
                Ok(Some(v)) => frame.stack.push(v),
                Err(e) => frame.throws = Some(e),
                _ => {}
            }
        } else {
            panic!(
                "Unable to execute {}::{} {}",
                &class_name, &field_name, &descriptor
            );
        }
    }
}

instruction! {@partial instanceof, 0xc1, u16}

impl InstructionAction for instanceof {
    fn exec(&self, frame: &mut StackFrame, jvm: &mut JVM) {
        let instanceof(class_index) = *self;

        let class = frame.constants[class_index as usize - 1]
            .expect_class()
            .unwrap();
        let class_name = frame.constants[class as usize - 1].expect_utf8().unwrap();

        let target = match frame.stack.pop() {
            Some(LocalVariable::Reference(Some(v))) => unsafe { (&*v.get()).expect_class() },
            Some(LocalVariable::Reference(None)) => {
                frame.stack.push(LocalVariable::Byte(0));
                return;
            }
            _ => panic!("Attempted to run instanceof, but did not find target object!"),
        }
        .unwrap();

        if class_name == target {
            frame.stack.push(LocalVariable::Byte(1));
            return;
        }

        frame.stack.push(LocalVariable::Byte(
            jvm.instanceof(&target, &class_name).unwrap() as _,
        ));
    }
}

pub fn raise_null_pointer_exception(frame: &mut StackFrame, jvm: &mut JVM) {
    jvm.init_class("java/lang/NullPointerException");

    warn!("Throwing java/lang/NullPointerException!");
    let object = jvm
        .class_loader
        .class("java/lang/NullPointerException")
        .unwrap()
        .build_object();
    frame.throws = Some(LocalVariable::Reference(Some(Rc::new(UnsafeCell::new(
        object,
    )))));
}

//  - getfield
//  - putfield
//  - getstatic
//  - invokevirtual
