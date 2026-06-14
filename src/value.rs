use std::fmt;
use std::rc::Rc;

use crate::mult::Mult;
use crate::syntax::{Level, Name, Tm};

#[derive(Clone, Debug, Default)]
pub struct Env(pub Vec<Val>);

impl Env {
    #[must_use]
    pub fn extend(&self, v: Val) -> Self {
        let mut new = self.clone();
        new.0.push(v);
        new
    }

    #[must_use]
    pub fn lookup(&self, ix: usize) -> Val {
        self.0[self.0.len() - 1 - ix].clone()
    }

    #[must_use]
    pub const fn len(&self) -> usize {
        self.0.len()
    }

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

/// A semantic closure $\langle \rho, t \rangle$: capture the environment now,
/// evaluate the single-binder body `t` when applied.
#[derive(Clone)]
pub struct Closure(pub Env, pub Rc<Tm>);

impl fmt::Debug for Closure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Closure({:?})", self.1)
    }
}

/// Semantic values in weak-head normal form. Introductions are concrete;
/// eliminations blocked on a neutral head accumulate in [`Stuck`].
#[derive(Clone, Debug)]
pub enum Val {
    /// $\lambda x.\, t$.
    Lam(Name, Closure),
    /// $(x \overset{\rho}{:} A) \to B$.
    Pi(Mult, Name, Rc<Self>, Closure),
    /// $(x \overset{\rho}{:} A) \otimes B$.
    Tensor(Mult, Name, Rc<Self>, Closure),
    /// $(a, b)$ -- tensor introduction.
    Pair(Rc<Self>, Rc<Self>),
    /// $A \mathbin{\&} B$.
    With(Rc<Self>, Rc<Self>),
    /// $\langle a, b \rangle$ -- with introduction.
    WPair(Rc<Self>, Rc<Self>),
    /// $\mathcal{U}$.
    U,
    /// $\mathbb{B}$.
    Bool,
    /// $\mathsf{true}$.
    True,
    /// $\mathsf{false}$.
    False,
    /// A neutral term: a rigid head under a spine of stuck eliminations.
    Stuck(Head, Spine),
}

/// Head of a neutral term. With no top-level signature inside [`Val`] only
/// rigid de Bruijn *levels* appear.
#[derive(Clone, Debug)]
pub enum Head {
    Var(Level),
}

/// Spine of eliminations applied to a neutral head, outermost first.
pub type Spine = Vec<Elim>;

/// A single elimination frame, the dual of a [`Val`] introduction.
#[derive(Clone, Debug)]
pub enum Elim {
    /// $\square\, v$ -- function application.
    App(Val),
    /// $\pi_1\, \square$.
    Fst,
    /// $\pi_2\, \square$.
    Snd,
    /// $\mathsf{if}\, \square\, \mathsf{then}\, a\, \mathsf{else}\, b$.
    If(Val, Val),
    /// $\mathsf{let}\, (x, y) = \square\, \mathsf{in}\, u$ -- the body `u`
    /// lives under two binders, closed over the captured environment.
    Split(Name, Name, Env, Rc<Tm>),
}

impl Val {
    #[must_use]
    pub const fn var(lvl: Level) -> Self {
        Self::Stuck(Head::Var(lvl), Vec::new())
    }
}
