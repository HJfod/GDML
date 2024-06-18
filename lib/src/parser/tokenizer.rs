
use std::marker::PhantomData;
use std::ops::Range;
use crate::shared::src::{Src, SrcCursor};
use crate::shared::logger::LoggerRef;

/// The main trait for introducing a custom syntax for a language. The 
/// language's token kind should implement this trait
pub trait Token: Sized + Clone {
    /// Get the next token in the source iterator. Errors and EOF should be represented as Tokens aswell
    fn next(iter: &mut SrcCursor) -> Self;

    /// Create a new instance of the EOF token for this type
    fn new_eof() -> Self;

    /// If this is the EOF token, returns true
    fn is_eof(&self) -> bool;
    /// Get the expected final character name if this is an EOF token
    fn get_eof_name(&self) -> Option<&str>;

    /// True if this token represents an error
    fn is_error(&self) -> bool;
    /// Get the error string for this token (if it's an error)
    fn get_error(&self) -> Option<&str>;
}

trait TokenIterator<T: Token> {
    fn next(&mut self) -> T;
}

pub struct Tokenizer<'s, T: Token> {
    src: &'s Src,
    cursor: SrcCursor<'s>,
    _phantom: PhantomData<T>,
}
impl<'s, T: Token> Tokenizer<'s, T> {
    pub fn new(src: &'s Src) -> Self {
        Self { src, cursor: src.cursor(), _phantom: PhantomData }
    }
}
impl<'s, T: Token> TokenIterator<T> for Tokenizer<'s, T> {
    fn next(&mut self) -> T {
        T::next(&mut self.cursor)
    }
}

pub struct TokenTree<'s, T: Token> {
    src: &'s Src,
    items: Vec<T>,
    iter_pos: usize,
    start_offset: usize,
    eof: Range<usize>,
}
impl<'s, T: Token> TokenIterator<T> for TokenTree<'s, T> {
    fn next(&mut self) -> T {
        self.items.get(self.iter_pos).cloned().inspect(|_| self.iter_pos += 1).unwrap_or(T::new_eof())
    }
}

pub struct TokenStream<'s, T: Token, const LOOKAHEAD: usize> {
    src: &'s Src,
    iter: Box<dyn TokenIterator<T>>,
    peek: [Option<T>; LOOKAHEAD],
    start_of_last_token: usize,
    last_was_braced: bool,
    logger: LoggerRef,
}

impl<'s, T: Token, const LOOKAHEAD: usize> TokenStream<'s, T, LOOKAHEAD> {

}

/// Skip C-like comments (`// ...` line comments and `/* ... */` block comments) 
/// in a source stream
pub fn skip_c_like_comments(cursor: &mut SrcCursor) {
    loop {
        // Ignore line comments
        if cursor.peek().is_some_and(|c| c == '/') && cursor.peek_n(1).is_some_and(|c| c == '/') {
            cursor.next();
            cursor.next();
            for c in &mut *cursor {
                if c == '\n' {
                    break;
                }
            }
            continue;
        }
        // Continue skipping until we encounter a non-whitespace character
        if cursor.peek().is_some_and(|c| c.is_whitespace()) {
            cursor.next();
            continue;
        }
        break;
    }
}
