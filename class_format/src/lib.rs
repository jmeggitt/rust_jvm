//! A crate aimed at parsing and working with java class files. The goal of this crate is being easy
//! to use and compliant with the Java SE 17 JVM Specification.

// TODO: Remove this when crate is fully implemented
#![allow(dead_code)]

pub mod attributes;
pub mod class;
pub mod constant;
pub mod loader;
pub mod path;
pub mod read;
