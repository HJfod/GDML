
fun getAncestorBinding(this: Reflect::Expr) -> Reflect::Field? {
    let parent = unwrap this.parentExpr;
    // skip uses of bindings inside functions
    if parent is Reflect::FunDecl {
        return none;
    }
    if parent is Reflect::Field {
        return parent;
    }
    return parent.getAncestorBinding();
}

// GemScript distinguishes between compile time and runtime operations
// Operations like evaluating a literal like `5` is available both at runtime and compile time
// Operations like calling a macro or codegen are compile time only
// Calling a function marked as `extern` is runtime only

public macro @reactive(prop: &var Reflect::Field) {
    let refs = Reflect::findRefs(prop);
    for ref in refs {
        if let bind = ref.getAncestorBinding() {
            // codegen is a special block (like quote! in rust) that parses the code
            // anything used inside `@(...)` will be interpolated inside the code
            prop.set.body.list.push(codegen {
                @(bind.parent.name).@(bind.name) = @(bind.value);
            });
        }
    }
}
