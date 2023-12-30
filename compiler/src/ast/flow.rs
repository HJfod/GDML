
use dash_macros::{Parse, Resolve};
use crate::{parser::parse::{Separated, SeparatedWithTrailing}, checker::resolve::Resolve};
use super::{token::{kw, delim, punct}, expr::{Expr, ExprList, IdentComponent}};

#[derive(Debug, Parse)]
pub struct If {
    if_kw: kw::If,
    cond: Expr,
    truthy: delim::Braced<ExprList>,
    falsy: Option<(kw::Else, Else)>
}

impl Resolve for If {
    fn try_resolve(&mut self, checker: &mut crate::checker::coherency::Checker) -> Option<crate::checker::ty::Ty> {
        todo!()
    }
}

#[derive(Debug, Parse, Resolve)]
#[parse(expected = "block or if statement")]
pub enum Else {
    Else(delim::Braced<ExprList>),
    ElseIf(Box<If>),
}

#[derive(Debug, Parse)]
pub struct Return {
    return_kw: kw::Return,
    expr: Option<Expr>,
}

impl Resolve for Return {
    fn try_resolve(&mut self, checker: &mut crate::checker::coherency::Checker) -> Option<crate::checker::ty::Ty> {
        todo!()
    }
}

#[derive(Debug, Parse)]
#[parse(expected = "identifier")]
enum UsingComponent {
    Multi(delim::Braced<SeparatedWithTrailing<UsingComponent, punct::Comma>>),
    Single(IdentComponent),
}

#[derive(Debug, Parse)]
struct UsingPath {
    absolute: Option<punct::Namespace>,
    path: Separated<UsingComponent, punct::Namespace>,
}

#[derive(Debug, Parse)]
pub struct UsingItem {
    using_kw: kw::Using,
    path: UsingPath,
}

impl Resolve for UsingItem {
    fn try_resolve(&mut self, checker: &mut crate::checker::coherency::Checker) -> Option<crate::checker::ty::Ty> {
        todo!()
    }
}

#[derive(Debug, Parse, Resolve)]
#[parse(expected = "control flow expression")]
pub enum Flow {
    If(If),
    Return(Return),
    UsingItem(UsingItem),
}
