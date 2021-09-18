use std::io;
use std::io::Cursor;

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

// use crate::r#mod::{AccessFlags, BufferedRead};
// use crate::constant_pool::{ClassElement, Constant};
use crate::class::constant::{ClassElement, Constant};
use crate::class::{AccessFlags, BufferedRead};
use crate::instruction::{Instruction, InstructionAction, StaticInstruct};
use crate::jvm::call::{clean_str, FlowControl, JavaEnvInvoke, StackFrame};
use crate::jvm::mem::FieldDescriptor;
use crate::jvm::mem::{JavaValue, ManualInstanceReference, ObjectHandle, ObjectReference};
use crate::jvm::JavaEnv;
use parking_lot::RwLock;
use std::sync::Arc;

instruction! {@partial getstatic, 0xb2, u16}

impl getstatic {
    pub fn check_static_init(
        jvm: &mut JavaEnv,
        class: &str,
        field: &str,
        desc: &str,
    ) -> Option<JavaValue> {
        let class_spec = jvm.class_loader.class(class)?;
        let field_spec = class_spec.get_field(field, desc)?;
        if !field_spec.access.contains(AccessFlags::STATIC) {
            return None;
        }

        let descriptor = FieldDescriptor::read_str(desc).ok()?;
        let ret = descriptor.initial_local();

        // let field_reference = format!("{}_{}", clean_str(class), clean_str(field));
        jvm.static_fields.set_static(&class, &field, ret);
        Some(ret)
    }
}

impl InstructionAction for getstatic {
    fn exec(
        &self,
        frame: &mut StackFrame,
        jvm: &mut Arc<RwLock<JavaEnv>>,
    ) -> Result<(), FlowControl> {
        let getstatic(field) = *self;

        if let Constant::FieldRef(reference) = &frame.constants[field as usize - 1] {
            let class = frame.constants[reference.class_index as usize - 1]
                .expect_class()
                .unwrap();
            let mut class_name = frame.constants[class as usize - 1].expect_utf8().unwrap();
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

            loop {
                let lock = jvm.read();
                let raw_class = lock.class_loader.class(&class_name).unwrap();
                if raw_class.get_field(&field_name, &descriptor).is_some() {
                    break;
                }

                // Reached base case, it will error anyway
                if class_name == "java/lang/Object" {
                    break;
                }

                class_name = raw_class.super_class();
            }

            // let field_reference = format!("{}_{}", clean_str(&class_name), clean_str(&field_name));
            let value = {
                let mut lock = jvm.write();
                match lock.static_fields.get_static(&class_name, &field_name) {
                    Some(v) => v,
                    None => match Self::check_static_init(
                        &mut *lock,
                        &class_name,
                        &field_name,
                        &descriptor,
                    ) {
                        Some(v) => v,
                        None => panic!("Static value not found: {}::{}", &class_name, &field_name),
                    },
                }
            };
            debug!(
                "Got value {:?} from {}::{} {}",
                &value, &class_name, &field_name, descriptor
            );

            if matches!(&value, JavaValue::Long(_) | JavaValue::Double(_)) {
                frame.stack.push(value.to_owned());
            }

            frame.stack.push(value);
        } else {
            panic!("Error in getstatic");
        }
        Ok(())
    }
}

instruction! {@partial invokestatic, 0xb8, u16}

impl InstructionAction for invokestatic {
    fn exec(
        &self,
        frame: &mut StackFrame,
        jvm: &mut Arc<RwLock<JavaEnv>>,
    ) -> Result<(), FlowControl> {
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
        jvm.init_class(&class_name);
        jvm.write().class_loader.attempt_load(&class_name).unwrap();

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
            let stack_args =
                frame.stack[frame.stack.len() - FieldDescriptor::word_len(&args)..].to_vec();

            for _ in 0..stack_args.len() {
                frame.stack.pop();
            }

            let method = ClassElement::new(class_name, field_name, descriptor);
            match jvm.invoke_static(method, stack_args) {
                Ok(Some(JavaValue::Long(v))) => {
                    frame.stack.push(JavaValue::Long(v));
                    frame.stack.push(JavaValue::Long(v));
                }
                Ok(Some(JavaValue::Double(v))) => {
                    frame.stack.push(JavaValue::Double(v));
                    frame.stack.push(JavaValue::Double(v));
                }
                Ok(Some(v)) => frame.stack.push(v),
                Ok(None) => {}
                Err(e) => return Err(e),
            }
        } else {
            panic!(
                "Unable to execute {}::{} {}",
                &class_name, &field_name, &descriptor
            );
        }
        Ok(())
    }
}

instruction! {@partial putstatic, 0xb3, u16}

impl InstructionAction for putstatic {
    fn exec(
        &self,
        frame: &mut StackFrame,
        jvm: &mut Arc<RwLock<JavaEnv>>,
    ) -> Result<(), FlowControl> {
        let putstatic(field) = *self;

        let (class_index, desc_index) = match &frame.constants[field as usize - 1] {
            Constant::FieldRef(v) => (v.class_index, v.name_and_type_index),
            x => panic!("Unexpected constant in putstatic: {:?}", x),
        };

        let class = frame.constants[class_index as usize - 1]
            .expect_class()
            .unwrap();
        let mut class_name = frame.constants[class as usize - 1].expect_utf8().unwrap();
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

        loop {
            let lock = jvm.read();
            let raw_class = lock.class_loader.class(&class_name).unwrap();
            if raw_class.get_field(&field_name, &descriptor).is_some() {
                break;
            }

            // Reached base case, it will error anyway
            if class_name == "java/lang/Object" {
                break;
            }

            class_name = raw_class.super_class();
        }

        let mut value = frame.stack.pop().expect("Unable to pop stack");

        if matches!(&value, JavaValue::Long(_) | JavaValue::Double(_)) {
            value = frame.stack.pop().unwrap();
        }

        debug!(
            "Put value {:?} into {}::{} {}",
            &value, &class_name, &field_name, descriptor
        );
        // let field_reference = format!("{}_{}", clean_str(&class_name), clean_str(&field_name));
        jvm.write()
            .static_fields
            .set_static(&class_name, &field_name, value);
        Ok(())
    }
}

instruction! {@partial invokevirtual, 0xb6, u16}

impl InstructionAction for invokevirtual {
    fn exec(
        &self,
        frame: &mut StackFrame,
        jvm: &mut Arc<RwLock<JavaEnv>>,
    ) -> Result<(), FlowControl> {
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
            let stack_args =
                frame.stack[frame.stack.len() - FieldDescriptor::word_len(&args)..].to_vec();

            for _ in 0..stack_args.len() {
                frame.stack.pop();
            }

            let target = match frame.stack.pop() {
                Some(JavaValue::Reference(Some(v))) => v,
                _ => {
                    // raise_null_pointer_exception(frame, jvm);
                    // warn!(
                    //     "Raised NullPointerException while trying to call {}::{} {}",
                    //     &class_name, &field_name, &descriptor
                    // );
                    // return;
                    return Err(FlowControl::throw("java/lang/NullPointerException"));
                } // x => panic!("Attempted to run invokevirtual, but did not find target object: {:?}", x),
            };

            // stack_args.insert(0, JavaValue::Reference(Some(target.clone())));
            let method = ClassElement::new(class_name, field_name, descriptor);
            match jvm.invoke_virtual(method, target, stack_args) {
                Ok(Some(JavaValue::Long(v))) => {
                    frame.stack.push(JavaValue::Long(v));
                    frame.stack.push(JavaValue::Long(v));
                }
                Ok(Some(JavaValue::Double(v))) => {
                    frame.stack.push(JavaValue::Double(v));
                    frame.stack.push(JavaValue::Double(v));
                }
                Ok(Some(v)) => frame.stack.push(v),
                Ok(None) => {}
                Err(e) => return Err(e),
            }
        } else {
            panic!(
                "Unable to execute {}::{} {}",
                &class_name, &field_name, &descriptor
            );
        }
        Ok(())
    }
}

instruction! {@partial new, 0xbb, u16}

impl InstructionAction for new {
    fn exec(
        &self,
        frame: &mut StackFrame,
        jvm: &mut Arc<RwLock<JavaEnv>>,
    ) -> Result<(), FlowControl> {
        let new(field) = *self;
        let class = frame.constants[field as usize - 1]
            .expect_class()
            .expect("Expected class from constant pool");
        let class_name = frame.constants[class as usize - 1].expect_utf8().unwrap();

        if class_name.ends_with("Exception") || class_name.ends_with("Error") {
            #[cfg(feature = "callstack")]
            jvm.read().thread_manager.debug_print();
            // panic!("Starting to prepare {}", class_name);
        }

        jvm.init_class(&class_name);
        jvm.write().class_loader.attempt_load(&class_name).unwrap();

        let object = ObjectHandle::new(jvm.write().class_schema(&class_name));
        frame.stack.push(JavaValue::Reference(Some(object)));
        debug!("Pushed new instance of {} to the stack", class_name);
        Ok(())
    }
}

instruction! {@partial invokespecial, 0xb7, u16}

impl InstructionAction for invokespecial {
    fn exec(
        &self,
        frame: &mut StackFrame,
        jvm: &mut Arc<RwLock<JavaEnv>>,
    ) -> Result<(), FlowControl> {
        let invokespecial(field) = *self;
        // frame.debug_print();

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

        let (method_class, _main_method, _constants) =
            match jvm
                .read()
                .find_instance_method(&class_name, &field_name, &descriptor)
            {
                Some(v) => v,
                _ => panic!(
                    "Unable to find {}::{} {}",
                    &class_name, &field_name, &descriptor
                ),
            };

        info!(
            "Calling special: {}::{} {}",
            &class_name, &field_name, &descriptor
        );
        if let Ok(FieldDescriptor::Method { args, .. }) = FieldDescriptor::read_str(&descriptor) {
            info!(
                "Popping {} args ({} slots)",
                args.len(),
                FieldDescriptor::word_len(&args)
            );
            info!("Frame size: {}", frame.stack.len());
            let stack_args =
                frame.stack[frame.stack.len() - FieldDescriptor::word_len(&args)..].to_vec();

            for _ in 0..stack_args.len() {
                frame.stack.pop();
            }

            let target = match frame.stack.pop() {
                Some(JavaValue::Reference(Some(v))) => v,
                _ => {
                    warn!(
                        "Raised NullPointerException while trying to call {}::{} {}",
                        &class_name, &field_name, &descriptor
                    );
                    return Err(FlowControl::throw("java/lang/NullPointerException"));
                }
            };

            info!("Got target: {}", target.get_class());

            // stack_args.insert(0, JavaValue::Reference(Some(target.clone())));
            let method = ClassElement::new(method_class, field_name, descriptor);
            match jvm.invoke_special(method, target, stack_args) {
                // match jvm.exec(target, &method_class, main_method, constants, stack_args) {
                Ok(Some(JavaValue::Long(v))) => {
                    frame.stack.push(JavaValue::Long(v));
                    frame.stack.push(JavaValue::Long(v));
                }
                Ok(Some(JavaValue::Double(v))) => {
                    frame.stack.push(JavaValue::Double(v));
                    frame.stack.push(JavaValue::Double(v));
                }
                Ok(Some(v)) => frame.stack.push(v),
                Ok(None) => {}
                Err(e) => return Err(e),
            }
        } else {
            panic!(
                "Unable to execute {}::{} {}",
                &class_name, &field_name, &descriptor
            );
        }
        Ok(())
    }
}

instruction! {@partial getfield, 0xb4, u16}

impl InstructionAction for getfield {
    fn exec(
        &self,
        frame: &mut StackFrame,
        _: &mut Arc<RwLock<JavaEnv>>,
    ) -> Result<(), FlowControl> {
        let getfield(field) = *self;

        let (_, desc_index) = match &frame.constants[field as usize - 1] {
            Constant::FieldRef(v) => (v.class_index, v.name_and_type_index),
            x => panic!("Unexpected constant in getfield: {:?}", x),
        };

        let field = frame.constants[desc_index as usize - 1]
            .expect_name_and_type()
            .unwrap();
        let field_name = frame.constants[field.name_index as usize - 1]
            .expect_utf8()
            .unwrap();

        if let Some(JavaValue::Reference(Some(obj))) = frame.stack.pop() {
            let instance = obj.expect_instance();

            let value = instance.read_named_field(&field_name);

            if matches!(&value, JavaValue::Long(_) | JavaValue::Double(_)) {
                frame.stack.push(value.to_owned());
            }

            frame.stack.push(value);

            // if let Object::Instance { fields, .. } = unsafe { &*obj.get() } {
            //     if let Some(v) = fields.get(&field_name) {
            //         frame.stack.push(v.clone());
            //     } else {
            //         panic!(
            //             "Attempted to get field that does not exist: {}",
            //             &field_name
            //         );
            //     }
            // } else {
            //     panic!("Attempted to get field from non-instance");
            // }
            Ok(())
        } else {
            // raise_null_pointer_exception(frame, jvm);
            Err(FlowControl::throw("java/lang/NullPointerException"))
        }
    }
}

instruction! {@partial putfield, 0xb5, u16}

impl InstructionAction for putfield {
    fn exec(
        &self,
        frame: &mut StackFrame,
        _: &mut Arc<RwLock<JavaEnv>>,
    ) -> Result<(), FlowControl> {
        let putfield(field) = *self;

        let (class_index, desc_index) = match &frame.constants[field as usize - 1] {
            Constant::FieldRef(v) => (v.class_index, v.name_and_type_index),
            x => panic!("Unexpected constant in putfield: {:?}", x),
        };

        let class_index = frame.constants[class_index as usize - 1]
            .expect_class()
            .unwrap();
        let class_name = frame.constants[class_index as usize - 1]
            .expect_utf8()
            .unwrap();
        let field = frame.constants[desc_index as usize - 1]
            .expect_name_and_type()
            .unwrap();
        let field_name = frame.constants[field.name_index as usize - 1]
            .expect_utf8()
            .unwrap();
        let desc_name = frame.constants[field.descriptor_index as usize - 1]
            .expect_utf8()
            .unwrap();

        // jvm.debug_print_call_stack();
        // frame.debug_print();
        let mut value = frame.stack.pop().unwrap();

        if matches!(&value, JavaValue::Long(_) | JavaValue::Double(_)) {
            value = frame.stack.pop().unwrap();
        }

        if let Some(JavaValue::Reference(Some(obj))) = frame.stack.pop() {
            let instance = obj.expect_instance();
            debug!(
                "Putting field {}::{} {}",
                &class_name, &field_name, &desc_name
            );
            instance.write_named_field(&field_name, value);
            // if let Object::Instance { fields, .. } = unsafe { &mut *obj.get() } {
            //     fields.insert(field_name, value);
            // } else {
            //     panic!("Attempted to get field from non-instance");
            // }
            Ok(())
        } else {
            // raise_null_pointer_exception(frame, jvm);
            Err(FlowControl::throw("java/lang/NullPointerException"))
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
    fn exec(
        &self,
        frame: &mut StackFrame,
        jvm: &mut Arc<RwLock<JavaEnv>>,
    ) -> Result<(), FlowControl> {
        <Self as InstructionAction>::exec(self, frame, jvm)
    }
}

impl StaticInstruct for invokeinterface {
    const FORM: u8 = 0xb9;

    fn read(_form: u8, buffer: &mut Cursor<Vec<u8>>) -> io::Result<Box<dyn Instruction>> {
        let ret = invokeinterface {
            index: buffer.read_u16::<BigEndian>()?,
            count: buffer.read_u8()?,
        };
        assert_eq!(buffer.read_u8()?, 0);
        Ok(Box::new(ret))
    }
}

impl InstructionAction for invokeinterface {
    fn exec(
        &self,
        frame: &mut StackFrame,
        jvm: &mut Arc<RwLock<JavaEnv>>,
    ) -> Result<(), FlowControl> {
        let invokeinterface { index, .. } = *self;

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
            let stack_args =
                frame.stack[frame.stack.len() - FieldDescriptor::word_len(&args)..].to_vec();

            for _ in 0..stack_args.len() {
                frame.stack.pop();
            }

            let target = match frame.stack.pop() {
                Some(JavaValue::Reference(Some(v))) => v,
                _ => {
                    // raise_null_pointer_exception(frame, jvm);
                    debug!(
                        "Raised NullPointerException while trying to call {}::{} {}",
                        &class_name, &field_name, &descriptor
                    );
                    return Err(FlowControl::throw("java/lang/NullPointerException"));
                }
            };

            // stack_args.insert(0, JavaValue::Reference(Some(target.clone())));
            let method = ClassElement::new(class_name, field_name, descriptor);
            match jvm.invoke_virtual(method, target, stack_args) {
                Ok(Some(JavaValue::Long(v))) => {
                    frame.stack.push(JavaValue::Long(v));
                    frame.stack.push(JavaValue::Long(v));
                }
                Ok(Some(JavaValue::Double(v))) => {
                    frame.stack.push(JavaValue::Double(v));
                    frame.stack.push(JavaValue::Double(v));
                }
                Ok(Some(v)) => frame.stack.push(v),
                Ok(None) => {}
                Err(e) => return Err(e),
            }
        } else {
            panic!(
                "Unable to execute {}::{} {}",
                &class_name, &field_name, &descriptor
            );
        }
        Ok(())
    }
}

instruction! {@partial instanceof, 0xc1, u16}

impl InstructionAction for instanceof {
    fn exec(
        &self,
        frame: &mut StackFrame,
        jvm: &mut Arc<RwLock<JavaEnv>>,
    ) -> Result<(), FlowControl> {
        let instanceof(class_index) = *self;

        let class = frame.constants[class_index as usize - 1]
            .expect_class()
            .unwrap();
        let class_name = frame.constants[class as usize - 1].expect_utf8().unwrap();

        let target = match frame.stack.pop() {
            // Some(JavaValue::Reference(Some(v))) => unsafe { (&*v.get()).expect_class() },
            Some(JavaValue::Reference(Some(v))) => v.get_class(),
            Some(JavaValue::Reference(None)) => {
                frame.stack.push(JavaValue::Byte(0));
                return Ok(());
            }
            _ => panic!("Attempted to run instanceof, but did not find target object!"),
        };

        if class_name == target {
            frame.stack.push(JavaValue::Byte(1));
            return Ok(());
        }

        frame.stack.push(JavaValue::Byte(
            jvm.read().instanceof(&target, &class_name).unwrap() as _,
        ));
        Ok(())
    }
}

// pub fn raise_null_pointer_exception(frame: &mut StackFrame, jvm: &mut Arc<RwLock<JavaEnv>>) {
//     jvm.init_class("java/lang/NullPointerException");
//
//     warn!("Throwing java/lang/NullPointerException!");
//     let object = ObjectHandle::new(jvm.class_schema("java/lang/NullPointerException"));
//     frame.throws = Some(JavaValue::Reference(Some(object)));
// }
