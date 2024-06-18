
use dash_macros::{ParseNode, ResolveNode};
use super::{expr::{Expr, IdentPath, ExprList}, token::{lit, kw}};
use crate::{
    ast::token::delim,
    checker::{resolve::ResolveNode, coherency::{Checker, ScopeLevel}, ty::Ty, path},
    parser::parse::{NodePool, Node},
    shared::{logger::{Message, Level, LoggerRef, Note}, src::ArcSpan}
};

#[derive(Debug, Default)]
pub enum PossibleMatch {
    /// Defined outside function
    Inaccessible(ArcSpan),
    /// An ephemeral variable whose definition hasn't shown up yet
    NotYetDefined(ArcSpan),
    /// Variable with nearly the same name
    NearlyTheSameName(path::Ident, ArcSpan),
    /// No possible matches. Write better code
    #[default]
    None,
}

impl PossibleMatch {
    fn precedence(&self) -> usize {
        match self {
            PossibleMatch::None => 0,
            PossibleMatch::NearlyTheSameName(_, _) => 1,
            PossibleMatch::Inaccessible(_) => 2,
            PossibleMatch::NotYetDefined(_) => 3,
        }
    }
    fn upgrade(&mut self, another: PossibleMatch) {
        if self.precedence() < another.precedence() {
            *self = another;
        }
    }
}

#[derive(Debug, ParseNode)]
#[parse(expected = "identifier")]
pub enum ItemUseNode {
    This(kw::This),
    Ident(IdentPath, #[parse(skip)] PossibleMatch),
}

impl ResolveNode for ItemUseNode {
    fn try_resolve_node(&mut self, pool: &NodePool, checker: &mut Checker) -> Option<Ty> {
        let mut outside_function_scope = false;
        for scope in checker.scopes() {
            let name = match self {
                Self::Ident(i, _) => i.get(pool).to_path(pool),
                Self::This(_) => path::IdentPath::new([path::Ident::from("this")], false)
            };
            if let Some(ent) = scope.entities().find(&name) {
                if ent.defined() && (!outside_function_scope || !ent.ephemeral()) {
                    return Some(ent.ty());
                }
                if let Self::Ident(_, possible_match) = self {
                    if outside_function_scope && ent.ephemeral() {
                        possible_match.upgrade(PossibleMatch::Inaccessible(ent.span()));
                    }
                    else if ent.ephemeral() && !ent.defined() {
                        possible_match.upgrade(PossibleMatch::NotYetDefined(ent.span()));
                    }
                }
            }
            if scope.level() >= ScopeLevel::Function {
                outside_function_scope = true;
            }
        }
        None
    }
    fn log_unresolved_reason(&self, pool: &NodePool, _checker: &Checker, logger: LoggerRef) {
        match self {
            Self::Ident(i, possible_match) => {
                let name = i.get(pool).to_path(pool);
                let span = i.get(pool).span_or_builtin(pool);
                let msg = Message::new(Level::Error, format!("Unknown item {name}"), span.as_ref());

                logger.lock().unwrap().log(
                    match possible_match {
                        PossibleMatch::Inaccessible(ref span) => msg.note(Note::new_at(
                            format!("Item {name} is not accessible outside this function"),
                            span.as_ref()
                        )),
                        PossibleMatch::NotYetDefined(ref span) => msg.note(Note::new_at(
                            format!("Item {name} is defined later"),
                            span.as_ref()
                        )),
                        PossibleMatch::NearlyTheSameName(ident, ref span) => msg.note(Note::new_at(
                            format!("A similarly named item '{ident}' exists, did you mean that?"),
                            span.as_ref()
                        )),
                        PossibleMatch::None => msg,
                    }
                );
            }
            Self::This(kw) => logger.lock().unwrap().log(Message::new(
                Level::Error,
                "'this' is not valid in this scope",
                kw.get(pool).span_or_builtin(pool).as_ref()
            )),
        }
    }
}

#[derive(Debug, ParseNode, ResolveNode)]
#[parse(expected = "expression")]
pub enum AtomNode {
    ClosedExpr(delim::Parenthesized<Expr>),
    Block(delim::Braced<ExprList>),
    ItemUse(ItemUse),
    String(lit::String),
    Float(lit::Float),
    Int(lit::Int),
    Bool(lit::Bool),
    Void(lit::Void),
}
