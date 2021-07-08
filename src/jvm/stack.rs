//! Maintains the opperand stack for the jvm. The primary objective of this system is to support the
//! native interface which requires stdcall support.
use std::ffi::c_void;
use std::mem::{size_of, transmute, zeroed};
use std::pin::Pin;

use jni::sys::{jobject, jvalue, JNIEnv, JNINativeInterface_, _jobject};

use crate::jvm::interface::build_interface;

pub struct OperandStack {
    stack: Vec<jvalue>,
    stack_top: usize,
    native_env: Pin<Box<JNINativeInterface_>>,
}

/// x86 calling convention notes:
///  1: %rdi
///  2: %rsi
///  3: %rdx
///  4: %rcx
///  5: %r8
///  6: %r9
///  >7: push to stack
///
///
///
/// Stack form:
///  0: Stack pointer
///  1: Stack base pointer
///  2: Return pointer
///  3: Arguments...
///
///
impl OperandStack {
    pub fn new(size: usize) -> Self {
        unsafe {
            OperandStack {
                stack: vec![zeroed(); size / size_of::<jvalue>()],
                stack_top: 0,
                native_env: Box::pin(build_interface()),
            }
        }
    }

    /// Loads values onto stack with at least enough padding for 6 values to be removed to place in
    /// registers.
    pub fn preload_stack(&mut self, args: Vec<jvalue>) {
        println!("Preloading stack");
        let mut index = self.stack.len() - (args.len()).max(6);
        self.stack_top = index;

        for value in args {
            self.stack[index] = value;
            index += 1;
        }
    }

    pub fn debug_info(&self) {
        println!("OperandStack top: {}/{}", self.stack_top, self.stack.len());
        unsafe {
            for idx in self.stack_top..self.stack.len() {
                println!("\tstack[{}] = {:0x}", idx, self.stack[idx].j);
            }
        }
    }

    unsafe fn native_call(&mut self, fn_ptr: *const c_void, args: Vec<jvalue>) -> jvalue {
        match args.len() {
            0 => transmute::<_, unsafe extern "C" fn() -> jvalue>(fn_ptr)(),
            1 => transmute::<_, unsafe extern "C" fn(jvalue) -> jvalue>(fn_ptr)(args[0]),
            2 => transmute::<_, unsafe extern "C" fn(jvalue, jvalue) -> jvalue>(fn_ptr)(
                args[0], args[1],
            ),
            3 => transmute::<_, unsafe extern "C" fn(jvalue, jvalue, jvalue) -> jvalue>(fn_ptr)(
                args[0], args[1], args[2],
            ),
            4 => transmute::<_, unsafe extern "C" fn(jvalue, jvalue, jvalue, jvalue) -> jvalue>(
                fn_ptr,
            )(args[0], args[1], args[2], args[3]),
            5 => transmute::<
                _,
                unsafe extern "C" fn(jvalue, jvalue, jvalue, jvalue, jvalue) -> jvalue,
            >(fn_ptr)(args[0], args[1], args[2], args[3], args[4]),
            6 => transmute::<
                _,
                unsafe extern "C" fn(jvalue, jvalue, jvalue, jvalue, jvalue, jvalue) -> jvalue,
            >(fn_ptr)(args[0], args[1], args[2], args[3], args[4], args[5]),
            7 => transmute::<
                _,
                unsafe extern "C" fn(
                    jvalue,
                    jvalue,
                    jvalue,
                    jvalue,
                    jvalue,
                    jvalue,
                    jvalue,
                ) -> jvalue,
            >(fn_ptr)(
                args[0], args[1], args[2], args[3], args[4], args[5], args[6],
            ),
            8 => transmute::<
                _,
                unsafe extern "C" fn(
                    jvalue,
                    jvalue,
                    jvalue,
                    jvalue,
                    jvalue,
                    jvalue,
                    jvalue,
                    jvalue,
                ) -> jvalue,
            >(fn_ptr)(
                args[0], args[1], args[2], args[3], args[4], args[5], args[6], args[7],
            ),
            9 => transmute::<
                _,
                unsafe extern "C" fn(
                    jvalue,
                    jvalue,
                    jvalue,
                    jvalue,
                    jvalue,
                    jvalue,
                    jvalue,
                    jvalue,
                    jvalue,
                ) -> jvalue,
            >(fn_ptr)(
                args[0], args[1], args[2], args[3], args[4], args[5], args[6], args[7], args[8],
            ),
            _ => {
                self.preload_stack(args);
                self.perform_raw_call(fn_ptr)
            }
        }
    }

    #[cfg(not(unix))]
    pub unsafe fn perform_raw_call(&self, _: *const c_void) -> jvalue {
        unimplemented!("Raw function calls are only supported on unix systems!")
    }

    #[cfg(unix)]
    pub unsafe fn perform_raw_call(&self, fn_ptr: *const c_void) -> jvalue {
        use crate::jvm::exec::exec_x86_with_stack;

        println!("About to perform call!");
        let rsp = &self.stack[self.stack_top] as *const _ as *const c_void;
        // let rbp = &self.stack[self.stack.len() - 1] as *const _ as *const c_void;
        let rbp = self.stack.as_ptr().add(self.stack.len()) as *const c_void;
        println!("rsp and rbp created!");
        println!("Calling function at {:p}", fn_ptr);
        // forget(rbp);
        exec_x86_with_stack(fn_ptr, rbp, rsp)
    }

    pub unsafe fn native_static_call(
        &mut self,
        fn_ptr: *const c_void,
        mut args: Vec<jvalue>,
    ) -> jvalue {
        // Push JNIEnv
        // self.stack_top -= 1;
        let env = &*self.native_env as JNIEnv;
        let env_ptr = &env as *const JNIEnv;

        // let value = jvalue {
        //     l: transmute(env_ptr),
        // };
        let value = jvalue {
            l: env_ptr as *mut _jobject,
        };
        args.insert(0, value);
        // self.stack[self.stack_top] = value;

        self.native_call(fn_ptr, args)
    }

    pub unsafe fn native_method_call(
        &mut self,
        fn_ptr: *const c_void,
        object: jobject,
        mut args: Vec<jvalue>,
    ) -> jvalue {
        args.insert(0, jvalue { l: object });
        self.native_static_call(fn_ptr, args)
    }
}

impl Default for OperandStack {
    fn default() -> Self {
        OperandStack::new(16384)
    }
}

#[test]
pub fn basic_functionality() {
    println!("Testing basic_functionality");
    extern "C" fn subtract(a: i32, b: i32) -> i32 {
        println!("A: {}, B: {}", a, b);
        a - b
    }

    unsafe {
        let mut stack = OperandStack::default();
        let args = vec![jvalue { i: 7 }, jvalue { i: 13 }];
        let fn_ptr = subtract as *const c_void;

        let result = stack.native_call(fn_ptr, args);
        assert_eq!(result.i, -6);
    }
}

// #[test]
// pub fn many_args() {
//     println!("Testing many_args");
//     extern "C" fn add(a: i32, b: i32, c: i32, d: i32, e: i32, f: i32, g: i32, h: i32, i: i32, j: i32) -> i32 {
//         a + b + c + d + e + f + g + h + i + j
//     }
//
//     unsafe {
//         let mut stack = OperandStack::default();
//         let mut args = Vec::new();
//         args.push(jvalue { i: 1 });
//         args.push(jvalue { i: 5 });
//         args.push(jvalue { i: 11 });
//         args.push(jvalue { i: 37 });
//         args.push(jvalue { i: 19 });
//         args.push(jvalue { i: -36 });
//         args.push(jvalue { i: 22 });
//         args.push(jvalue { i: -1 });
//         args.push(jvalue { i: -1234 });
//         args.push(jvalue { i: 107 });
//
//         let result = stack.native_call(add as *const c_void, args);
//         assert_eq!(result.i, 1 + 5 + 11 + 37 + 19 - 36 + 22 - 1 - 1234 + 107);
//     }
// }
//
