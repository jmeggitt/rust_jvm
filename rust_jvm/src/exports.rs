#![allow(unused_variables, clippy::missing_safety_doc)]
#![deny(improper_ctypes_definitions)]

use crate::class::constant::ClassElement;
use crate::class::{AccessFlags, BufferedRead, ClassLoader, ClassPath};
use crate::jvm::call::{build_interface, FlowControl, JavaEnvInvoke, RawJNIEnv};
use crate::jvm::mem::{
    ConstTypeId, FieldDescriptor, GcBox, JavaValue, ManualInstanceReference, ObjectHandle,
    ObjectReference, ObjectType, RawObject,
};
use crate::jvm::JavaEnv;
use jni::sys::*;
use log::LevelFilter;
use simplelog::{CombinedLogger, ConfigBuilder, ThreadLogMode, WriteLogger};
use std::collections::hash_map::DefaultHasher;
use std::env::var;
use std::ffi::{c_void, CStr};
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::os::raw::c_char;
use std::path::PathBuf;
use std::process::exit;
use std::ptr::{null_mut, write_bytes};
use std::sync::atomic::{AtomicI64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

macro_rules! obj_expect {
    ($env:expr, $obj:expr) => {
        obj_expect!($env, $obj, Default::default())
    };
    ($env:expr, $obj:expr, $ret:expr) => {
        match ObjectHandle::from_ptr($obj as _) {
            Some(v) => v,
            None => {
                // TODO: throw null pointer exception
                $env.write_thrown(None);
                return $ret;
            }
        }
    };
}

#[no_mangle]
pub unsafe extern "system" fn JNI_GetDefaultJavaVMInitArgs_impl(args: *mut c_void) -> jint {
    *(args as *mut JavaVMInitArgs) = JavaVMInitArgs {
        version: JNI_VERSION_1_8,
        nOptions: 0,
        options: null_mut(),
        ignoreUnrecognized: JNI_TRUE,
    };
    0
}

#[no_mangle]
pub unsafe extern "system" fn JNI_CreateJavaVM_impl(
    pvm: &mut *mut JavaVM,  // Fill with created JavaVM
    penv: &mut *mut JNIEnv, // Fill with created JNIEnv
    args: *mut c_void,
) -> jint {
    // TODO: Apply init arguments instead of hard coding values
    let _init_args = *(args as *mut JavaVMInitArgs);

    CombinedLogger::init(vec![
        simplelog::TermLogger::new(
            LevelFilter::Debug,
            simplelog::Config::default(),
            simplelog::TerminalMode::Mixed,
            simplelog::ColorChoice::Always,
        ),
        WriteLogger::new(
            LevelFilter::Trace,
            ConfigBuilder::new()
                .set_thread_mode(ThreadLogMode::IDs)
                .build(),
            File::create("execution.log").unwrap(),
        ),
    ])
    .unwrap();

    let java_dir = var("JAVA_HOME").ok().map(PathBuf::from);
    let class_path = match ClassPath::new(
        java_dir,
        Some(vec![
            // "/mnt/c/Users/Jasper/CLionProjects/JavaClassTests/java_std/out".into(),
            ".".into(),
        ]),
    ) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Error indexing class path: {:?}", e);
            exit(1);
        }
    };

    let mut class_loader = ClassLoader::from_class_path(class_path);
    if let Err(e) = class_loader.preload_class_path() {
        eprintln!("Error loading class path: {:?}", e);
        return -1;
    }

    // class_loader.attempt_load("java/lang/reflect/Modifier").unwrap();
    // class_loader.class("java/lang/reflect/Modifier").unwrap().print_method();
    // panic!();

    let jvm = Box::new(JavaEnv::new(class_loader));
    let interface = build_interface(&jvm);

    let vm = &jvm.read().jni_vm as *const _;
    // let vm = JNIInvokeInterface_ {
    //     reserved0: Box::leak(jvm) as *mut Arc<RwLock<JavaEnv>> as _,
    //     reserved1: null_mut(),
    //     reserved2: null_mut(),
    //     DestroyJavaVM: None,
    //     AttachCurrentThread: None,
    //     DetachCurrentThread: None,
    //     GetEnv: None,
    //     AttachCurrentThreadAsDaemon: None,
    // };

    *pvm = Box::leak(Box::new(vm)) as *mut _ as *mut JavaVM;
    *penv = Box::leak(Box::new(Box::new(interface))) as *mut _ as *mut JNIEnv;

    0
}

#[no_mangle]
pub unsafe extern "system" fn JNI_GetCreatedJavaVMs_impl(
    vm_buf: *mut *mut JavaVM,
    buf_len: jsize,
    n_vms: *mut jsize,
) -> jint {
    unimplemented!("This feature is not supported due to functional programming approach")
}

#[no_mangle]
pub extern "system" fn JVM_GetInterfaceVersion_impl() -> i32 {
    4
}

/*************************************************************************
PART 1: Functions for Native Libraries
************************************************************************/
/*
 * java.lang.Object
 */
#[no_mangle]
pub unsafe extern "system" fn JVM_IHashCode_impl(env: RawJNIEnv, obj: jobject) -> jint {
    let mut hasher = DefaultHasher::new();
    obj_expect!(env, obj, 0).hash(&mut hasher);
    let finish = hasher.finish();

    ((finish >> 32) & 0xFFFF_FFFF) as jint ^ (finish & 0xFFFF_FFFF) as jint
}

#[no_mangle]
pub unsafe extern "system" fn JVM_Clone_impl(
    env: RawJNIEnv,
    obj: Option<ObjectHandle>,
) -> Option<ObjectHandle> {
    // let obj = obj_expect!(env, obj, null_mut());

    Some(match obj?.memory_layout() {
        ObjectType::Instance => {
            GcBox::new(RawObject::clone(&*obj?.expect_instance().lock())).into_unknown()
            // (&*obj?.expect_instance().lock()).clone().into_unknown()
            // ObjectWrapper::new(RawObject::clone(&*obj.expect_instance())).into_raw()
        }
        // TODO: Do arrays even implement clone?
        ObjectType::Array(jboolean::ID) => {
            GcBox::new(RawObject::clone(&*obj?.expect_array::<jboolean>().lock())).into_unknown()
        }
        ObjectType::Array(jbyte::ID) => {
            GcBox::new(RawObject::clone(&*obj?.expect_array::<jbyte>().lock())).into_unknown()
        }
        ObjectType::Array(jchar::ID) => {
            GcBox::new(RawObject::clone(&*obj?.expect_array::<jchar>().lock())).into_unknown()
        }
        ObjectType::Array(jshort::ID) => {
            GcBox::new(RawObject::clone(&*obj?.expect_array::<jshort>().lock())).into_unknown()
        }
        ObjectType::Array(jint::ID) => {
            GcBox::new(RawObject::clone(&*obj?.expect_array::<jint>().lock())).into_unknown()
        }
        ObjectType::Array(jlong::ID) => {
            GcBox::new(RawObject::clone(&*obj?.expect_array::<jlong>().lock())).into_unknown()
        }
        ObjectType::Array(jfloat::ID) => {
            GcBox::new(RawObject::clone(&*obj?.expect_array::<jfloat>().lock())).into_unknown()
        }
        ObjectType::Array(jdouble::ID) => {
            GcBox::new(RawObject::clone(&*obj?.expect_array::<jdouble>().lock())).into_unknown()
        }
        ObjectType::Array(<Option<ObjectHandle> as ConstTypeId>::ID) => GcBox::new(
            RawObject::clone(&*obj?.expect_array::<Option<ObjectHandle>>().lock()),
        )
        .into_unknown(),
        x => panic!("Unable to hash object of type {:?}", x),
    })
}

/*
 * java.lang.String
 */
#[no_mangle]
pub unsafe extern "system" fn JVM_InternString_impl(env: RawJNIEnv, str: jstring) -> jstring {
    let obj = obj_expect!(env, str, null_mut());
    let raw_str = obj.expect_string();

    let mut jvm = env.write();
    if jvm.interned_strings.contains_key(&raw_str) {
        return jvm.interned_strings.get(&raw_str).unwrap().ptr();
    }

    jvm.interned_strings.insert(raw_str, obj);
    str
}

/*
 * java.lang.System
 */
#[no_mangle]
pub unsafe extern "system" fn JVM_CurrentTimeMillis_impl(env: RawJNIEnv, ignored: jclass) -> jlong {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time travel occurred")
        .as_millis() as jlong
}

#[no_mangle]
pub unsafe extern "system" fn JVM_NanoTime_impl(env: RawJNIEnv, ignored: jclass) -> jlong {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time travel occurred")
        .as_nanos() as jlong
}

#[no_mangle]
pub unsafe extern "system" fn JVM_ArrayCopy_impl(
    env: RawJNIEnv,
    ignored: jclass,
    src: jobject,
    src_pos: jint,
    dst: jobject,
    dst_pos: jint,
    length: jint,
) {
    // FIXME: Panic and throw exceptions on null or invalid arguments
    let src_object = ObjectHandle::from_ptr(src).unwrap();
    let dst_object = ObjectHandle::from_ptr(dst).unwrap();

    if src_object.memory_layout() != dst_object.memory_layout() {
        panic!("Attempted arraycopy with different typed arrays!");
    }

    match src_object.memory_layout() {
        ObjectType::Array(jboolean::ID) => src_object.expect_array::<jboolean>().lock().array_copy(
            dst_object,
            src_pos as usize,
            dst_pos as usize,
            length as usize,
        ),
        ObjectType::Array(jbyte::ID) => src_object.expect_array::<jbyte>().lock().array_copy(
            dst_object,
            src_pos as usize,
            dst_pos as usize,
            length as usize,
        ),
        ObjectType::Array(jchar::ID) => src_object.expect_array::<jchar>().lock().array_copy(
            dst_object,
            src_pos as usize,
            dst_pos as usize,
            length as usize,
        ),
        ObjectType::Array(jshort::ID) => src_object.expect_array::<jshort>().lock().array_copy(
            dst_object,
            src_pos as usize,
            dst_pos as usize,
            length as usize,
        ),
        ObjectType::Array(jint::ID) => src_object.expect_array::<jint>().lock().array_copy(
            dst_object,
            src_pos as usize,
            dst_pos as usize,
            length as usize,
        ),
        ObjectType::Array(jlong::ID) => src_object.expect_array::<jlong>().lock().array_copy(
            dst_object,
            src_pos as usize,
            dst_pos as usize,
            length as usize,
        ),
        ObjectType::Array(jfloat::ID) => src_object.expect_array::<jfloat>().lock().array_copy(
            dst_object,
            src_pos as usize,
            dst_pos as usize,
            length as usize,
        ),
        ObjectType::Array(jdouble::ID) => src_object.expect_array::<jdouble>().lock().array_copy(
            dst_object,
            src_pos as usize,
            dst_pos as usize,
            length as usize,
        ),
        ObjectType::Array(<Option<ObjectHandle>>::ID) => src_object
            .expect_array::<Option<ObjectHandle>>()
            .lock()
            .array_copy(
                dst_object,
                src_pos as usize,
                dst_pos as usize,
                length as usize,
            ),
        x => panic!("Array copy can not be preformed with type {:?}", x),
    };
}

#[no_mangle]
pub unsafe extern "system" fn JVM_InitProperties_impl(env: RawJNIEnv, p: jobject) -> jobject {
    // TODO: Move properties initialization here from hooks
    unimplemented!()
}

/*
 * java.io.File
 */
#[no_mangle]
pub unsafe extern "system" fn JVM_OnExit_impl(f: unsafe extern "C" fn()) {
    // TODO: Save on exit function
    unimplemented!()
}

/*
 * java.lang.Runtime
 */
#[no_mangle]
pub unsafe extern "system" fn JVM_Exit_impl(code: jint) {
    // TODO: Call exit function passed to jvm
    exit(code)
}

#[no_mangle]
pub unsafe extern "system" fn JVM_Halt_impl(code: jint) {
    // What does this do?
    unimplemented!()
}

#[no_mangle]
pub unsafe extern "system" fn JVM_GC_impl() {
    // Nah
}

/* Returns the number of real-time milliseconds that have elapsed since the
 * least-recently-inspected heap object was last inspected by the garbage
 * collector.
 *
 * For simple stop-the-world collectors this value is just the time
 * since the most recent collection.  For generational collectors it is the
 * time since the oldest generation was most recently collected.  Other
 * collectors are free to return a pessimistic estimate of the elapsed time, or
 * simply the time since the last full collection was performed.
 *
 * Note that in the presence of reference objects, a given object that is no
 * longer strongly reachable may have to be inspected multiple times before it
 * can be reclaimed.
 */
#[no_mangle]
pub unsafe extern "system" fn JVM_MaxObjectInspectionAge_impl() -> jlong {
    0 // Rust checks objects as soon as any reference is dropped. This is way easier than storing this info
}

#[no_mangle]
pub unsafe extern "system" fn JVM_TraceInstructions_impl(on: jboolean) {
    unimplemented!()
}

#[no_mangle]
pub unsafe extern "system" fn JVM_TraceMethodCalls_impl(on: jboolean) {
    unimplemented!()
}

#[no_mangle]
pub unsafe extern "system" fn JVM_TotalMemory_impl() -> jlong {
    unimplemented!()
}

#[no_mangle]
pub unsafe extern "system" fn JVM_FreeMemory_impl() -> jlong {
    unimplemented!()
}

#[no_mangle]
pub unsafe extern "system" fn JVM_MaxMemory_impl() -> jlong {
    // We're not using a fixed gc so pretend we're on a theoretical turring machine
    jlong::MAX
}

#[no_mangle]
pub unsafe extern "system" fn JVM_ActiveProcessorCount_impl() -> jint {
    num_cpus::get() as jint
}

#[no_mangle]
pub unsafe extern "system" fn JVM_LoadLibrary_impl(name: *mut c_char) -> *mut c_void {
    warn!("JVM_LoadLibrary({:?})", CStr::from_ptr(name));
    unimplemented!()
}

#[no_mangle]
pub unsafe extern "system" fn JVM_UnloadLibrary_impl(handle: *mut c_void) {
    warn!("JVM_UnloadLibrary({:p})", handle);
    unimplemented!()
}

#[no_mangle]
pub unsafe extern "system" fn JVM_FindLibraryEntry_impl(
    handle: *mut c_void,
    name: *mut c_char,
) -> *mut c_void {
    // warn!("JVM_FindLibraryEntry({:p}, {:?})", handle, CStr::from_ptr(name));

    #[cfg(windows)]
    let ret = winapi::um::libloaderapi::GetProcAddress(handle as *mut _, name) as *mut c_void;

    // if direct_attempt.is_null() {
    //     let mut buffer = vec![null_mut(); 512];
    //     let byte_len = size_of::<*mut winapi::shared::minwindef::HMODULE>() * buffer.len();
    //     let mut req_len = 0;
    //
    //     let process_handle = winapi::um::processthreadsapi::GetCurrentProcess();
    //     warn!("Passed Handle: {:p}, Process Handle: {:p}", handle, process_handle);
    //     if winapi::um::psapi::EnumProcessModules(process_handle, buffer.as_mut_ptr(), byte_len as u32, &mut req_len) == 0 {
    //         let err = winapi::um::errhandlingapi::GetLastError();
    //         error!("Failed to get all loaded modules for process! (Err: {})", err);
    //         return direct_attempt;
    //     }
    //
    //     let entries = req_len as usize / size_of::<*mut winapi::shared::minwindef::HMODULE>();
    //     warn!("Found {} loaded modules", entries);
    //     for index in 0..entries {
    //         direct_attempt = winapi::um::libloaderapi::GetProcAddress(buffer[index], name) as *mut c_void;
    //         if !direct_attempt.is_null() {
    //             break
    //         }
    //     }
    // }

    #[cfg_attr(any(target_os = "linux", target_os = "android"), link(name = "dl"))]
    #[cfg_attr(any(target_os = "freebsd", target_os = "dragonfly"), link(name = "c"))]
    #[cfg(unix)]
    extern "C" {
        fn dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void;
    }

    #[cfg(unix)]
    let ret: *mut c_void = dlsym(handle, name);

    if ret.is_null() {
        warn!(
            "JVM_FindLibraryEntry({:p}, {:?}) -> Symbol Not Found",
            handle,
            CStr::from_ptr(name)
        );
    } else {
        warn!(
            "JVM_FindLibraryEntry({:p}, {:?}) -> {:p}",
            handle,
            CStr::from_ptr(name),
            ret
        );
    }

    ret
    // let name = CStr::from_ptr(name);
    // unimplemented!("JVM_FindLibraryEntry_impl({:?}, {:?})", handle, name)
}

#[no_mangle]
pub unsafe extern "system" fn JVM_IsSupportedJNIVersion_impl(version: jint) -> jboolean {
    unimplemented!()
}

/*
 * java.lang.Float and java.lang.Double
 */
#[no_mangle]
pub unsafe extern "system" fn JVM_IsNaN_impl(d: jdouble) -> jboolean {
    d.is_nan() as jboolean
}

/*
 * java.lang.Throwable
 */
#[no_mangle]
pub unsafe extern "system" fn JVM_FillInStackTrace_impl(env: RawJNIEnv, throwable: jobject) {
    let exception = obj_expect!(env, throwable);
    // env.read().dump_debug_info();
    // unimplemented!("JVM_FillInStackTrace")
}

#[no_mangle]
pub unsafe extern "system" fn JVM_GetStackTraceDepth_impl(
    env: RawJNIEnv,
    throwable: jobject,
) -> jint {
    unimplemented!()
}

#[no_mangle]
pub unsafe extern "system" fn JVM_GetStackTraceElement_impl(
    env: RawJNIEnv,
    throwable: jobject,
    index: jint,
) -> jobject {
    unimplemented!()
}

/*
 * java.lang.Compiler
 */
#[no_mangle]
pub unsafe extern "system" fn JVM_InitializeCompiler_impl(env: RawJNIEnv, comp_cls: jclass) {
    unimplemented!()
}

#[no_mangle]
pub unsafe extern "system" fn JVM_IsSilentCompiler_impl(
    env: RawJNIEnv,
    comp_cls: jclass,
) -> jboolean {
    unimplemented!()
}

#[no_mangle]
pub unsafe extern "system" fn JVM_CompileClass_impl(
    env: RawJNIEnv,
    comp_cls: jclass,
    cls: jclass,
) -> jboolean {
    unimplemented!()
}

#[no_mangle]
pub unsafe extern "system" fn JVM_CompileClasses_impl(
    env: RawJNIEnv,
    cls: jclass,
    jname: jstring,
) -> jboolean {
    unimplemented!()
}

#[no_mangle]
pub unsafe extern "system" fn JVM_CompilerCommand_impl(
    env: RawJNIEnv,
    comp_cls: jclass,
    arg: jobject,
) -> jobject {
    unimplemented!()
}

#[no_mangle]
pub unsafe extern "system" fn JVM_EnableCompiler_impl(env: RawJNIEnv, comp_cls: jclass) {
    unimplemented!()
}

#[no_mangle]
pub unsafe extern "system" fn JVM_DisableCompiler_impl(env: RawJNIEnv, comp_cls: jclass) {
    unimplemented!()
}

/*
 * java.lang.SecurityManager
 */
#[no_mangle]
pub unsafe extern "system" fn JVM_CurrentLoadedClass_impl(env: RawJNIEnv) -> jclass {
    unimplemented!()
}

#[no_mangle]
pub unsafe extern "system" fn JVM_CurrentClassLoader_impl(env: RawJNIEnv) -> jobject {
    unimplemented!()
}

#[no_mangle]
pub unsafe extern "system" fn JVM_GetClassContext_impl(env: RawJNIEnv) -> jobjectArray {
    unimplemented!()
}

#[no_mangle]
pub unsafe extern "system" fn JVM_ClassDepth_impl(env: RawJNIEnv, name: jstring) -> jint {
    unimplemented!()
}

#[no_mangle]
pub unsafe extern "system" fn JVM_ClassLoaderDepth_impl(env: RawJNIEnv) -> jint {
    unimplemented!()
}

/*
 * java.lang.Package
 */
#[no_mangle]
pub unsafe extern "system" fn JVM_GetSystemPackage_impl(env: RawJNIEnv, name: jstring) -> jstring {
    unimplemented!()
}

#[no_mangle]
pub unsafe extern "system" fn JVM_GetSystemPackages_impl(env: RawJNIEnv) -> jobjectArray {
    unimplemented!()
}

/*
 * java.io.ObjectInputStream
 */
#[no_mangle]
pub unsafe extern "system" fn JVM_AllocateNewObject_impl(
    env: RawJNIEnv,
    obj: jobject,
    curr_class: jclass,
    init_class: jclass,
) -> jobject {
    unimplemented!()
}

#[no_mangle]
pub unsafe extern "system" fn JVM_AllocateNewArray_impl(
    env: RawJNIEnv,
    obj: jobject,
    curr_class: jclass,
    length: jint,
) -> jobject {
    unimplemented!()
}

#[no_mangle]
pub unsafe extern "system" fn JVM_LatestUserDefinedLoader_impl(env: RawJNIEnv) -> jobject {
    unimplemented!()
}

/*
 * This function has been deprecated and should not be considered
 * part of the specified JVM interface.
 */
#[no_mangle]
pub unsafe extern "system" fn JVM_LoadClass0_impl(
    env: RawJNIEnv,
    obj: jobject,
    curr_class: jclass,
    curr_class_name: jstring,
) -> jclass {
    unimplemented!()
}

/*
 * java.lang.reflect.Array
 */
#[no_mangle]
pub unsafe extern "system" fn JVM_GetArrayLength_impl(env: RawJNIEnv, arr: jobject) -> jint {
    unimplemented!()
}

#[no_mangle]
pub unsafe extern "system" fn JVM_GetArrayElement_impl(
    env: RawJNIEnv,
    arr: jobject,
    index: jint,
) -> jobject {
    unimplemented!()
}

#[no_mangle]
pub unsafe extern "system" fn JVM_GetPrimitiveArrayElement_impl(
    env: RawJNIEnv,
    arr: jobject,
    index: jint,
    w_code: jint,
) -> jvalue {
    unimplemented!()
}

#[no_mangle]
pub unsafe extern "system" fn JVM_SetArrayElement_impl(
    env: RawJNIEnv,
    arr: jobject,
    index: jint,
    val: jobject,
) {
    unimplemented!()
}

#[no_mangle]
pub unsafe extern "system" fn JVM_SetPrimitiveArrayElement_impl(
    env: RawJNIEnv,
    arr: jobject,
    index: jint,
    v: jvalue,
    v_code: u8,
) {
    unimplemented!()
}

#[no_mangle]
pub unsafe extern "system" fn JVM_NewArray_impl(
    env: RawJNIEnv,
    elt_class: jclass,
    length: jint,
) -> jobject {
    unimplemented!()
}

#[no_mangle]
pub unsafe extern "system" fn JVM_NewMultiArray_impl(
    env: RawJNIEnv,
    elt_class: jclass,
    dim: jintArray,
) -> jobject {
    unimplemented!()
}

/*
 * java.lang.Class and java.lang.ClassLoader
 */

// TODO: What is this used for?
// #define JVM_CALLER_DEPTH -1

/*
 * Returns the immediate caller class of the native method invoking
 * JVM_GetCallerClass.  The Method.invoke and other frames due to
 * reflection machinery are skipped.
 *
 * The depth parameter must be -1 (JVM_DEPTH). The caller is expected
 * to be marked with sun.reflect.CallerSensitive.  The JVM will throw
 * an error if it is not marked propertly.
 */
#[no_mangle]
pub unsafe extern "system" fn JVM_GetCallerClass_impl(env: RawJNIEnv, depth: i32) -> jclass {
    let jvm = env.read();
    let call_stack = jvm.thread_manager.get_current_call_stack().unwrap();
    // let len = jvm.call_stack.len();

    // if call_stack.len() < 2 - depth as usize {
    //     panic!("Attempted to call Java_sun_reflect_Reflection_getCallerClass__ without caller");
    // }

    // len - 1 = Reflection.class
    // len - 2 = Target class
    // len - 3 = Caller class

    // jvm.debug_print_call_stack();
    let index = (call_stack.len() as jint + depth - 2) as usize;
    // jvm.thread_manager.info_print_call_stack();
    // if index < call_stack.len() {
    //     warn!("JVM_GetCallerClass({}) -> {:?}", depth, call_stack[index].1);
    // } else {
    //     warn!("JVM_GetCallerClass({}) -> ?", depth);
    //     // jvm.debug_print_call_stack();
    // }
    if index < call_stack.len() {
        return call_stack[index].0.ptr();
    }

    null_mut()
}

/*
 * Find primitive classes
 * utf: class name
 */
#[no_mangle]
pub unsafe extern "system" fn JVM_FindPrimitiveClass_impl(
    env: RawJNIEnv,
    utf: *const i8,
) -> jclass {
    let str = CStr::from_ptr(utf);
    env.write().class_instance(&str.to_string_lossy()).ptr()
}

/*
 * Link the class
 */
#[no_mangle]
pub unsafe extern "system" fn JVM_ResolveClass_impl(env: RawJNIEnv, cls: jclass) {
    unimplemented!()
}

/*
 * Find a class from a boot class loader. Returns NULL if class not found.
 */
#[no_mangle]
pub unsafe extern "system" fn JVM_FindClassFromBootLoader_impl(
    env: RawJNIEnv,
    name: *const c_char,
) -> jclass {
    JVM_FindPrimitiveClass_impl(env, name)
}

#[no_mangle]
pub unsafe extern "system" fn JVM_FindClassFromCaller_impl(
    env: RawJNIEnv,
    name: *const c_char,
    init: jboolean,
    loader: jobject,
    caller: jclass,
) -> jclass {
    JVM_FindPrimitiveClass_impl(env, name)
}
/*
 * Find a class from a given class loader. Throw ClassNotFoundException
 * or NoClassDefFoundError depending on the value of the last
 * argument.
 */
#[no_mangle]
pub unsafe extern "system" fn JVM_FindClassFromClassLoader_impl(
    env: RawJNIEnv,
    name: *const c_char,
    init: jboolean,
    loader: jobject,
    throw_error: jboolean,
) -> jclass {
    JVM_FindPrimitiveClass_impl(env, name)
}

/*
 * Find a class from a given class.
 */
#[no_mangle]
pub unsafe extern "system" fn JVM_FindClassFromClass_impl(
    env: RawJNIEnv,
    name: *const c_char,
    init: jboolean,
    from: jclass,
) -> jclass {
    JVM_FindPrimitiveClass_impl(env, name)
}

/* Find a loaded class cached by the VM */
#[no_mangle]
pub unsafe extern "system" fn JVM_FindLoadedClass_impl(
    env: RawJNIEnv,
    loader: jobject,
    name: jstring,
) -> jclass {
    let name = obj_expect!(env, name, null_mut())
        .expect_string()
        .replace('.', "/");
    env.write().class_instance(&name).ptr()
}

/* Define a class */
#[no_mangle]
pub unsafe extern "system" fn JVM_DefineClass_impl(
    env: RawJNIEnv,
    name: *mut u8,
    loader: jobject,
    buf: *mut jbyte,
    len: jsize,
    pd: jobject,
) -> jclass {
    unimplemented!()
}

/* Define a class with a source _impl(added in JDK1.5) */
#[no_mangle]
pub unsafe extern "system" fn JVM_DefineClassWithSource_impl(
    env: RawJNIEnv,
    name: *mut u8,
    loader: jobject,
    buf: *mut jbyte,
    len: jsize,
    pd: jobject,
    source: *mut u8,
) -> jclass {
    unimplemented!()
}

/*
 * Reflection support functions
 */

#[no_mangle]
pub unsafe extern "system" fn JVM_GetClassName_impl(env: RawJNIEnv, cls: jclass) -> jstring {
    unimplemented!()
}

#[no_mangle]
pub unsafe extern "system" fn JVM_GetClassInterfaces_impl(
    env: RawJNIEnv,
    cls: jclass,
) -> jobjectArray {
    unimplemented!()
}

#[no_mangle]
pub unsafe extern "system" fn JVM_GetClassLoader_impl(env: RawJNIEnv, cls: jclass) -> jobject {
    unimplemented!()
}

#[no_mangle]
pub unsafe extern "system" fn JVM_IsInterface_impl(env: RawJNIEnv, cls: jclass) -> jboolean {
    let jstring_name: Option<ObjectHandle> = obj_expect!(env, cls, JNI_FALSE)
        .expect_instance()
        .lock()
        .read_named_field("name");
    let class_name = jstring_name.unwrap().expect_string().replace('.', "/");

    // TODO: Is this right? I picked it at random
    let mut jvm = env.write();
    jvm.class_loader.attempt_load(&class_name).unwrap();
    jvm.class_loader
        .class(&class_name)
        .unwrap()
        .access_flags
        .contains(AccessFlags::INTERFACE) as jboolean
}

#[no_mangle]
pub unsafe extern "system" fn JVM_GetClassSigners_impl(
    env: RawJNIEnv,
    cls: jclass,
) -> jobjectArray {
    unimplemented!()
}

#[no_mangle]
pub unsafe extern "system" fn JVM_SetClassSigners_impl(
    env: RawJNIEnv,
    cls: jclass,
    signers: jobjectArray,
) {
    unimplemented!()
}

#[no_mangle]
pub unsafe extern "system" fn JVM_GetProtectionDomain_impl(env: RawJNIEnv, cls: jclass) -> jobject {
    unimplemented!()
}

#[no_mangle]
pub unsafe extern "system" fn JVM_IsArrayClass_impl(env: RawJNIEnv, cls: jclass) -> jboolean {
    obj_expect!(env, cls, JNI_FALSE)
        .unwrap_as_class()
        .starts_with('[') as jboolean
}

#[no_mangle]
pub unsafe extern "system" fn JVM_IsPrimitiveClass_impl(env: RawJNIEnv, cls: jclass) -> jboolean {
    matches!(
        obj_expect!(env, cls, JNI_FALSE).unwrap_as_class().as_ref(),
        "boolean" | "byte" | "char" | "short" | "int" | "float" | "long" | "double"
    ) as jboolean
}

#[no_mangle]
pub unsafe extern "system" fn JVM_GetComponentType_impl(env: RawJNIEnv, cls: jclass) -> jclass {
    let class_name = obj_expect!(env, cls, null_mut()).unwrap_as_class();
    if !class_name.starts_with('[') {
        return null_mut();
    }

    env.write().class_instance(&class_name[1..]).ptr()
}

#[no_mangle]
pub unsafe extern "system" fn JVM_GetClassModifiers_impl(env: RawJNIEnv, cls: jclass) -> jint {
    // Idk what the differance is and they didn't document it
    JVM_GetClassAccessFlags_impl(env, cls)
}

#[no_mangle]
pub unsafe extern "system" fn JVM_GetDeclaredClasses_impl(
    env: RawJNIEnv,
    of_class: jclass,
) -> jobjectArray {
    unimplemented!()
}

#[no_mangle]
pub unsafe extern "system" fn JVM_GetDeclaringClass_impl(
    env: RawJNIEnv,
    of_class: Option<ObjectHandle>,
) -> Option<ObjectHandle> {
    // Can check NestHost attribute for nested classes
    // Can check EnclosingClass for local
    // How to check inner class?
    let name = of_class?.unwrap_as_class();
    let mut lock = env.write();
    let class = lock.class_loader.class(&name)?;

    if let Some(host) = class.nest_host() {
        return Some(lock.class_instance(&host));
    }

    if let Some((enclosing, _, _)) = class.enclosing_method() {
        return Some(lock.class_instance(&enclosing));
    }

    None
}

/* Generics support _impl(JDK 1.5) */
#[no_mangle]
pub unsafe extern "system" fn JVM_GetClassSignature_impl(env: RawJNIEnv, cls: jclass) -> jstring {
    unimplemented!()
}

/* Annotations support _impl(JDK 1.5) */
#[no_mangle]
pub unsafe extern "system" fn JVM_GetClassAnnotations_impl(
    env: RawJNIEnv,
    cls: jclass,
) -> jbyteArray {
    unimplemented!()
}

/* Type use annotations support _impl(JDK 1.8) */

#[no_mangle]
pub unsafe extern "system" fn JVM_GetClassTypeAnnotations_impl(
    env: RawJNIEnv,
    cls: jclass,
) -> jbyteArray {
    unimplemented!()
}

#[no_mangle]
pub unsafe extern "system" fn JVM_GetFieldTypeAnnotations_impl(
    env: RawJNIEnv,
    field: jobject,
) -> jbyteArray {
    unimplemented!()
}

#[no_mangle]
pub unsafe extern "system" fn JVM_GetMethodTypeAnnotations_impl(
    env: RawJNIEnv,
    method: jobject,
) -> jbyteArray {
    unimplemented!()
}

/*
 * New _impl(JDK 1.4) reflection implementation
 */

#[no_mangle]
pub unsafe extern "system" fn JVM_GetClassDeclaredMethods_impl(
    env: RawJNIEnv,
    of_class: jclass,
    public_only: jboolean,
) -> jobjectArray {
    unimplemented!()
}

#[no_mangle]
pub unsafe extern "system" fn JVM_GetClassDeclaredFields_impl(
    env: RawJNIEnv,
    of_class: jclass,
    public_only: jboolean,
) -> jobjectArray {
    let class = obj_expect!(env, of_class, null_mut());

    let mut jvm = env.write();
    let field_schema = jvm.class_schema("java/lang/reflect/Field");

    let jstring_name: Option<ObjectHandle> =
        class.expect_instance().lock().read_named_field("name");
    let class_name = jstring_name.unwrap().expect_string().replace('.', "/");

    let target_class_schema = jvm.class_schema(&class_name);
    let raw_class = jvm.class_loader.class(&class_name).unwrap().to_owned();

    let mut fields = Vec::with_capacity(target_class_schema.field_lookup.len());

    for raw_field in &raw_class.fields {
        if public_only == JNI_TRUE && !raw_field.access.contains(AccessFlags::PUBLIC) {
            continue;
        }

        let field_obj = ObjectHandle::new(field_schema.clone());
        let instance = field_obj.expect_instance();
        let mut lock = instance.lock();
        lock.write_named_field("clazz", Some(class));

        let name = raw_field.name(&raw_class.constants).unwrap();
        let slot = match target_class_schema.field_offsets.get(&name) {
            Some(v) => v.offset as jint,
            None => -1,
        };
        lock.write_named_field("slot", slot);
        lock.write_named_field("name", jvm.build_string(&name));

        let type_class =
            match FieldDescriptor::read_str(&raw_field.descriptor(&raw_class.constants).unwrap())
                .unwrap()
            {
                FieldDescriptor::Byte => jvm.class_instance("byte"),
                FieldDescriptor::Char => jvm.class_instance("char"),
                FieldDescriptor::Double => jvm.class_instance("double"),
                FieldDescriptor::Float => jvm.class_instance("float"),
                FieldDescriptor::Int => jvm.class_instance("int"),
                FieldDescriptor::Long => jvm.class_instance("long"),
                FieldDescriptor::Short => jvm.class_instance("short"),
                FieldDescriptor::Boolean => jvm.class_instance("boolean"),
                FieldDescriptor::Object(x) => jvm.class_instance(&x),
                FieldDescriptor::Array(x) => jvm.class_instance(&format!("[{:?}", x)),
                _ => panic!("Can't get classes for these types"),
            };
        lock.write_named_field("type", Some(type_class));
        lock.write_named_field("modifiers", raw_field.access.bits() as jint);
        fields.push(Some(field_obj));
    }

    // for (slot, field) in target_class_schema.field_lookup.iter().enumerate() {
    //     let raw_field = raw_class
    //         .get_field(&field.name, &format!("{:?}", &field.desc))
    //         .unwrap();
    //     if public_only == JNI_TRUE && !raw_field.access.contains(AccessFlags::PUBLIC) {
    //         continue;
    //     }
    //
    //     let field_obj = ObjectHandle::new(field_schema.clone());
    //     let instance = field_obj.expect_instance();
    //     instance.write_named_field("clazz", Some(class));
    //     instance.write_named_field("slot", slot as jint);
    //     instance.write_named_field("name", jvm.build_string(&field.name));
    //
    //     let type_class = match &field.desc {
    //         FieldDescriptor::Byte => jvm.class_instance("byte"),
    //         FieldDescriptor::Char => jvm.class_instance("char"),
    //         FieldDescriptor::Double => jvm.class_instance("double"),
    //         FieldDescriptor::Float => jvm.class_instance("float"),
    //         FieldDescriptor::Int => jvm.class_instance("int"),
    //         FieldDescriptor::Long => jvm.class_instance("long"),
    //         FieldDescriptor::Short => jvm.class_instance("short"),
    //         FieldDescriptor::Boolean => jvm.class_instance("boolean"),
    //         FieldDescriptor::Object(x) => jvm.class_instance(x),
    //         FieldDescriptor::Array(x) => jvm.class_instance(&format!("[{:?}", x)),
    //         _ => panic!("Can't get classes for these types"),
    //     };
    //     instance.write_named_field("type", Some(type_class));
    //     instance.write_named_field("modifiers", raw_field.access.bits() as jint);
    //     fields.push(Some(field_obj));
    // }

    ObjectHandle::array_from_data(fields).ptr()
}

#[no_mangle]
pub unsafe extern "system" fn JVM_GetClassDeclaredConstructors_impl(
    env: RawJNIEnv,
    of_class: jclass,
    public_only: jboolean,
) -> jobjectArray {
    let class = obj_expect!(env, of_class, null_mut());

    let mut jvm = env.write();
    let constructor_schema = jvm.class_schema("java/lang/reflect/Constructor");

    let jstring_name: Option<ObjectHandle> =
        class.expect_instance().lock().read_named_field("name");
    let class_name = jstring_name.unwrap().expect_string().replace('.', "/");
    // warn!("Getting Constructors for {}", &class_name);

    let target_class_schema = jvm.class_schema(&class_name);
    let raw_class = jvm.class_loader.class(&class_name).unwrap().to_owned();
    // warn!("{:?}", &raw_class);

    let mut constructors = Vec::with_capacity(target_class_schema.field_lookup.len());

    for raw_field in raw_class.methods {
        // warn!("Field: {:?}", raw_field.name(&raw_class.constants));

        // let raw_field = raw_class.get_field(&field.name, &format!("{:?}", &field.desc)).unwrap();
        if public_only == JNI_TRUE && !raw_field.access.contains(AccessFlags::PUBLIC) {
            // warn!("Skipping Non-public: {:?}", raw_field.name(&raw_class.constants));
            continue;
        }

        if raw_field.name(&raw_class.constants).unwrap() != "<init>" {
            // warn!("Skipping Non-constructor: {:?}", raw_field.name(&raw_class.constants));
            continue;
        }

        let field_obj = ObjectHandle::new(constructor_schema.clone());
        let instance = field_obj.expect_instance();
        let mut lock = instance.lock();
        lock.write_named_field("clazz", Some(class));
        lock.write_named_field("slot", constructors.len() as jint);
        lock.write_named_field("override", JavaValue::Byte(1));

        let desc = FieldDescriptor::read_str(&raw_field.descriptor(&raw_class.constants).unwrap())
            .unwrap();

        if let FieldDescriptor::Method { args, .. } = desc {
            let mut arg_types = Vec::with_capacity(args.len());

            for arg in &args {
                arg_types.push(Some(match arg {
                    FieldDescriptor::Byte => jvm.class_instance("byte"),
                    FieldDescriptor::Char => jvm.class_instance("char"),
                    FieldDescriptor::Double => jvm.class_instance("double"),
                    FieldDescriptor::Float => jvm.class_instance("float"),
                    FieldDescriptor::Int => jvm.class_instance("int"),
                    FieldDescriptor::Long => jvm.class_instance("long"),
                    FieldDescriptor::Short => jvm.class_instance("short"),
                    FieldDescriptor::Boolean => jvm.class_instance("boolean"),
                    FieldDescriptor::Object(x) => jvm.class_instance(x),
                    FieldDescriptor::Array(x) => jvm.class_instance(&format!("[{:?}", x)),
                    _ => panic!("Can't get classes for these types"),
                }));
            }

            lock.write_named_field(
                "parameterTypes",
                Some(ObjectHandle::array_from_data(arg_types)),
            );
        } else {
            panic!("Invalid descriptor on constructor");
        }

        // TODO: Check exception types
        lock.write_named_field(
            "exceptionTypes",
            Some(ObjectHandle::array_from_data::<Option<ObjectHandle>>(
                vec![],
            )),
        );
        lock.write_named_field("modifiers", raw_field.access.bits() as jint);
        constructors.push(Some(field_obj));
    }

    // warn!("Got constructors: {:?}", &constructors);
    ObjectHandle::array_from_data(constructors).ptr()
}

/* Differs from JVM_GetClassModifiers in treatment of inner classes.
 This returns the access flags for the class as specified in the
 class file rather than searching the InnerClasses attribute _impl(if
) to find the source-level access flags. Only the values of
 the low 13 bits _impl(i.e., a mask of 0x1FFF) are guaranteed to be
 valid. */
#[no_mangle]
pub unsafe extern "system" fn JVM_GetClassAccessFlags_impl(env: RawJNIEnv, cls: jclass) -> jint {
    let jstring_name: Option<ObjectHandle> = obj_expect!(env, cls, 0)
        .expect_instance()
        .lock()
        .read_named_field("name");
    let class_name = jstring_name.unwrap().expect_string().replace('.', "/");

    let mut flags = env
        .read()
        .class_loader
        .class(&class_name)
        .unwrap()
        .access_flags
        .bits();

    // Force everything to show up as public to resolve some errors due to reflection
    flags &= !(AccessFlags::PUBLIC | AccessFlags::PRIVATE | AccessFlags::PROTECTED).bits();
    flags |= AccessFlags::PUBLIC.bits();

    flags as jint
}

/* The following two reflection routines are still needed due to startup time issues */
/*
 * java.lang.reflect.Method
 */
#[no_mangle]
pub unsafe extern "system" fn JVM_InvokeMethod_impl(
    env: RawJNIEnv,
    method: jobject,
    obj: jobject,
    args0: jobjectArray,
) -> jobject {
    unimplemented!()
}

/*
 * java.lang.reflect.Constructor
 */
#[no_mangle]
pub unsafe extern "system" fn JVM_NewInstanceFromConstructor_impl(
    mut env: RawJNIEnv,
    c: jobject,
    args0: jobjectArray,
) -> jobject {
    let constructor = obj_expect!(env, c, null_mut()).expect_instance();
    let constructor_lock = constructor.lock();
    let target_class: Option<ObjectHandle> = constructor_lock.read_named_field("clazz");
    let parameters: Option<ObjectHandle> = constructor_lock.read_named_field("parameterTypes");

    let mut arg_descriptors = Vec::new();

    // TODO: Separate out functionality for use with other JVM calls
    for arg_class in parameters
        .unwrap()
        .expect_array::<Option<ObjectHandle>>()
        .lock()
        .iter()
    {
        let class_name = arg_class.unwrap().unwrap_as_class();
        arg_descriptors.push(match class_name.as_ref() {
            "boolean" => FieldDescriptor::Boolean,
            "byte" => FieldDescriptor::Byte,
            "short" => FieldDescriptor::Short,
            "char" => FieldDescriptor::Char,
            "int" => FieldDescriptor::Int,
            "float" => FieldDescriptor::Float,
            "long" => FieldDescriptor::Long,
            "double" => FieldDescriptor::Double,
            x if x.starts_with('[') => FieldDescriptor::read_str(x).unwrap(),
            x => FieldDescriptor::Object(x.to_owned()),
        });
    }

    let element_ref = ClassElement {
        class: target_class.unwrap().unwrap_as_class(),
        element: "<init>".to_string(),
        desc: format!(
            "{:?}",
            FieldDescriptor::Method {
                args: arg_descriptors.clone(),
                returns: Box::new(FieldDescriptor::Void),
            }
        ),
    };

    let mut args = Vec::with_capacity(arg_descriptors.len());

    if !args0.is_null() {
        for (obj_arg, desc) in obj_expect!(env, args0, null_mut())
            .expect_array::<Option<ObjectHandle>>()
            .lock()
            .iter()
            .zip(arg_descriptors)
        {
            let next = match desc {
                FieldDescriptor::Byte => JavaValue::Byte(
                    obj_arg
                        .unwrap()
                        .expect_instance()
                        .lock()
                        .read_named_field("value"),
                ),
                FieldDescriptor::Char => JavaValue::Char(
                    obj_arg
                        .unwrap()
                        .expect_instance()
                        .lock()
                        .read_named_field("value"),
                ),
                FieldDescriptor::Double => {
                    let value = JavaValue::Double(
                        obj_arg
                            .unwrap()
                            .expect_instance()
                            .lock()
                            .read_named_field("value"),
                    );
                    args.push(value); // Push extra to comply with standard
                    value
                }
                FieldDescriptor::Float => JavaValue::Float(
                    obj_arg
                        .unwrap()
                        .expect_instance()
                        .lock()
                        .read_named_field("value"),
                ),
                FieldDescriptor::Int => JavaValue::Int(
                    obj_arg
                        .unwrap()
                        .expect_instance()
                        .lock()
                        .read_named_field("value"),
                ),
                FieldDescriptor::Long => {
                    let value = JavaValue::Long(
                        obj_arg
                            .unwrap()
                            .expect_instance()
                            .lock()
                            .read_named_field("value"),
                    );
                    args.push(value); // Push extra to comply with standard
                    value
                }
                FieldDescriptor::Short => JavaValue::Short(
                    obj_arg
                        .unwrap()
                        .expect_instance()
                        .lock()
                        .read_named_field("value"),
                ),
                FieldDescriptor::Boolean => JavaValue::Byte(
                    obj_arg
                        .unwrap()
                        .expect_instance()
                        .lock()
                        .read_named_field("value"),
                ),
                FieldDescriptor::Object(_) | FieldDescriptor::Array(_) => {
                    JavaValue::Reference(*obj_arg)
                }
                _ => unreachable!(),
            };

            args.push(next);
        }
    }

    let ret = ObjectHandle::new(
        env.write()
            .class_schema(&target_class.unwrap().unwrap_as_class()),
    );
    match env.invoke_virtual(element_ref, ret, args) {
        Ok(None) => {}
        Err(FlowControl::Throws(x)) => env.write_thrown(x),
        x => panic!("{:?}", x),
    };

    ret.ptr()
}

/*
 *access:  Constant pool, currently used to implement reflective access to annotations (JDK 1.5)
 */

#[no_mangle]
pub unsafe extern "system" fn JVM_GetClassConstantPool_impl(
    env: RawJNIEnv,
    cls: jclass,
) -> jobject {
    unimplemented!()
}

#[no_mangle]
pub unsafe extern "system" fn JVM_ConstantPoolGetSize_impl(
    env: RawJNIEnv,
    _unused: jobject,
    jcpool: jobject,
) -> jint {
    unimplemented!()
}

#[no_mangle]
pub unsafe extern "system" fn JVM_ConstantPoolGetClassAt_impl(
    env: RawJNIEnv,
    _unused: jobject,
    jcpool: jobject,
    index: jint,
) -> jclass {
    unimplemented!()
}

#[no_mangle]
pub unsafe extern "system" fn JVM_ConstantPoolGetClassAtIfLoaded_impl(
    env: RawJNIEnv,
    _unused: jobject,
    jcpool: jobject,
    index: jint,
) -> jclass {
    unimplemented!()
}

#[no_mangle]
pub unsafe extern "system" fn JVM_ConstantPoolGetMethodAt_impl(
    env: RawJNIEnv,
    _unused: jobject,
    jcpool: jobject,
    index: jint,
) -> jobject {
    unimplemented!()
}

#[no_mangle]
pub unsafe extern "system" fn JVM_ConstantPoolGetMethodAtIfLoaded_impl(
    env: RawJNIEnv,
    _unused: jobject,
    jcpool: jobject,
    index: jint,
) -> jobject {
    unimplemented!()
}

#[no_mangle]
pub unsafe extern "system" fn JVM_ConstantPoolGetFieldAt_impl(
    env: RawJNIEnv,
    _unused: jobject,
    jcpool: jobject,
    index: jint,
) -> jobject {
    unimplemented!()
}

#[no_mangle]
pub unsafe extern "system" fn JVM_ConstantPoolGetFieldAtIfLoaded_impl(
    env: RawJNIEnv,
    _unused: jobject,
    jcpool: jobject,
    index: jint,
) -> jobject {
    unimplemented!()
}

#[no_mangle]
pub unsafe extern "system" fn JVM_ConstantPoolGetMemberRefInfoAt_impl(
    env: RawJNIEnv,
    _unused: jobject,
    jcpool: jobject,
    index: jint,
) -> jobjectArray {
    unimplemented!()
}

#[no_mangle]
pub unsafe extern "system" fn JVM_ConstantPoolGetIntAt_impl(
    env: RawJNIEnv,
    _unused: jobject,
    jcpool: jobject,
    index: jint,
) -> jint {
    unimplemented!()
}

#[no_mangle]
pub unsafe extern "system" fn JVM_ConstantPoolGetLongAt_impl(
    env: RawJNIEnv,
    _unused: jobject,
    jcpool: jobject,
    index: jint,
) -> jlong {
    unimplemented!()
}

#[no_mangle]
pub unsafe extern "system" fn JVM_ConstantPoolGetFloatAt_impl(
    env: RawJNIEnv,
    _unused: jobject,
    jcpool: jobject,
    index: jint,
) -> jfloat {
    unimplemented!()
}

#[no_mangle]
pub unsafe extern "system" fn JVM_ConstantPoolGetDoubleAt_impl(
    env: RawJNIEnv,
    _unused: jobject,
    jcpool: jobject,
    index: jint,
) -> jdouble {
    unimplemented!()
}

#[no_mangle]
pub unsafe extern "system" fn JVM_ConstantPoolGetStringAt_impl(
    env: RawJNIEnv,
    _unused: jobject,
    jcpool: jobject,
    index: jint,
) -> jstring {
    unimplemented!()
}

#[no_mangle]
pub unsafe extern "system" fn JVM_ConstantPoolGetUTF8At_impl(
    env: RawJNIEnv,
    _unused: jobject,
    jcpool: jobject,
    index: jint,
) -> jstring {
    unimplemented!()
}

/*
 * Parameter reflection
 */

#[no_mangle]
pub unsafe extern "system" fn JVM_GetMethodParameters_impl(
    env: RawJNIEnv,
    method: jobject,
) -> jobjectArray {
    unimplemented!()
}

/*
 * java.security.*
 */

#[no_mangle]
pub unsafe extern "system" fn JVM_DoPrivileged_impl(
    mut env: RawJNIEnv,
    cls: jclass,
    action: jobject,
    context: jobject,
    wrap_exception: jboolean,
) -> jobject {
    let action = obj_expect!(env, action, null_mut());
    let element = ClassElement {
        class: action.get_class(),
        element: "run".to_string(),
        desc: "()Ljava/lang/Object;".to_string(),
    };

    match env.invoke_virtual(element, action, vec![]) {
        Ok(Some(JavaValue::Reference(None))) => null_mut(),
        Ok(Some(JavaValue::Reference(Some(v)))) => v.ptr(),
        Err(FlowControl::Throws(x)) => {
            env.write_thrown(x);
            null_mut()
        }
        x => panic!("{:?}", x),
    }
    // panic!("Action: {:?}", ObjectHandle::from_ptr(action))
}

#[no_mangle]
pub unsafe extern "system" fn JVM_GetInheritedAccessControlContext_impl(
    env: RawJNIEnv,
    cls: jclass,
) -> jobject {
    unimplemented!()
}

#[no_mangle]
pub unsafe extern "system" fn JVM_GetStackAccessControlContext_impl(
    env: RawJNIEnv,
    cls: jclass,
) -> jobject {
    // TODO: Actually implement JVM_GetStackAccessControlContext
    null_mut()
}

/*
 * Signal support, used to implement the shutdown sequence.  Every VM must
 * support JVM_SIGINT and JVM_SIGTERM, raising the former for user interrupts
 * (^C) and the latter for external termination (kill, system shutdown, etc.).
 * Other platform-dependent signal values may also be supported.
 */

#[no_mangle]
pub unsafe extern "system" fn JVM_RegisterSignal_impl(
    sig: jint,
    handler: *mut c_void,
) -> *mut c_void {
    unimplemented!()
}

#[no_mangle]
pub unsafe extern "system" fn JVM_RaiseSignal_impl(x: jint) -> jboolean {
    unimplemented!()
}

#[no_mangle]
pub unsafe extern "system" fn JVM_FindSignal_impl(name: *mut u8) -> jint {
    unimplemented!()
}

/*
 * Retrieve the assertion directives for the specified class.
 */
#[no_mangle]
pub unsafe extern "system" fn JVM_DesiredAssertionStatus_impl(
    env: RawJNIEnv,
    _unused: jclass,
    cls: jclass,
) -> jboolean {
    // TODO: Allow assertions on specific classes
    JNI_TRUE
}

/*
 * Retrieve the assertion directives from the VM.
 */
#[no_mangle]
pub unsafe extern "system" fn JVM_AssertionStatusDirectives_impl(
    env: RawJNIEnv,
    _unused: jclass,
) -> jobject {
    unimplemented!()
}

/*
 * java.util.concurrent.atomic.AtomicLong
 */
#[no_mangle]
pub unsafe extern "system" fn JVM_SupportsCX8_impl() -> jboolean {
    JNI_TRUE
}

#[no_mangle]
pub unsafe extern "system" fn JVM_CX8Field_impl(
    env: RawJNIEnv,
    obj: jobject,
    field: jfieldID,
    expected: jlong,
    x: jlong,
) -> jboolean {
    let obj = obj_expect!(env, obj, JNI_FALSE);
    let instance = obj.expect_instance();
    let mut lock = instance.lock();
    // assert_eq!(offset as usize % size_of::<jvalue>(), 0);
    // let index = offset as usize / size_of::<jvalue>();
    let index = field as usize;

    // let fields = instance.raw_fields();
    assert!(index < lock.len());

    let ptr = &mut lock[index] as *mut jvalue as *const AtomicI64;

    let res = (*ptr).compare_exchange(expected, x, Ordering::SeqCst, Ordering::Relaxed);
    res.is_ok() as jboolean
}

/* Define a class with a source with conditional verification (added HSX 14)
 * -Xverify:all will verify anyway, -Xverify:none will not verify,
 * -Xverify:remote (default) will obey this conditional
 * i.e. true = should_verify_class
 */
#[no_mangle]
pub unsafe extern "system" fn JVM_DefineClassWithSourceCond_impl(
    env: JNIEnv,
    name: *const u8,
    loader: jobject,
    buf: *mut jbyte,
    len: jsize,
    pd: jobject,
    source: *mut u8,
    verify: jboolean,
) -> jclass {
    unimplemented!()
}

/* Annotations support (JDK 1.6) */

// field is a handle to a java.lang.reflect.Field object
#[no_mangle]
pub unsafe extern "system" fn JVM_GetFieldAnnotations_impl(
    env: RawJNIEnv,
    field: jobject,
) -> jbyteArray {
    unimplemented!()
}

// method is a handle to a java.lang.reflect.Method object
#[no_mangle]
pub unsafe extern "system" fn JVM_GetMethodAnnotations_impl(
    env: RawJNIEnv,
    method: jobject,
) -> jbyteArray {
    unimplemented!()
}

// method is a handle to a java.lang.reflect.Method object
#[no_mangle]
pub unsafe extern "system" fn JVM_GetMethodDefaultAnnotationValue_impl(
    env: RawJNIEnv,
    method: jobject,
) -> jbyteArray {
    unimplemented!()
}

// method is a handle to a java.lang.reflect.Method object
#[no_mangle]
pub unsafe extern "system" fn JVM_GetMethodParameterAnnotations_impl(
    env: RawJNIEnv,
    method: jobject,
) -> jbyteArray {
    unimplemented!()
}

/*
 * com.sun.dtrace.jsdt support
 */

// #define JVM_TRACING_DTRACE_VERSION 1

/*
 * Structure to pass one probe description to JVM
 */
#[repr(C)]
pub struct JVM_DTraceProbe {
    method: jmethodID,
    function: jstring,
    name: jstring,
    reserved: [*mut c_void; 4], // for future use
}

/**
 * Encapsulates the stability ratings for a DTrace provider field
 */
#[repr(C)]
pub struct JVM_DTraceInterfaceAttributes {
    name_stability: jint,
    data_stability: jint,
    dependency_class: jint,
}

/*
 * Structure to pass one provider description to JVM
 */
#[repr(C)]
pub struct JVM_DTraceProvider {
    name: jstring,
    probes: *mut JVM_DTraceProbe,
    probe_count: jint,
    provider_attributes: JVM_DTraceInterfaceAttributes,
    module_attributes: JVM_DTraceInterfaceAttributes,
    function_attributes: JVM_DTraceInterfaceAttributes,
    name_attributes: JVM_DTraceInterfaceAttributes,
    args_attributes: JVM_DTraceInterfaceAttributes,
    reserved: [*mut c_void; 4], // for future use
}

/*
 * Get the version number the JVM was built with
 */
#[no_mangle]
pub unsafe extern "system" fn JVM_DTraceGetVersion_impl(env: RawJNIEnv) -> jint {
    unimplemented!()
}

/*
 * Register new probe with given signature, return global handle
 *
 * The version passed in is the version that the library code was
 * built with.
 */
#[no_mangle]
pub unsafe extern "system" fn JVM_DTraceActivate_impl(
    env: RawJNIEnv,
    version: jint,
    module_name: jstring,
    providers_count: jint,
    providers: *mut JVM_DTraceProvider,
) -> jlong {
    unimplemented!()
}

/*
 * Check JSDT probe
 */
#[no_mangle]
pub unsafe extern "system" fn JVM_DTraceIsProbeEnabled_impl(
    env: RawJNIEnv,
    method: jmethodID,
) -> jboolean {
    unimplemented!()
}

/*
 * Destroy custom DOF
 */
#[no_mangle]
pub unsafe extern "system" fn JVM_DTraceDispose_impl(env: RawJNIEnv, activation_handle: jlong) {
    unimplemented!()
}

/*
 * Check to see if DTrace is supported by OS
 */
#[no_mangle]
pub unsafe extern "system" fn JVM_DTraceIsSupported_impl(env: RawJNIEnv) -> jboolean {
    unimplemented!()
}

/*************************************************************************
PART 2: Support for the Verifier and Class File Format Checker
************************************************************************/
/*
 * Return the class name in UTF format. The result is valid
 * until JVM_ReleaseUTf is called.
 *
 * The caller must treat the string as a constant and not modify it
 * in any way.
 */
#[no_mangle]
pub unsafe extern "system" fn JVM_GetClassNameUTF_impl(env: RawJNIEnv, cb: jclass) -> *const u8 {
    unimplemented!()
}

/*
 * Returns the constant pool types in the buffer provided by "types."
 */
#[no_mangle]
pub unsafe extern "system" fn JVM_GetClassCPTypes_impl(env: RawJNIEnv, cb: jclass, types: *mut u8) {
    unimplemented!()
}

/*
 * Returns the number of Constant Pool entries.
 */
#[no_mangle]
pub unsafe extern "system" fn JVM_GetClassCPEntriesCount_impl(env: RawJNIEnv, cb: jclass) -> jint {
    unimplemented!()
}

/*
 * Returns the number of *declared* fields or methods.
 */
#[no_mangle]
pub unsafe extern "system" fn JVM_GetClassFieldsCount_impl(env: RawJNIEnv, cb: jclass) -> jint {
    unimplemented!()
}

#[no_mangle]
pub unsafe extern "system" fn JVM_GetClassMethodsCount_impl(env: RawJNIEnv, cb: jclass) -> jint {
    unimplemented!()
}

/*
 * Returns the CP indexes of exceptions raised by a given method.
 * Places the result in the given buffer.
 *
 * The method is identified by method_index.
 */
#[no_mangle]
pub unsafe extern "system" fn JVM_GetMethodIxExceptionIndexes_impl(
    env: RawJNIEnv,
    cb: jclass,
    method_index: jint,
    exceptions: *mut u16,
) {
    unimplemented!()
}
/*
 * Returns the number of exceptions raised by a given method.
 * The method is identified by method_index.
 */
#[no_mangle]
pub unsafe extern "system" fn JVM_GetMethodIxExceptionsCount_impl(
    env: RawJNIEnv,
    cb: jclass,
    method_index: jint,
) -> jint {
    unimplemented!()
}

/*
 * Returns the byte code sequence of a given method.
 * Places the result in the given buffer.
 *
 * The method is identified by method_index.
 */
#[no_mangle]
pub unsafe extern "system" fn JVM_GetMethodIxByteCode_impl(
    env: RawJNIEnv,
    cb: jclass,
    method_index: jint,
    code: *mut u8,
) {
    unimplemented!()
}

/*
 * Returns the length of the byte code sequence of a given method.
 * The method is identified by method_index.
 */
#[no_mangle]
pub unsafe extern "system" fn JVM_GetMethodIxByteCodeLength_impl(
    env: RawJNIEnv,
    cb: jclass,
    method_index: jint,
) -> jint {
    unimplemented!()
}

/*
 * A structure used to a capture exception table entry in a Java method.
 */
#[repr(C)]
pub struct JVM_ExceptionTableEntryType {
    start_pc: jint,
    end_pc: jint,
    handler_pc: jint,
    catch_type: jint,
}

/*
 * Returns the exception table entry at entry_index of a given method.
 * Places the result in the given buffer.
 *
 * The method is identified by method_index.
 */
#[no_mangle]
pub unsafe extern "system" fn JVM_GetMethodIxExceptionTableEntry_impl(
    env: RawJNIEnv,
    cb: jclass,
    method_index: jint,
    entry_index: jint,
    entry: *mut JVM_ExceptionTableEntryType,
) {
    unimplemented!()
}

/*
 * Returns the length of the exception table of a given method.
 * The method is identified by method_index.
 */
#[no_mangle]
pub unsafe extern "system" fn JVM_GetMethodIxExceptionTableLength_impl(
    env: RawJNIEnv,
    cb: jclass,
    index: i32,
) -> jint {
    unimplemented!()
}

/*
 * Returns the modifiers of a given field.
 * The field is identified by field_index.
 */
#[no_mangle]
pub unsafe extern "system" fn JVM_GetFieldIxModifiers_impl(
    env: RawJNIEnv,
    cb: jclass,
    index: i32,
) -> jint {
    unimplemented!()
}

/*
 * Returns the modifiers of a given method.
 * The method is identified by method_index.
 */
#[no_mangle]
pub unsafe extern "system" fn JVM_GetMethodIxModifiers_impl(
    env: RawJNIEnv,
    cb: jclass,
    index: i32,
) -> jint {
    unimplemented!()
}

/*
 * Returns the number of local variables of a given method.
 * The method is identified by method_index.
 */
#[no_mangle]
pub unsafe extern "system" fn JVM_GetMethodIxLocalsCount_impl(
    env: RawJNIEnv,
    cb: jclass,
    index: i32,
) -> jint {
    unimplemented!()
}

/*
 * Returns the number of arguments (including this pointer) of a given method.
 * The method is identified by method_index.
 */
#[no_mangle]
pub unsafe extern "system" fn JVM_GetMethodIxArgsSize_impl(
    env: RawJNIEnv,
    cb: jclass,
    index: i32,
) -> jint {
    unimplemented!()
}

/*
 * Returns the maximum amount of stack (in words) used by a given method.
 * The method is identified by method_index.
 */
#[no_mangle]
pub unsafe extern "system" fn JVM_GetMethodIxMaxStack_impl(
    env: RawJNIEnv,
    cb: jclass,
    index: i32,
) -> jint {
    unimplemented!()
}

/*
 * Is a given method a constructor.
 * The method is identified by method_index.
 */
#[no_mangle]
pub unsafe extern "system" fn JVM_IsConstructorIx_impl(
    env: RawJNIEnv,
    cb: jclass,
    index: i32,
) -> jboolean {
    unimplemented!()
}

/*
 * Is the given method generated by the VM.
 * The method is identified by method_index.
 */
#[no_mangle]
pub unsafe extern "system" fn JVM_IsVMGeneratedMethodIx_impl(
    env: RawJNIEnv,
    cb: jclass,
    index: i32,
) -> jboolean {
    unimplemented!()
}

/*
 * Returns the name of a given method in UTF format.
 * The result remains valid until JVM_ReleaseUTF is called.
 *
 * The caller must treat the string as a constant and not modify it
 * in any way.
 */
#[no_mangle]
pub unsafe extern "system" fn JVM_GetMethodIxNameUTF_impl(
    env: RawJNIEnv,
    cb: jclass,
    index: jint,
) -> *const u8 {
    unimplemented!()
}

/*
 * Returns the signature of a given method in UTF format.
 * The result remains valid until JVM_ReleaseUTF is called.
 *
 * The caller must treat the string as a constant and not modify it
 * in any way.
 */
#[no_mangle]
pub unsafe extern "system" fn JVM_GetMethodIxSignatureUTF_impl(
    env: RawJNIEnv,
    cb: jclass,
    index: jint,
) -> *const u8 {
    unimplemented!()
}

/*
 * Returns the name of the field referred to at a given constant pool
 * index.
 *
 * The result is in UTF format and remains valid until JVM_ReleaseUTF
 * is called.
 *
 * The caller must treat the string as a constant and not modify it
 * in any way.
 */
#[no_mangle]
pub unsafe extern "system" fn JVM_GetCPFieldNameUTF_impl(
    env: RawJNIEnv,
    cb: jclass,
    index: jint,
) -> *const u8 {
    unimplemented!()
}

/*
 * Returns the name of the method referred to at a given constant pool
 * index.
 *
 * The result is in UTF format and remains valid until JVM_ReleaseUTF
 * is called.
 *
 * The caller must treat the string as a constant and not modify it
 * in any way.
 */
#[no_mangle]
pub unsafe extern "system" fn JVM_GetCPMethodNameUTF_impl(
    env: RawJNIEnv,
    cb: jclass,
    index: jint,
) -> *const u8 {
    unimplemented!()
}

/*
 * Returns the signature of the method referred to at a given constant pool
 * index.
 *
 * The result is in UTF format and remains valid until JVM_ReleaseUTF
 * is called.
 *
 * The caller must treat the string as a constant and not modify it
 * in any way.
 */
#[no_mangle]
pub unsafe extern "system" fn JVM_GetCPMethodSignatureUTF_impl(
    env: RawJNIEnv,
    cb: jclass,
    index: jint,
) -> *const u8 {
    unimplemented!()
}

/*
 * Returns the signature of the field referred to at a given constant pool
 * index.
 *
 * The result is in UTF format and remains valid until JVM_ReleaseUTF
 * is called.
 *
 * The caller must treat the string as a constant and not modify it
 * in any way.
 */
#[no_mangle]
pub unsafe extern "system" fn JVM_GetCPFieldSignatureUTF_impl(
    env: RawJNIEnv,
    cb: jclass,
    index: jint,
) -> *const u8 {
    unimplemented!()
}

/*
 * Returns the class name referred to at a given constant pool index.
 *
 * The result is in UTF format and remains valid until JVM_ReleaseUTF
 * is called.
 *
 * The caller must treat the string as a constant and not modify it
 * in any way.
 */
#[no_mangle]
pub unsafe extern "system" fn JVM_GetCPClassNameUTF_impl(
    env: RawJNIEnv,
    cb: jclass,
    index: jint,
) -> *const u8 {
    unimplemented!()
}

/*
 * Returns the class name referred to at a given constant pool index.
 *
 * The constant pool entry must refer to a CONSTANT_Fieldref.
 *
 * The result is in UTF format and remains valid until JVM_ReleaseUTF
 * is called.
 *
 * The caller must treat the string as a constant and not modify it
 * in any way.
 */
#[no_mangle]
pub unsafe extern "system" fn JVM_GetCPFieldClassNameUTF_impl(
    env: RawJNIEnv,
    cb: jclass,
    index: jint,
) -> *const u8 {
    unimplemented!()
}

/*
 * Returns the class name referred to at a given constant pool index.
 *
 * The constant pool entry must refer to CONSTANT_Methodref or
 * CONSTANT_InterfaceMethodref.
 *
 * The result is in UTF format and remains valid until JVM_ReleaseUTF
 * is called.
 *
 * The caller must treat the string as a constant and not modify it
 * in any way.
 */
#[no_mangle]
pub unsafe extern "system" fn JVM_GetCPMethodClassNameUTF_impl(
    env: RawJNIEnv,
    cb: jclass,
    index: jint,
) -> *const u8 {
    unimplemented!()
}

/*
 * Returns the modifiers of a field in calledClass. The field is
 * referred to in class cb at constant pool entry index.
 *
 * The caller must treat the string as a constant and not modify it
 * in any way.
 *
 * Returns -1 if the field does not exist in calledClass.
 */
#[no_mangle]
pub unsafe extern "system" fn JVM_GetCPFieldModifiers_impl(
    env: RawJNIEnv,
    cb: jclass,
    index: i32,
    called_class: jclass,
) -> jint {
    unimplemented!()
}

/*
 * Returns the modifiers of a method in calledClass. The method is
 * referred to in class cb at constant pool entry index.
 *
 * Returns -1 if the method does not exist in calledClass.
 */
#[no_mangle]
pub unsafe extern "system" fn JVM_GetCPMethodModifiers_impl(
    env: RawJNIEnv,
    cb: jclass,
    index: i32,
    called_class: jclass,
) -> jint {
    unimplemented!()
}

/*
 * Releases the UTF string obtained from the VM.
 */
#[no_mangle]
pub unsafe extern "system" fn JVM_ReleaseUTF_impl(utf: *const u8) {
    unimplemented!()
}

/*
 * Compare if two classes are in the same package.
 */
#[no_mangle]
pub unsafe extern "system" fn JVM_IsSameClassPackage_impl(
    env: RawJNIEnv,
    class1: jclass,
    class2: jclass,
) -> jboolean {
    unimplemented!()
}

/* Get classfile constants */
// TODO: #include "classfile_constants.h"

/*
 * A function defined by the byte-code verifier and called by the VM.
 * This is not a function implemented in the VM.
 *
 * Returns JNI_FALSE if verification fails. A detailed error message
 * will be places in msg_buf, whose length is specified by buf_len.
 */
// type verifier_fn_t =
//     unsafe extern "C" fn(env: RawJNIEnv, cb: jclass, msg_buf: *mut u8, buf_len: jint) -> jboolean;
// typedef jboolean (*verifier_fn_t)(JNIEnv *env,
// jclass cb,
// char * msg_buf,
// jint buf_len);

/*
 * Support for a VM-independent class format checker.
 */
#[repr(C)]
pub struct method_size_info {
    code: u64,
    /* byte code */
    excs: u64,
    /* exceptions */
    etab: u64,
    /* catch table */
    lnum: u64,
    /* line number */
    lvar: u64,
    /* local vars */
}

#[repr(C)]
pub struct class_size_info {
    constants: u32,
    /* constant pool */
    fields: u32,
    methods: u32,
    interfaces: u32,
    fields2: u32,
    /* number of static 2-word fields */
    innerclasses: u32,
    /* # of records in InnerClasses attr */
    clinit: method_size_info,
    /* memory used in clinit */
    main: method_size_info,
    /* used everywhere else */
}

/*
 * Functions defined in libjava.so to perform string conversions.
 *
 */

// type to_java_string_fn_t = unsafe extern "C" fn(env: RawJNIEnv, str: *mut u8) -> jstring;
// typedef jstring (*to_java_string_fn_t)(JNIEnv *env, char *str);

// type to_c_string_fn_t =
//     unsafe extern "C" fn(env: RawJNIEnv, s: jstring, b: *mut jboolean) -> *mut u8;
// typedef char *(*to_c_string_fn_t)(JNIEnv *env, jstring s, jboolean *b);

/* This is the function defined in libjava.so that performs class
 * format checks. This functions fills in size information about
 * the class file and returns:
 *
 *   0: good
 *  -1: out of memory
 *  -2: bad format
 *  -3: unsupported version
 *  -4: bad class name
 */

// type check_format_fn_t = unsafe extern "C" fn(
//     class_name: *mut u8,
//     data: *mut u8,
//     data_size: u32,
//     class_size: *mut class_size_info,
//     message_buffer: *mut u8,
//     buffer_length: jint,
//     measure_only: jboolean,
//     check_relaxed: jboolean,
// ) -> jint;
// typedef jint (*check_format_fn_t)(char *class_name,
// *mut u8data,
// u32 data_size,
// class_size_info *class_size,
// char *message_buffer,
// jint buffer_length,
// jboolean measure_only,
// jboolean check_relaxed);

// #define JVM_RECOGNIZED_CLASS_MODIFIERS (JVM_ACC_PUBLIC | \
// JVM_ACC_FINAL | \
// JVM_ACC_SUPER | \
// JVM_ACC_INTERFACE | \
// JVM_ACC_ABSTRACT | \
// JVM_ACC_ANNOTATION | \
// JVM_ACC_ENUM | \
// JVM_ACC_SYNTHETIC)

// #define JVM_RECOGNIZED_FIELD_MODIFIERS (JVM_ACC_PUBLIC | \
// JVM_ACC_PRIVATE | \
// JVM_ACC_PROTECTED | \
// JVM_ACC_STATIC | \
// JVM_ACC_FINAL | \
// JVM_ACC_VOLATILE | \
// JVM_ACC_TRANSIENT | \
// JVM_ACC_ENUM | \
// JVM_ACC_SYNTHETIC)

// #define JVM_RECOGNIZED_METHOD_MODIFIERS (JVM_ACC_PUBLIC | \
// JVM_ACC_PRIVATE | \
// JVM_ACC_PROTECTED | \
// JVM_ACC_STATIC | \
// JVM_ACC_FINAL | \
// JVM_ACC_SYNCHRONIZED | \
// JVM_ACC_BRIDGE | \
// JVM_ACC_VARARGS | \
// JVM_ACC_NATIVE | \
// JVM_ACC_ABSTRACT | \
// JVM_ACC_STRICT | \
// JVM_ACC_SYNTHETIC)

/*
 * This is the function defined in libjava.so to perform path
 * canonicalization. VM call this function before opening jar files
 * to load system classes.
 *
 */

// type canonicalize_fn_t =
//     unsafe extern "C" fn(env: RawJNIEnv, orig: *mut u8, out: *mut u8, len: i32) -> i32;
// typedef int (*canonicalize_fn_t)(JNIEnv *env, char *orig, char *out, int len);

/*************************************************************************
PART 3: I/O and Network Support
************************************************************************/

/* Note that the JVM IO functions are expected to return JVM_IO_ERR
 * when there is any kind of error. The caller can then use the
 * platform specific support (e.g., errno) to get the detailed
 * error info.  The JVM_GetLastErrorString procedure may also be used
 * to obtain a descriptive error string.
 */
// #define JVM_IO_ERR  (-1)

/* For interruptible IO. Returning JVM_IO_INTR indicates that an IO
 * operation has been disrupted by Thread.interrupt. There are a
 * number of technical difficulties related to interruptible IO that
 * need to be solved. For example, most existing programs do not handle
 * InterruptedIOExceptions specially, they simply treat those as any
 * IOExceptions, which typically indicate fatal errors.
 *
 * There are also two modes of operation for interruptible IO. In the
 * resumption mode, an interrupted IO operation is guaranteed not to
 * have any side-effects, and can be restarted. In the termination mode,
 * an interrupted IO operation corrupts the underlying IO stream, so
 * that the only reasonable operation on an interrupted stream is to
 * close that stream. The resumption mode seems to be impossible to
 * implement on Win32 and Solaris. Implementing the termination mode is
 * easier, but it's not clear that's the right semantics.
 *
 * Interruptible IO is not supported on Win32.It can be enabled/disabled
 * using a compile-time flag on Solaris. Third-party JVM ports do not
 * need to implement interruptible IO.
 */
// #define JVM_IO_INTR (-2)

/* Write a string into the given buffer, in the platform's local encoding,
 * that describes the most recent system-level error to occur in this thread.
 * Return the length of the string or zero if no error occurred.
 */
#[no_mangle]
pub unsafe extern "system" fn JVM_GetLastErrorString_impl(buf: *mut u8, len: i32) -> jint {
    0 // TODO: It would probably be helpful to implement this
}

/*
 * Convert a pathname into native format.  This function does syntactic
 * cleanup, such as removing redundant separator characters.  It modifies
 * the given pathname string in place.
 */
#[no_mangle]
pub unsafe extern "system" fn JVM_NativePath_impl(x: *mut u8) -> *mut u8 {
    unimplemented!()
}

/*
 * JVM I/O error codes
 */
// #define JVM_EEXIST       -100

/*
 * Open a file descriptor. This function returns a negative error code
 * on error, and a non-negative integer that is the file descriptor on
 * success.
 */
#[no_mangle]
pub unsafe extern "system" fn JVM_Open_impl(fname: *const u8, flags: jint, mode: jint) -> jint {
    unimplemented!()
}

/*
 * Close a file descriptor. This function returns -1 on error, and 0
 * on success.
 *
 * fd        the file descriptor to close.
 */
#[no_mangle]
pub unsafe extern "system" fn JVM_Close_impl(fd: jint) -> jint {
    unimplemented!()
}

/*
 * Read data from a file decriptor into a char array.
 *
 * fd        the file descriptor to read from.
 * buf       the buffer where to put the read data.
 * nbytes    the number of bytes to read.
 *
 * This function returns -1 on error, and 0 on success.
 */
#[no_mangle]
pub unsafe extern "system" fn JVM_Read_impl(fd: jint, buf: *mut u8, nbytes: jint) -> jint {
    unimplemented!()
}

/*
 * Write data from a char array to a file decriptor.
 *
 * fd        the file descriptor to read from.
 * buf       the buffer from which to fetch the data.
 * nbytes    the number of bytes to write.
 *
 * This function returns -1 on error, and 0 on success.
 */
#[no_mangle]
pub unsafe extern "system" fn JVM_Write_impl(fd: jint, buf: *mut u8, nbytes: jint) -> jint {
    unimplemented!()
}

/*
 * Returns the number of bytes available for reading from a given file
 * descriptor
 */
#[no_mangle]
pub unsafe extern "system" fn JVM_Available_impl(fd: jint, pbytes: *mut jlong) -> jint {
    unimplemented!()
}

/*
 * Move the file descriptor pointer from whence by offset.
 *
 * fd        the file descriptor to move.
 * offset    the number of bytes to move it by.
 * whence    the start from where to move it.
 *
 * This function returns the resulting pointer location.
 */
#[no_mangle]
pub unsafe extern "system" fn JVM_Lseek_impl(fd: jint, offset: jlong, whence: jint) -> jlong {
    unimplemented!()
}

/*
 * Set the length of the file associated with the given descriptor to the given
 * length.  If the new length is longer than the current length then the file
 *extended:  is, the contents of the extended portion are not defined.  The
 * value of the file pointer is undefined after this procedure returns.
 */
#[no_mangle]
pub unsafe extern "system" fn JVM_SetLength_impl(fd: jint, length: jlong) -> jint {
    unimplemented!()
}

/*
 * Synchronize the file descriptor's in memory state with that of the
 * physical device.  Return of -1 is an error, 0 is OK.
 */
#[no_mangle]
pub unsafe extern "system" fn JVM_Sync_impl(fd: jint) -> jint {
    unimplemented!()
}

/*
 * Networking library support
 */

#[no_mangle]
pub unsafe extern "system" fn JVM_InitializeSocketLibrary_impl() -> jint {
    unimplemented!()
}

#[repr(C)]
pub struct sockaddr;

#[no_mangle]
pub unsafe extern "system" fn JVM_Socket_impl(
    domain: jint,
    type_name: jint,
    protocol: jint,
) -> jint {
    info!("JVM_Socket({}, {}, {})", domain, type_name, protocol);
    #[cfg(unix)]
    {
        #[link(name = "c")]
        extern "C" {
            pub fn socket(domain: i32, socket_type: i32, protocol: i32) -> i32;
        }

        return socket(domain, type_name, protocol);
    }

    #[cfg(windows)]
    return winapi::um::winsock2::socket(domain, type_name, protocol) as jint;
}

#[no_mangle]
pub unsafe extern "system" fn JVM_SocketClose_impl(fd: jint) -> jint {
    unimplemented!()
}

#[no_mangle]
pub unsafe extern "system" fn JVM_SocketShutdown_impl(fd: jint, howto: jint) -> jint {
    unimplemented!()
}

#[no_mangle]
pub unsafe extern "system" fn JVM_Recv_impl(
    fd: jint,
    buf: *mut u8,
    n_bytes: jint,
    flags: jint,
) -> jint {
    unimplemented!()
}

#[no_mangle]
pub unsafe extern "system" fn JVM_Send_impl(
    fd: jint,
    buf: *mut u8,
    n_bytes: jint,
    flags: jint,
) -> jint {
    unimplemented!()
}

#[no_mangle]
pub unsafe extern "system" fn JVM_Timeout_impl(fd: i32, timeout: i64) -> jint {
    unimplemented!()
}

#[no_mangle]
pub unsafe extern "system" fn JVM_Listen_impl(fd: jint, count: jint) -> jint {
    unimplemented!()
}

#[no_mangle]
pub unsafe extern "system" fn JVM_Connect_impl(fd: jint, him: *mut sockaddr, len: jint) -> jint {
    unimplemented!()
}

#[no_mangle]
pub unsafe extern "system" fn JVM_Bind_impl(fd: jint, him: *mut sockaddr, len: jint) -> jint {
    unimplemented!()
}

#[no_mangle]
pub unsafe extern "system" fn JVM_Accept_impl(
    fd: jint,
    him: *mut sockaddr,
    len: *mut jint,
) -> jint {
    unimplemented!()
}

#[no_mangle]
pub unsafe extern "system" fn JVM_RecvFrom_impl(
    fd: jint,
    buf: *mut u8,
    n_bytes: i32,
    flags: i32,
    from: *mut sockaddr,
    fromlen: *mut i32,
) -> jint {
    unimplemented!()
}

#[no_mangle]
pub unsafe extern "system" fn JVM_SendTo_impl(
    fd: jint,
    buf: *mut u8,
    len: i32,
    flags: i32,
    to: *mut sockaddr,
    tolen: i32,
) -> jint {
    unimplemented!()
}

#[no_mangle]
pub unsafe extern "system" fn JVM_SocketAvailable_impl(fd: jint, result: *mut jint) -> jint {
    unimplemented!()
}

#[no_mangle]
pub unsafe extern "system" fn JVM_GetSockName_impl(
    fd: jint,
    him: *mut sockaddr,
    len: *mut i32,
) -> jint {
    unimplemented!()
}

#[no_mangle]
pub unsafe extern "system" fn JVM_GetSockOpt_impl(
    fd: jint,
    level: i32,
    optname: i32,
    optval: *mut u8,
    optlen: *mut i32,
) -> jint {
    unimplemented!()
}

#[no_mangle]
pub unsafe extern "system" fn JVM_SetSockOpt_impl(
    fd: jint,
    level: i32,
    optname: i32,
    optval: *const u8,
    optlen: i32,
) -> jint {
    unimplemented!()
}

#[no_mangle]
pub unsafe extern "system" fn JVM_GetHostName_impl(name: *mut u8, namelen: i32) -> i32 {
    unimplemented!()
}

/*
 * The standard printing functions supported by the Java VM. (Should they
 * be renamed to JVM_* in the future?
 */

/*
 * BE CAREFUL! The following functions do not implement the
 * full feature set of standard C printf formats.
 */
#[no_mangle]
pub unsafe extern "C" fn jio_vsnprintf(
    str: *mut u8,
    count: usize,
    fmt: *const u8,
    args: va_list,
) -> i32 {
    unimplemented!()
}

// TODO: Fix variadic functions
// #[no_mangle]
// pub unsafe extern "C" fn jio_snprintf(str: *mut char,count:  usize ,fmt:  *const u8, ...) -> i32;

// #[no_mangle]
// pub unsafe extern "C" fn jio_fprintf(*mut FILE,fmt:  *const u8, ...) -> i32;

#[no_mangle]
pub unsafe extern "C" fn jio_vfprintf(fd: *mut c_void, fmt: *const u8, args: va_list) -> i32 {
    unimplemented!()
}

#[no_mangle]
pub unsafe extern "system" fn JVM_RawMonitorCreate_impl() -> *mut c_void {
    unimplemented!()
}

#[no_mangle]
pub unsafe extern "system" fn JVM_RawMonitorDestroy_impl(mon: *mut c_void) {
    unimplemented!()
}

#[no_mangle]
pub unsafe extern "system" fn JVM_RawMonitorEnter_impl(mon: *mut c_void) -> jint {
    unimplemented!()
}

#[no_mangle]
pub unsafe extern "system" fn JVM_RawMonitorExit_impl(mon: *mut c_void) {
    unimplemented!()
}

/*
 * java.lang.management support
 */
#[no_mangle]
pub unsafe extern "system" fn JVM_GetManagement_impl(version: jint) -> *mut c_void {
    unimplemented!()
}

/*
 * com.sun.tools.attach.VirtualMachine support
 *
 * Initialize the agent properties with the properties maintained in the VM.
 */
#[no_mangle]
pub unsafe extern "system" fn JVM_InitAgentProperties_impl(
    env: RawJNIEnv,
    agent_props: jobject,
) -> jobject {
    unimplemented!()
}

/* Generics reflection support.
 *
 * Returns information about the given class's EnclosingMethod
 * attribute, if present, or null if the class had no enclosing
 * method.
 *
 * If non-null, the returned array contains three elements. Element 0
 * is the java.lang.Class of which the enclosing method is a member,
 * and elements 1 and 2 are the java.lang.Strings for the enclosing
 * method's name and descriptor, respectively.
 */
#[no_mangle]
pub unsafe extern "system" fn JVM_GetEnclosingMethodInfo_impl(
    env: RawJNIEnv,
    of_class: Option<ObjectHandle>,
) -> Option<ObjectHandle> {
    let name = of_class?.unwrap_as_class();
    let mut lock = env.write();
    let class = lock.class_loader.class(&name)?;
    let (enclosing, method, desc) = class.enclosing_method()?;

    let enclosing_class = lock.class_instance(&enclosing);
    let method = lock.build_string(&method).expect_object();
    let desc = lock.build_string(&desc).expect_object();
    Some(ObjectHandle::array_from_data(vec![
        Some(enclosing_class),
        Some(method),
        Some(desc),
    ]))
}

/*
 * Java thread state support
 */
pub const JAVA_THREAD_STATE_NEW: jint = 0;
pub const JAVA_THREAD_STATE_RUNNABLE: jint = 1;
pub const JAVA_THREAD_STATE_BLOCKED: jint = 2;
pub const JAVA_THREAD_STATE_WAITING: jint = 3;
pub const JAVA_THREAD_STATE_TIMED_WAITING: jint = 4;
pub const JAVA_THREAD_STATE_TERMINATED: jint = 5;
pub const JAVA_THREAD_STATE_COUNT: jint = 6;

/*
 * Returns an array of the threadStatus values representing the
 * given Java thread state.  Returns NULL if the VM version is
 * incompatible with the JDK or doesn't support the given
 * Java thread state.
 */
#[no_mangle]
pub unsafe extern "system" fn JVM_GetThreadStateValues_impl(
    env: RawJNIEnv,
    java_thread_state: jint,
) -> jintArray {
    unimplemented!()
}

/*
 * Returns an array of the substate names representing the
 * given Java thread state.  Returns NULL if the VM version is
 * incompatible with the JDK or the VM doesn't support
 * the given Java thread state.
 * values must be the jintArray returned from JVM_GetThreadStateValues
 * and javaThreadState.
 */
#[no_mangle]
pub unsafe extern "system" fn JVM_GetThreadStateNames_impl(
    env: RawJNIEnv,
    java_thread_state: jint,
    values: jintArray,
) -> jobjectArray {
    unimplemented!()
}

/* =========================================================================
 * The following defines a private JVM interface that the JDK can query
 * for the JVM version and capabilities.  sun.misc.Version defines
 * the methods for getting the VM version and its capabilities.
 *
 * When a new bit is added, the following should be updated to provide
 * access to the new capability:
 *    HS:   JVM_GetVersionInfo and Abstract_VM_Version class
 *    SDK:  Version class
 *
 * Similary, a private JDK interface JDK_GetVersionInfo0 is defined for
 * JVM to query for the JDK version and capabilities.
 *
 * When a new bit is added, the following should be updated to provide
 * access to the new capability:
 *    HS:   JDK_Version class
 *    SDK:  JDK_GetVersionInfo0
 *
 * ==========================================================================
 */
#[repr(C)]
pub struct jvm_version_info {
    /* Naming convention of RE build version string: n.n.n[_uu[c]][-<identifier>]-bxx */
    jvm_version: u32,
    /* Consists of major, minor, micro (n.n.n) */
    /* and build number (xx) */
    // u32 update_version : 8;         /* Update release version (uu) */
    // u32 special_update_version : 8; /* Special update release version (c)*/
    // u32 reserved1 : 16;
    version_reserved1: u32,
    reserved2: u32,

    /* The following bits represents JVM supports that JDK has dependency on.
     * JDK can use these bits to determine which JVM version
     * and support it has to maintain runtime compatibility.
     *
     * When a new bit is added in a minor or update release, make sure
     * the new bit is also added in the main/baseline.
     */
    // u32 is_attach_supported : 1;
    // u32 : 31;
    is_attach_supported: u32,
    // u32 : 32;
    // u32 : 32;
    reserved3: u32,
    reserved4: u32,
}

// #define JVM_VERSION_MAJOR(version) ((version & 0xFF000000) >> 24)
// #define JVM_VERSION_MINOR(version) ((version & 0x00FF0000) >> 16)
// #define JVM_VERSION_MICRO(version) ((version & 0x0000FF00) >> 8)

/* Build number is available only for RE builds.
 * It will be zero for internal builds.
 */
// #define JVM_VERSION_BUILD(version) ((version & 0x000000FF))

#[no_mangle]
pub unsafe extern "system" fn JVM_GetVersionInfo_impl(
    env: RawJNIEnv,
    info: *mut jvm_version_info,
    info_size: usize,
) {
    write_bytes(info as *mut u8, 0, info_size);

    // Pretend to be java 16.0.0
    (*info).jvm_version = 16u32 << 24;
}

#[repr(C)]
pub struct jdk_version_info {
    // Naming convention of RE build version string: n.n.n[_uu[c]][-<identifier>]-bxx
    jdk_version: u32,
    /* Consists of major, minor, micro (n.n.n) */
    /* and build number (xx) */
    // u32 update_version : 8;         /* Update release version (uu) */
    // u32 special_update_version : 8; /* Special update release version (c)*/
    // u32 reserved1 : 16;
    version_reserved1: u32,
    reserved2: u32,

    /* The following bits represents new JDK supports that VM has dependency on.
     * VM implementation can use these bits to determine which JDK version
     * and support it has to maintain runtime compatibility.
     *
     * When a new bit is added in a minor or update release, make sure
     * the new bit is also added in the main/baseline.
     */
    // u32 thread_park_blocker : 1;
    // u32 post_vm_init_hook_enabled : 1;
    // u32 pending_list_uses_discovered_field : 1;
    // u32 : 29;
    thread_reserved: u32,
    // u32 : 32;
    // u32 : 32;
    reserved3: u32,
    reserved4: u32,
}

// #define JDK_VERSION_MAJOR(version) ((version & 0xFF000000) >> 24)
// #define JDK_VERSION_MINOR(version) ((version & 0x00FF0000) >> 16)
// #define JDK_VERSION_MICRO(version) ((version & 0x0000FF00) >> 8)

/* Build number is available only for RE build (i.e. JDK_BUILD_NUMBER is set to bNN)
 * It will be zero for internal builds.
 */
// #define JDK_VERSION_BUILD(version) ((version & 0x000000FF))

/*
 * This is the function JDK_GetVersionInfo0 defined in libjava.so
 * that is dynamically looked up by JVM.
 */

// type jdk_version_info_fn_t = unsafe extern "C" fn(info: jdk_version_info, info_size: usize);
// typedef void (*jdk_version_info_fn_t)(jdk_version_info* info, usize info_size);

/*
 * This structure is used by the launcher to get the default thread
 * stack size from the VM using JNI_GetDefaultJavaVMInitArgs() with a
 * version of 1.1.  As it is not supported otherwise, it has been removed
 * from jni.h
 */
#[repr(C)]
pub struct JDK1_1InitArgs {
    version: jint,

    properties: *mut *mut u8,
    check_source: jint,
    native_stack_size: jint,
    java_stack_size: jint,
    min_heap_size: jint,
    max_heap_size: jint,
    verify_mode: jint,
    classpath: *mut u8,

    vfprintf: unsafe extern "system" fn(fp: *mut c_void, format: *mut u8, args: va_list) -> jint,
    // jint (JNICALL *vfprintf)(FILE *fp, *const u8format, va_list args);
    exit: unsafe extern "system" fn(code: jint),
    abort: unsafe extern "system" fn(),
    // void (JNICALL *exit)(jint code);
    // void (JNICALL *abort)(void);
    enable_class_gc: jint,
    enable_verbose_gc: jint,
    disable_async_gc: jint,
    verbose: jint,
    debugging: jboolean,
    debug_port: jint,
}

extern "C" {
    #[allow(dead_code)]
    static JDK1_1InitArgs: JDK1_1InitArgs;
}

#[no_mangle]
pub unsafe extern "system" fn JVM_GetResourceLookupCacheURLs_impl(
    _env: RawJNIEnv,
    _loader: jobject,
) -> jobjectArray {
    warn!("JVM_GetResourceLookupCacheURLs is not supported on this JVM");
    // Return null to indicate the lookup cache is unavailable
    null_mut()
}

#[no_mangle]
pub unsafe extern "system" fn JVM_GetResourceLookupCache_impl(
    _env: RawJNIEnv,
    _loader: jobject,
    _resource_name: *const c_char,
) -> jintArray {
    warn!("JVM_GetResourceLookupCache is not supported on this JVM");
    null_mut()
}

#[no_mangle]
pub unsafe extern "system" fn JVM_KnownToNotExist_impl(
    _env: RawJNIEnv,
    _loader: jobject,
    _classname: *const c_char,
) -> jboolean {
    warn!("JVM_KnownToNotExist is not supported on this JVM");
    JNI_FALSE
}

#[no_mangle]
pub unsafe extern "system" fn JVM_Unsupported() -> u64 {
    error!("Attempted to call function not supported in Java 8");
    // 0  // Zero return register since we don't know what the call was for
    panic!()
}
