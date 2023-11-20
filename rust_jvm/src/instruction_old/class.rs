use std::io;
use std::io::Cursor;

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

use crate::class::constant::{
    ClassElement, Constant, ConstantClass, ConstantDouble, ConstantFloat, ConstantInteger,
    ConstantLong, ConstantMethodHandle, ConstantMethodType, ConstantString, ReferenceKind,
};
use crate::class::{AccessFlags, BufferedRead};
use crate::instruction::{Instruction, InstructionAction, StaticInstruct};
use crate::jvm::call::{FlowControl, JavaEnvInvoke, StackFrame};
use crate::jvm::mem::FieldDescriptor;
use crate::jvm::mem::{JavaValue, ManualInstanceReference, ObjectHandle, ObjectReference};
use crate::jvm::JavaEnv;
use parking_lot::RwLock;
use std::sync::Arc;

instruction! {getstatic, 0xb2, u16}

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

        if let Constant::FieldRef(reference) = &frame.constants[field] {
            let mut class_name = frame.constants.class_name(reference.class_index).to_owned();
            jvm.init_class(&class_name);

            let (field_name, descriptor) =
                frame.constants.name_and_type(reference.name_and_type_index);

            let mut lock = jvm.write();

            loop {
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

            let value = match lock.static_fields.get_static(&class_name, &field_name) {
                Some(v) => v,
                None => {
                    match Self::check_static_init(&mut *lock, &class_name, &field_name, &descriptor)
                    {
                        Some(v) => v,
                        None => panic!("Static value not found: {}::{}", &class_name, &field_name),
                    }
                }
            };

            trace!(
                "Get static request for {}::{} {}",
                &class_name,
                &field_name,
                descriptor
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

instruction! {invokestatic, 0xb8, u16}

impl InstructionAction for invokestatic {
    fn exec(
        &self,
        frame: &mut StackFrame,
        jvm: &mut Arc<RwLock<JavaEnv>>,
    ) -> Result<(), FlowControl> {
        let invokestatic(field) = *self;

        let (class_name, field_name, descriptor) = frame.constants.class_element_ref(field);
        jvm.init_class(&class_name);
        jvm.write().class_loader.attempt_load(&class_name).unwrap();

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

instruction! {putstatic, 0xb3, u16}

impl InstructionAction for putstatic {
    fn exec(
        &self,
        frame: &mut StackFrame,
        jvm: &mut Arc<RwLock<JavaEnv>>,
    ) -> Result<(), FlowControl> {
        let putstatic(field) = *self;

        let (class_name, field_name, descriptor) = frame.constants.class_element_ref(field);

        let mut class_name = class_name.to_owned();

        jvm.init_class(&class_name);

        // TODO: This probably is not required by specification
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

        trace!(
            "Put value {:?} into {}::{} {}",
            &value,
            &class_name,
            &field_name,
            descriptor
        );

        jvm.write()
            .static_fields
            .set_static(&class_name, &field_name, value);
        Ok(())
    }
}

instruction! {invokevirtual, 0xb6, u16}

impl InstructionAction for invokevirtual {
    fn exec(
        &self,
        frame: &mut StackFrame,
        jvm: &mut Arc<RwLock<JavaEnv>>,
    ) -> Result<(), FlowControl> {
        let invokevirtual(field) = *self;

        let (class_name, field_name, descriptor) = frame.constants.class_element_ref(field);

        if let Ok(FieldDescriptor::Method { args, .. }) = FieldDescriptor::read_str(&descriptor) {
            let stack_args =
                frame.stack[frame.stack.len() - FieldDescriptor::word_len(&args)..].to_vec();

            for _ in 0..stack_args.len() {
                frame.stack.pop();
            }

            let target = match frame.stack.pop() {
                Some(JavaValue::Reference(Some(v))) => v,
                x => {
                    error!("================================================================");
                    frame.debug_print();
                    error!("================================================================");
                    error!("Popped Arguments:");
                    for arg in stack_args {
                        error!("\t{:?}", arg);
                    }
                    error!("================================================================");
                    error!("Expected invocation target, got: {:?}", x);
                    error!(
                        "Attempted to run {}::{} {}",
                        class_name, field_name, descriptor
                    );
                    jvm.read().debug_print_call_stack();
                    return Err(FlowControl::throw("java/lang/NullPointerException"));
                }
            };

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

instruction! {new, 0xbb, u16}

impl InstructionAction for new {
    fn exec(
        &self,
        frame: &mut StackFrame,
        jvm: &mut Arc<RwLock<JavaEnv>>,
    ) -> Result<(), FlowControl> {
        let new(field) = *self;

        let class_name = frame.constants.class_name(field);

        if class_name.ends_with("Exception") || class_name.ends_with("Error") {
            #[cfg(feature = "callstack")]
            jvm.read().thread_manager.debug_print();
        }

        jvm.init_class(&class_name);
        jvm.write().class_loader.attempt_load(&class_name).unwrap();

        let object = ObjectHandle::new(jvm.write().class_schema(&class_name));
        frame.stack.push(JavaValue::Reference(Some(object)));
        trace!("Pushed new instance of {} to the stack", class_name);
        Ok(())
    }
}

instruction! {invokespecial, 0xb7, u16}

impl InstructionAction for invokespecial {
    fn exec(
        &self,
        frame: &mut StackFrame,
        jvm: &mut Arc<RwLock<JavaEnv>>,
    ) -> Result<(), FlowControl> {
        let invokespecial(field) = *self;

        let (class_name, field_name, descriptor) = frame.constants.class_element_ref(field);

        let (method_class, _main_method) =
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

        trace!(
            "Calling special: {}::{} {}",
            &class_name,
            &field_name,
            &descriptor
        );
        if let Ok(FieldDescriptor::Method { args, .. }) = FieldDescriptor::read_str(&descriptor) {
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

            let method = ClassElement::new(method_class.as_str(), field_name, descriptor);
            match jvm.invoke_special(method, target, stack_args) {
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

instruction! {getfield, 0xb4, u16}

impl InstructionAction for getfield {
    fn exec(
        &self,
        frame: &mut StackFrame,
        _: &mut Arc<RwLock<JavaEnv>>,
    ) -> Result<(), FlowControl> {
        let getfield(field) = *self;

        let (_class_name, field_name, _descriptor) = frame.constants.class_element_ref(field);

        if let Some(JavaValue::Reference(Some(obj))) = frame.stack.pop() {
            let instance = obj.expect_instance();

            let value = instance.lock().read_named_field(&field_name);

            if matches!(&value, JavaValue::Long(_) | JavaValue::Double(_)) {
                frame.stack.push(value.to_owned());
            }

            frame.stack.push(value);

            Ok(())
        } else {
            Err(FlowControl::throw("java/lang/NullPointerException"))
        }
    }
}

instruction! {putfield, 0xb5, u16}

impl InstructionAction for putfield {
    fn exec(
        &self,
        frame: &mut StackFrame,
        _: &mut Arc<RwLock<JavaEnv>>,
    ) -> Result<(), FlowControl> {
        let putfield(field) = *self;

        let (class_name, field_name, desc_name) = frame.constants.class_element_ref(field);

        let mut value = frame.stack.pop().unwrap();

        if matches!(&value, JavaValue::Long(_) | JavaValue::Double(_)) {
            value = frame.stack.pop().unwrap();
        }

        if let Some(JavaValue::Reference(Some(obj))) = frame.stack.pop() {
            let instance = obj.expect_instance();
            trace!(
                "Putting field {}::{} {}",
                &class_name,
                &field_name,
                &desc_name
            );
            instance.lock().write_named_field(&field_name, value);
            Ok(())
        } else {
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

        let (class_name, field_name, descriptor) = frame.constants.class_element_ref(index);

        if let Ok(FieldDescriptor::Method { args, .. }) = FieldDescriptor::read_str(&descriptor) {
            let stack_args =
                frame.stack[frame.stack.len() - FieldDescriptor::word_len(&args)..].to_vec();

            for _ in 0..stack_args.len() {
                frame.stack.pop();
            }

            let target = match frame.stack.pop() {
                Some(JavaValue::Reference(Some(v))) => v,
                _ => {
                    debug!(
                        "Raised NullPointerException while trying to call {}::{} {}",
                        &class_name, &field_name, &descriptor
                    );
                    return Err(FlowControl::throw("java/lang/NullPointerException"));
                }
            };

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

instruction! {instanceof, 0xc1, u16}

impl InstructionAction for instanceof {
    fn exec(
        &self,
        frame: &mut StackFrame,
        jvm: &mut Arc<RwLock<JavaEnv>>,
    ) -> Result<(), FlowControl> {
        let instanceof(class_index) = *self;

        let class_name = frame.constants.class_name(class_index);
        let target = match frame.stack.pop() {
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

#[derive(Debug, Copy, Clone)]
pub struct invokedynamic(u16);

impl crate::instruction::Instruction for invokedynamic {
    fn write(&self, buffer: &mut std::io::Cursor<Vec<u8>>) -> std::io::Result<()> {
        buffer.write_u8(<Self as crate::instruction::StaticInstruct>::FORM)?;
        buffer.write_u16::<byteorder::BigEndian>(self.0)?;
        buffer.write_u16::<byteorder::BigEndian>(0)
    }

    fn exec(
        &self,
        stack: &mut crate::jvm::call::StackFrame,
        jvm: &mut std::sync::Arc<parking_lot::RwLock<crate::jvm::JavaEnv>>,
    ) -> Result<(), crate::jvm::call::FlowControl> {
        <Self as crate::instruction::InstructionAction>::exec(self, stack, jvm)
    }
}

impl crate::instruction::StaticInstruct for invokedynamic {
    const FORM: u8 = 0xba;

    fn read(
        _: u8,
        buffer: &mut std::io::Cursor<Vec<u8>>,
    ) -> std::io::Result<Box<dyn crate::instruction::Instruction>> {
        let index = buffer.read_u16::<byteorder::BigEndian>()?;
        assert_eq!(buffer.read_u16::<byteorder::BigEndian>()?, 0);
        Ok(Box::new(invokedynamic(index)))
    }
}

impl InstructionAction for invokedynamic {
    fn exec(
        &self,
        frame: &mut StackFrame,
        jvm: &mut Arc<RwLock<JavaEnv>>,
    ) -> Result<(), FlowControl> {
        let invokedynamic(field) = *self;

        let (bootstrap_idx, desc_index) = match &frame.constants[field] {
            Constant::InvokeDynamic(v) => (v.bootstrap_method_attr_index, v.name_and_type_index),
            x => panic!("Unexpected constant in invokedynamic: {:?}", x),
        };

        let (field_name, descriptor) = frame.constants.name_and_type(desc_index);

        info!("Dynamic: {} {}", &field_name, &descriptor);

        // TODO: This should be more straightforward
        let (current_class, class_name) = {
            let lock = jvm.read();
            let current_class = lock
                .thread_manager
                .get_current_call_stack()
                .unwrap()
                .last()
                .unwrap()
                .0;
            (current_class, current_class.unwrap_as_class())
        };

        let bootstrap_method = {
            let lock = jvm.read();
            lock.class_loader
                .class(&class_name)
                .and_then(|x| x.bootstrap_methods())
                .map(|mut x| x.remove(bootstrap_idx as usize))
                .expect("Unable to find bootstrap method to invoke dynamic")
        };

        // let bootstrap_method = match bootstrap {
        //     Some(v) => v[bootstrap_idx as usize].to_owned(),
        //     None => panic!("No bootstrap method attribute on {}", &class_name),
        // };

        info!("{:?}", &bootstrap_method);

        let lookup = {
            jvm.write()
                .class_loader
                .attempt_load("java/lang/invoke/MethodHandles")
                .unwrap();

            let element = ClassElement::new(
                "java/lang/invoke/MethodHandles",
                "lookup",
                "()Ljava/lang/invoke/MethodHandles$Lookup;",
            );

            match jvm.invoke_static(element, vec![])? {
                Some(JavaValue::Reference(Some(v))) => v,
                x => panic!("{:?}", x),
            }
        };

        let mut dyn_args = Vec::with_capacity(bootstrap_method.bootstrap_arguments.len() + 2);
        dyn_args.push(JavaValue::Reference(Some(lookup)));
        dyn_args.push(jvm.write().build_string(&field_name));

        let desc = FieldDescriptor::read_str(&descriptor)
            .unwrap()
            .to_class(jvm);
        dyn_args.push(JavaValue::Reference(Some(desc)));

        for arg in bootstrap_method.bootstrap_arguments {
            match &frame.constants[arg] {
                Constant::Class(ConstantClass { name_index }) => {
                    let name = frame.constants.text(*name_index);
                    info!("\tClass {}", &name);
                    dyn_args.push(JavaValue::Reference(Some(
                        jvm.write().class_instance(&name),
                    )));
                }
                Constant::String(ConstantString { string_index }) => {
                    let name = frame.constants.text(*string_index);
                    info!("\tString {}", &name);
                    dyn_args.push(jvm.write().build_string(&name));
                }
                Constant::Int(ConstantInteger { value }) => {
                    info!("\tInt {}", value);
                    dyn_args.push(JavaValue::Int(*value));
                }
                Constant::Float(ConstantFloat { value }) => {
                    info!("\tFloat {}", value);
                    dyn_args.push(JavaValue::Float(*value));
                }
                Constant::Long(ConstantLong { value }) => {
                    info!("\tLong {}", value);
                    dyn_args.push(JavaValue::Long(*value));
                    dyn_args.push(JavaValue::Long(*value));
                }
                Constant::Double(ConstantDouble { value }) => {
                    info!("\tDouble {}", value);
                    dyn_args.push(JavaValue::Double(*value));
                    dyn_args.push(JavaValue::Double(*value));
                }
                Constant::MethodType(ConstantMethodType { descriptor_index }) => {
                    let desc = frame.constants.text(*descriptor_index);
                    info!("\tMethod Type {}", &desc);
                    let desc = FieldDescriptor::read_str(&desc).unwrap().to_class(jvm);
                    dyn_args.push(JavaValue::Reference(Some(desc)));
                }
                Constant::MethodHandle(ConstantMethodHandle {
                    reference_kind,
                    index,
                }) => {
                    let (class, field, desc) = frame.constants.class_element_ref(*index);
                    let class_instance =
                        JavaValue::Reference(Some(jvm.write().class_instance(class)));
                    let field_name = jvm.write().build_string(field);
                    let field_type = JavaValue::Reference(Some(
                        FieldDescriptor::read_str(desc).unwrap().to_class(jvm),
                    ));

                    let handle = match reference_kind {
                        ReferenceKind::GetField => {
                            let element = ClassElement::new("java/lang/invoke/MethodHandles$Lookup", "findGetter", "(Ljava/lang/Class;Ljava/lang/String;Ljava/lang/Class;)Ljava/lang/invoke/MethodHandle;");
                            jvm.invoke_virtual(
                                element,
                                lookup,
                                vec![class_instance, field_name, field_type],
                            )?
                        }
                        ReferenceKind::GetStatic => {
                            let element = ClassElement::new("java/lang/invoke/MethodHandles$Lookup", "findStaticGetter", "(Ljava/lang/Class;Ljava/lang/String;Ljava/lang/Class;)Ljava/lang/invoke/MethodHandle;");
                            jvm.invoke_virtual(
                                element,
                                lookup,
                                vec![class_instance, field_name, field_type],
                            )?
                        }
                        ReferenceKind::PutField => {
                            let element = ClassElement::new("java/lang/invoke/MethodHandles$Lookup", "findSetter", "(Ljava/lang/Class;Ljava/lang/String;Ljava/lang/Class;)Ljava/lang/invoke/MethodHandle;");
                            jvm.invoke_virtual(
                                element,
                                lookup,
                                vec![class_instance, field_name, field_type],
                            )?
                        }
                        ReferenceKind::PutStatic => {
                            let element = ClassElement::new("java/lang/invoke/MethodHandles$Lookup", "findStaticSetter", "(Ljava/lang/Class;Ljava/lang/String;Ljava/lang/Class;)Ljava/lang/invoke/MethodHandle;");
                            jvm.invoke_virtual(
                                element,
                                lookup,
                                vec![class_instance, field_name, field_type],
                            )?
                        }
                        ReferenceKind::InvokeVirtual => {
                            let element = ClassElement::new("java/lang/invoke/MethodHandles$Lookup", "findVirtual", "(Ljava/lang/Class;Ljava/lang/String;Ljava/lang/invoke/MethodType;)Ljava/lang/invoke/MethodHandle;");
                            jvm.invoke_virtual(
                                element,
                                lookup,
                                vec![class_instance, field_name, field_type],
                            )?
                        }
                        ReferenceKind::InvokeStatic => {
                            let element = ClassElement::new("java/lang/invoke/MethodHandles$Lookup", "findStatic", "(Ljava/lang/Class;Ljava/lang/String;Ljava/lang/invoke/MethodType;)Ljava/lang/invoke/MethodHandle;");
                            jvm.invoke_virtual(
                                element,
                                lookup,
                                vec![class_instance, field_name, field_type],
                            )?
                        }
                        ReferenceKind::InvokeSpecial => {
                            let element = ClassElement::new("java/lang/invoke/MethodHandles$Lookup", "findSpecial", "(Ljava/lang/Class;Ljava/lang/String;Ljava/lang/invoke/MethodType;Ljava/lang/Class;)Ljava/lang/invoke/MethodHandle;");
                            jvm.invoke_virtual(
                                element,
                                lookup,
                                vec![
                                    class_instance,
                                    field_name,
                                    field_type,
                                    JavaValue::Reference(Some(current_class)),
                                ],
                            )?
                        }
                        ReferenceKind::NewInvokeSpecial => {
                            let element = ClassElement::new("java/lang/invoke/MethodHandles$Lookup", "findConstructor", "(Ljava/lang/Class;Ljava/lang/invoke/MethodType;)Ljava/lang/invoke/MethodHandle;");
                            jvm.invoke_virtual(element, lookup, vec![class_instance, field_type])?
                        }
                        ReferenceKind::InvokeInterface => {
                            let element = ClassElement::new("java/lang/invoke/MethodHandles$Lookup", "findVirtual", "(Ljava/lang/Class;Ljava/lang/String;Ljava/lang/invoke/MethodType;)Ljava/lang/invoke/MethodHandle;");
                            jvm.invoke_virtual(
                                element,
                                lookup,
                                vec![class_instance, field_name, field_type],
                            )?
                        }
                    };

                    dyn_args.push(handle.unwrap());
                }
                Constant::Dynamic(x) => panic!("Not sure what to do: {:?}", x),
                x => panic!("Constant {:?} is not stack loadable", x),
            }
        }

        let (class_name, field_name, descriptor) = frame
            .constants
            .class_element_ref(bootstrap_method.bootstrap_method_ref);

        let method = ClassElement::new(class_name, field_name, descriptor);

        let call_site = match jvm.invoke_static(method, dyn_args)? {
            Some(JavaValue::Reference(Some(x))) => x,
            x => panic!("Expected callsite, received {:?}", x),
        };

        let get_target = ClassElement::new(
            "java/lang/invoke/CallSite",
            "getTarget",
            "()Ljava/lang/invoke/MethodHandle;",
        );
        let target = match jvm.invoke_virtual(get_target, call_site, Vec::new())? {
            Some(JavaValue::Reference(Some(x))) => x,
            x => panic!("Expected callsite target, received {:?}", x),
        };

        let invoke_target = ClassElement::new(
            "java/lang/invoke/MethodHandle",
            "invoke",
            "([Ljava/lang/Object;)Ljava/lang/Object;",
        );
        let target = match jvm.invoke_virtual(invoke_target, target, Vec::new())? {
            Some(JavaValue::Reference(Some(x))) => x,
            x => panic!("Expected object, received {:?}", x),
        };

        frame.stack.push(JavaValue::Reference(Some(target)));

        // match jvm.invoke_static(method, dyn_args) {
        //     Ok(Some(JavaValue::Long(v))) => {
        //         frame.stack.push(JavaValue::Long(v));
        //         frame.stack.push(JavaValue::Long(v));
        //     }
        //     Ok(Some(JavaValue::Double(v))) => {
        //         frame.stack.push(JavaValue::Double(v));
        //         frame.stack.push(JavaValue::Double(v));
        //     }
        //     Ok(Some(v)) => frame.stack.push(v),
        //     Ok(None) => {}
        //     Err(e) => return Err(e),
        // }

        Ok(())
    }
}
