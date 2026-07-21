// The lexer module exposes the Lexer itself and its error type,
// plus the token definitions that everything else in the compiler depends on.

pub mod token;
pub mod lexer;

// Re-export the most commonly used types so other modules can write
// `use crate::lexer::Lexer` instead of `crate::lexer::lexer::Lexer`
pub use lexer::Lexer;
