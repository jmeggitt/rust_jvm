//! Instructions I have yet to implement, but can still be parsed


use crate::instruction::InstructionAction;
use crate::jvm::{JVM, LocalVariable};
use crate::constant_pool::{Constant, ConstantInteger, ConstantFloat, ConstantString, ConstantMethodRef};
instruction! {aaload, 0x32}
instruction! {aastore, 0x53}
instruction! {anewarray, 0xbd, u16}
instruction! {areturn, 0xb0}
instruction! {arraylength, 0xbe}
instruction! {athrow, 0xbf}
instruction! {baload, 0x33}
instruction! {bastore, 0x54}
instruction! {bipush, 0x10, u8}
instruction! {caload, 0x34}
instruction! {castore, 0x55}
instruction! {checkcast, 0xc0, u16}
instruction! {d2f, 0x90}
instruction! {d2i, 0x8e}
instruction! {d2l, 0x8f}
instruction! {dadd, 0x63}
instruction! {daload, 0x31}
instruction! {dastore, 0x52}
instruction! {dcmpg, 0x98}
instruction! {dcmpl, 0x97}
instruction! {ddiv, 0x6f}
instruction! {dmul, 0x6b}
instruction! {dneg, 0x77}
instruction! {drem, 0x73}
instruction! {dreturn, 0xaf}
instruction! {dsub, 0x67}
instruction! {f2d, 0x8d}
instruction! {f2i, 0x8b}
instruction! {f2l, 0x8c}
instruction! {fadd, 0x62}
instruction! {faload, 0x30}
instruction! {fastore, 0x51}
instruction! {fcmpg, 0x96}
instruction! {fcmpl, 0x95}
instruction! {fdiv, 0x6e}
instruction! {fmul, 0x6a}
instruction! {fneg, 0x76}
instruction! {frem, 0x72}
instruction! {freturn, 0xae}
instruction! {fsub, 0x66}
instruction! {getfield, 0xb4, u16}
instruction! {goto, 0xa7, u16}
// TODO: goto_w
instruction! {i2b, 0x91}
instruction! {i2c, 0x92}
instruction! {i2d, 0x87}
instruction! {i2f, 0x86}
instruction! {i2l, 0x85}
instruction! {i2s, 0x93}
instruction! {iadd, 0x60}
instruction! {iaload, 0x2e}
instruction! {iand, 0x7e}
instruction! {iastore, 0x4f}
instruction! {idiv, 0x6c}
instruction! {if_acmpeq, 0xa5}
instruction! {if_acmpne, 0xa6}
instruction! {if_icmpeq, 0x9f}
instruction! {if_icmpne, 0xa0}
instruction! {if_icmplt, 0xa1}
instruction! {if_icmpge, 0xa2}
instruction! {if_icmpgt, 0xa3}
instruction! {if_icmple, 0xa4}
instruction! {ifeq, 0x99}
instruction! {ifne, 0x9a}
instruction! {iflt, 0x9b}
instruction! {ifge, 0x9c}
instruction! {ifgt, 0x9d}
instruction! {ifle, 0x9e}
instruction! {ifnonnull, 0xc7, u16}
instruction! {ifnull, 0xc6, u16}
// TODO: iinc, 0x84, u8, u8
instruction! {imul, 0x68}
instruction! {ineg, 0x74}
instruction! {instanceof, 0xc1, u16}
// TODO: invokedynamic
// TODO: invokeinterface
instruction! {invokespecial, 0xb7, u16}
instruction! {ior, 0x80}
instruction! {irem, 0x70}
instruction! {ireturn, 0xac}
instruction! {ishl, 0x78}
instruction! {ishr, 0x7a}
instruction! {isub, 0x64}
instruction! {iushr, 0x7c}
instruction! {ixor, 0x82}
instruction! {jsr, 0xa8, u16}
// TODO: jsr_w
instruction! {l2d, 0x8a}
instruction! {l2f, 0x89}
instruction! {l2i, 0x88}
instruction! {ladd, 0x61}
instruction! {laload, 0x2f}
instruction! {land, 0x7f}
instruction! {lastore, 0x50}
instruction! {lcmp, 0x94}
instruction! {ldc_w, 0x13, u16}
instruction! {ldc2_w, 0x14, u16}
instruction! {ldiv, 0x6d}
instruction! {lmul, 0x69}
instruction! {lneg, 0x75}
// TODO: lookupswitch
instruction! {lor, 0x81}
instruction! {lrem, 0x71}
instruction! {lreturn, 0xad}
instruction! {lshl, 0x79}
instruction! {lshr, 0x7b}
instruction! {lsub, 0x65}
instruction! {lushr, 0x7d}
instruction! {lxor, 0x83}
instruction! {monitorenter, 0xc2}
instruction! {monitorexit, 0xc3}
// TODO: multianewarray
instruction! {newarray, 0xbc, u8}
instruction! {nop, 0x0}
instruction! {pop, 0x57}
instruction! {pop2, 0x58}
instruction! {putfield, 0xb5, u16}
instruction! {ret, 0xa9, u8}
instruction! {r#return, 0xb1}
instruction! {saload, 0x35}
instruction! {sastore, 0x56}
instruction! {sipush, 0x11, u16}
instruction! {swap, 0x5f}
// TODO: tableswitch
// TODO: wide


instruction! {@partial ldc, 0x12, u8}

impl InstructionAction for ldc {
    fn exec(&self, stack: &mut Vec<LocalVariable>, pool: &[Constant], jvm: &mut JVM) {
        let ldc(index) = *self;

        stack.push(match &pool[index as usize] {
            Constant::Int(ConstantInteger {value}) => LocalVariable::Int(*value),
            Constant::Float(ConstantFloat{value}) => LocalVariable::Float(*value),
            Constant::String(ConstantString{string_index}) => {
                todo!("Implement ldc for strings")
            },
            Constant::MethodRef(ConstantMethodRef { class_index, name_and_type_index }) => {
                let class = pool[*class_index as usize - 1].expect_class().unwrap();
                let class_name = pool[class as usize - 1].expect_utf8().unwrap();
                jvm.init_class(&class_name);

                let field = pool[*name_and_type_index as usize - 1]
                    .expect_name_and_type()
                    .unwrap();
                let field_name = pool[field.name_index as usize - 1].expect_utf8().unwrap();
                let descriptor = pool[field.descriptor_index as usize - 1]
                    .expect_utf8()
                    .unwrap();


                info!("Attempted to load constant to stack: {}::{} {}", class_name, field_name, descriptor);
                return
            }
            x => panic!("Attempted to push {:?} to the stack", x),
        });
    }
}

