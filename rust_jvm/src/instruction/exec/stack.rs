use crate::jvm::call::StackFrame;
use crate::jvm::mem::JavaValue;

pub fn dup(frame: &mut StackFrame) {
    let peeked = frame.stack[frame.stack.len() - 1];

    if matches!(&peeked, JavaValue::Long(_) | JavaValue::Double(_)) {
        panic!("dup can only be used on category 1 computational types")
    }

    frame.stack.push(peeked);
}

pub fn dup_x1(frame: &mut StackFrame) {
    let value1 = frame.stack.pop().unwrap();
    let value2 = frame.stack.pop().unwrap();

    if matches!(&value1, JavaValue::Long(_) | JavaValue::Double(_))
        || matches!(&value2, JavaValue::Long(_) | JavaValue::Double(_))
    {
        panic!("dup_x1 can only be used on category 1 computational types")
    }

    frame.stack.push(value1);
    frame.stack.push(value2);
    frame.stack.push(value1);
}

pub fn dup_x2(frame: &mut StackFrame) {
    let value1 = frame.stack.pop().unwrap();
    let value2 = frame.stack.pop().unwrap();
    let value3 = frame.stack.pop().unwrap();

    if matches!(&value1, JavaValue::Long(_) | JavaValue::Double(_)) {
        panic!("dup_x2 value1 can only be used on category 1 computational types")
    }

    // value2 and 3 must either be a single category 2 type or 2 category 1s
    assert_eq!(
        matches!(&value2, JavaValue::Long(_) | JavaValue::Double(_)),
        matches!(&value3, JavaValue::Long(_) | JavaValue::Double(_))
    );

    frame.stack.push(value1);
    frame.stack.push(value3);
    frame.stack.push(value2);
    frame.stack.push(value1);
}

pub fn dup2(frame: &mut StackFrame) {
    let value1 = frame.stack[frame.stack.len() - 1];
    let value2 = frame.stack[frame.stack.len() - 2];

    // value2 and 3 must either be a single category 2 type or 2 category 1s
    assert_eq!(
        matches!(&value1, JavaValue::Long(_) | JavaValue::Double(_)),
        matches!(&value2, JavaValue::Long(_) | JavaValue::Double(_))
    );

    frame.stack.push(value2);
    frame.stack.push(value1);
}

pub fn dup2_x1(frame: &mut StackFrame) {
    let value1 = frame.stack.pop().unwrap();
    let value2 = frame.stack.pop().unwrap();
    let value3 = frame.stack.pop().unwrap();

    if matches!(&value3, JavaValue::Long(_) | JavaValue::Double(_)) {
        panic!("dup2_x1 value3 can only be used on category 1 computational types")
    }

    // value2 and 3 must either be a single category 2 type or 2 category 1s
    assert_eq!(
        matches!(&value1, JavaValue::Long(_) | JavaValue::Double(_)),
        matches!(&value2, JavaValue::Long(_) | JavaValue::Double(_))
    );

    frame.stack.push(value2);
    frame.stack.push(value1);
    frame.stack.push(value3);
    frame.stack.push(value2);
    frame.stack.push(value1);
}

pub fn dup2_x2(frame: &mut StackFrame) {
    let value1 = frame.stack.pop().unwrap();
    let value2 = frame.stack.pop().unwrap();
    let value3 = frame.stack.pop().unwrap();
    let value4 = frame.stack.pop().unwrap();

    // value 1 and 2 must either be a single category 2 type or 2 category 1s
    assert_eq!(
        matches!(&value1, JavaValue::Long(_) | JavaValue::Double(_)),
        matches!(&value2, JavaValue::Long(_) | JavaValue::Double(_))
    );

    // value 3 and 4 must either be a single category 2 type or 2 category 1s
    assert_eq!(
        matches!(&value3, JavaValue::Long(_) | JavaValue::Double(_)),
        matches!(&value4, JavaValue::Long(_) | JavaValue::Double(_))
    );

    frame.stack.push(value2);
    frame.stack.push(value1);
    frame.stack.push(value4);
    frame.stack.push(value3);
    frame.stack.push(value2);
    frame.stack.push(value1);
}

pub fn nop() {}

pub fn pop(frame: &mut StackFrame) {
    assert!(!matches!(
        frame.stack.pop().unwrap(),
        JavaValue::Long(_) | JavaValue::Double(_)
    ));
}

pub fn pop2(frame: &mut StackFrame) {
    // value 1 and 2 must either be a single category 2 type or 2 category 1s
    assert_eq!(
        matches!(
            frame.stack.pop().unwrap(),
            JavaValue::Long(_) | JavaValue::Double(_)
        ),
        matches!(
            frame.stack.pop().unwrap(),
            JavaValue::Long(_) | JavaValue::Double(_)
        )
    );
}

pub fn swap(frame: &mut StackFrame) {
    let value1 = frame.stack.pop().unwrap();
    let value2 = frame.stack.pop().unwrap();

    if matches!(&value1, JavaValue::Long(_) | JavaValue::Double(_))
        || matches!(&value2, JavaValue::Long(_) | JavaValue::Double(_))
    {
        panic!("swap can only be used on category 1 computational types")
    }

    frame.stack.push(value1);
    frame.stack.push(value2);
}
