use num_traits::PrimInt;

use crate::instruction::InstructionAction;
use crate::jvm::{LocalVariable, StackFrame, JVM};

macro_rules! math_instruction {
    ($name:ident, $inst:literal, $type:ident ($a:ident $(,$x:ident)*) => $res:expr) => {
        instruction! {@partial $name, $inst}

        impl InstructionAction for $name {
            fn exec(&self, frame: &mut StackFrame, _: &mut JVM) {
                math_instruction!(@impl $type frame ($($x,)* $a) => $res);
            }
        }
    };
    (@impl Long $frame:ident ($($x:ident),+) => $res:expr) => {
        $(math_instruction!(@pop_int $frame $x -> i64);)+
        $frame.stack.push(LocalVariable::Long($res));
    };
    (@impl Int $frame:ident ($($x:ident),+) => $res:expr) => {
        $(math_instruction!(@pop_int $frame $x -> i32);)+
        $frame.stack.push(LocalVariable::Int($res));
    };
    (@impl Double $frame:ident ($($x:ident),+) => $res:expr) => {
        $(math_instruction!(@pop_float $frame $x -> f64);)+
        $frame.stack.push(LocalVariable::Double($res));
    };
    (@impl Float $frame:ident ($($x:ident),+) => $res:expr) => {
        $(math_instruction!(@pop_float $frame $x -> f32);)+
        $frame.stack.push(LocalVariable::Float($res));
    };
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
math_instruction! {iadd, 0x60, Int (x, y) => x + y}
math_instruction! {iand, 0x7e, Int (x, y) => x & y}
math_instruction! {idiv, 0x6c, Int (x, y) => x / y}
math_instruction! {imul, 0x68, Int (x, y) => x * y}
math_instruction! {ineg, 0x74, Int (x) => -x}
math_instruction! {ior, 0x80, Int (x, y) => x | y}
math_instruction! {irem, 0x70, Int (x, y) =>  x % y}
math_instruction! {ishl, 0x78, Int (x, y) => x.overflowing_shl(y as _).0}
math_instruction! {ishr, 0x7a, Int (x, y) => x.overflowing_shr(y as _).0}
math_instruction! {isub, 0x64, Int (x, y) => x - y}
math_instruction! {iushr, 0x7c, Int (x, y) => x.unsigned_shr(y as _)} // FIXME: Does not handle cases where y < 0
math_instruction! {ixor, 0x82, Int (x, y) => x ^ y}
math_instruction! {ladd, 0x61, Long (x, y) => x + y}
math_instruction! {land, 0x7f, Long (x, y) => x & y}
math_instruction! {ldiv, 0x6d, Long (x, y) => x / y}
math_instruction! {lmul, 0x69, Long (x, y) => x * y}
math_instruction! {lneg, 0x75, Long (x) => -x}
math_instruction! {lor, 0x81, Long (x, y) => x | y}
math_instruction! {lrem, 0x71, Long (x, y) => x % y}
math_instruction! {lshl, 0x79, Long (x, y) => x.overflowing_shl(y as _).0}
math_instruction! {lshr, 0x7b, Long (x, y) => x.overflowing_shr(y as _).0}
math_instruction! {lsub, 0x65, Long (x, y) => x - y}
math_instruction! {lushr, 0x7d, Long (x, y) => x.unsigned_shr(y as _)}
math_instruction! {lxor, 0x83, Long (x, y) => x ^ y}
