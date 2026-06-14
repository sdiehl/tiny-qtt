use std::rc::Rc;

use crate::mult::Mult;

pub type Name = Rc<str>;

#[must_use]
pub fn name(s: &str) -> Name {
    Rc::from(s)
}

pub type Index = usize;
pub type Level = usize;

/// Surface syntax produced by the parser: named binders, multiplicity
/// annotations, and sugar (`A -> B`, `A * B`, `A & B`) that elaboration
/// resolves into [`Tm`].
#[derive(Clone, Debug)]
pub enum Raw {
    Var(Name),
    U,
    /// $(x \overset{\rho}{:} A) \to B$ -- function whose argument is used with
    /// multiplicity $\rho$.
    Pi(Mult, Name, Box<Self>, Box<Self>),
    Lam(Vec<Name>, Box<Self>),
    App(Box<Self>, Box<Self>),
    /// $(x \overset{\rho}{:} A) \otimes B$ -- multiplicative (linear) pair; a
    /// split hands you both components at once.
    Tensor(Mult, Name, Box<Self>, Box<Self>),
    Pair(Box<Self>, Box<Self>),
    /// $\mathsf{let}\, (x, y) = t\, \mathsf{in}\, u$ -- tensor elimination.
    LetPair(Name, Name, Box<Self>, Box<Self>),
    /// $A \mathbin{\&} B$ -- additive (with) pair; you may project either side
    /// but only one, so both share the same resources.
    With(Box<Self>, Box<Self>),
    WPair(Box<Self>, Box<Self>),
    Fst(Box<Self>),
    Snd(Box<Self>),
    Bool,
    True,
    False,
    /// $\mathsf{if}\, c\, \mathsf{then}\, a\, \mathsf{else}\, b$ -- additive
    /// elimination: both branches must consume the same resources.
    If(Box<Self>, Box<Self>, Box<Self>),
    /// $\mathsf{let}\, x : A := t\, \mathsf{in}\, u$ -- a cut whose bound
    /// variable's multiplicity is whatever the body demands of it.
    Let(Name, Box<Self>, Box<Self>, Box<Self>),
    /// $t : A$ -- type annotation (turns checking into inference).
    Ann(Box<Self>, Box<Self>),
}

/// Core syntax: scope-resolved, sugar-free, de Bruijn indexed.
#[derive(Clone, Debug)]
pub enum Tm {
    Var(Index),
    U,
    Pi(Mult, Name, Rc<Self>, Rc<Self>),
    Lam(Name, Rc<Self>),
    App(Rc<Self>, Rc<Self>),
    Tensor(Mult, Name, Rc<Self>, Rc<Self>),
    Pair(Rc<Self>, Rc<Self>),
    /// `body` lives under the two binders `x` and `y`.
    LetPair(Name, Name, Rc<Self>, Rc<Self>),
    With(Rc<Self>, Rc<Self>),
    WPair(Rc<Self>, Rc<Self>),
    Fst(Rc<Self>),
    Snd(Rc<Self>),
    Bool,
    True,
    False,
    If(Rc<Self>, Rc<Self>, Rc<Self>),
    Let(Name, Rc<Self>, Rc<Self>, Rc<Self>),
}

/// Top-level declaration in a `.qtt` file.
#[derive(Clone, Debug)]
pub enum Decl {
    Def(Name, Raw, Raw),
    Eval(Raw),
    Check(Raw, Raw),
}

#[derive(Clone, Debug)]
pub enum ReplInput {
    Decl(Decl),
    Term(Raw),
}
