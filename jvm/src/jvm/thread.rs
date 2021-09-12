//! This module handles all of the threading and synchronous activities of the jvm
#![allow(unused_variables)]

use crate::jvm::call::RawJNIEnv;
use crate::jvm::mem::ObjectHandle;
use hashbrown::HashMap;
use jni::sys::{jboolean, jclass, jint, jlong, jobject, jobjectArray, jstring, JNI_FALSE};
use std::thread::{Thread, ThreadId};

pub struct JavaThreadManager {
    java_threads: HashMap<Thread, ObjectHandle>,
    thread_handles: HashMap<ObjectHandle, Thread>,
}

impl JavaThreadManager {}

#[no_mangle]
pub unsafe extern "system" fn JVM_StartThread_impl(env: RawJNIEnv, thread: jobject) {
    unimplemented!()
}

#[no_mangle]
pub unsafe extern "system" fn JVM_StopThread_impl(
    env: RawJNIEnv,
    thread: jobject,
    exception: jobject,
) {
    unimplemented!()
}

#[no_mangle]
pub unsafe extern "system" fn JVM_IsThreadAlive_impl(env: RawJNIEnv, thread: jobject) -> jboolean {
    JNI_FALSE
    // TODO: Actually handle threading in the future
}

#[no_mangle]
pub unsafe extern "system" fn JVM_SuspendThread_impl(env: RawJNIEnv, thread: jobject) {
    // TODO: Actually handle threading in the future
}

#[no_mangle]
pub unsafe extern "system" fn JVM_ResumeThread_impl(env: RawJNIEnv, thread: jobject) {
    // TODO: Actually handle threading in the future
}

#[no_mangle]
pub unsafe extern "system" fn JVM_SetThreadPriority_impl(
    env: RawJNIEnv,
    thread: jobject,
    prio: jint,
) {
    // TODO: Actually handle threading in the future
}

#[no_mangle]
pub unsafe extern "system" fn JVM_Yield_impl(env: RawJNIEnv, thread_class: jclass) {
    unimplemented!()
}

#[no_mangle]
pub unsafe extern "system" fn JVM_Sleep_impl(env: RawJNIEnv, thread_class: jclass, millis: jlong) {
    unimplemented!()
}

#[no_mangle]
pub unsafe extern "system" fn JVM_CurrentThread_impl(
    env: RawJNIEnv,
    thread_class: jclass,
) -> jobject {
    unimplemented!()
}

#[no_mangle]
pub unsafe extern "system" fn JVM_CountStackFrames_impl(env: RawJNIEnv, thread: jobject) -> jint {
    env.read().call_stack.len() as jint
}

#[no_mangle]
pub unsafe extern "system" fn JVM_Interrupt_impl(env: RawJNIEnv, thread: jobject) {
    unimplemented!()
}

#[no_mangle]
pub unsafe extern "system" fn JVM_IsInterrupted_impl(
    env: RawJNIEnv,
    thread: jobject,
    clear_interrupted: jboolean,
) -> jboolean {
    unimplemented!()
}

#[no_mangle]
pub unsafe extern "system" fn JVM_HoldsLock_impl(
    env: RawJNIEnv,
    thread_class: jclass,
    obj: jobject,
) -> jboolean {
    unimplemented!()
}

#[no_mangle]
pub unsafe extern "system" fn JVM_DumpAllStacks_impl(env: RawJNIEnv, unused: jclass) {
    unimplemented!()
}

#[no_mangle]
pub unsafe extern "system" fn JVM_GetAllThreads_impl(
    env: RawJNIEnv,
    dummy: jclass,
) -> jobjectArray {
    unimplemented!()
}

#[no_mangle]
pub unsafe extern "system" fn JVM_SetNativeThreadName_impl(
    env: RawJNIEnv,
    jthread: jobject,
    name: jstring,
) {
    unimplemented!()
}

/* getStackTrace_impl() and getAllStackTraces_impl() method */
#[no_mangle]
pub unsafe extern "system" fn JVM_DumpThreads_impl(
    env: RawJNIEnv,
    thread_class: jclass,
    threads: jobjectArray,
) -> jobjectArray {
    unimplemented!()
}
