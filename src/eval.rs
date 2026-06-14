use std::rc::Rc;

use crate::syntax::{Level, Name, Tm};
use crate::value::{Closure, Elim, Env, Head, Spine, Val};

#[must_use]
pub fn eval(env: &Env, tm: &Tm) -> Val {
    match tm {
        Tm::Var(ix) => env.lookup(*ix),
        Tm::U => Val::U,
        Tm::Bool => Val::Bool,
        Tm::True => Val::True,
        Tm::False => Val::False,
        Tm::Pi(m, n, a, b) => Val::Pi(
            *m,
            n.clone(),
            Rc::new(eval(env, a)),
            Closure(env.clone(), b.clone()),
        ),
        Tm::Lam(n, body) => Val::Lam(n.clone(), Closure(env.clone(), body.clone())),
        Tm::App(f, x) => apply(eval(env, f), eval(env, x)),
        Tm::Tensor(m, n, a, b) => Val::Tensor(
            *m,
            n.clone(),
            Rc::new(eval(env, a)),
            Closure(env.clone(), b.clone()),
        ),
        Tm::Pair(a, b) => Val::Pair(Rc::new(eval(env, a)), Rc::new(eval(env, b))),
        Tm::LetPair(x, y, t, body) => do_split(eval(env, t), x, y, env, body),
        Tm::With(a, b) => Val::With(Rc::new(eval(env, a)), Rc::new(eval(env, b))),
        Tm::WPair(a, b) => Val::WPair(Rc::new(eval(env, a)), Rc::new(eval(env, b))),
        Tm::Fst(t) => do_fst(eval(env, t)),
        Tm::Snd(t) => do_snd(eval(env, t)),
        Tm::If(c, a, b) => do_if(eval(env, c), eval(env, a), eval(env, b)),
        Tm::Let(_, _, t, body) => eval(&env.extend(eval(env, t)), body),
    }
}

#[must_use]
pub fn closure_apply(cl: &Closure, v: Val) -> Val {
    eval(&cl.0.extend(v), &cl.1)
}

#[must_use]
pub fn apply(f: Val, x: Val) -> Val {
    match f {
        Val::Lam(_, cl) => closure_apply(&cl, x),
        Val::Stuck(h, mut sp) => {
            sp.push(Elim::App(x));
            Val::Stuck(h, sp)
        }
        v => panic!("apply: not a function: {v:?}"),
    }
}

#[must_use]
pub fn do_fst(v: Val) -> Val {
    match v {
        Val::WPair(a, _) => (*a).clone(),
        Val::Stuck(h, mut sp) => {
            sp.push(Elim::Fst);
            Val::Stuck(h, sp)
        }
        v => panic!("fst: not a with-pair: {v:?}"),
    }
}

#[must_use]
pub fn do_snd(v: Val) -> Val {
    match v {
        Val::WPair(_, b) => (*b).clone(),
        Val::Stuck(h, mut sp) => {
            sp.push(Elim::Snd);
            Val::Stuck(h, sp)
        }
        v => panic!("snd: not a with-pair: {v:?}"),
    }
}

#[must_use]
pub fn do_if(c: Val, a: Val, b: Val) -> Val {
    match c {
        Val::True => a,
        Val::False => b,
        Val::Stuck(h, mut sp) => {
            sp.push(Elim::If(a, b));
            Val::Stuck(h, sp)
        }
        v => panic!("if: not a Bool: {v:?}"),
    }
}

#[must_use]
pub fn do_split(t: Val, x: &Name, y: &Name, env: &Env, body: &Rc<Tm>) -> Val {
    match t {
        Val::Pair(a, b) => eval(&env.extend((*a).clone()).extend((*b).clone()), body),
        Val::Stuck(h, mut sp) => {
            sp.push(Elim::Split(x.clone(), y.clone(), env.clone(), body.clone()));
            Val::Stuck(h, sp)
        }
        v => panic!("let-pair: not a tensor: {v:?}"),
    }
}

#[must_use]
pub fn quote(lvl: Level, v: &Val) -> Tm {
    match v {
        Val::Lam(n, cl) => {
            let body = closure_apply(cl, Val::var(lvl));
            Tm::Lam(n.clone(), Rc::new(quote(lvl + 1, &body)))
        }
        Val::Pi(m, n, dom, cod) => {
            let body = closure_apply(cod, Val::var(lvl));
            Tm::Pi(
                *m,
                n.clone(),
                Rc::new(quote(lvl, dom)),
                Rc::new(quote(lvl + 1, &body)),
            )
        }
        Val::Tensor(m, n, dom, cod) => {
            let body = closure_apply(cod, Val::var(lvl));
            Tm::Tensor(
                *m,
                n.clone(),
                Rc::new(quote(lvl, dom)),
                Rc::new(quote(lvl + 1, &body)),
            )
        }
        Val::Pair(a, b) => Tm::Pair(Rc::new(quote(lvl, a)), Rc::new(quote(lvl, b))),
        Val::With(a, b) => Tm::With(Rc::new(quote(lvl, a)), Rc::new(quote(lvl, b))),
        Val::WPair(a, b) => Tm::WPair(Rc::new(quote(lvl, a)), Rc::new(quote(lvl, b))),
        Val::U => Tm::U,
        Val::Bool => Tm::Bool,
        Val::True => Tm::True,
        Val::False => Tm::False,
        Val::Stuck(h, sp) => quote_stuck(lvl, h, sp),
    }
}

fn quote_stuck(lvl: Level, h: &Head, sp: &Spine) -> Tm {
    let Head::Var(l) = h;
    let mut acc = Tm::Var(lvl - l - 1);
    for e in sp {
        acc = match e {
            Elim::App(v) => Tm::App(Rc::new(acc), Rc::new(quote(lvl, v))),
            Elim::Fst => Tm::Fst(Rc::new(acc)),
            Elim::Snd => Tm::Snd(Rc::new(acc)),
            Elim::If(a, b) => Tm::If(Rc::new(acc), Rc::new(quote(lvl, a)), Rc::new(quote(lvl, b))),
            Elim::Split(x, y, env, body) => {
                let opened = eval(&env.extend(Val::var(lvl)).extend(Val::var(lvl + 1)), body);
                Tm::LetPair(
                    x.clone(),
                    y.clone(),
                    Rc::new(acc),
                    Rc::new(quote(lvl + 2, &opened)),
                )
            }
        };
    }
    acc
}

#[must_use]
pub fn nf(env: &Env, tm: &Tm) -> Tm {
    quote(env.len(), &eval(env, tm))
}

#[must_use]
pub fn conv(lvl: Level, v1: &Val, v2: &Val) -> bool {
    match (v1, v2) {
        (Val::U, Val::U)
        | (Val::Bool, Val::Bool)
        | (Val::True, Val::True)
        | (Val::False, Val::False) => true,
        (Val::Pi(m1, _, d1, c1), Val::Pi(m2, _, d2, c2))
        | (Val::Tensor(m1, _, d1, c1), Val::Tensor(m2, _, d2, c2)) => {
            m1 == m2 && conv(lvl, d1, d2) && {
                let a = Val::var(lvl);
                conv(
                    lvl + 1,
                    &closure_apply(c1, a.clone()),
                    &closure_apply(c2, a),
                )
            }
        }
        (Val::With(a1, b1), Val::With(a2, b2))
        | (Val::Pair(a1, b1), Val::Pair(a2, b2))
        | (Val::WPair(a1, b1), Val::WPair(a2, b2)) => conv(lvl, a1, a2) && conv(lvl, b1, b2),
        (Val::Lam(_, c1), Val::Lam(_, c2)) => {
            let a = Val::var(lvl);
            conv(
                lvl + 1,
                &closure_apply(c1, a.clone()),
                &closure_apply(c2, a),
            )
        }
        (Val::Lam(_, c), other) | (other, Val::Lam(_, c)) => {
            let a = Val::var(lvl);
            conv(
                lvl + 1,
                &closure_apply(c, a.clone()),
                &apply(other.clone(), a),
            )
        }
        (Val::WPair(a, b), other) | (other, Val::WPair(a, b)) => {
            conv(lvl, a, &do_fst(other.clone())) && conv(lvl, b, &do_snd(other.clone()))
        }
        (Val::Stuck(Head::Var(l1), s1), Val::Stuck(Head::Var(l2), s2)) => {
            l1 == l2 && conv_spine(lvl, s1, s2)
        }
        _ => false,
    }
}

fn conv_spine(lvl: Level, s1: &Spine, s2: &Spine) -> bool {
    s1.len() == s2.len()
        && s1.iter().zip(s2).all(|(a, b)| match (a, b) {
            (Elim::App(x), Elim::App(y)) => conv(lvl, x, y),
            (Elim::Fst, Elim::Fst) | (Elim::Snd, Elim::Snd) => true,
            (Elim::If(a1, b1), Elim::If(a2, b2)) => conv(lvl, a1, a2) && conv(lvl, b1, b2),
            (Elim::Split(_, _, e1, b1), Elim::Split(_, _, e2, b2)) => {
                let l = Val::var(lvl);
                let r = Val::var(lvl + 1);
                let v1 = eval(&e1.extend(l.clone()).extend(r.clone()), b1);
                let v2 = eval(&e2.extend(l).extend(r), b2);
                conv(lvl + 2, &v1, &v2)
            }
            _ => false,
        })
}
