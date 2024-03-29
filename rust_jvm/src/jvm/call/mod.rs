mod interface;
/// Java Calling Convention:
///  - Local Variable table is maintained between calls
///  - Returned value is placed on operand stack of previous call
///
/// Invocation Types:
///   - Virtual: The standard call invocation of an instance method. Target object ref is taken from
///     stack and call implementation based on the target class's vtable.
///   - Static: Similar to virtual, but the class is determined from the constant table so there is
///     no object ref to place in local variable table.
///   - Interface: Invokes an interface method on an unknown class implementing that interface.
///     Requires special lookup since the regular vtable can not be referenced directly.
///   - Special: Similar to virtual, but call implementation of class specified in constant table.
///   - Dynamic: Use reflection to find the call site.
///
/// TODO: Signature Polymorphic (§2.9.3) are not yet supported
///
mod interpreter;
mod native;

#[cfg(feature = "callstack")]
pub mod callstack_trace;
mod ffi;

use crate::class::constant::ClassElement;
use crate::class::AccessFlags;
use crate::jvm::mem::{JavaValue, ObjectHandle, ObjectReference};
use crate::jvm::thread::{handle_thread_updates, SynchronousMonitor};
use crate::jvm::JavaEnv;
pub use interface::build_interface;
pub use interpreter::*;
use jni::sys::JNIEnv;
use log::Level;
pub use native::*;
use parking_lot::RwLock;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;

pub trait Method: 'static {
    fn exec(&self, jvm: &mut JavaEnv, args: &[JavaValue]) -> Result<Option<JavaValue>, JavaValue>;
}

#[derive(Debug)]
pub enum FlowControl {
    Branch(i64),
    Return(Option<JavaValue>),
    Throws(Option<ObjectHandle>),
    ThreadInterrupt,
}

impl FlowControl {
    #[track_caller]
    pub fn error<S: AsRef<str>>(class: S) -> Self {
        debug!("Attempted to build error: {}", class.as_ref());
        unimplemented!("Attempted to build error: {}", class.as_ref())
    }

    #[track_caller]
    pub fn throw<S: AsRef<str>>(class: S) -> Self {
        debug!("Attempted to build exception: {}", class.as_ref());
        unimplemented!("Attempted to build exception: {}", class.as_ref())
    }
}

// pub struct JavaVTable {
//     fns: Vec<Box<dyn Method>>,
// }

/// TODO: If the method is synchronized, the monitor associated with the resolved Class object is
/// entered or reentered as if by execution of a monitorenter instruction (§monitorenter) in the
/// current thread.
///
/// TODO: Support invokedynamic, invokeinterface, invokespecial, invokestatic, and invokevirtual
// impl JavaVTable {
//     // TODO: Impl index instead
//     pub fn fn_from_offset(&self, index: usize) -> &dyn Method {
//         &*self.fns[index]
//     }
//
//     /// Call a regular instance method
//     /// ... [arg1, [arg2 ...]] ->
//     /// ... [result]
//     ///
//     pub fn invoke_virtual(&self, index: usize) {
//         unimplemented!()
//     }
//
//     /// Invokes the superclass implementation of a method or the default implementation of an
//     /// interface.
//     pub fn invoke_special(&self, index: usize) {
//         unimplemented!()
//     }
//
//     pub fn invoke_interface(&self, index: usize) {
//         unimplemented!()
//     }
//
//     /// TODO: This will be the hardest to implement. Come back to later.
//     /// Invoke a dynamically-computed call site. Formed from CONSTANT_InvokeDynamic_info.
//     /// ... [arg1, [arg2 ...]] ->
//     /// ... [result]
//     ///
//     /// The symbolic reference is resolved (§5.4.3.6) for this specific invokedynamic instruction to
//     /// obtain a reference to an instance of java.lang.invoke.CallSite. The instance of
//     /// java.lang.invoke.CallSite is considered "bound" to this specific invokedynamic instruction.
//     ///
//     pub fn invoke_dynamic(&self, index: usize) {
//         unimplemented!()
//     }
//
//     /// Invoke a class (static) method
//     /// ... [arg1, [arg2 ...]] ->
//     /// ... [result]
//     ///
//     /// The nargs argument values are consecutively made the values of local variables of the new
//     /// frame, with arg1 in local variable 0 (or, if arg1 is of type long or double, in local
//     /// variables 0 and 1) and so on. Any argument value that is of a floating-point type undergoes
//     /// value set conversion (§2.8.3) prior to being stored in a local variable.
//     ///
//     /// Example:
//     /// ```java
//     /// public static long add(int a, int b) {
//     ///     return (long) (a + b);
//     /// }
//     /// ```
//     pub fn invoke_static(&self, index: usize) {
//         unimplemented!()
//     }
//
//     // pub fn perform(&self, index: usize, jvm: &mut JavaEnv, target: )
// }

pub trait JavaEnvInvoke {
    fn init_class(&mut self, class: &str);

    fn invoke(
        &mut self,
        element: ClassElement,
        locals: Vec<JavaValue>,
    ) -> Result<Option<JavaValue>, FlowControl>;

    fn invoke_special(
        &mut self,
        method: ClassElement,
        target: ObjectHandle,
        args: Vec<JavaValue>,
    ) -> Result<Option<JavaValue>, FlowControl>;

    fn invoke_virtual(
        &mut self,
        method: ClassElement,
        target: ObjectHandle,
        args: Vec<JavaValue>,
    ) -> Result<Option<JavaValue>, FlowControl>;

    fn invoke_static(
        &mut self,
        method: ClassElement,
        args: Vec<JavaValue>,
    ) -> Result<Option<JavaValue>, FlowControl>;
}

impl JavaEnvInvoke for Arc<RwLock<JavaEnv>> {
    fn init_class(&mut self, class: &str) {
        #[cfg(feature = "profile")]
        let mut profile_scope = thread_profiler::ProfileScope::new("init_class".into());

        let class_instance = {
            let mut jvm = self.write();
            jvm.class_instance(class)
        };

        self.lock(class_instance);
        if !self.read().static_load.contains(class) {
            {
                let mut jvm = self.write();
                // jvm.class_loader.attempt_load(class).unwrap();
                jvm.static_load.insert(class.to_string());
            }

            if class != "java/lang/Object" {
                let super_class = self
                    .write()
                    .class_loader
                    .class(class)
                    .unwrap()
                    .super_class();
                self.init_class(&super_class);
            }

            let method = {
                let jvm = self.write();
                let instance = jvm.class_loader.class(class).unwrap();

                instance
                    .get_method("<clinit>", "()V")
                    .map(|_| ClassElement::new(class, "<clinit>", "()V"))
            };

            if let Some(method_ref) = method {
                // This is the most common place for a panic to occur
                if let Err(x) = self.invoke_static(method_ref, vec![]) {
                    let (properties, stdout) = {
                        let jvm = self.write();
                        let properties = jvm
                            .static_fields
                            .get_static("java/lang/System", "props")
                            .unwrap()
                            .expect_object();
                        let stdout = jvm
                            .static_fields
                            .get_static("java/lang/System", "out")
                            .unwrap();
                        (properties, stdout)
                    };

                    // Attempt to print properties to stdout before panic
                    let element = ClassElement::new(
                        "java/util/Properties",
                        "store",
                        "(Ljava/io/OutputStream;Ljava/lang/String;)V",
                    );
                    self.invoke_virtual(
                        element,
                        properties,
                        vec![stdout, JavaValue::Reference(None)],
                    )
                    .unwrap();

                    panic!("Error during <clinit> of {}: {:?}", class, x);
                }
            }
            // let instance = self.write().class_loader.class(class).unwrap();
            // if instance.get_method("<clinit>", "()V").is_some() {
            //     let method = ClassElement::new(class, "<clinit>", "()V");
            //     self.invoke_static(method, vec![]).unwrap();
            //     // self.exec_static(class, "<clinit>", "()V", vec![]).unwrap();
            // }
        }
        self.unlock(class_instance);

        // FIXME: This is a bad approach that won't be able to find classes with additional nesting.
        // It would be better to check for an InnerClass attribute
        if let Some(x) = class.find('$') {
            self.init_class(&class[..x])
        }

        #[cfg(feature = "profile")]
        std::mem::drop(profile_scope);
    }

    fn invoke(
        &mut self,
        element: ClassElement,
        mut locals: Vec<JavaValue>,
    ) -> Result<Option<JavaValue>, FlowControl> {
        // Check for sticky actions on current thread
        handle_thread_updates(self)?;

        if log::max_level() >= Level::Trace {
            trace!("Running {:?}", &element);
            // for (idx, local) in locals.iter().enumerate() {
            //     debug!("\t{}: {:?}", idx, local);
            // }
            self.read().debug_print_call_stack();
        }
        if !StackFrame::verify_computational_types(&locals) {
            error!("Failed local verification");
            for (idx, local) in locals.iter().enumerate() {
                error!("\t{}/{}: {:?}", idx, locals.len(), local);
            }
            panic!("Failed local verification");
        }
        let (class_name, method) =
            match self
                .read()
                .find_instance_method(&element.class, &element.element, &element.desc)
            {
                Some(v) => v,
                _ => panic!("Unable to find {:?}", element),
            };

        let class_constants = self
            .read()
            .class_loader
            .class(&class_name)
            .unwrap()
            .constants
            .to_owned();

        {
            let mut jvm = self.write();
            let class = jvm.class_instance(&class_name);
            jvm.thread_manager
                .push_call_stack(class, element.clone(), &locals);
            // jvm.call_stack.push((class, format!("{:?}", &method)));
        }

        let ret = if method.access.contains(AccessFlags::NATIVE) {
            // If attempting to call a native method, the class must be initialized first
            self.init_class(&class_name);

            let fn_ptr = match self.write().linked_libraries.get_fn_ptr(
                &class_name,
                &element.element,
                &element.desc,
            ) {
                Some(v) => v,
                None => panic!("Unable to find function {:?}", element),
            };
            let native_call = NativeCall::new(fn_ptr, element.build_desc());

            // Native static methods require the class
            let target = if method.access.contains(AccessFlags::STATIC) {
                self.write().class_instance(&class_name)
            } else {
                match locals.remove(0) {
                    JavaValue::Reference(Some(v)) => v,
                    _ => return Err(FlowControl::throw("java/lang/NullPointerException")),
                }
            };

            native_call.exec(self, target, locals)
        } else {
            let instructions = method.code(&class_constants);
            let mut frame = StackFrame::new(
                instructions.max_locals as usize,
                instructions.max_stack as usize,
                &class_constants,
                locals,
            );
            frame.exec(self, &instructions)
        };

        self.write().thread_manager.pop_call_stack(&ret);

        match ret {
            Err(FlowControl::Return(v)) => Ok(v),
            x => x,
        }
    }

    fn invoke_special(
        &mut self,
        method: ClassElement,
        target: ObjectHandle,
        mut args: Vec<JavaValue>,
    ) -> Result<Option<JavaValue>, FlowControl> {
        #[cfg(feature = "profile")]
        let profile_scope = thread_profiler::ProfileScope::new(format!("special {:?}", &method));

        assert!(self
            .read()
            .instanceof(&target.get_class(), &method.class)
            .unwrap());

        if !StackFrame::verify_computational_types(&args) {
            error!("Failed local verification");
            for (idx, local) in args.iter().enumerate() {
                error!("\t{}/{}: {:?}", idx, args.len(), local);
            }
            self.write().debug_print_call_stack();
            panic!("Failed local verification");
        }
        args.insert(0, JavaValue::Reference(Some(target)));
        #[cfg(feature = "profile")]
        {
            let ret = self.invoke(method, args);
            std::mem::drop(profile_scope);
            ret
        }

        #[cfg(not(feature = "profile"))]
        self.invoke(method, args)
    }

    fn invoke_virtual(
        &mut self,
        mut method: ClassElement,
        target: ObjectHandle,
        mut args: Vec<JavaValue>,
    ) -> Result<Option<JavaValue>, FlowControl> {
        #[cfg(feature = "profile")]
        let profile_scope = thread_profiler::ProfileScope::new(format!("virtual {:?}", &method));

        method.class = target.get_class();
        args.insert(0, JavaValue::Reference(Some(target)));
        #[cfg(feature = "profile")]
        {
            let ret = self.invoke(method, args);
            std::mem::drop(profile_scope);
            ret
        }

        #[cfg(not(feature = "profile"))]
        self.invoke(method, args)
    }

    fn invoke_static(
        &mut self,
        method: ClassElement,
        args: Vec<JavaValue>,
    ) -> Result<Option<JavaValue>, FlowControl> {
        #[cfg(feature = "profile")]
        {
            let mut profile_scope =
                thread_profiler::ProfileScope::new(format!("static {:?}", &method));
            let ret = self.invoke(method, args);
            std::mem::drop(profile_scope);
            ret
        }

        #[cfg(not(feature = "profile"))]
        self.invoke(method, args)
    }
}

#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct RawJNIEnv<'a> {
    ptr: *mut JNIEnv,
    _phantom: PhantomData<&'a ()>,
}

impl<'a> RawJNIEnv<'a> {
    pub fn new(ptr: *mut JNIEnv) -> Self {
        RawJNIEnv {
            ptr,
            _phantom: PhantomData,
        }
    }

    pub fn write_thrown(&self, throwable: Option<ObjectHandle>) {
        self.write().thread_manager.set_sticky_exception(throwable)
    }

    pub fn read_thrown(&self) -> Option<ObjectHandle> {
        self.read().thread_manager.get_sticky_exception()
    }
}

impl<'a> Deref for RawJNIEnv<'a> {
    type Target = Arc<RwLock<JavaEnv>>;

    fn deref(&self) -> &Self::Target {
        unsafe {
            let jvm = (**self.ptr).reserved0 as *mut Arc<RwLock<JavaEnv>>;
            &*jvm
        }
    }
}

impl<'a> DerefMut for RawJNIEnv<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            let jvm = (**self.ptr).reserved0 as *mut Arc<RwLock<JavaEnv>>;
            &mut *jvm
        }
    }
}
