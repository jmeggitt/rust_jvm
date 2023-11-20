use crate::jvm::call::StackFrame;
use crate::jvm::mem::{FieldDescriptor, JavaValue};
use num_traits::PrimInt;

macro_rules! math_instruction {
    ($name:ident, $type:ident ($a:ident $(,$x:ident)*) => $res:expr) => {
        pub fn $name(frame: &mut StackFrame) {
            // jvm.read().debug_print_call_stack();
            math_instruction!(@impl $type frame ($($x,)* $a) => $res);
        }

        #[cfg(test)]
        struct $name;

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
math_instruction! {dadd, Double (x, y) => x + y}
math_instruction! {ddiv, Double (x, y) => x / y}
math_instruction! {dmul, Double (x, y) => x * y}
math_instruction! {dneg, Double (x) => -x} // FIXME: This isn't technically correct, but meh
math_instruction! {drem, Double (x, y) => x % y}
math_instruction! {dsub, Double (x, y) => x - y}
math_instruction! {fadd, Float (x, y) => x + y}
math_instruction! {fdiv, Float (x, y) => x / y}
math_instruction! {fmul, Float (x, y) => x * y}
math_instruction! {fneg, Float (x) => -x}
math_instruction! {frem, Float (x, y) => x % y}
math_instruction! {fsub, Float (x, y) => x - y}
math_instruction! {iadd, Int (x, y) => x.overflowing_add(y).0}
math_instruction! {iand, Int (x, y) => x & y}
math_instruction! {idiv, Int (x, y) => x / y}
math_instruction! {imul, Int (x, y) => x.overflowing_mul(y).0}
math_instruction! {ineg, Int (x) => -x}
math_instruction! {ior, Int (x, y) => x | y}
math_instruction! {irem, Int (x, y) =>  x % y}
math_instruction! {ishl, Int (x, y) => x.overflowing_shl(y as _).0}
math_instruction! {ishr, Int (x, y) => x.overflowing_shr(y as _).0}
math_instruction! {isub, Int (x, y) => x.overflowing_sub(y).0}
math_instruction! {iushr, Int (x, y) => x.unsigned_shr(y as _)} // FIXME: Does not handle cases where y < 0
math_instruction! {ixor, Int (x, y) => x ^ y}
math_instruction! {ladd, Long (x, y) => x + y}
math_instruction! {land, Long (x, y) => x & y}
math_instruction! {ldiv, Long (x, y) => x / y}
math_instruction! {lmul, Long (x, y) => x * y}
math_instruction! {lneg, Long (x) => -x}
math_instruction! {lor, Long (x, y) => x | y}
math_instruction! {lrem, Long (x, y) => x % y}
math_instruction! {lsub, Long (x, y) => x - y}
math_instruction! {lxor, Long (x, y) => x ^ y}

pub fn lshl(frame: &mut StackFrame) {
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
}

pub fn lshr(frame: &mut StackFrame) {
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
}

pub fn lushr(frame: &mut StackFrame) {
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
}
