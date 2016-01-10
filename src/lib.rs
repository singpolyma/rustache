#![deny(missing_docs)]
#![deny(warnings)]

//! The main crate for the Rustache library.
//!
//! Rustache is a flexible template engine for Rust.

extern crate rustc_serialize;

use std::fmt;
use std::cell::RefCell;
use std::collections::HashMap;
use std::error::Error;

use self::RustacheError::*;
use self::Data::*;

pub use build::{HashBuilder, VecBuilder};
pub use rustache::{render_file, render_text, Render};

/// Alias for Result<T, RustacheError>
pub type RustacheResult<T> = Result<T, RustacheError>;

/// Enum to handle errors from the Rustache library.
pub enum RustacheError {
    //ParserErrorType(ParserError),
    //CompilerErrorType(CompilerError),
    /// Error parsing JSON data
    JsonError(String),
    /// Error opening or reading a file
    FileError(String),
    /// Generic enum value for any errors from the template module.
    TemplateErrorType(template::TemplateError)
}

impl fmt::Debug for RustacheError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &JsonError(ref val) => write!(f, "JsonError: {:?}", val),
            &FileError(ref val) => write!(f, "FileError: {:?}", val),
            &TemplateErrorType(ref val) => write!(f, "{:?}", val),
        }
    }
}

impl fmt::Display for RustacheError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &JsonError(ref val) => write!(f, "{:?}", val),
            &FileError(ref val) => write!(f, "{:?}", val),
            &TemplateErrorType(ref val) => write!(f, "{:?}", val),
        }
    }
}

impl Error for RustacheError {
    fn description(&self) -> &str {
        match self {
            &JsonError(ref val) => val,
            &FileError(ref val) => val,
            &TemplateErrorType(ref val) => val.description(),
        }
    }
}

// Represents the possible types that passed in data may take on
#[doc(hidden)]
pub enum Data<'a> {
    Strng(String),
    Bool(bool),
    Integer(i32),
    Float(f64),
    Vector(Vec<Data<'a>>),
    Hash(HashMap<String, Data<'a>>),
    Lambda(RefCell<&'a mut FnMut(String) -> String>)
}
// |String|: 'a -> String : F Above

// Implementing custom PartialEq for Data
impl<'a> PartialEq for Data<'a> {
    fn eq(&self, other: &Data<'a>) -> bool {
        match (self, other) {
            (&Strng(ref val0), &Strng(ref val1)) => val0 == val1,
            (&Bool(ref val0), &Bool(ref val1)) => val0 == val1,
            (&Integer(ref val0), &Integer(ref val1)) => val0 == val1,
            (&Float(ref val0), &Float(ref val1)) => val0 == val1,
            (&Vector(ref val0), &Vector(ref val1)) => val0 == val1,
            (&Hash(ref val0), &Hash(ref val1)) => val0 == val1,
            (&Lambda(_), &Lambda(_)) => panic!("Can't compare closures"),
            (_, _) => false
        }
    }
}

// Implementing custom Show for Data
impl<'a> fmt::Debug for Data<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Strng(ref val)   => write!(f, "Strng({:?})", val),
            Bool(val)        => write!(f, "Boolean({:?})", val),
            Integer(ref val) => write!(f, "Integer({:?})", val),
            Float(ref val)   => write!(f, "Float({:?})", val),
            Vector(ref val)  => write!(f, "Vector({:?})", val),
            Hash(ref val)    => write!(f, "Hash({:?})", val),
            Lambda(_)        => write!(f, "Lambda(...)")
        }
    }
}

// Internal Modules
mod rustache;
mod compiler;
mod parser;
mod build;
mod template;
