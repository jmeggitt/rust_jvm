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
mod stack;

use crate::class::AccessFlags;
use crate::constant_pool::ClassElement;
use crate::jvm::mem::{JavaValue, ObjectHandle, ObjectReference};
use crate::jvm::thread::handle_thread_updates;
use crate::jvm::JavaEnv;
use crate::profile_scope_cfg;
pub use interface::build_interface;
pub use interpreter::*;
use jni::sys::JNIEnv;
pub use native::*;
use parking_lot::RwLock;
pub use stack::*;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::ptr::null_mut;
use std::sync::Arc;

pub trait Method: 'static {
    fn exec(&self, jvm: &mut JavaEnv, args: &[JavaValue]) -> Result<Option<JavaValue>, JavaValue>;
}

#[repr(transparent)]
pub struct CallSite {
    reference: ClassElement,
}

#[derive(Debug)]
pub enum FlowControl {
    Branch(i64),
    Return(Option<JavaValue>),
    Throws(Option<ObjectHandle>),
    ThreadInterrupt,
}

impl FlowControl {
    pub fn error<S: AsRef<str>>(class: S) -> Self {
        debug!("Attempted to build error: {}", class.as_ref());
        unimplemented!("Attempted to build error: {}", class.as_ref())
    }

    pub fn throw<S: AsRef<str>>(class: S) -> Self {
        debug!("Attempted to build exception: {}", class.as_ref());
        unimplemented!("Attempted to build exception: {}", class.as_ref())
    }
}

pub struct JavaVTable {
    fns: Vec<Box<dyn Method>>,
}

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
        if !self.read().static_load.contains(class) {
            {
                let mut jvm = self.write();
                jvm.class_loader.attempt_load(class).unwrap();
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

                match instance.get_method("<clinit>", "()V") {
                    Some(_) => Some(ClassElement::new(class, "<clinit>", "()V")),
                    None => None,
                }
            };

            if let Some(method_ref) = method {
                self.invoke_static(method_ref, vec![]).unwrap();
            }
            // let instance = self.write().class_loader.class(class).unwrap();
            // if instance.get_method("<clinit>", "()V").is_some() {
            //     let method = ClassElement::new(class, "<clinit>", "()V");
            //     self.invoke_static(method, vec![]).unwrap();
            //     // self.exec_static(class, "<clinit>", "()V", vec![]).unwrap();
            // }
        }
    }

    fn invoke(
        &mut self,
        element: ClassElement,
        mut locals: Vec<JavaValue>,
    ) -> Result<Option<JavaValue>, FlowControl> {
        // Check for sticky actions on current thread
        handle_thread_updates(self)?;

        debug!("Running {:?}", &element);
        for (idx, local) in locals.iter().enumerate() {
            debug!("\t{}: {:?}", idx, local);
        }
        self.read().debug_print_call_stack();
        StackFrame::verify_computational_types(&locals);
        let (class_name, method, constants) =
            match self
                .read()
                .find_instance_method(&element.class, &element.element, &element.desc)
            {
                Some(v) => v,
                _ => panic!("Unable to find {:?}", element),
            };

        {
            let mut jvm = self.write();
            let class = jvm.class_instance(&class_name);
            jvm.thread_manager.push_call_stack(class, element.clone());
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

            unsafe { native_call.exec(self, target, locals) }
        } else {
            let instructions = method.code(&constants);
            let mut frame = StackFrame::new(
                instructions.max_locals as usize,
                instructions.max_stack as usize,
                constants,
                locals,
            );
            frame.exec(self, &instructions)
        };

        self.write().thread_manager.pop_call_stack();

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
        profile_scope_cfg!("special {:?}", &method);

        assert!(self
            .read()
            .instanceof(&target.get_class(), &method.class)
            .unwrap());

        // if !self.instanceof(&method.class, &target.get_class()).unwrap() {
        //     panic!("Expected: {:?}, Got: {:?}", &method.class, &target.get_class());
        // }

        // let descriptor = match FieldDescriptor::read_str(&method.desc).unwrap() {
        //     FieldDescriptor::Method {args, ..} => args,
        //     _ => panic!("Called method with non method element!"),
        // };

        // let mut locals_idx = 0;
        // for idx in 0..descriptor.len() {
        //     match descriptor[idx].assign_from(args[locals_idx]) {
        //         Some(JavaValue::Long(x)) => {
        //             args[locals_idx] = JavaValue::Long(x);
        //             args[locals_idx + 1] = JavaValue::Long(x);
        //             locals_idx += 1;
        //         },
        //         Some(JavaValue::Double(x)) => {
        //             args[locals_idx] = JavaValue::Double(x);
        //             args[locals_idx + 1] = JavaValue::Double(x);
        //             locals_idx += 1;
        //         }
        //         Some(x) => args[idx] = x,
        //         None => panic!("Expected: {:?}, Got: {:?}", &descriptor, &args),
        //     };
        //     locals_idx += 1;
        // }

        StackFrame::verify_computational_types(&args);

        // {
        //     let mut jvm = self.write();
        //     let class = jvm.class_instance(&target.get_class());
        //     jvm.call_stack.push((class, format!("{:?}", &method)));
        // }
        args.insert(0, JavaValue::Reference(Some(target)));
        let ret = self.invoke(method, args);
        // self.write().call_stack.pop().unwrap();
        ret
    }

    fn invoke_virtual(
        &mut self,
        mut method: ClassElement,
        target: ObjectHandle,
        mut args: Vec<JavaValue>,
    ) -> Result<Option<JavaValue>, FlowControl> {
        profile_scope_cfg!("virtual {:?}", &method);

        assert!(self
            .read()
            .instanceof(&target.get_class(), &method.class)
            .unwrap());

        // if !self.instanceof(&target.get_class(), &method.class).unwrap() {
        //     panic!("Expected: {:?}, Got: {:?}", &method.class, &target.get_class());
        // }

        // let descriptor = match FieldDescriptor::read_str(&method.desc).unwrap() {
        //     FieldDescriptor::Method { args, .. } => args,
        //     _ => panic!("Called method with non method element!"),
        // };

        // let mut locals_idx = 0;
        // for idx in 0..descriptor.len() {
        //     match descriptor[idx].assign_from(args[locals_idx]) {
        //         Some(JavaValue::Long(x)) => {
        //             args[locals_idx] = JavaValue::Long(x);
        //             args[locals_idx+1] = JavaValue::Long(x);
        //             locals_idx += 1;
        //         },
        //         Some(JavaValue::Double(x)) => {
        //             args[locals_idx] = JavaValue::Double(x);
        //             args[locals_idx + 1] = JavaValue::Double(x);
        //             locals_idx += 1;
        //         }
        //         Some(x) => args[idx] = x,
        //         None => panic!("Expected: {:?}, Got: {:?}", &descriptor, &args),
        //     };
        //     locals_idx += 1;
        // }

        method.class = target.get_class();

        // {
        //     let mut jvm = self.write();
        //     let class = jvm.class_instance(&target.get_class());
        //     jvm.call_stack.push((class, format!("{:?}", &method)));
        // }
        args.insert(0, JavaValue::Reference(Some(target)));
        let ret = self.invoke(method, args);
        // self.write().call_stack.pop().unwrap();
        ret
    }

    fn invoke_static(
        &mut self,
        method: ClassElement,
        args: Vec<JavaValue>,
    ) -> Result<Option<JavaValue>, FlowControl> {
        profile_scope_cfg!("static {:?}", &method);

        // let descriptor = match FieldDescriptor::read_str(&method.desc).unwrap() {
        //     FieldDescriptor::Method {args, ..} => args,
        //     _ => panic!("Called method with non method element!"),
        // };

        // let mut locals_idx = 0;
        // for idx in 0..descriptor.len() {
        //     match descriptor[idx].assign_from(args[locals_idx]) {
        //         Some(JavaValue::Long(x)) => {
        //             args[locals_idx] = JavaValue::Long(x);
        //             args[locals_idx + 1] = JavaValue::Long(x);
        //             locals_idx += 1;
        //         },
        //         Some(JavaValue::Double(x)) => {
        //             args[locals_idx + 1] = JavaValue::Double(x);
        //             locals_idx += 1;
        //         }
        //         Some(x) => args[idx] = x,
        //         None => panic!("Expected: {:?}, Got: {:?}", &descriptor, &args),
        //     };
        //     locals_idx += 1;
        // }

        // {
        //     let mut jvm = self.write();
        //     let class = jvm.class_instance(&method.class);
        //     jvm.call_stack.push((class, format!("{:?}", &method)));
        // }
        let ret = self.invoke(method, args);
        // self.write().call_stack.pop().unwrap();
        ret
    }

    // pub fn exec(
    //     &mut self,
    //     target: ObjectHandle,
    //     class_name: &str,
    //     method: MethodInfo,
    //     constants: Vec<Constant>,
    //     mut args: Vec<JavaValue>,
    // ) -> Result<Option<JavaValue>, FlowControl> {
    //     let call_string = format!(
    //         "{}::{} {}",
    //         class_name,
    //         method.name(&constants).unwrap(),
    //         method.descriptor(&constants).unwrap()
    //     );
    //     debug!("Executing method {} for target {:?}", &call_string, &target);
    //
    //     let target_class = self.class_instance(class_name);
    //     self.call_stack.push((target_class, call_string));
    //     self.debug_print_call_stack();
    //
    //     let ret = if method.access.contains(AccessFlags::NATIVE) {
    //         let fn_ptr = match self.linked_libraries.get_fn_ptr(
    //             class_name,
    //             &method.name(&constants).unwrap(),
    //             &method.descriptor(&constants).unwrap(),
    //         ) {
    //             Some(v) => v,
    //             None => panic!(
    //                 "Unable to find function ptr {}::{} {}",
    //                 class_name,
    //                 &method.name(&constants).unwrap(),
    //                 &method.descriptor(&constants).unwrap()
    //             ),
    //         };
    //
    //         if let Ok(FieldDescriptor::Method { returns, .. }) =
    //         FieldDescriptor::read_str(&method.descriptor(&constants).unwrap())
    //         {
    //             unimplemented!(
    //                 "Native methods will be implemented once libffi integration is completed"
    //             )
    //             // let ret = unsafe {
    //             //     debug!("Native method arguments:");
    //             //     let raw_args = args
    //             //         .iter_mut()
    //             //         .map(|x| {
    //             //             debug!("\t{:?}", x);
    //             //             match x {
    //             //                 JavaValue::Reference(Some(v)) => jvalue {
    //             //                     l: v as *mut _ as jobject,
    //             //                 },
    //             //                 x => {
    //             //                     let value: jvalue = x.clone().into();
    //             //                     value
    //             //                 }
    //             //             }
    //             //         })
    //             //         .collect();
    //             //
    //             //     self.native_stack.native_method_call(
    //             //         fn_ptr,
    //             //         &target as *const _ as jobject,
    //             //         raw_args,
    //             //     )
    //             // };
    //             //
    //             // Ok(returns.cast(ret))
    //         } else {
    //             panic!("Method descriptor can not be correctly parsed")
    //         }
    //     } else {
    //         args.insert(0, JavaValue::Reference(Some(target.clone())));
    //         let code = method.code(&constants);
    //         let mut frame = StackFrame::new(
    //             code.max_locals as usize,
    //             code.max_stack as usize,
    //             constants,
    //             args,
    //         );
    //         frame.exec(self, &code)
    //     };
    //
    //     self.call_stack.pop();
    //     ret
    // }
    //
    // pub fn exec_method(
    //     &mut self,
    //     target: ObjectHandle,
    //     method: &str,
    //     desc: &str,
    //     args: Vec<JavaValue>,
    // ) -> Result<Option<JavaValue>, FlowControl> {
    //     let class = target.get_class();
    //
    //     let (class_name, main_method, constants) =
    //         match self.find_instance_method(&class, method, desc) {
    //             Some(v) => v,
    //             _ => fatal_error!("Unable to find {}::{} {}", class, method, desc),
    //         };
    //
    //     self.exec(target, &class_name, main_method, constants, args)
    // }
    //
    // pub fn exec_static(
    //     &mut self,
    //     class: &str,
    //     method: &str,
    //     desc: &str,
    //     args: Vec<JavaValue>,
    // ) -> Result<Option<JavaValue>, FlowControl> {
    //     let target = self.class_instance(class);
    //     self.exec_method(target, method, desc, args)
    // }
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
        // lock.
        // (&mut **(self.ptr as *mut *mut JNINativeInterface_)).reserved1 = match throwable {
        //     None => null_mut(),
        //     Some(v) => v.ptr() as _,
        // }
    }

    pub fn read_thrown(&self) -> Option<ObjectHandle> {
        self.read().thread_manager.get_sticky_exception()
        // let ptr = (&mut **(self.ptr as *mut *mut JNINativeInterface_)).reserved1 as jthrowable;
        // ObjectHandle::from_ptr(ptr)
    }
}

impl<'a> Deref for RawJNIEnv<'a> {
    type Target = Arc<RwLock<JavaEnv>>;

    fn deref(&self) -> &Self::Target {
        unsafe {
            let jvm = (&**self.ptr).reserved0 as *mut Arc<RwLock<JavaEnv>>;
            &*jvm
        }
    }
}

impl<'a> DerefMut for RawJNIEnv<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            let jvm = (&**self.ptr).reserved0 as *mut Arc<RwLock<JavaEnv>>;
            &mut *jvm
        }
    }
}
