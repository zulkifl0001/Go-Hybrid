use hy_emit::emit_go;
use hy_syntax::parse_file;

fn emit(src: &str) -> String {
    let ast = parse_file("test.hy", src).expect("test source must parse");
    emit_go(&ast).expect("test source must emit cleanly")
}

#[test]
fn parenthesized_arg_to_pointer_param_auto_addresses() {
    let go = emit(
        "struct holder { n int }\n\
         fn inc(p *holder) {\n\
             p.n = p.n + 1\n\
         }\n\
         fn main() {\n\
             h := holder{n: 1}\n\
             inc((h))\n\
         }\n",
    );
    assert!(
        go.contains("inc(&h)"),
        "expected `inc(&h)` for parenthesized addressable arg, got:\n{go}"
    );
}

#[test]
fn user_struct_method_pointer_param_auto_addresses() {
    let go = emit(
        "struct holder { n int }\n\
         fn (a *holder) take(p *holder) {\n\
             a.n = a.n + p.n\n\
         }\n\
         fn main() {\n\
             a := holder{n: 1}\n\
             b := holder{n: 2}\n\
             a.take(b)\n\
         }\n",
    );
    assert!(
        go.contains("a.take(&b)"),
        "expected `a.take(&b)` for user struct method pointer param, got:\n{go}"
    );
}

#[test]
fn local_closure_pointer_param_auto_addresses() {
    let go = emit(
        "struct holder { n int }\n\
         fn main() {\n\
             h := holder{n: 1}\n\
             f := fn(p *holder) {\n\
                 p.n = p.n + 1\n\
             }\n\
             f(h)\n\
         }\n",
    );
    assert!(
        go.contains("f(&h)"),
        "expected `f(&h)` for local closure pointer param, got:\n{go}"
    );
}

#[test]
fn iife_closure_literal_pointer_param_auto_addresses() {
    let go = emit(
        "struct holder { n int }\n\
         fn main() {\n\
             h := holder{n: 1}\n\
             (fn(p *holder) {\n\
                 p.n = p.n + 1\n\
             })(h)\n\
         }\n",
    );
    assert!(
        go.contains("(&h)"),
        "expected `(&h)` for immediately-invoked closure literal pointer param, got:\n{go}"
    );
}

#[test]
fn pointer_param_negative_controls_do_not_over_address() {
    let go = emit(
        "struct holder { n int }\n\
         fn inc(p *holder) {\n\
             p.n = p.n + 1\n\
         }\n\
         fn read(v holder) int {\n\
             return v.n\n\
         }\n\
         fn main() {\n\
             h := holder{n: 1}\n\
             p := &h\n\
             inc((p))\n\
             _ = read(h)\n\
             inc := fn(v holder) {\n\
                 _ = v.n\n\
             }\n\
             inc(h)\n\
         }\n",
    );
    assert!(
        !go.contains("inc(&p)"),
        "must not double-address pointer variable `(p)`:\n{go}"
    );
    assert!(
        go.contains("read(h)"),
        "value parameter `read(h)` must stay `read(h)`:\n{go}"
    );
    assert!(
        go.contains("inc(h)"),
        "shadowed local closure `inc(v holder)` must not inherit top-level `inc(p *holder)`:\n{go}"
    );
}
