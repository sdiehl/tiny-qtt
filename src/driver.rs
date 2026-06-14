use std::fmt::Write;

use crate::elab::{check, infer, is_type, Cxt};
use crate::errors::{TinyQttError, TinyQttResult, TypeError};
use crate::eval::{eval, quote};
use crate::parse::Parser;
use crate::pretty::pretty_tm;
use crate::syntax::Decl;

pub fn check_str(src: &str) -> Result<String, String> {
    check_str_pretty(src).map_err(|e| match e {
        TinyQttError::Type(t) => t.message,
        other => other.to_string(),
    })
}

pub fn check_str_pretty(src: &str) -> TinyQttResult<String> {
    let parser = Parser::new();
    let decls = parser.parse_module(src)?;
    let mut cx = Cxt::default();
    let mut out = String::new();
    for d in decls {
        run_decl(&mut cx, &mut out, d)?;
    }
    Ok(out)
}

fn run_decl(cx: &mut Cxt, out: &mut String, d: Decl) -> TinyQttResult<()> {
    match d {
        Decl::Def(n, ty_raw, body_raw) => {
            let ty_tm = is_type(cx, &ty_raw).map_err(TypeError::new)?;
            let ty_val = eval(&cx.env, &ty_tm);
            let (body_tm, _usage) = check(cx, false, &body_raw, &ty_val).map_err(TypeError::new)?;
            let body_val = eval(&cx.env, &body_tm);
            writeln!(
                out,
                "def {n}\n  : {}\n  := {}",
                pretty_tm(&cx.names, &ty_tm),
                pretty_tm(&cx.names, &body_tm)
            )
            .unwrap();
            *cx = cx.define(n, body_val, ty_val);
        }
        Decl::Eval(raw) => {
            let (tm, ty, _u) = infer(cx, false, &raw).map_err(TypeError::new)?;
            let v = eval(&cx.env, &tm);
            let nf_tm = quote(cx.level(), &v);
            let ty_tm = quote(cx.level(), &ty);
            writeln!(
                out,
                "eval\n  = {}\n  : {}",
                pretty_tm(&cx.names, &nf_tm),
                pretty_tm(&cx.names, &ty_tm)
            )
            .unwrap();
        }
        Decl::Check(raw_tm, raw_ty) => {
            let ty_tm = is_type(cx, &raw_ty).map_err(TypeError::new)?;
            let ty_val = eval(&cx.env, &ty_tm);
            let (tm, _u) = check(cx, false, &raw_tm, &ty_val).map_err(TypeError::new)?;
            writeln!(
                out,
                "check ok\n  : {}\n  ~ {}",
                pretty_tm(&cx.names, &ty_tm),
                pretty_tm(&cx.names, &tm)
            )
            .unwrap();
        }
    }
    Ok(())
}
