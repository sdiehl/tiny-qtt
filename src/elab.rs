use std::rc::Rc;

use crate::eval::{closure_apply, conv, eval, quote};
use crate::mult::{Mult, Use};
use crate::pretty::pretty_tm;
use crate::syntax::{Index, Level, Name, Raw, Tm};
use crate::value::{Env, Val};

#[derive(Clone, Default, Debug)]
pub struct Cxt {
    pub env: Env,
    pub types: Vec<Val>,
    pub names: Vec<Name>,
}

impl Cxt {
    #[must_use]
    pub const fn level(&self) -> Level {
        self.env.len()
    }

    /// Introduce a fresh rigid variable of type `ty`.
    #[must_use]
    pub fn bind(&self, n: Name, ty: Val) -> Self {
        let mut cx = self.clone();
        let lvl = cx.env.len();
        cx.env.0.push(Val::var(lvl));
        cx.types.push(ty);
        cx.names.push(n);
        cx
    }

    /// Introduce a definition that unfolds to `v` during evaluation.
    #[must_use]
    pub fn define(&self, n: Name, v: Val, ty: Val) -> Self {
        let mut cx = self.clone();
        cx.env.0.push(v);
        cx.types.push(ty);
        cx.names.push(n);
        cx
    }

    #[must_use]
    pub fn lookup(&self, n: &Name) -> Option<(Index, Val)> {
        for i in (0..self.names.len()).rev() {
            if &self.names[i] == n {
                return Some((self.names.len() - 1 - i, self.types[i].clone()));
            }
        }
        None
    }
}

fn err(cx: &Cxt, msg: &str) -> String {
    format!("{msg} (at depth {})", cx.level())
}

fn show(cx: &Cxt, v: &Val) -> String {
    pretty_tm(&cx.names, &quote(cx.level(), v))
}

/// Elaborate a term expected to denote a type. Types live in the erased
/// fragment, so usage is discarded.
pub fn is_type(cx: &Cxt, raw: &Raw) -> Result<Tm, String> {
    let (tm, ty, _) = infer(cx, true, raw)?;
    if conv(cx.level(), &ty, &Val::U) {
        Ok(tm)
    } else {
        Err(err(
            cx,
            &format!("expected a type, found: {}", show(cx, &ty)),
        ))
    }
}

/// Check `raw` against `expected`, returning the elaborated core term and
/// the usage it charges to the context. `erased` marks the 0-fragment,
/// where resource discipline is suspended.
pub fn check(cx: &Cxt, erased: bool, raw: &Raw, expected: &Val) -> Result<(Tm, Use), String> {
    match raw {
        Raw::Lam(ns, body) => check_lam(cx, erased, ns, body, expected),
        Raw::Pair(a, b) => match expected {
            Val::Tensor(rho, _, dom, cod) => {
                let a_erased = erased || *rho == Mult::Zero;
                let (a_tm, u_a) = check(cx, a_erased, a, dom)?;
                let cod_at = closure_apply(cod, eval(&cx.env, &a_tm));
                let (b_tm, u_b) = check(cx, erased, b, &cod_at)?;
                let usage = u_a.scale(*rho).add(&u_b);
                Ok((Tm::Pair(Rc::new(a_tm), Rc::new(b_tm)), usage))
            }
            other => Err(err(
                cx,
                &format!("pair against non-tensor: {}", show(cx, other)),
            )),
        },
        Raw::WPair(a, b) => match expected {
            Val::With(ta, tb) => {
                let (a_tm, u_a) = check(cx, erased, a, ta)?;
                let (b_tm, u_b) = check(cx, erased, b, tb)?;
                if !erased && !use_eq(&u_a, &u_b) {
                    return Err(err(cx, "with-pair components consume different resources"));
                }
                Ok((Tm::WPair(Rc::new(a_tm), Rc::new(b_tm)), u_a))
            }
            other => Err(err(
                cx,
                &format!("with-pair against non-&: {}", show(cx, other)),
            )),
        },
        Raw::If(c, a, b) => {
            let (c_tm, u_c) = check(cx, erased, c, &Val::Bool)?;
            let (a_tm, u_a) = check(cx, erased, a, expected)?;
            let (b_tm, u_b) = check(cx, erased, b, expected)?;
            if !erased && !use_eq(&u_a, &u_b) {
                return Err(err(cx, "branches of `if` consume different resources"));
            }
            let usage = u_c.add(&u_a);
            Ok((Tm::If(Rc::new(c_tm), Rc::new(a_tm), Rc::new(b_tm)), usage))
        }
        Raw::LetPair(x, y, t, body) => check_letpair(cx, erased, x, y, t, body, expected),
        Raw::Let(n, ty, val, body) => {
            let ty_tm = is_type(cx, ty)?;
            let ty_val = eval(&cx.env, &ty_tm);
            let (val_tm, u_val) = check(cx, erased, val, &ty_val)?;
            let val_v = eval(&cx.env, &val_tm);
            let cx2 = cx.define(n.clone(), val_v, ty_val);
            let (body_tm, u_body) = check(&cx2, erased, body, expected)?;
            let (rho_x, u_outer) = u_body.pop();
            let usage = u_outer.add(&u_val.scale(rho_x));
            Ok((
                Tm::Let(n.clone(), Rc::new(ty_tm), Rc::new(val_tm), Rc::new(body_tm)),
                usage,
            ))
        }
        _ => {
            let (tm, inferred, usage) = infer(cx, erased, raw)?;
            if conv(cx.level(), &inferred, expected) {
                Ok((tm, usage))
            } else {
                Err(err(
                    cx,
                    &format!(
                        "type mismatch\n  expected: {}\n  inferred: {}",
                        show(cx, expected),
                        show(cx, &inferred)
                    ),
                ))
            }
        }
    }
}

fn check_lam(
    cx: &Cxt,
    erased: bool,
    ns: &[Name],
    body: &Raw,
    expected: &Val,
) -> Result<(Tm, Use), String> {
    if ns.is_empty() {
        return check(cx, erased, body, expected);
    }
    let (n, rest) = (&ns[0], &ns[1..]);
    match expected {
        Val::Pi(rho, _, dom, cod) => {
            let lvl = cx.level();
            let cx2 = cx.bind(n.clone(), (**dom).clone());
            let cod_at = closure_apply(cod, Val::var(lvl));
            let (body_tm, u_body) = check_lam(&cx2, erased, rest, body, &cod_at)?;
            let (used, u_outer) = u_body.pop();
            if !erased && !used.fits(*rho) {
                return Err(err(
                    cx,
                    &format!("variable `{n}` has multiplicity {rho} but is used {used}"),
                ));
            }
            Ok((Tm::Lam(n.clone(), Rc::new(body_tm)), u_outer))
        }
        other => Err(err(
            cx,
            &format!(
                "lambda binder `{n}` against non-function: {}",
                show(cx, other)
            ),
        )),
    }
}

fn check_letpair(
    cx: &Cxt,
    erased: bool,
    x: &Name,
    y: &Name,
    t: &Raw,
    body: &Raw,
    expected: &Val,
) -> Result<(Tm, Use), String> {
    let (t_tm, t_ty, u_t) = infer(cx, erased, t)?;
    let Val::Tensor(rho, _, dom, cod) = t_ty else {
        return Err(err(
            cx,
            &format!("let-pair on non-tensor: {}", show(cx, &t_ty)),
        ));
    };
    let lvl = cx.level();
    let cx1 = cx.bind(x.clone(), (*dom).clone());
    let cod_at = closure_apply(&cod, Val::var(lvl));
    let cx2 = cx1.bind(y.clone(), cod_at);
    let (body_tm, u_body) = check(&cx2, erased, body, expected)?;
    let (used_x, used_y, u_outer) = u_body.pop2();
    if !erased && !used_x.fits(rho) {
        return Err(err(
            cx,
            &format!("variable `{x}` has multiplicity {rho} but is used {used_x}"),
        ));
    }
    if !erased && !used_y.fits(Mult::One) {
        return Err(err(
            cx,
            &format!("variable `{y}` is linear but is used {used_y}"),
        ));
    }
    let usage = u_outer.add(&u_t);
    Ok((
        Tm::LetPair(x.clone(), y.clone(), Rc::new(t_tm), Rc::new(body_tm)),
        usage,
    ))
}

/// Elaborate the domain and codomain of a dependent binder (`Pi` or
/// `Tensor`). Both live in the erased fragment, so no usage is charged.
fn binder_type(cx: &Cxt, x: &Name, dom: &Raw, body: &Raw) -> Result<(Rc<Tm>, Rc<Tm>), String> {
    let dom_tm = is_type(cx, dom)?;
    let dom_val = eval(&cx.env, &dom_tm);
    let body_tm = is_type(&cx.bind(x.clone(), dom_val), body)?;
    Ok((Rc::new(dom_tm), Rc::new(body_tm)))
}

pub fn infer(cx: &Cxt, erased: bool, raw: &Raw) -> Result<(Tm, Val, Use), String> {
    let n = cx.level();
    match raw {
        Raw::Var(name) => match cx.lookup(name) {
            Some((ix, ty)) => {
                let usage = if erased {
                    Use::zeros(n)
                } else {
                    Use::one(n, n - 1 - ix)
                };
                Ok((Tm::Var(ix), ty, usage))
            }
            None => Err(err(cx, &format!("unbound variable: {name}"))),
        },
        Raw::U => Ok((Tm::U, Val::U, Use::zeros(n))),
        Raw::Bool => Ok((Tm::Bool, Val::U, Use::zeros(n))),
        Raw::True => Ok((Tm::True, Val::Bool, Use::zeros(n))),
        Raw::False => Ok((Tm::False, Val::Bool, Use::zeros(n))),
        Raw::Pi(m, x, dom, body) => {
            let (d, b) = binder_type(cx, x, dom, body)?;
            Ok((Tm::Pi(*m, x.clone(), d, b), Val::U, Use::zeros(n)))
        }
        Raw::Tensor(m, x, dom, body) => {
            let (d, b) = binder_type(cx, x, dom, body)?;
            Ok((Tm::Tensor(*m, x.clone(), d, b), Val::U, Use::zeros(n)))
        }
        Raw::With(a, b) => {
            let a_tm = is_type(cx, a)?;
            let b_tm = is_type(cx, b)?;
            Ok((
                Tm::With(Rc::new(a_tm), Rc::new(b_tm)),
                Val::U,
                Use::zeros(n),
            ))
        }
        Raw::App(f, x) => {
            let (f_tm, f_ty, u_f) = infer(cx, erased, f)?;
            match f_ty {
                Val::Pi(rho, _, dom, cod) => {
                    let arg_erased = erased || rho == Mult::Zero;
                    let (x_tm, u_x) = check(cx, arg_erased, x, &dom)?;
                    let res_ty = closure_apply(&cod, eval(&cx.env, &x_tm));
                    let usage = u_f.add(&u_x.scale(rho));
                    Ok((Tm::App(Rc::new(f_tm), Rc::new(x_tm)), res_ty, usage))
                }
                other => Err(err(
                    cx,
                    &format!("applying non-function: {}", show(cx, &other)),
                )),
            }
        }
        Raw::Fst(t) => {
            let (t_tm, t_ty, u_t) = infer(cx, erased, t)?;
            match t_ty {
                Val::With(a, _) => Ok((Tm::Fst(Rc::new(t_tm)), (*a).clone(), u_t)),
                other => Err(err(cx, &format!("fst on non-&: {}", show(cx, &other)))),
            }
        }
        Raw::Snd(t) => {
            let (t_tm, t_ty, u_t) = infer(cx, erased, t)?;
            match t_ty {
                Val::With(_, b) => Ok((Tm::Snd(Rc::new(t_tm)), (*b).clone(), u_t)),
                other => Err(err(cx, &format!("snd on non-&: {}", show(cx, &other)))),
            }
        }
        Raw::Let(name, ty, val, body) => {
            let ty_tm = is_type(cx, ty)?;
            let ty_val = eval(&cx.env, &ty_tm);
            let (val_tm, u_val) = check(cx, erased, val, &ty_val)?;
            let val_v = eval(&cx.env, &val_tm);
            let cx2 = cx.define(name.clone(), val_v, ty_val);
            let (body_tm, body_ty, u_body) = infer(&cx2, erased, body)?;
            let (rho_x, u_outer) = u_body.pop();
            let usage = u_outer.add(&u_val.scale(rho_x));
            Ok((
                Tm::Let(
                    name.clone(),
                    Rc::new(ty_tm),
                    Rc::new(val_tm),
                    Rc::new(body_tm),
                ),
                body_ty,
                usage,
            ))
        }
        Raw::Ann(t, ty) => {
            let ty_tm = is_type(cx, ty)?;
            let ty_val = eval(&cx.env, &ty_tm);
            let (t_tm, usage) = check(cx, erased, t, &ty_val)?;
            Ok((t_tm, ty_val, usage))
        }
        Raw::Lam(..) => Err(err(cx, "cannot infer a lambda; add a type annotation")),
        Raw::Pair(..) | Raw::WPair(..) | Raw::If(..) | Raw::LetPair(..) => {
            Err(err(cx, "cannot infer this form; add a type annotation"))
        }
    }
}

fn use_eq(a: &Use, b: &Use) -> bool {
    a.0 == b.0
}
