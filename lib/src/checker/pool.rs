
use crate::parser::tokenizer::Tokenizer;
use crate::shared::src::SrcPool;
use crate::shared::logger::LoggerRef;
use crate::parser::parse::{ParseRef, NodePool};

pub struct ASTPool<AST: ParseRef> {
    asts: Vec<AST>,
}

impl<AST: ParseRef> ASTPool<AST> {
    pub fn parse_src_pool(list: &mut NodePool, pool: &SrcPool, logger: LoggerRef) -> Self {
        Self {
            asts: pool.iter()
                .filter_map(|src| AST::parse_complete(
                    list,
                    src.clone(),
                    Tokenizer::new(&src, logger.clone())
                ).ok())
                .collect(),
        }
    }
    pub fn iter(&self) -> <&Vec<AST> as IntoIterator>::IntoIter {
        self.into_iter()
    }
}

impl<'a, AST: ParseRef> IntoIterator for &'a ASTPool<AST> {
    type Item = &'a AST;
    type IntoIter = <&'a Vec<AST> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.asts.iter()
    }
}

impl<'a, AST: ParseRef> IntoIterator for &'a mut ASTPool<AST> {
    type Item = &'a mut AST;
    type IntoIter = <&'a mut Vec<AST> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.asts.iter_mut()
    }
}
