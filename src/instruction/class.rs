use crate::constant_pool::Constant;
use crate::instruction::InstructionAction;
use crate::jvm::{LocalVariable, JVM, clean_str};
use std::rc::Rc;


instruction! {@partial getstatic, 0xb2, u16}

impl InstructionAction for getstatic {
    fn exec(&self, stack: &mut Vec<LocalVariable>, pool: &[Constant], jvm: &mut JVM) {
        let getstatic(field) = *self;

        if let Constant::FieldRef(reference) = &pool[field as usize - 1] {
            let class = pool[reference.class_index as usize - 1]
                .expect_class()
                .unwrap();
            let class_name = pool[class as usize - 1].expect_utf8().unwrap();
            jvm.init_class(&class_name);

            let field = pool[reference.name_and_type_index as usize - 1]
                .expect_name_and_type()
                .unwrap();
            let field_name = pool[field.name_index as usize - 1].expect_utf8().unwrap();
            let descriptor = pool[field.descriptor_index as usize - 1]
                .expect_utf8()
                .unwrap();

            let field_reference = format!("{}_{}", clean_str(&class_name), clean_str(&field_name));
            let value = jvm.static_fields.get(&field_reference).expect("Static value not found").clone();
            debug!("Got value {:?} from {}::{} {}", &value, &class_name, &field_name, descriptor);
            stack.push(value);
        } else {
            panic!("Error in getstatic");
        }
    }
}

instruction! {@partial invokestatic, 0xb8, u16}

impl InstructionAction for invokestatic {
    fn exec(&self, stack: &mut Vec<LocalVariable>, pool: &[Constant], jvm: &mut JVM) {
        let invokestatic(field) = *self;

        let (class_index, desc_index) = match &pool[field as usize - 1] {
            Constant::MethodRef(v) => (v.class_index, v.name_and_type_index),
            Constant::InterfaceMethodRef(v) => (v.class_index, v.name_and_type_index),
            _ => panic!(),
        };

        let class = pool[class_index as usize - 1].expect_class().unwrap();
        let class_name = pool[class as usize - 1].expect_utf8().unwrap();
        jvm.init_class(&class_name);

        let field = pool[desc_index as usize - 1]
            .expect_name_and_type()
            .unwrap();
        let field_name = pool[field.name_index as usize - 1].expect_utf8().unwrap();
        let descriptor = pool[field.descriptor_index as usize - 1]
            .expect_utf8()
            .unwrap();

        jvm.exec_static(&class_name, &field_name, &descriptor);
    }
}

instruction! {@partial putstatic, 0xb3, u16}

impl InstructionAction for putstatic {
    fn exec(&self, stack: &mut Vec<LocalVariable>, pool: &[Constant], jvm: &mut JVM) {
        let putstatic(field) = *self;

        let (class_index, desc_index) = match &pool[field as usize - 1] {
            Constant::FieldRef(v) => (v.class_index, v.name_and_type_index),
            x => panic!("Unexpected constant in putstatic: {:?}", x),
        };

        let class = pool[class_index as usize - 1].expect_class().unwrap();
        let class_name = pool[class as usize - 1].expect_utf8().unwrap();
        jvm.init_class(&class_name);

        let field = pool[desc_index as usize - 1]
            .expect_name_and_type()
            .unwrap();
        let field_name = pool[field.name_index as usize - 1].expect_utf8().unwrap();
        let descriptor = pool[field.descriptor_index as usize - 1]
            .expect_utf8()
            .unwrap();

        let value = stack.pop().expect("Unable to pop stack");
        debug!("Put value {:?} into {}::{} {}", &value, &class_name, &field_name, descriptor);
        let field_reference = format!("{}_{}", clean_str(&class_name), clean_str(&field_name));
        jvm.static_fields.insert(field_reference, value);
    }
}

instruction! {@partial invokevirtual, 0xb6, u16}

impl InstructionAction for invokevirtual {
    fn exec(&self, stack: &mut Vec<LocalVariable>, pool: &[Constant], jvm: &mut JVM) {
        let invokevirtual(field) = *self;

        let (class_index, desc_index) = match &pool[field as usize - 1] {
            Constant::MethodRef(v) => (v.class_index, v.name_and_type_index),
            Constant::InterfaceMethodRef(v) => (v.class_index, v.name_and_type_index),
            x => panic!("Unexpected constant in putstatic: {:?}", x),
        };

        let class = pool[class_index as usize - 1].expect_class().unwrap();
        let class_name = pool[class as usize - 1].expect_utf8().unwrap();
        jvm.init_class(&class_name);

        let field = pool[desc_index as usize - 1]
            .expect_name_and_type()
            .unwrap();
        let field_name = pool[field.name_index as usize - 1].expect_utf8().unwrap();
        let descriptor = pool[field.descriptor_index as usize - 1]
            .expect_utf8()
            .unwrap();

        let value = stack.pop().expect("Unable to pop stack");
        debug!("Put value {:?} into {}::{} {}", &value, &class_name, &field_name, descriptor);
        let field_reference = format!("{}_{}", clean_str(&class_name), clean_str(&field_name));
        jvm.static_fields.insert(field_reference, value);
    }
}

instruction! {@partial new, 0xbb, u16}


impl InstructionAction for new {
    fn exec(&self, stack: &mut Vec<LocalVariable>, pool: &[Constant], jvm: &mut JVM) {
        let new(field) = *self;
        let class = pool[field as usize - 1].expect_class().expect("Expected class from constant pool");
        let class_name = pool[class as usize - 1].expect_utf8().unwrap();

        jvm.init_class(&class_name);
        let object = jvm.class_loader.class(&class_name).unwrap().build_object();
        stack.push(LocalVariable::Reference(Some(Rc::new(object))));
        debug!("Pushed new instance of {} to the stack", class_name);
    }
}

//  - getfield
//  - putfield
//  - getstatic
//  - invokevirtual
