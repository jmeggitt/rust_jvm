//! This module is for helper functions for getting data about instructions and their usages.

use crate::instruction::Instruction;
use crate::instruction::Instruction::*;

/// Helper functions
impl Instruction {
    /// Get a static slice of all runtime exception classes a given instruction can produce when
    /// executed.
    pub fn runtime_exceptions(&self) -> &'static [&'static str] {
        match self {
            aaload | baload | bastore | caload | castore | daload | dastore | faload | fastore
            | iaload | iastore | laload | lastore | saload | sastore => &[
                "java/lang/NullPointerException",
                "java/lang/ArrayIndexOutOfBoundsException",
            ],
            aastore => &[
                "java/lang/NullPointerException",
                "java/lang/ArrayIndexOutOfBoundsException",
                "java/lang/ArrayStoreException",
            ],
            anewarray(_) | multianewarray { .. } | newarray(_) => {
                &["java/lang/NegativeArraySizeException"]
            }
            areturn | dreturn | freturn | ireturn | lreturn | r#return => {
                &["java/lang/IllegalMonitorStateException"]
            }
            arraylength | getfield(_) | monitorenter | putfield(_) => {
                &["java/lang/NullPointerException"]
            }
            monitorexit => &[
                "java/lang/NullPointerException",
                "java/lang/IllegalMonitorStateException",
            ],
            athrow => &["java/lang/Object"],
            checkcast(_) => &["java/lang/ClassCastException"],
            // Error as described in §5.5
            getstatic(_) | putstatic(_) => &["java/lang/Error"],
            // Error as described in JLS §15.9.4
            new(_) => &["java/lang/Error"],
            idiv | irem | ldiv | lrem => &["java/lang/ArithmeticException"],
            invokedynamic(_)
            | invokeinterface { .. }
            | invokespecial(_)
            | invokestatic(_)
            | invokevirtual(_) => &["java/lang/Object"],
            _ => &[],
        }
    }

    /// Get the mnemonic for a given instruction
    pub fn mnemonic(&self) -> &'static str {
        match self {
            aaload => "aaload",
            aastore => "aastore",
            aconst_null => "aconst_null",
            aload(_) => "aload",
            anewarray(_) => "anewarray",
            areturn => "areturn",
            arraylength => "arraylength",
            astore(_) => "astore",
            athrow => "athrow",
            baload => "baload",
            bastore => "bastore",
            bipush(_) => "bipush",
            caload => "caload",
            castore => "castore",
            checkcast(_) => "checkcast",
            d2f => "d2f",
            d2i => "d2i",
            d2l => "d2l",
            dadd => "dadd",
            daload => "daload",
            dastore => "dastore",
            dcmpg => "dcmpg",
            dcmpl => "dcmpl",
            dconst_0 => "dconst_0",
            dconst_1 => "dconst_1",
            ddiv => "ddiv",
            dload(_) => "dload",
            dmul => "dmul",
            dneg => "dneg",
            drem => "drem",
            dreturn => "dreturn",
            dstore(_) => "dstore",
            dsub => "dsub",
            dup => "dup",
            dup_x1 => "dup_x1",
            dup_x2 => "dup_x2",
            dup2 => "dup2",
            dup2_x1 => "dup2_x1",
            dup2_x2 => "dup2_x2",
            f2d => "f2d",
            f2i => "f2i",
            f2l => "f2l",
            fadd => "fadd",
            faload => "faload",
            fastore => "fastore",
            fcmpg => "fcmpg",
            fcmpl => "fcmpl",
            fconst_0 => "fconst_0",
            fconst_1 => "fconst_1",
            fconst_2 => "fconst_2",
            fdiv => "fdiv",
            fload(_) => "fload",
            fmul => "fmul",
            fneg => "fneg",
            frem => "frem",
            freturn => "freturn",
            fstore(_) => "fstore",
            fsub => "fsub",
            getfield(_) => "getfield",
            getstatic(_) => "getstatic",
            goto(_) => "goto",
            goto_w(_) => "goto_w",
            i2b => "i2b",
            i2c => "i2c",
            i2d => "i2d",
            i2f => "i2f",
            i2l => "i2l",
            i2s => "i2s",
            iadd => "iadd",
            iaload => "iaload",
            iand => "iand",
            iastore => "iastore",
            iconst_m1 => "iconst_m1",
            iconst_0 => "iconst_0",
            iconst_1 => "iconst_1",
            iconst_2 => "iconst_2",
            iconst_3 => "iconst_3",
            iconst_4 => "iconst_4",
            iconst_5 => "iconst_5",
            idiv => "idiv",
            if_acmpeq(_) => "if_acmpeq",
            if_acmpne(_) => "if_acmpne",
            if_icmpeq(_) => "if_icmpeq",
            if_icmpne(_) => "if_icmpne",
            if_icmplt(_) => "if_icmplt",
            if_icmpge(_) => "if_icmpge",
            if_icmpgt(_) => "if_icmpgt",
            if_icmple(_) => "if_icmple",
            ifeq(_) => "ifeq",
            ifne(_) => "ifne",
            iflt(_) => "iflt",
            ifge(_) => "ifge",
            ifgt(_) => "ifgt",
            ifle(_) => "ifle",
            ifnonnull(_) => "ifnonnull",
            ifnull(_) => "ifnull",
            iload(_) => "iload",
            imul => "imul",
            ineg => "ineg",
            instanceof(_) => "instanceof",
            invokespecial(_) => "invokespecial",
            invokestatic(_) => "invokestatic",
            invokevirtual(_) => "invokevirtual",
            ior => "ior",
            irem => "irem",
            ireturn => "ireturn",
            ishl => "ishl",
            ishr => "ishr",
            istore(_) => "istore",
            isub => "isub",
            iushr => "iushr",
            ixor => "ixor",
            jsr(_) => "jsr",
            jsr_w(_) => "jsr_w",
            l2d => "l2d",
            l2f => "l2f",
            l2i => "l2i",
            ladd => "ladd",
            laload => "laload",
            land => "land",
            lastore => "lastore",
            lcmp => "lcmp",
            lconst_0 => "lconst_0",
            lconst_1 => "lconst_1",
            ldc(_) => "ldc",
            ldc_w(_) => "ldc_w",
            ldc2_w(_) => "ldc2_w",
            ldiv => "ldiv",
            lload(_) => "lload",
            lmul => "lmul",
            lneg => "lneg",
            lor => "lor",
            lrem => "lrem",
            lreturn => "lreturn",
            lshl => "lshl",
            lshr => "lshr",
            lstore(_) => "lstore",
            lsub => "lsub",
            lushr => "lushr",
            lxor => "lxor",
            monitorenter => "monitorenter",
            monitorexit => "monitorexit",
            multianewarray { .. } => "multianewarray",
            new(_) => "new",
            newarray(_) => "newarray",
            nop => "nop",
            pop => "pop",
            pop2 => "pop2",
            putfield(_) => "putfield",
            putstatic(_) => "putstatic",
            ret(_) => "ret",
            r#return => "return",
            saload => "saload",
            sastore => "sastore",
            sipush(_) => "sipush",
            swap => "swap",
            invokeinterface { .. } => "invokeinterface",
            iinc { .. } => "iinc",
            lookupswitch { .. } => "lookupswitch",
            tableswitch { .. } => "tableswitch",
            invokedynamic(_) => "invokedynamic",
        }
    }
}

/// The category of an instruction as defined by chapter 7 of the JVM specification
pub enum InstructionCategory {
    Constants,
    Loads,
    Stores,
    Stack,
    Math,
    Conversions,
    Comparisons,
    Control,
    References,
    Extended,
    Reserved,
}

impl Instruction {
    pub fn category(&self) -> InstructionCategory {
        todo!()
    }
}
