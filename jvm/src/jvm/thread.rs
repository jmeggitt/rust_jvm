//! This module handles all of the threading and synchronous activities of the jvm
#![allow(unused_variables)]

use crate::constant_pool::ClassElement;
use crate::jvm::call::{clean_str, FlowControl, JavaEnvInvoke, RawJNIEnv};
use crate::jvm::mem::{
    ArrayReference, InstanceReference, JavaValue, ManualInstanceReference, ObjectHandle,
};
use hashbrown::HashMap;
use jni::sys::{
    jboolean, jclass, jint, jlong, jobject, jobjectArray, jstring, JNI_FALSE, JNI_TRUE,
};
use std::thread::{current, park, sleep, spawn, yield_now, Thread, ThreadId};

use crate::instruction::getstatic;
use crate::jvm::JavaEnv;
use parking_lot::{Condvar, Mutex, RwLock};
use std::sync::{Arc, Barrier};
use std::time::Duration;
#[cfg(feature = "thread-priority")]
use thread_priority::{ThreadId as NativeThreadId, ThreadPriority};
#[cfg(feature = "callstack")]
use crate::jvm::call::callstack_trace::CallTracer;

pub trait SynchronousMonitor<T> {
    fn lock(&self, target: T);
    fn unlock(&self, target: T);
    fn check_lock(&self, target: T) -> bool;
}

#[derive(Default)]
pub struct JavaThreadManager {
    thread_handles: HashMap<ObjectHandle, ThreadId>,
    threads: HashMap<ThreadId, ThreadInfo>,
    monitor: HashMap<ObjectHandle, Arc<(Mutex<Option<(ThreadId, u64)>>, Condvar)>>,
    system_thread_group: Option<ObjectHandle>,
}

impl JavaThreadManager {
    pub fn init_headless_current_thread(&mut self) {
        let info = ThreadInfo {
            java_thread: None,
            state: ThreadState::Running,
            state_request: None,
            rust_thread: current(),
            #[cfg(feature = "thread-priority")]
            native_thread: thread_priority::thread_native_id(),
            call_stack: vec![],
            #[cfg(feature = "callstack")]
            call_trace: CallTracer::new(),
            sticky_exception: None,
        };

        self.threads.insert(current().id(), info);
    }

    pub fn mut_info(&mut self, obj: ObjectHandle) -> Option<&mut ThreadInfo> {
        let handle = self.thread_handles.get(&obj)?.to_owned();
        self.threads.get_mut(&handle)
    }

    pub fn get_info(&self, obj: ObjectHandle) -> Option<&ThreadInfo> {
        self.thread_handles
            .get(&obj)
            .and_then(move |x| self.threads.get(x))
    }

    pub fn get_current_call_stack(&self) -> Option<&[(ObjectHandle, ClassElement)]> {
        self.threads.get(&current().id()).map(|x| &x.call_stack[..])
    }

    pub fn push_call_stack(&mut self, target: ObjectHandle, element: ClassElement, args: &[JavaValue]) {
        let current_thread = current();
        let mut info = self
            .threads
            .get_mut(&current_thread.id())
            .expect("Unable to find current thread");

        if info.call_stack.is_empty() {
            info.state = ThreadState::Running;
        }


        #[cfg(feature = "callstack")]
        info.call_trace.push_call(&element, args);
        info.call_stack.push((target, element));
    }

    #[cfg(feature = "callstack")]
    pub fn debug_print(&self) {
        let current_thread = current();
        let mut info = self
            .threads
            .get(&current_thread.id())
            .expect("Unable to find current thread");

        info.call_trace.dump();
    }

    pub fn pop_call_stack(&mut self, ret: &Result<Option<JavaValue>, FlowControl>) {
        let current_thread = current();
        let mut info = self
            .threads
            .get_mut(&current_thread.id())
            .expect("Unable to find current thread");
        info.call_stack.pop().unwrap();


        #[cfg(feature = "callstack")]
        info.call_trace.pop_call(ret);

        if info.call_stack.is_empty() {
            info.state = ThreadState::Stopped;
        }
    }

    pub fn debug_print_call_stack(&self) {
        let current_thread = current();
        let mut info = self
            .threads
            .get(&current_thread.id())
            .expect("Unable to find current thread");

        let mut padding = String::new();
        for debug_str in &info.call_stack {
            debug!("{}{:?}", &padding, debug_str.1);
            padding.push_str("   ");
        }
    }

    pub fn set_sticky_exception(&mut self, throwable: Option<ObjectHandle>) {
        self.threads
            .get_mut(&current().id())
            .unwrap()
            .sticky_exception = throwable;
    }

    pub fn get_sticky_exception(&self) -> Option<ObjectHandle> {
        self.threads.get(&current().id()).unwrap().sticky_exception
    }
}

impl SynchronousMonitor<ObjectHandle> for Arc<RwLock<JavaEnv>> {
    fn lock(&self, target: ObjectHandle) {
        let pair = self
            .write()
            .thread_manager
            .monitor
            .entry(target)
            .or_insert_with(|| Arc::new((Mutex::new(None), Condvar::new())))
            .clone();

        let current_thread = current().id();
        let mut guard = pair.0.lock();

        while guard.is_some() {
            // Lock is already held by this thread increment counter and continue
            if guard.unwrap().0 == current_thread {
                guard.unwrap().1 += 1;
                return
            }

            pair.1.wait(&mut guard);
        }

        *guard = Some((current().id(), 1));
    }

    fn unlock(&self, target: ObjectHandle) {
        let pair = self
            .read()
            .thread_manager
            .monitor
            .get(&target)
            .unwrap()
            .clone();
        let mut guard = pair.0.lock();
        let mut break_lock = false;

        if let Some((lock_holder, count)) = &mut *guard {
            if *lock_holder == current().id() {
                *count -= 1;
            }

            if *count == 0 {
                break_lock = true;
            }
        }

        if break_lock {
            *guard = None;
        }

        pair.1.notify_one();
    }

    fn check_lock(&self, target: ObjectHandle) -> bool {
        match self.read().thread_manager.monitor.get(&target) {
            Some(v) => v.0.lock().is_some(),
            None => false,
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn JVM_MonitorWait_impl(env: RawJNIEnv, obj: jobject, ms: jlong) {
    // TODO: Should this also acquire the lock?
    let target = obj_expect!(env, obj);
    let pair = env
        .write()
        .thread_manager
        .monitor
        .entry(target)
        .or_insert_with(|| Arc::new((Mutex::new(None), Condvar::new())))
        .clone();

    let mut guard = pair.0.lock();
    pair.1
        .wait_for(&mut guard, Duration::from_millis(ms as u64));
}

#[no_mangle]
pub unsafe extern "system" fn JVM_MonitorNotify_impl(env: RawJNIEnv, obj: jobject) {
    let target = obj_expect!(env, obj);
    let pair = env
        .write()
        .thread_manager
        .monitor
        .entry(target)
        .or_insert_with(|| Arc::new((Mutex::new(None), Condvar::new())))
        .clone();
    pair.1.notify_one();
}

#[no_mangle]
pub unsafe extern "system" fn JVM_MonitorNotifyAll_impl(env: RawJNIEnv, obj: jobject) {
    let target = obj_expect!(env, obj);
    let pair = env
        .write()
        .thread_manager
        .monitor
        .entry(target)
        .or_insert_with(|| Arc::new((Mutex::new(None), Condvar::new())))
        .clone();
    pair.1.notify_all();
}

pub struct ThreadInfo {
    java_thread: Option<ObjectHandle>,
    state: ThreadState,
    state_request: Option<StateRequest>,
    rust_thread: Thread,
    #[cfg(feature = "thread-priority")]
    native_thread: NativeThreadId,
    call_stack: Vec<(ObjectHandle, ClassElement)>,
    #[cfg(feature = "callstack")]
    call_trace: CallTracer,
    // Unlike state_request, this holds regular exceptions thrown in native functions
    sticky_exception: Option<ObjectHandle>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ThreadState {
    Running,
    Suspended,
    Stopped,
    Interrupted,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum StateRequest {
    Park,
    Interrupt,
    Throw(ObjectHandle),
}

pub fn prepare_sys_thread_group(env: &mut Arc<RwLock<JavaEnv>>) {
    if env.read().thread_manager.system_thread_group.is_none() {
        let sys_group = ObjectHandle::new(env.write().class_schema("java/lang/ThreadGroup"));

        env.invoke_virtual(
            ClassElement {
                class: "java/lang/ThreadGroup".into(),
                element: "<init>".into(),
                desc: "()V".into(),
            },
            sys_group,
            vec![],
        )
            .unwrap();

        env.write().thread_manager.system_thread_group = Some(sys_group);
    }
}

pub fn first_time_sys_thread_init(env: &mut Arc<RwLock<JavaEnv>>) {
    env.write().thread_manager.init_headless_current_thread();
    prepare_sys_thread_group(env);

    let obj = {
        let mut jvm = env.write();
        let obj = ObjectHandle::new(jvm.class_schema("java/lang/Thread"));

        // Thread must be set up manually :(
        let tid = match getstatic::check_static_init(
            &mut *jvm,
            "java/lang/Thread",
            "threadSeqNumber",
            "J",
        ) {
            Some(JavaValue::Long(0)) => JavaValue::Long(0),
            Some(JavaValue::Long(x)) => JavaValue::Long(x + 1),
            _ => panic!("Error while retrieving java/lang/Thread::threadSeqNumber"),
        };

        let field_reference = format!(
            "{}_{}",
            clean_str("java/lang/Thread"),
            clean_str("threadSeqNumber")
        );
        jvm.static_fields.insert(field_reference, tid);

        let instance = obj.expect_instance();
        instance.write_named_field("tid", tid);
        instance.write_named_field("priority", JavaValue::Int(5));
        instance.write_named_field("group", jvm.thread_manager.system_thread_group);

        if let JavaValue::Long(thread_id) = tid {
            if thread_id == 0 {
                instance.write_named_field("name", jvm.build_string("main"));
            } else {
                instance.write_named_field(
                    "name",
                    jvm.build_string(&format!("Sys-Thread-{}", thread_id)),
                );
            }
        }

        obj
    };

    let group = env.read().thread_manager.system_thread_group.unwrap();

    // Hard code the operation of java/lang/ThreadGroup::add(Ljava/lang/Thread;)V to avoid an infinite loop
    env.lock(group);
    let mut group_instance = group.expect_instance();

    let group_threads: Option<ObjectHandle> = group_instance.read_named_field("threads");
    let n_threads: jint = group_instance.read_named_field("nthreads");

    if group_threads.is_none() {
        let mut new_threads = vec![None; 4];
        new_threads[0] = Some(obj);
        group_instance
            .write_named_field("threads", Some(ObjectHandle::array_from_data(new_threads)));
    } else if n_threads as usize == group_threads.unwrap().unknown_array_length().unwrap() {
        let new_array: Vec<Option<ObjectHandle>> = vec![None; n_threads as usize * 2];
        let new_array = ObjectHandle::array_from_data(new_array);
        group_threads
            .unwrap()
            .expect_array::<Option<ObjectHandle>>()
            .array_copy(new_array, 0, 0, n_threads as usize);
        group_instance.write_named_field("threads", Some(new_array));
    }

    let group_threads: Option<ObjectHandle> = group_instance.read_named_field("threads");
    group_threads
        .unwrap()
        .expect_array()
        .write_array(n_threads as usize, Some(obj));
    // group_threads.unwrap().expect_array()[n_threads as usize] = Some(obj);
    group_instance.write_named_field("nthreads", n_threads + 1);

    let unstarted: jint = group_instance.read_named_field("nUnstartedThreads");
    group_instance.write_named_field("nUnstartedThreads", unstarted - 1);
    env.unlock(group);

    let thread_handle = current().id();
    // let info = ThreadInfo {
    //     java_thread: Some(obj),
    //     state: ThreadState::Running,
    //     state_request: None,
    //     rust_thread: thread_handle.clone(),
    //     #[cfg(feature = "thread-priority")]
    //     native_thread: thread_priority::thread_native_id(),
    //     sticky_exception: None,
    //     call_stack: vec![],
    // };

    let mut jvm = env.write();
    jvm.thread_manager.thread_handles.insert(obj, thread_handle);
    jvm.thread_manager
        .threads
        .get_mut(&thread_handle)
        .unwrap()
        .java_thread = Some(obj);
    // jvm.thread_manager.threads.insert(thread_handle.id(), info);
}

pub fn handle_thread_updates(env: &mut Arc<RwLock<JavaEnv>>) -> Result<(), FlowControl> {
    // Change to park or sent interrupt exception
    let handle = current().id();

    // TODO: Maybe more to a system where all jni invoke functions will check if the current thread is initialized
    // Must be doing first time setup of the thread
    if env.read().thread_manager.threads.get(&handle).is_none() {
        return Ok(());
    }

    loop {
        // Early return if possible without acquiring global write guard
        if env
            .read()
            .thread_manager
            .threads
            .get(&handle)
            .unwrap()
            .state_request
            .is_none()
        {
            return Ok(());
        }

        let action = {
            let mut lock = env.write();
            let mut info = lock.thread_manager.threads.get_mut(&handle).unwrap();
            match info.state_request.take() {
                None => return Ok(()), // Park request may have been rescinded between locks
                Some(StateRequest::Park) => {
                    info.state = ThreadState::Suspended;
                    StateRequest::Park
                }
                Some(StateRequest::Interrupt) => {
                    info.state = ThreadState::Interrupted;
                    StateRequest::Interrupt
                }
                Some(x) => x,
            }
        };

        match action {
            StateRequest::Park => {
                park();
                let mut lock = env.write();
                let mut info = lock.thread_manager.threads.get_mut(&handle).unwrap();
                info.state = ThreadState::Running;
            }
            StateRequest::Throw(x) => return Err(FlowControl::Throws(Some(x))),
            StateRequest::Interrupt => return Err(FlowControl::ThreadInterrupt),
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn JVM_StartThread_impl(env: RawJNIEnv, thread: jobject) {
    let target_thread = obj_expect!(env, thread);

    // Use a barrier so we can register the new thread before is starts execution
    let barrier = Arc::new(Barrier::new(2));
    let mut env_handle: Arc<RwLock<JavaEnv>> = Arc::clone(&*env);
    let mut barrier_clone = barrier.clone();

    #[cfg(feature = "crossbeam-channel")]
        let (send, recv) = crossbeam_channel::bounded(1);

    let new_thread = spawn(move || {
        #[cfg(all(feature = "thread-priority", feature = "crossbeam-channel"))]
            {
                send.send(thread_priority::thread_native_id()).unwrap();
            }
        {
            barrier_clone.wait();
        }

        let field = ClassElement {
            class: "java/lang/Thread".to_string(),
            element: "run".to_string(),
            desc: "()V".to_string(),
        };

        let exit_state = match env_handle.invoke_virtual(field, target_thread, vec![]) {
            Ok(_) => ThreadState::Stopped,
            Err(FlowControl::ThreadInterrupt) => ThreadState::Interrupted,
            Err(FlowControl::Throws(x)) => panic!(
                "Thread {:?} terminated with throwable {:?}",
                current().name(),
                x
            ),
            Err(x) => panic!("What? Thread exited with strange result: {:?}", x),
        };

        info!(
            "Thread {:?} exited with state {:?}",
            current().name(),
            exit_state
        );
        let mut lock = env_handle.write();
        if let Some(info) = lock.thread_manager.threads.get_mut(&current().id()) {
            if info.state == ThreadState::Running {
                info.state = exit_state;
            }
        }
    })
        .thread()
        .to_owned();

    let info = ThreadInfo {
        java_thread: Some(target_thread),
        state: ThreadState::Running,
        state_request: None,
        rust_thread: new_thread.clone(),
        #[cfg(all(feature = "thread-priority", feature = "crossbeam-channel"))]
        native_thread: recv.recv().unwrap(),
        sticky_exception: None,
        call_stack: vec![],
        #[cfg(feature = "callstack")]
        call_trace: CallTracer::new(),
    };

    let mut lock = env.write();
    lock.thread_manager
        .thread_handles
        .insert(target_thread, new_thread.id());
    lock.thread_manager.threads.insert(new_thread.id(), info);

    // Release barrier by reach barrier thread target
    barrier.wait();
}

#[no_mangle]
pub unsafe extern "system" fn JVM_StopThread_impl(
    env: RawJNIEnv,
    thread: jobject,
    exception: jobject,
) {
    let exception_handle = obj_expect!(env, exception);

    let mut lock = env.write();
    if let Some(info) = lock.thread_manager.threads.get_mut(&current().id()) {
        if matches!(info.state, ThreadState::Stopped | ThreadState::Interrupted) {
            return;
        }

        if info.state_request != Some(StateRequest::Interrupt) {
            info.state_request = Some(StateRequest::Throw(exception_handle));

            // If suspended, unpark the thread so it can handle the state change
            if info.state == ThreadState::Suspended {
                info.rust_thread.unpark();
            }
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn JVM_IsThreadAlive_impl(env: RawJNIEnv, thread: jobject) -> jboolean {
    let thread_handle = ObjectHandle::from_ptr(thread).unwrap();
    let lock = env.read();

    match lock.thread_manager.get_info(thread_handle).map(|x| x.state) {
        Some(ThreadState::Running) => JNI_TRUE,
        Some(ThreadState::Suspended) => JNI_TRUE,
        _ => JNI_FALSE,
    }
}

#[no_mangle]
pub unsafe extern "system" fn JVM_SuspendThread_impl(env: RawJNIEnv, thread: jobject) {
    let mut lock = env.write();
    if let Some(info) = lock.thread_manager.threads.get_mut(&current().id()) {
        if info.state == ThreadState::Running && info.state_request.is_none() {
            info.state_request = Some(StateRequest::Park);
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn JVM_ResumeThread_impl(env: RawJNIEnv, thread: jobject) {
    let mut lock = env.write();
    if let Some(info) = lock.thread_manager.threads.get_mut(&current().id()) {
        if info.state == ThreadState::Suspended {
            if info.state_request == Some(StateRequest::Park) {
                info.state_request = None;
            }

            info.rust_thread.unpark();
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn JVM_SetThreadPriority_impl(
    env: RawJNIEnv,
    thread: jobject,
    prio: jint,
) {
    // I'm not sure if I should trust the requested thread priority, so make it optional
    #[cfg(not(feature = "thread-priority"))]
    warn!("Opting to ignore request to set thread priority");

    #[cfg(feature = "thread-priority")]
        {
            let thread_handle = ObjectHandle::from_ptr(thread).unwrap();
            let lock = env.read();

            // Java object field is set for me
            if let Some(info) = lock.thread_manager.get_info(thread_handle) {
                let priority = ThreadPriority::Specific(prio as _);

                #[cfg(windows)]
                    thread_priority::set_thread_priority(info.native_thread, priority).unwrap();

                #[cfg(unix)]
                    {
                        use thread_priority::unix::{NormalThreadSchedulePolicy, ThreadSchedulePolicy};
                        let policy = ThreadSchedulePolicy::Normal(NormalThreadSchedulePolicy::Normal);
                        thread_priority::set_thread_priority_and_policy(
                            info.native_thread,
                            priority,
                            policy,
                        )
                            .unwrap();
                    }
            }
        }
}

#[no_mangle]
pub unsafe extern "system" fn JVM_Yield_impl(env: RawJNIEnv, thread_class: jclass) {
    yield_now()
}

#[no_mangle]
pub unsafe extern "system" fn JVM_Sleep_impl(env: RawJNIEnv, thread_class: jclass, millis: jlong) {
    sleep(Duration::from_millis(millis as _))
}

#[no_mangle]
pub unsafe extern "system" fn JVM_CurrentThread_impl(
    mut env: RawJNIEnv,
    thread_class: jclass,
) -> jobject {
    let handle = current().id();

    if env.read().thread_manager.threads.get(&handle).is_none() {
        first_time_sys_thread_init(&mut *env);
    }

    env.read()
        .thread_manager
        .threads
        .get(&handle)
        .unwrap()
        .java_thread
        .unwrap()
        .ptr()
}

#[no_mangle]
pub unsafe extern "system" fn JVM_CountStackFrames_impl(env: RawJNIEnv, thread: jobject) -> jint {
    let thread_handle = ObjectHandle::from_ptr(thread).unwrap();
    let lock = env.read();

    if let Some(info) = lock.thread_manager.get_info(thread_handle) {
        return info.call_stack.len() as jint;
    }

    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn JVM_Interrupt_impl(env: RawJNIEnv, thread: jobject) {
    let mut lock = env.write();
    if let Some(info) = lock.thread_manager.threads.get_mut(&current().id()) {
        if matches!(info.state, ThreadState::Stopped | ThreadState::Interrupted) {
            info.state = ThreadState::Interrupted;
            return;
        }

        info.state_request = Some(StateRequest::Interrupt);

        // If suspended, unpark the thread so it can handle the interrupt
        if info.state == ThreadState::Suspended {
            info.rust_thread.unpark();
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn JVM_IsInterrupted_impl(
    env: RawJNIEnv,
    thread: jobject,
    clear_interrupted: jboolean,
) -> jboolean {
    let mut lock = env.write();
    if let Some(info) = lock.thread_manager.threads.get_mut(&current().id()) {
        let ret = info.state == ThreadState::Interrupted
            || info.state_request == Some(StateRequest::Interrupt);

        if clear_interrupted == JNI_TRUE {
            if info.state_request == Some(StateRequest::Interrupt) {
                info.state_request = None;
            }

            if info.state == ThreadState::Interrupted {
                info.state = ThreadState::Stopped;
            }
        }

        return ret as jboolean;
    }

    JNI_FALSE
}

#[no_mangle]
pub unsafe extern "system" fn JVM_HoldsLock_impl(
    env: RawJNIEnv,
    thread_class: jclass,
    obj: jobject,
) -> jboolean {
    let obj_handle = obj_expect!(env, obj, JNI_FALSE);
    env.check_lock(obj_handle) as jboolean
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
    let mut threads = Vec::new();

    let lock = env.read();
    for info in lock.thread_manager.threads.values() {
        if matches!(info.state, ThreadState::Running | ThreadState::Suspended) {
            threads.push(info.java_thread);
        }
    }

    // TODO: This might drop the object too soon
    ObjectHandle::array_from_data(threads).ptr()
}

#[no_mangle]
pub unsafe extern "system" fn JVM_SetNativeThreadName_impl(
    env: RawJNIEnv,
    jthread: jobject,
    name: jstring,
) {
    // TODO: This is not possible in rust std
    warn!("Ignoring request to set native thread name")
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
