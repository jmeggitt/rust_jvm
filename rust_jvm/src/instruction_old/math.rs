use num_traits::PrimInt;
use parking_lot::RwLock;
use std::sync::Arc;

use crate::instruction::InstructionAction;
use crate::jvm::call::{FlowControl, StackFrame};
use crate::jvm::mem::{FieldDescriptor, JavaValue};
use crate::jvm::JavaEnv;

macro_rules! math_instruction {
    ($name:ident, $inst:literal, $type:ident ($a:ident $(,$x:ident)*) => $res:expr) => {
        instruction! {$name, $inst}

        impl InstructionAction for $name {
            fn exec(&self, frame: &mut StackFrame, _jvm: &mut Arc<RwLock<JavaEnv>>) -> Result<(), FlowControl> {
                // jvm.read().debug_print_call_stack();
                math_instruction!(@impl $type frame ($($x,)* $a) => $res);
                Ok(())
            }
        }

        #[cfg(test)]
        impl $name {
            // math_instruction!{@type $type OpType}
            /// Function stub to allow for testing
            pub fn oper($a: math_instruction!{@type $type} $(,$x: math_instruction!{@type $type})*) -> math_instruction!{@type $type} {
                $res
            }
        }
    };
    (@impl Long $frame:ident ($($x:ident),+) => $res:expr) => {
        $(
            $frame.stack.pop().unwrap();
            math_instruction!(@pop_int $frame $x -> i64);
        )+
        $frame.stack.push(JavaValue::Long($res));
        $frame.stack.push(JavaValue::Long($res));
    };
    (@impl Int $frame:ident ($($x:ident),+) => $res:expr) => {
        // $frame.debug_print();
        $(math_instruction!(@pop_int $frame $x -> i32);)+
        $frame.stack.push(JavaValue::Int($res));
    };
    (@impl Double $frame:ident ($($x:ident),+) => $res:expr) => {
        $(
            $frame.stack.pop().unwrap();
            math_instruction!(@pop_float $frame $x -> f64);
        )+
        $frame.stack.push(JavaValue::Double($res));
        $frame.stack.push(JavaValue::Double($res));
    };
    (@impl Float $frame:ident ($($x:ident),+) => $res:expr) => {
        $(math_instruction!(@pop_float $frame $x -> f32);)+
        $frame.stack.push(JavaValue::Float($res));
    };
    (@type Long) => { jlong };
    (@type Int) => { jint };
    (@type Double) => { jdouble };
    (@type Float) => { jfloat };
    (@pop_int $frame:ident $x:ident -> $type:ty) => {
        let a = $frame.stack.pop().unwrap();
        let $x = match a.as_int() {
            Some(x) => x as $type,
            _ => panic!("Unable to convert {:?} to int for math operations", a),
        };
    };
    (@pop_float $frame:ident $x:ident -> $type:ty) => {
        let a = $frame.stack.pop().unwrap();
        let $x = match a.as_float() {
            Some(x) => x as $type,
            _ => panic!("Unable to convert {:?} to int for math operations", a),
        };
    };
}

// TODO: flip x and y to match stack pop order
// TODO: I don't perform set conversion described in 2.8.3 of the specification
math_instruction! {dadd, 0x63, Double (x, y) => x + y}
math_instruction! {ddiv, 0x6f, Double (x, y) => x / y}
math_instruction! {dmul, 0x6b, Double (x, y) => x * y}
math_instruction! {dneg, 0x77, Double (x) => -x} // FIXME: This isn't technically correct, but meh
math_instruction! {drem, 0x73, Double (x, y) => x % y}
math_instruction! {dsub, 0x67, Double (x, y) => x - y}
math_instruction! {fadd, 0x62, Float (x, y) => x + y}
math_instruction! {fdiv, 0x6e, Float (x, y) => x / y}
math_instruction! {fmul, 0x6a, Float (x, y) => x * y}
math_instruction! {fneg, 0x76, Float (x) => -x}
math_instruction! {frem, 0x72, Float (x, y) => x % y}
math_instruction! {fsub, 0x66, Float (x, y) => x - y}
math_instruction! {iadd, 0x60, Int (x, y) => x.overflowing_add(y).0}
math_instruction! {iand, 0x7e, Int (x, y) => x & y}
math_instruction! {idiv, 0x6c, Int (x, y) => x / y}
math_instruction! {imul, 0x68, Int (x, y) => x.overflowing_mul(y).0}
math_instruction! {ineg, 0x74, Int (x) => -x}
math_instruction! {ior, 0x80, Int (x, y) => x | y}
math_instruction! {irem, 0x70, Int (x, y) =>  x % y}
math_instruction! {ishl, 0x78, Int (x, y) => x.overflowing_shl(y as _).0}
math_instruction! {ishr, 0x7a, Int (x, y) => x.overflowing_shr(y as _).0}
math_instruction! {isub, 0x64, Int (x, y) => x.overflowing_sub(y).0}
math_instruction! {iushr, 0x7c, Int (x, y) => x.unsigned_shr(y as _)} // FIXME: Does not handle cases where y < 0
math_instruction! {ixor, 0x82, Int (x, y) => x ^ y}
math_instruction! {ladd, 0x61, Long (x, y) => x + y}
math_instruction! {land, 0x7f, Long (x, y) => x & y}
math_instruction! {ldiv, 0x6d, Long (x, y) => x / y}
math_instruction! {lmul, 0x69, Long (x, y) => x * y}
math_instruction! {lneg, 0x75, Long (x) => -x}
math_instruction! {lor, 0x81, Long (x, y) => x | y}
math_instruction! {lrem, 0x71, Long (x, y) => x % y}
// math_instruction! {lshl, 0x79, Long (x, y) => x.overflowing_shl(y as _).0}
// math_instruction! {lshr, 0x7b, Long (x, y) => x.overflowing_shr(y as _).0}
math_instruction! {lsub, 0x65, Long (x, y) => x - y}
// math_instruction! {lushr, 0x7d, Long (x, y) => x.unsigned_shr(y as _)}
math_instruction! {lxor, 0x83, Long (x, y) => x ^ y}

instruction! {lshl, 0x79}
instruction! {lshr, 0x7b}
instruction! {lushr, 0x7d}

impl InstructionAction for lshl {
    fn exec(
        &self,
        frame: &mut StackFrame,
        _: &mut Arc<RwLock<JavaEnv>>,
    ) -> Result<(), FlowControl> {
        let value2 = frame.stack.pop().unwrap();
        frame.stack.pop().unwrap();
        let value1 = frame.stack.pop().unwrap();

        if let (JavaValue::Long(x), Some(JavaValue::Int(shift))) =
            (value1, FieldDescriptor::Int.assign_from(value2))
        {
            frame
                .stack
                .push(JavaValue::Long(x.overflowing_shl(shift as _).0));
            frame
                .stack
                .push(JavaValue::Long(x.overflowing_shl(shift as _).0));
        } else {
            panic!("Expected Long with Int shift for lsh")
        }

        Ok(())
    }
}

impl InstructionAction for lshr {
    fn exec(
        &self,
        frame: &mut StackFrame,
        _: &mut Arc<RwLock<JavaEnv>>,
    ) -> Result<(), FlowControl> {
        let value2 = frame.stack.pop().unwrap();
        frame.stack.pop().unwrap();
        let value1 = frame.stack.pop().unwrap();

        if let (JavaValue::Long(x), Some(JavaValue::Int(shift))) =
            (value1, FieldDescriptor::Int.assign_from(value2))
        {
            frame
                .stack
                .push(JavaValue::Long(x.overflowing_shr(shift as _).0));
            frame
                .stack
                .push(JavaValue::Long(x.overflowing_shr(shift as _).0));
        } else {
            panic!("Expected Long with Int shift for lsh")
        }

        Ok(())
    }
}

impl InstructionAction for lushr {
    fn exec(
        &self,
        frame: &mut StackFrame,
        _: &mut Arc<RwLock<JavaEnv>>,
    ) -> Result<(), FlowControl> {
        let value2 = frame.stack.pop().unwrap();
        frame.stack.pop().unwrap();
        let value1 = frame.stack.pop().unwrap();

        if let (JavaValue::Long(x), Some(JavaValue::Int(shift))) =
            (value1, FieldDescriptor::Int.assign_from(value2))
        {
            frame
                .stack
                .push(JavaValue::Long(x.unsigned_shr(shift as _)));
            frame
                .stack
                .push(JavaValue::Long(x.unsigned_shr(shift as _)));
        } else {
            panic!("Expected Long with Int shift for lushr")
        }

        Ok(())
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::constant_pool::ClassElement;
    use crate::jvm::call::JavaEnvInvoke;
    use crate::r#mod::{ClassLoader, ClassPath};

    #[test]
    pub fn int_mul() {
        assert_eq!(imul::oper(47483647i32, 8752i32), -1034949168i32);
        assert_eq!(imul::oper(8752i32, 47483647i32), -1034949168i32);
    }

    #[test]
    pub fn int_remainder() {
        assert_eq!(irem::oper(47483647i32, 8752i32), 4047i32);
        assert_eq!(irem::oper(-47483647i32, 8752i32), -4047i32);
        assert_eq!(irem::oper(47483647i32, -8752i32), 4047i32);
        assert_eq!(irem::oper(-47483647i32, -8752i32), -4047i32);

        assert_eq!(irem::oper(8752i32, 47483647i32), 8752i32);
        assert_eq!(irem::oper(8752i32, -47483647i32), 8752i32);
        assert_eq!(irem::oper(-8752i32, 47483647i32), -8752i32);
        assert_eq!(irem::oper(-8752i32, -47483647i32), -8752i32);
    }

    #[test]
    pub fn shift_right() {
        assert_eq!(ishr::oper(-12345, 0), -12345);
        assert_eq!(ishr::oper(-12345, 3), -1544);
        assert_eq!(ishr::oper(-12345, -3), -1);
        assert_eq!(ishr::oper(12345, 3), 1543);
        assert_eq!(ishr::oper(12345, -3), 0);
        assert_eq!(ishr::oper(-1, 4), -1);
    }

    #[test]
    pub fn shift_spot_test() {
        assert_eq!(iand::oper(ishr::oper(111607186, 0), 1023), 402);
        assert_eq!(iand::oper(ishr::oper(111607186, 1), 31), 9);
    }

    #[test]
    pub fn string_hash() {
        let class_path = ClassPath::new(None, Some(Vec::new())).unwrap();
        let mut class_loader = ClassLoader::from_class_path(class_path);
        class_loader.preload_class_path().unwrap();
        let mut jvm = JavaEnv::new(class_loader);

        let element = ClassElement::new("java/lang/Object", "hashCode", "()I");

        let empty = jvm.write().build_string("").expect_object();
        let simple = jvm.write().build_string("abc").expect_object();
        let simple2 = jvm.write().build_string("utf-8").expect_object();
        let longer = jvm
            .write()
            .build_string("qwertyuiopasdfghjklzxcvbnm")
            .expect_object();

        assert_eq!(
            jvm.invoke_virtual(element.clone(), empty, vec![]).unwrap(),
            Some(JavaValue::Int(0))
        );
        assert_eq!(
            jvm.invoke_virtual(element.clone(), simple, vec![]).unwrap(),
            Some(JavaValue::Int(96354))
        );
        assert_eq!(
            jvm.invoke_virtual(element.clone(), simple2, vec![])
                .unwrap(),
            Some(JavaValue::Int(111607186))
        );
        assert_eq!(
            jvm.invoke_virtual(element.clone(), longer, vec![]).unwrap(),
            Some(JavaValue::Int(144599175))
        );
    }
}
