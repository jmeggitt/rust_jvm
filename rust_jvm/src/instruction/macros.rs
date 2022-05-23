macro_rules! instruction {
    ($name:ident) => {
        #[derive(Debug, Copy, Clone)]
        pub struct $name;
    };
    ($name:ident, $arg:ty) => {
        #[derive(Debug, Copy, Clone)]
        pub struct $name(pub $arg);
    };
    ($name:ident, $inst:literal) => {
        instruction!($name);
        instruction!(@write $name, self, buffer, {byteorder::WriteBytesExt::write_u8(buffer, <Self as crate::instruction::StaticInstruct>::FORM)});

        impl crate::instruction::StaticInstruct for $name {
            const FORM: u8 = $inst;

            fn read(_: u8, _: &mut std::io::Cursor<Vec<u8>>) -> std::io::Result<Box<dyn crate::instruction::Instruction>> {
                Ok(Box::new($name))
            }
        }
    };
    ($name:ident, $inst:literal, u16) => {
        instruction!($name, u16);
        instruction!(@write $name, self, buffer, {
            use byteorder::WriteBytesExt;
            buffer.write_u8(<Self as crate::instruction::StaticInstruct>::FORM)?;
            buffer.write_u16::<byteorder::BigEndian>(self.0)
        });

        impl crate::instruction::StaticInstruct for $name {
            const FORM: u8 = $inst;

            fn read(_: u8, buffer: &mut std::io::Cursor<Vec<u8>>) -> std::io::Result<Box<dyn crate::instruction::Instruction>> {
                use byteorder::ReadBytesExt;
                Ok(Box::new($name(buffer.read_u16::<byteorder::BigEndian>()?)))
            }
        }
    };
    ($name:ident, $inst:literal, i16) => {
        instruction!($name, i16);
        instruction!(@write $name, self, buffer, {
            use byteorder::WriteBytesExt;
            buffer.write_u8(<Self as crate::instruction::StaticInstruct>::FORM)?;
            buffer.write_i16::<byteorder::BigEndian>(self.0)
        });

        impl crate::instruction::StaticInstruct for $name {
            const FORM: u8 = $inst;

            fn read(_: u8, buffer: &mut std::io::Cursor<Vec<u8>>) -> std::io::Result<Box<dyn crate::instruction::Instruction>> {
                use byteorder::ReadBytesExt;
                Ok(Box::new($name(buffer.read_i16::<byteorder::BigEndian>()?)))
            }
        }
    };
    ($name:ident, $inst:literal, u8) => {
        instruction!($name, u8);
        instruction!(@write $name, self, buffer, {
            use byteorder::WriteBytesExt;
            buffer.write_u8(<Self as crate::instruction::StaticInstruct>::FORM)?;
            buffer.write_u8(self.0)
        });

        impl crate::instruction::StaticInstruct for $name {
            const FORM: u8 = $inst;

            fn read(_: u8, buffer: &mut std::io::Cursor<Vec<u8>>) -> std::io::Result<Box<dyn crate::instruction::Instruction>> {
                use byteorder::ReadBytesExt;
                Ok(Box::new($name(buffer.read_u8()?)))
            }
        }
    };
    ($name:ident, $inst:literal, u8, $start:literal <-> $stop:literal) => {
        instruction!($name, u8);
        instruction!(@write $name, self, buffer, {
            use byteorder::WriteBytesExt;
            use std::io::Write;
            if self.0 <= $stop - $start {
                buffer.write_u8($start + self.0)
            } else {
                buffer.write_all(&[$inst, self.0])?;
                Ok(())
            }
        });

        impl crate::instruction::StaticInstruct for $name {
            const FORM: u8 = $inst;
            const STRIDE: Option<std::ops::RangeInclusive<u8>> = Some($start..=$stop);

            fn read(form: u8, buffer: &mut std::io::Cursor<Vec<u8>>) -> std::io::Result<Box<dyn crate::instruction::Instruction>> {
                use byteorder::ReadBytesExt;
                Ok(Box::new(match form {
                    $inst => $name(buffer.read_u8()?),
                    x => $name(x - $start),
                }))
            }
        }
    };
    (@write $name:ident, $self:ident, $buffer:ident, $x:block) => {
        impl crate::instruction::Instruction for $name {
            fn write(&$self, $buffer: &mut std::io::Cursor<Vec<u8>>) -> std::io::Result<()> { $x }
            fn exec(&self, stack: &mut crate::jvm::call::StackFrame, jvm: &mut std::sync::Arc<parking_lot::RwLock<crate::jvm::JavaEnv>>) -> Result<(), crate::jvm::call::FlowControl> {
                <Self as crate::instruction::InstructionAction>::exec(self, stack, jvm)
            }
        }
    };
}

macro_rules! llvm_instr_todo {
    ($($self:ty),*) => {
        $(#[cfg(feature = "llvm")]
        impl crate::instruction::LLVMInstruction for $self {
            unsafe fn add_impl(&self, _builder: llvm_sys::prelude::LLVMBuilderRef, _cxt: &mut crate::class::llvm::FunctionContext) {
                unimplemented!("LLVM codegen is not implemented for {:?}", self)
            }
        })*
    };
}
