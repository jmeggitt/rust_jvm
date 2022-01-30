use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::fmt::Debug;
use std::pin::Pin;
use std::ptr::{null, null_mut};
use libc::{c_char, c_int};
use llvm_sys::core::{LLVMAddFunction, LLVMAppendBasicBlock, LLVMArrayType, LLVMBuildAlloca, LLVMBuildBr, LLVMBuildFPExt, LLVMBuildLoad, LLVMBuildStore, LLVMCountParams, LLVMCreateBuilder, LLVMCreatePassManager, LLVMDisposeMessage, LLVMDisposePassManager, LLVMDoubleType, LLVMFloatType, LLVMFunctionType, LLVMGetArgOperand, LLVMGetModuleContext, LLVMGetNumArgOperands, LLVMGetOrInsertNamedMetadata, LLVMGetParam, LLVMGetTarget, LLVMInt16Type, LLVMInt32Type, LLVMInt64Type, LLVMInt8Type, LLVMIntType, LLVMModuleCreateWithName, LLVMPointerType, LLVMPositionBuilderAtEnd, LLVMPrintModuleToString, LLVMPrintValueToString, LLVMRunPassManager, LLVMSetTarget, LLVMStructCreateNamed, LLVMStructSetBody, LLVMVoidType};
use llvm_sys::{LLVMContext, LLVMModule};
use llvm_sys::execution_engine::{LLVMCreateGenericValueOfFloat, LLVMCreateGenericValueOfInt, LLVMCreateGenericValueOfPointer, LLVMGenericValueRef};
use llvm_sys::prelude::{LLVMBasicBlockRef, LLVMBool, LLVMBuilderRef, LLVMContextRef, LLVMTypeRef, LLVMValueRef};
use llvm_sys::target::{LLVM_InitializeAllAsmParsers, LLVM_InitializeAllAsmPrinters, LLVM_InitializeAllTargetInfos, LLVM_InitializeAllTargetMCs, LLVM_InitializeAllTargets};
use llvm_sys::target_machine::{LLVMCodeGenOptLevel, LLVMCodeModel, LLVMCreateTargetMachine, LLVMDisposeTargetMachine, LLVMGetDefaultTargetTriple, LLVMGetTargetFromTriple, LLVMRelocMode, LLVMTargetMachineRef};
use llvm_sys::transforms::pass_manager_builder::{LLVMPassManagerBuilderCreate, LLVMPassManagerBuilderDispose, LLVMPassManagerBuilderPopulateModulePassManager, LLVMPassManagerBuilderSetOptLevel};
use crate::class::{AccessFlags, BufferedRead, ClassLoader};
use crate::jvm::call::{clean_str, NativeManager};
use crate::jvm::mem::FieldDescriptor;
use crate::c_str;




pub unsafe fn llvm_target() -> CString {
    let target_ptr = LLVMGetDefaultTargetTriple();
    let ret = CStr::from_ptr(target_ptr as _).to_owned();
    LLVMDisposeMessage(target_ptr);
    ret
}

pub fn debug_value_ref(llvm: LLVMValueRef) -> String {
    unsafe {
        let msg = LLVMPrintValueToString(llvm);
        let ret = CStr::from_ptr(msg).to_string_lossy().to_string();
        LLVMDisposeMessage(msg);
        ret
    }
}

pub fn llvm_print<T>(func: unsafe extern "C" fn(T) -> *mut c_char, val: T) -> String {
    unsafe {
        let msg = func(val);
        let ret = CStr::from_ptr(msg).to_string_lossy().to_string();
        LLVMDisposeMessage(msg);
        ret
    }
}

pub unsafe fn init_llvm() {
    LLVM_InitializeAllTargetInfos();
    LLVM_InitializeAllTargets();
    LLVM_InitializeAllTargetMCs();
    LLVM_InitializeAllAsmParsers();
    LLVM_InitializeAllAsmPrinters();
}

#[derive(Debug, Default)]
pub struct CStringArena {
    owned_strings: HashMap<String, Pin<Box<CString>>>,
}

impl CStringArena {
    pub fn str_ptr<S: AsRef<str>>(&mut self, string: S) -> *const c_char {
        if let Some(ptr) = self.owned_strings.get(string.as_ref()) {
            return ptr.as_ptr();
        } else {
            let cstr = Box::pin(CString::new(string.as_ref()).unwrap());
            let ret = cstr.as_ptr();
            self.owned_strings.insert(string.as_ref().to_owned(), cstr);
            ret
        }
    }
}


#[repr(transparent)]
pub struct TargetMachine(LLVMTargetMachineRef);

impl Drop for TargetMachine {
    fn drop(&mut self) {
        unsafe { LLVMDisposeTargetMachine(self.0) }
    }
}


#[derive(Debug)]
struct Module {
    ptr: *mut LLVMModule,
    owned_strings: CStringArena,
}

impl Module {
    pub unsafe fn new<S: AsRef<str>>(name: S) -> Self {
        let mut str_arena = CStringArena::default();
        let ptr = LLVMModuleCreateWithName(str_arena.str_ptr(name));
        LLVMSetTarget(ptr, str_arena.str_ptr(llvm_target().to_string_lossy()));

        Module {
            ptr,
            owned_strings: str_arena,
        }
    }

    pub unsafe fn context(&self) -> LLVMContextRef {
        LLVMGetModuleContext(self.ptr)
    }

    pub unsafe fn add_fn<S: AsRef<str>>(
        &mut self,
        name: S,
        args: &mut [LLVMTypeRef],
        ret: LLVMTypeRef,
    ) -> LLVMValueRef {
        let fn_type = LLVMFunctionType(ret, args.as_mut_ptr(), args.len() as _, 0);
        LLVMAddFunction(self.ptr, self.owned_strings.str_ptr(name), fn_type)
    }


    pub unsafe fn ir_str(&self) -> CString {
        let str_ptr = LLVMPrintModuleToString(self.ptr);
        let ret = CStr::from_ptr(str_ptr as *const _).to_owned();
        LLVMDisposeMessage(str_ptr);
        ret
    }

    pub unsafe fn optimize_ir(&mut self, opt_level: u32) {
        let builder = LLVMPassManagerBuilderCreate();
        LLVMPassManagerBuilderSetOptLevel(builder, opt_level);

        let pass_manager = LLVMCreatePassManager();
        LLVMPassManagerBuilderPopulateModulePassManager(builder, pass_manager);
        LLVMPassManagerBuilderDispose(builder);

        // LLVMRunPassManager(pass_manager, self.ptr);
        LLVMRunPassManager(pass_manager, self.ptr);

        LLVMDisposePassManager(pass_manager);
    }

    pub unsafe fn target_machine(&mut self) -> TargetMachine {
        let mut target = null_mut();
        let mut error = null_mut();

        let target_triple = LLVMGetTarget(self.ptr);
        LLVMGetTargetFromTriple(target_triple, &mut target, &mut error);

        if target.is_null() {
            panic!("{:?}", CStr::from_ptr(error));
        }

        let cpu = c_str!("generic");
        let features = c_str!("");
        TargetMachine(LLVMCreateTargetMachine(
            target,
            target_triple,
            cpu,
            features,
            LLVMCodeGenOptLevel::LLVMCodeGenLevelAggressive,
            LLVMRelocMode::LLVMRelocPIC,
            LLVMCodeModel::LLVMCodeModelDefault,
        ))
    }
}

unsafe fn type_from_descriptor(desc: FieldDescriptor) -> LLVMTypeRef {
    match desc {
        FieldDescriptor::Byte => LLVMInt8Type(),
        FieldDescriptor::Char => LLVMInt16Type(),
        FieldDescriptor::Double => LLVMDoubleType(),
        FieldDescriptor::Float => LLVMFloatType(),
        FieldDescriptor::Int => LLVMInt32Type(),
        FieldDescriptor::Long => LLVMInt64Type(),
        FieldDescriptor::Short => LLVMInt16Type(),
        FieldDescriptor::Boolean => LLVMInt8Type(),
        FieldDescriptor::Void => LLVMVoidType(),
        _ => panic!("Unable to create LLVM type for objects yet!"),
    }
}

pub struct ObjectFieldSlot {
    name: String,
    field_type: FieldDescriptor,
}

pub struct ObjectTypeBuilder {
    types: HashMap<String, LLVMTypeRef>,
    layouts: HashMap<String, Vec<ObjectFieldSlot>>,
}

impl ObjectTypeBuilder {
    pub fn new() -> Self {
        ObjectTypeBuilder {
            types: HashMap::new(),
            layouts: HashMap::new(),
        }
    }

    pub unsafe fn type_for_array(&mut self, context: LLVMContextRef, str_arena: &mut CStringArena, loader: &mut ClassLoader, element: &FieldDescriptor) -> LLVMTypeRef {
        let name = format!("[{}", element);

        if let Some(type_ref) = self.types.get(&name) {
            return *type_ref;
        }

        let llvm_type = LLVMStructCreateNamed(context, str_arena.str_ptr(clean_str(&name)));
        self.types.insert(name.to_string(), llvm_type);

        let element_type = self.type_for_desc(context, str_arena, loader, element);

        let mut body = vec![LLVMInt32Type(), LLVMArrayType(element_type, 0)];
        LLVMStructSetBody(llvm_type, body.as_mut_ptr(), 2, true as LLVMBool);

        llvm_type
    }

    pub unsafe fn type_for_desc(&mut self, context: LLVMContextRef, str_arena: &mut CStringArena, loader: &mut ClassLoader, desc: &FieldDescriptor) -> LLVMTypeRef {
        match desc {
            FieldDescriptor::Byte => LLVMInt8Type(),
            FieldDescriptor::Char => LLVMInt16Type(),
            FieldDescriptor::Double => LLVMDoubleType(),
            FieldDescriptor::Float => LLVMFloatType(),
            FieldDescriptor::Int => LLVMInt32Type(),
            FieldDescriptor::Long => LLVMInt64Type(),
            FieldDescriptor::Short => LLVMInt16Type(),
            FieldDescriptor::Boolean => LLVMInt8Type(),
            FieldDescriptor::Void => LLVMVoidType(),
            FieldDescriptor::Object(name) => {
                let type_ref = self.type_for(context, str_arena, loader, name);
                LLVMPointerType(type_ref, 0)
            }
            FieldDescriptor::Array(element) => {
                let type_ref = self.type_for_array(context, str_arena, loader, element);
                LLVMPointerType(type_ref, 0)
            }
            _ => panic!("Unable to create LLVM type for objects yet!"),
        }
    }

    pub unsafe fn type_for(&mut self, context: LLVMContextRef, str_arena: &mut CStringArena, loader: &mut ClassLoader, name: &str) -> LLVMTypeRef {
        if let Some(type_ref) = self.types.get(name) {
            return *type_ref;
        }

        let llvm_type = LLVMStructCreateNamed(context, str_arena.str_ptr(clean_str(name)));
        self.types.insert(name.to_string(), llvm_type);

        let raw_class = match loader.class(name) {
            Some(v) => v.to_owned(),
            None => panic!("Unable to find class: {:?}", name),
        };

        let mut slots = Vec::new();
        let mut types = Vec::new();

        if raw_class.super_class != 0 {
            let super_class = raw_class.super_class();

            types.push(self.type_for(context, str_arena, loader, &super_class));
            slots.push(ObjectFieldSlot {
                name: super_class.clone(),
                field_type: FieldDescriptor::Object(super_class),
            });
        }

        for field in &raw_class.fields {
            if field.access.contains(AccessFlags::STATIC) {
                continue;
            }

            let field_name = field.name(&raw_class.constants).unwrap();
            let descriptor = field.field_type(&raw_class.constants).unwrap();

            slots.push(ObjectFieldSlot {
                name: field_name,
                field_type: descriptor.clone(),
            });

            types.push(self.type_for_desc(context, str_arena, loader, &descriptor));
            // match descriptor {
            //     FieldDescriptor::Object(field_type) => {
            //         let base_type = self.type_for(context, str_arena, loader, &field_type);
            //         types.push(LLVMPointerType(base_type, 0));
            //     },
            //     x => types.push(type_from_descriptor(x)),
            // };
        }

        LLVMStructSetBody(llvm_type, types.as_mut_ptr(), types.len() as _, false as LLVMBool);
        llvm_type
    }
}


pub struct FunctionContext<'c> {
    jmp_labels: HashMap<u64, LLVMBasicBlockRef>,
    stack_values: Vec<LLVMValueRef>,
    // local_values: Vec<LLVMValueRef>,
    init_builder: LLVMBuilderRef,
    str_arena: &'c mut CStringArena,
    alloca_fields: HashMap<String, LLVMValueRef>,
    stack_idx: u64,
}

impl<'c> FunctionContext<'c> {
    pub fn new(init_builder: LLVMBuilderRef, str_arena: &'c mut CStringArena) -> Self {
        FunctionContext {
            jmp_labels: HashMap::new(),
            stack_values: Vec::new(),
            // local_values: Vec::new(),
            init_builder,
            str_arena,
            alloca_fields: HashMap::default(),
            stack_idx: 0,
        }
    }

    pub fn push_stack_alloca(&mut self, alloc_type: &FieldDescriptor) -> LLVMValueRef {
        self.stack_idx += 1;
        self.get_stack_alloca(alloc_type, self.stack_idx - 1)
    }

    pub fn pop_stack_alloca(&mut self, alloc_type: &FieldDescriptor) -> LLVMValueRef {
        self.stack_idx -= 1;
        self.get_stack_alloca(alloc_type, self.stack_idx)
    }

    pub fn get_operand_alloca(&mut self, alloc_type: &FieldDescriptor, idx: u64) -> LLVMValueRef {
        self.get_alloca(alloc_type, idx, "local")
    }

    pub fn get_stack_alloca(&mut self, alloc_type: &FieldDescriptor, idx: u64) -> LLVMValueRef {
        self.get_alloca(alloc_type, idx, "stack")
    }

    pub fn get_alloca(&mut self, alloc_type: &FieldDescriptor, idx: u64, prefix: &str) -> LLVMValueRef {
        unsafe {
            let (llvm_type, name) = match alloc_type {
                // FieldDescriptor::Byte => (LLVMIntType(8), format!("local{}b", idx)),
                // FieldDescriptor::Char => (LLVMIntType(16), format!("local{}c", idx)),
                FieldDescriptor::Double => (LLVMDoubleType(), format!("{}{}d", prefix, idx)),
                FieldDescriptor::Float => (LLVMFloatType(), format!("{}{}f", prefix, idx)),
                FieldDescriptor::Int | FieldDescriptor::Byte | FieldDescriptor::Char | FieldDescriptor::Short | FieldDescriptor::Boolean
                => (LLVMIntType(32), format!("{}{}i", prefix, idx)),
                FieldDescriptor::Long => (LLVMIntType(64), format!("{}{}j", prefix, idx)),
                // FieldDescriptor::Short => (LLVMIntType(16), format!("local{}s", idx)),
                // FieldDescriptor::Boolean => (LLVMIntType(8), format!("local{}z", idx)),
                FieldDescriptor::Object(_) | FieldDescriptor::Array(_) => (LLVMPointerType(LLVMVoidType(), 0), format!("{}{}a", prefix, idx)),
                _ => panic!(),
            };

            if let Some(alloca) = self.alloca_fields.get(&name) {
                return *alloca
            }

            let alloca = LLVMBuildAlloca(self.init_builder, llvm_type, self.str_arena.str_ptr(&name));
            self.alloca_fields.insert(name, alloca);
            alloca
        }
    }
}

pub trait LLVMInstruction {
    unsafe fn add_impl(&self, builder: LLVMBuilderRef, cxt: &mut FunctionContext);
}

// #[cfg(feature = "llvm")]
// impl LLVMInstruction for crate::instruction::instr::aload {
//     unsafe fn add_impl(&self, builder: LLVMBuilderRef, cxt: &mut FunctionContext) {
//         let crate::instruction::instr::aload(index) = *self;
//
//         let operand_type = FieldDescriptor::Object("java/lang/Object".to_string());
//         let local = cxt.get_operand_alloca(&operand_type, index as _);
//         let value = LLVMBuildLoad(builder, local, c_str!("aload"));
//
//         let destination = cxt.push_stack_alloca(&operand_type);
//
//         LLVMBuildStore(builder, value, destination);
//         // LLVMBuildLoad();
//     }
// }

pub unsafe fn build_for_class(mut loader: ClassLoader, name: &str) {
    let target = loader.class(name).unwrap().to_owned();
    let mut module = Module::new(&clean_str(name));

    let jni_interface = LLVMStructCreateNamed(module.context(), c_str!("JNINativeInterface_"));
    let jni_env = LLVMPointerType(jni_interface, 0);
    let jni_env_ref = LLVMPointerType(jni_env, 0);

    let mut type_builder = ObjectTypeBuilder::new();

    let this_obj = type_builder.type_for(module.context(), &mut module.owned_strings, &mut loader, name);
    let this_obj_ptr = LLVMPointerType(this_obj, 0);

    // let jclass = type_builder.type_for(module.context(), &mut module.owned_strings, &mut loader, "java/lang/Class");
    // let jclass_ptr = LLVMPointerType(jclass, 0);

    let jclass = LLVMStructCreateNamed(module.context(), c_str!("jclass"));
    let jclass_ptr = LLVMPointerType(jclass, 0);


    for method in &target.methods {
        let method_name = method.name(&target.constants).unwrap();
        let method_desc = method.descriptor(&target.constants).unwrap();
        let long_name = format!(
            "Java_{}_{}__{}",
            clean_str(name),
            clean_str(&method_name),
            NativeManager::clean_desc(&method_desc).unwrap()
        );
        // let short_name = format!("Java_{}_{}", clean_str(class), clean_str(name));


        let (function, args, ret) = match FieldDescriptor::read_str(&method_desc).unwrap() {
            FieldDescriptor::Method { args, returns } => {
                let mut llvm_args = Vec::with_capacity(2 + args.len());
                llvm_args.push(jni_env_ref);

                if method.access.contains(AccessFlags::STATIC) {
                    llvm_args.push(jclass_ptr);
                } else {
                    llvm_args.push(this_obj_ptr);
                }
                for arg in &args {
                    let llvm_ty = type_builder.type_for_desc(module.context(), &mut module.owned_strings, &mut loader, arg);
                    llvm_args.push(llvm_ty);
                }

                let ret_ty = type_builder.type_for_desc(module.context(), &mut module.owned_strings, &mut loader, &*returns);

                // llvm_args.extend(args.iter().cloned().map(|x|unsafe {type_from_descriptor(x)}));
                (module.add_fn(&long_name, &mut llvm_args[..], ret_ty), args, returns)
            }
            _ => panic!("Unexpected method descriptor!"),
        };


        if method.access.contains(AccessFlags::NATIVE) {
            continue;
        }

        let code = method.code(&target.constants());

        let init_block = LLVMAppendBasicBlock(function, c_str!("init"));
        let init_builder = LLVMCreateBuilder();
        LLVMPositionBuilderAtEnd(init_builder, init_block);

        let mut function_context = FunctionContext::new(init_builder, &mut module.owned_strings);

        // Pre-generate labels so they can be jumped to for branching if needed (plus make debugging easier)
        for (instruction_idx, _) in &code.instructions {
            let block = LLVMAppendBasicBlock(function, function_context.str_arena.str_ptr(format!("{}", instruction_idx)));
            function_context.jmp_labels.insert(*instruction_idx, block);
        }

        let mut arg_idx = 0;

        if !method.access.contains(AccessFlags::STATIC) {
            let local = function_context.get_operand_alloca(&FieldDescriptor::Object("java/lang/Object".to_string()), arg_idx as u64);
            let arg_value = LLVMGetParam(function, 1);
            LLVMBuildStore(function_context.init_builder, arg_value, local);
            arg_idx += 1;
        }

        for (idx, arg) in args.iter().enumerate() {
            // let generic_value_slot = match arg {
            //     FieldDescriptor::Byte => LLVMCreateGenericValueOfInt(LLVMInt8Type(), 0, 1),
            //     FieldDescriptor::Char => LLVMCreateGenericValueOfInt(LLVMInt16Type(), 0, 0),
            //     FieldDescriptor::Double => LLVMCreateGenericValueOfFloat(LLVMDoubleType(), 0.0),
            //     FieldDescriptor::Float => LLVMCreateGenericValueOfFloat(LLVMFloatType(), 0.0),
            //     FieldDescriptor::Int => LLVMCreateGenericValueOfInt(LLVMInt32Type(), 0, 1),
            //     FieldDescriptor::Long => LLVMCreateGenericValueOfInt(LLVMInt64Type(), 0, 1),
            //     FieldDescriptor::Short => LLVMCreateGenericValueOfInt(LLVMInt16Type(), 0, 1),
            //     FieldDescriptor::Boolean => LLVMCreateGenericValueOfInt(LLVMInt8Type(), 0, 0),
            //     FieldDescriptor::Object(_) | FieldDescriptor::Array(_) => LLVMCreateGenericValueOfPointer(null_mut()),
            //     _ => unreachable!("Class was malformed"),
            // };

            // let loc_idx = function_context.local_values.len();
            // ;
            // LLVMBuild

            // LLVMBuildAlloca()
            // function_context.local_values.push(LLVMGetParam(function, arg_idx));
            let local = function_context.get_operand_alloca(arg, arg_idx as u64);
            // println!("Getting operand: {:?}", debug_value_ref(local));

            // println!("{:?}", LLVMCountParams(function));
            let arg_value = LLVMGetParam(function, 2 + idx as u32);
            // println!("Got operand: {:?}", debug_value_ref(arg_value));

            // LLVMPrintValueToString(local);
            LLVMBuildStore(function_context.init_builder, arg_value, local);
            // println!("Built store!");

            if matches!(arg, FieldDescriptor::Double | FieldDescriptor::Long) {
                // function_context.local_values.push()
                arg_idx += 1;
            }

            // LLVMCreateGenericValueOfInt()
            // LLVMGetArgOperand()
            // LLVMGeneric
            arg_idx += 1;
        }

        // Fill in each instruction
        for (instr_idx, instruction) in &code.instructions {
            let block = *function_context.jmp_labels.get(instr_idx).unwrap();
            let builder = LLVMCreateBuilder();
            LLVMPositionBuilderAtEnd(builder, block);

            // let mut comment = format!("{:?}\0", &instruction);
            // let node = LLVMGetOrInsertNamedMetadata(module.ptr, comment.as_ptr() as _, comment.len() - 1);
            // LLVMBuildFPExt()
            instruction.add_impl(builder, &mut function_context);

            println!("{}", llvm_print(LLVMPrintModuleToString, module.ptr));
            println!("-------------------------------------------------------------------------\n\n\n\n\n\n\n\n");
        }
    }


    println!("{}", module.ir_str().to_string_lossy());
}


