use crate::constant_pool::ClassElement;
use crate::jvm::call::FlowControl;
use crate::jvm::mem::JavaValue;
use serde_json::{Map, Value};
use std::fs::File;
use std::io::BufWriter;

pub struct CallTracer {
    tree: Value,
    depth: usize,
}

impl CallTracer {
    pub fn new() -> Self {
        let mut map = Map::new();
        map.insert("name".to_string(), Value::String("<entry>".to_string()));
        map.insert("operands".to_string(), Value::Array(Vec::new()));
        map.insert("stack".to_string(), Value::Array(Vec::new()));

        CallTracer {
            tree: Value::Object(map),
            depth: 0,
        }
    }

    pub fn push_call(&mut self, element: &ClassElement, args: &[JavaValue]) {
        let mut depth = self.depth;
        self.depth += 1;

        let mut node = &mut self.tree;
        while depth > 0 {
            let calls = node.get_mut("stack").and_then(Value::as_array_mut).unwrap();
            let len = calls.len();
            node = &mut calls[len - 1];
            depth -= 1;
        }

        let calls = node.get_mut("stack").and_then(Value::as_array_mut).unwrap();
        let mut map = Map::new();
        map.insert("name".to_string(), Value::String(format!("{:?}", element)));
        map.insert(
            "operands".to_string(),
            Value::Array(
                args.iter()
                    .map(|x| Value::String(format!("{:?}", x)))
                    .collect(),
            ),
        );
        map.insert("stack".to_string(), Value::Array(Vec::new()));
        calls.push(Value::Object(map));
    }

    pub fn pop_call(&mut self, ret: &Result<Option<JavaValue>, FlowControl>) {
        let mut depth = self.depth;
        self.depth -= 1;

        let mut node = &mut self.tree;
        while depth > 0 {
            let calls = node.get_mut("stack").and_then(Value::as_array_mut).unwrap();
            let len = calls.len();
            node = &mut calls[len - 1];
            depth -= 1;
        }

        node.as_object_mut()
            .unwrap()
            .insert("returned".to_string(), Value::String(format!("{:?}", ret)));
    }

    pub fn dump(&self) {
        let mut out = BufWriter::new(File::create("callstack-dump.json").unwrap());
        serde_json::to_writer_pretty(out, &self.tree).unwrap();
    }
}
