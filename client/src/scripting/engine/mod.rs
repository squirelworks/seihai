//! GS1 script engine.
//!
//! Source text goes through [`parse`] into a [`Script`], which a
//! [`ScriptWorld`] runs against NPC and weapon objects.
//! This is written from observed behaviour and the cleaned
//! Graal_analysis notes, not from any other implementation.

pub mod lexer;
pub mod value;
pub mod vocab;

pub use value::Val;
