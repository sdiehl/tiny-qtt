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
    /// $x$ -- free or bound name, resolved during elaboration.
    Var(Name),
    /// $\mathcal{U}$ -- the universe (`Type` in the surface).
    U,
    /// $(x \overset{\rho}{:} A) \to B$ -- function type whose argument is
    /// used with multiplicity $\rho$.
    Pi(Mult, Name, Box<Self>, Box<Self>),
    /// $\lambda x_1\, \ldots\, x_n.\, t$ -- multi-binder lambda (sugar).
    Lam(Vec<Name>, Box<Self>),
    /// $t\, u$ -- application.
    App(Box<Self>, Box<Self>),
    /// $(x \overset{\rho}{:} A) \otimes B$ -- multiplicative (linear) pair
    /// type. Splitting it hands you both components at once.
    Tensor(Mult, Name, Box<Self>, Box<Self>),
    /// $(a, b)$ -- tensor introduction.
    Pair(Box<Self>, Box<Self>),
    /// $\mathsf{let}\, (x, y) = t\, \mathsf{in}\, u$ -- tensor elimination.
    LetPair(Name, Name, Box<Self>, Box<Self>),
    /// $A \mathbin{\&} B$ -- additive (with) pair type. You may project
    /// either side, but only one, so both share the same resources.
    With(Box<Self>, Box<Self>),
    /// $\langle a, b \rangle$ -- with introduction.
    WPair(Box<Self>, Box<Self>),
    /// $\pi_1\, t$ -- first projection of a with-pair.
    Fst(Box<Self>),
    /// $\pi_2\, t$ -- second projection of a with-pair.
    Snd(Box<Self>),
    /// $\mathbb{B}$.
    Bool,
    /// $\mathsf{true}$.
    True,
    /// $\mathsf{false}$.
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
    /// $x$ -- de Bruijn index.
    Var(Index),
    /// $\mathcal{U}$ -- type-in-type universe (single level for simplicity).
    U,
    /// $(x \overset{\rho}{:} A) \to B$.
    Pi(Mult, Name, Rc<Self>, Rc<Self>),
    /// $\lambda x.\, t$.
    Lam(Name, Rc<Self>),
    /// $t\, u$.
    App(Rc<Self>, Rc<Self>),
    /// $(x \overset{\rho}{:} A) \otimes B$.
    Tensor(Mult, Name, Rc<Self>, Rc<Self>),
    /// $(a, b)$.
    Pair(Rc<Self>, Rc<Self>),
    /// $\mathsf{let}\, (x, y) = t\, \mathsf{in}\, u$; `body` lives under two
    /// extra binders.
    LetPair(Name, Name, Rc<Self>, Rc<Self>),
    /// $A \mathbin{\&} B$.
    With(Rc<Self>, Rc<Self>),
    /// $\langle a, b \rangle$.
    WPair(Rc<Self>, Rc<Self>),
    /// $\pi_1\, t$.
    Fst(Rc<Self>),
    /// $\pi_2\, t$.
    Snd(Rc<Self>),
    /// $\mathbb{B}$.
    Bool,
    /// $\mathsf{true}$.
    True,
    /// $\mathsf{false}$.
    False,
    /// $\mathsf{if}\, c\, \mathsf{then}\, a\, \mathsf{else}\, b$.
    If(Rc<Self>, Rc<Self>, Rc<Self>),
    /// $\mathsf{let}\, x : A := t\, \mathsf{in}\, u$.
    Let(Name, Rc<Self>, Rc<Self>, Rc<Self>),
}

/// Top-level declaration in a `.qtt` file.
#[derive(Clone, Debug)]
pub enum Decl {
    /// `def f : A := t` -- check `t : A` at runtime multiplicity and bind it.
    Def(Name, Raw, Raw),
    /// `eval t` -- elaborate, normalize, and print the result.
    Eval(Raw),
    /// `check t : A` -- assert that `t` checks at `A`, printing its usage.
    Check(Raw, Raw),
}

/// What the REPL grammar accepts on a single line.
#[derive(Clone, Debug)]
pub enum ReplInput {
    Decl(Decl),
    Term(Raw),
}
