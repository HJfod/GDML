#include <lang/State.hpp>
#include <lang/Expr.hpp>
#include "../Debug.hpp"

using namespace geode::prelude;
using namespace gdml::lang;
using namespace gdml;

ExprResult<AttrExpr> AttrExpr::pull(Stream& stream) {
    Rollback rb(stream);
    GEODE_UNWRAP(Token::pull('@', stream));
    GEODE_UNWRAP_INTO(auto ident, IdentExpr::pull(stream));
    Option<Rc<Expr>> value;
    if (Token::draw('(', stream)) {
        GEODE_UNWRAP_INTO(value, Expr::pull(stream));
        GEODE_UNWRAP(Token::pull(')', stream));
    }
    return rb.commit<AttrExpr>(ident, value);
}

Type AttrExpr::typecheck(UnitParser& state) const {
    if (value) {
        value.value()->typecheck(state);
    }
    return Primitive::Void;
}

std::string AttrExpr::debug(size_t indent) const {
    return DebugPrint("AttrExpr", indent)
        .member("attribute", attribute)
        .member("value", value);
}

ExprResult<ExportExpr> ExportExpr::pull(Stream& stream) {
    Rollback rb(stream);
    GEODE_UNWRAP(Token::pull(Keyword::Export, stream));
    GEODE_UNWRAP_INTO(auto expr, Expr::pull(stream));
    return rb.commit<ExportExpr>(expr);
}

Type ExportExpr::typecheck(UnitParser& state) const {
    auto ty = expr->typecheck(state);
    if (!ty.isExportable()) {
        state.error(range, "Only named structs and nodes are exportable");
    }
    if (!state.isRootScope()) {
        state.error(range, "Export statements may only appear at top-level");
    }
    state.getParsedSrc()->addExportedType(state, ty);
    return ty;
}

std::string ExportExpr::debug(size_t indent) const {
    return DebugPrint("ExportExpr", indent)
        .member("expr", expr);
}

ExprResult<ImportExpr> ImportExpr::pull(Stream& stream) {
    Rollback rb(stream);
    GEODE_UNWRAP(Token::pull(Keyword::Import, stream));
    Vec<Rc<IdentExpr>> imports;
    if (!Token::pull('*', stream)) {
        rb.clearMessages();
        GEODE_UNWRAP(Token::pull('{', stream));
        while (true) {
            stream.debugTick();
            GEODE_UNWRAP_INTO(auto ident, IdentExpr::pull(stream));
            imports.push_back(ident);
            GEODE_UNWRAP_INTO(auto brk, Token::pullSeparator(',', '}', stream));
            if (brk) {
                break;
            }
        }
        GEODE_UNWRAP(Token::pull('}', stream));
    }
    GEODE_UNWRAP(Token::pull(Keyword::From, stream));
    Rollback litrb(stream);
    GEODE_UNWRAP_INTO(auto lit, Token::pull<Lit>(stream));
    if (!std::holds_alternative<StrLit>(lit)) {
        return litrb.error("Expected string literal");
    }
    litrb.commit();
    return rb.commit<ImportExpr>(std::get<StrLit>(lit), imports);
}

Type ImportExpr::typecheck(UnitParser& state) const {
    auto file = state.getSrc()->getSearchDir() / from;
    auto src = SrcFile::from(file);
    if (!src) {
        state.error(range, "{}", src.unwrapErr());
        return Type(VoidType(), nullptr);
    }
    auto parsed = UnitParser::parse(state.getShared(), src.unwrap());
    Vec<Type> imported;
    // empty = import everything
    if (imports.empty()) {
        for (auto& ty : parsed->getExportedTypes()) {
            imported.push_back(ty);
        }
    }
    else {
        for (auto& i : imports) {
            if (auto ty = parsed->getExportedType(FullIdentPath(i->path))) {
                imported.push_back(ty.value());
            }
            else {
                state.error(i->range, "Type \"{}\" not found in \"{}\"", i->path, from);
            }
        }
    }
    for (auto& ty : imported) {
        if (auto name = ty.getName()) {
            if (state.getType(name.value(), true)) {
                state.error(range, "Type \"{}\" already exists in this scope", ty.toString());
            }
            else {
                state.pushType(ty);
            }
        }
    }
    return Type(VoidType(), nullptr);
}

std::string ImportExpr::debug(size_t indent) const {
    return DebugPrint("ImportExpr", indent)
        .member("from", from)
        .member("imports", imports);
}

ExprResult<ListExpr> ListExpr::pull(Stream& stream) {
    Rollback rb(stream);
    Vec<Rc<Expr>> list;
    // handle just {}
    if (Token::peek('}', stream)) {
        return rb.commit<ListExpr>(list);
    }
    while (true) {
        stream.debugTick();
        GEODE_UNWRAP_INTO(auto expr, Expr::pull(stream));
        list.push_back(expr);
        // Allow omitting last semicolon
        if (!Token::pullSemicolons(stream) && !Token::peek('}', stream)) {
            return rb.error("Expected semicolon");
        }
        // End at EOF or }
        if (!Token::peek(stream) || Token::peek('}', stream)) {
            break;
        }
    }
    return rb.commit<ListExpr>(list);
}

Type ListExpr::typecheck(UnitParser& state) const {
    for (auto& expr : exprs) {
        expr->typecheck(state);
    }
    // todo: return types
    return Type(VoidType(), nullptr);
}

std::string ListExpr::debug(size_t indent) const {
    return DebugPrint("ListExpr", indent)
        .member("exprs", exprs);
}

ExprResult<ReturnExpr> ReturnExpr::pull(Stream& stream) {
    Rollback rb(stream);
    GEODE_UNWRAP(Token::pull(Keyword::Return, stream));
    Option<Rc<Expr>> expr;
    if (!Token::peek(';', stream)) {
        GEODE_UNWRAP_INTO(expr, Expr::pull(stream));
    }
    Option<Rc<IdentExpr>> from;
    if (Token::draw(Keyword::From, stream)) {
        GEODE_UNWRAP_INTO(from, IdentExpr::pull(stream));
    }
    return rb.commit<ReturnExpr>(expr, from);
}

Type ReturnExpr::typecheck(UnitParser& state) const {
    if (expr) {
        return expr.value()->typecheck(state);
    }
    else {
        return Primitive::Void;
    }
}

std::string ReturnExpr::debug(size_t indent) const {
    return DebugPrint("ReturnExpr", indent)
        .member("expr", expr)
        .member("from", from);
}

ExprResult<BlockExpr> BlockExpr::pull(Stream& stream) {
    Rollback rb(stream);
    GEODE_UNWRAP(Token::pull('{', stream));
    GEODE_UNWRAP_INTO(auto expr, ListExpr::pull(stream));
    GEODE_UNWRAP(Token::pull('}', stream));
    return rb.commit<BlockExpr>(expr);
}

Type BlockExpr::typecheck(UnitParser& state) const {
    // todo: get label from attribute
    state.pushScope(None, false);
    auto ret = expr->typecheck(state);
    state.popScope();
    return ret;
}

std::string BlockExpr::debug(size_t indent) const {
    return DebugPrint("BlockExpr", indent)
        .member("expr", expr);
}

ExprResult<AST> AST::pull(Stream& stream) {
    Token::skipToNext(stream);
    Rollback rb(stream);
    Vec<Rc<Expr>> exprs;
    while (true) {
        stream.debugTick();
        GEODE_UNWRAP_INTO(auto expr, Expr::pull(stream));
        exprs.push_back(expr);
        GEODE_UNWRAP(Token::pullSemicolons(stream));
        if (!Token::peek(stream)) {
            break;
        }
    }
    return rb.commit<AST>(exprs);
}

Type AST::typecheck(UnitParser& state) const {
    for (auto& expr : exprs) {
        expr->typecheck(state);
    }
    return Type(VoidType(), nullptr);
}

std::string AST::debug(size_t indent) const {
    return DebugPrint("AST", indent)
        .member("exprs", exprs);
}
