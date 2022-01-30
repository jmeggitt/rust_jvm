//! A crate aimed at parsing and working with java class files. The goal of this crate is being easy
//! to use and compliant with the Java SE 17 JVM Specification.

pub mod attributes;
pub mod constant;
pub mod read;
pub mod class;
pub mod path;
pub mod loader;
