use std::ffi::c_void;

use jni::sys::jvalue;

extern "C" {

    /// About as unsafe as it gets. I wrote it myself in assembly and fingers crossed the linker is
    /// working.
    pub fn exec_x86_with_stack(
        fn_ptr: *const c_void,
        rbp: *const c_void,
        rsp: *const c_void,
    ) -> jvalue;
}

#[test]
pub fn simple_asm_test() {
    use std::mem::size_of;

    println!("Testing simple_asm_test");
    extern "C" fn add(a: i32, b: i32) -> i32 {
        println!("A: {}, B: {}", a, b);
        a + b
    }

    unsafe {
        let mut stack = vec![0u64; 512];
        stack[506] = 7;
        stack[507] = 13;

        let rsp = &stack[506] as *const _ as *const c_void;
        let rbp = rsp.add(size_of::<[u64; 6]>());
        println!("rsp: {:?}", rsp);
        println!("rbp: {:?}", rbp);

        let fn_ptr = add as *const c_void;
        let out = exec_x86_with_stack(fn_ptr, rbp, rsp);
        assert_eq!(out.i, 20);
    }
}
