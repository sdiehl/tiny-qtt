use pretty::RcDoc;

use crate::mult::Mult;
use crate::syntax::{Name, Tm};

const WIDTH: usize = 100;

const ATOM: u8 = 5;
const APP: u8 = 4;
const STAR: u8 = 3;
const AMP: u8 = 2;
const ARROW: u8 = 1;
const TOP: u8 = 0;

#[must_use]
pub fn pretty_tm(names: &[Name], tm: &Tm) -> String {
    let mut env: Vec<Name> = names.to_vec();
    doc(&mut env, tm, TOP).pretty(WIDTH).to_string()
}

fn text(s: impl Into<String>) -> RcDoc<'static> {
    RcDoc::text(s.into())
}

fn lookup(env: &[Name], ix: usize) -> RcDoc<'static> {
    if ix >= env.len() {
        text(format!("?{ix}"))
    } else {
        text(env[env.len() - 1 - ix].as_ref().to_string())
    }
}

fn paren(level: u8, my: u8, d: RcDoc<'static>) -> RcDoc<'static> {
    if level > my {
        text("(").append(d).append(text(")"))
    } else {
        d
    }
}

fn mult_prefix(m: Mult) -> String {
    match m {
        Mult::Many => String::new(),
        other => format!("{other} "),
    }
}

fn binder_head(
    env: &mut Vec<Name>,
    m: Mult,
    n: &Name,
    dom: &Tm,
    op: &str,
    default: Mult,
    dom_lvl: u8,
) -> RcDoc<'static> {
    let show_mult = m != default;
    if n.as_ref() == "_" && !show_mult {
        doc(env, dom, dom_lvl).append(text(format!(" {op} ")))
    } else {
        let prefix = if show_mult {
            mult_prefix(m)
        } else {
            String::new()
        };
        text(format!("({prefix}{n} : "))
            .append(doc(env, dom, TOP))
            .append(text(format!(") {op} ")))
    }
}

fn doc(env: &mut Vec<Name>, tm: &Tm, lvl: u8) -> RcDoc<'static> {
    match tm {
        Tm::Var(ix) => lookup(env, *ix),
        Tm::U => text("Type"),
        Tm::Bool => text("Bool"),
        Tm::True => text("true"),
        Tm::False => text("false"),
        Tm::App(f, x) => paren(
            lvl,
            APP,
            doc(env, f, APP).append(text(" ")).append(doc(env, x, ATOM)),
        ),
        Tm::Lam(n, body) => {
            let head = text(format!("\\{n} => "));
            env.push(n.clone());
            let body_doc = doc(env, body, TOP);
            env.pop();
            paren(lvl, TOP, head.append(body_doc))
        }
        Tm::Pi(m, n, dom, cod) => {
            let head = binder_head(env, *m, n, dom, "->", Mult::Many, AMP);
            env.push(n.clone());
            let cod_doc = doc(env, cod, ARROW);
            env.pop();
            paren(lvl, ARROW, head.append(cod_doc))
        }
        Tm::Tensor(m, n, dom, cod) => {
            let head = binder_head(env, *m, n, dom, "*", Mult::One, APP);
            env.push(n.clone());
            let cod_doc = doc(env, cod, STAR);
            env.pop();
            paren(lvl, STAR, head.append(cod_doc))
        }
        Tm::With(a, b) => paren(
            lvl,
            AMP,
            doc(env, a, STAR)
                .append(text(" & "))
                .append(doc(env, b, AMP)),
        ),
        Tm::Pair(a, b) => text("(")
            .append(doc(env, a, TOP))
            .append(text(", "))
            .append(doc(env, b, TOP))
            .append(text(")")),
        Tm::WPair(a, b) => text("<")
            .append(doc(env, a, TOP))
            .append(text(", "))
            .append(doc(env, b, TOP))
            .append(text(">")),
        Tm::Fst(t) => paren(lvl, APP, text("fst ").append(doc(env, t, ATOM))),
        Tm::Snd(t) => paren(lvl, APP, text("snd ").append(doc(env, t, ATOM))),
        Tm::If(c, a, b) => {
            let d = text("if ")
                .append(doc(env, c, APP))
                .append(text(" then "))
                .append(doc(env, a, APP))
                .append(text(" else "))
                .append(doc(env, b, APP));
            paren(lvl, TOP, d)
        }
        Tm::LetPair(x, y, t, body) => {
            let head = text(format!("let ({x}, {y}) = "))
                .append(doc(env, t, TOP))
                .append(text(" in "));
            env.push(x.clone());
            env.push(y.clone());
            let body_doc = doc(env, body, TOP);
            env.pop();
            env.pop();
            paren(lvl, TOP, head.append(body_doc))
        }
        Tm::Let(n, ty, val, body) => {
            let head = text(format!("let {n} : "))
                .append(doc(env, ty, TOP))
                .append(text(" := "))
                .append(doc(env, val, TOP))
                .append(text(" in "));
            env.push(n.clone());
            let body_doc = doc(env, body, TOP);
            env.pop();
            paren(lvl, TOP, head.append(body_doc))
        }
    }
}
