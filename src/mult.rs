use std::fmt;

/// A resource multiplicity from the zero-one-many semiring $\{0, 1, \omega\}$.
///
/// This is the rig (semiring) that grades every binder in Quantitative
/// Type Theory: `Zero` erases a variable, `One` demands it be used exactly
/// once, and `Many` ($\omega$) permits any number of uses.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mult {
    Zero,
    One,
    Many,
}

impl Mult {
    /// Semiring addition: combine the demands of two independent positions.
    /// $1 + 1 = \omega$ because two linear uses are no longer linear.
    #[must_use]
    pub const fn add(self, other: Self) -> Self {
        match (self, other) {
            (Self::Zero, m) | (m, Self::Zero) => m,
            _ => Self::Many,
        }
    }

    /// Semiring multiplication: scale a usage by how often its context runs.
    /// Absorbing at `Zero`, identity at `One`.
    #[must_use]
    pub const fn mul(self, other: Self) -> Self {
        match (self, other) {
            (Self::Zero, _) | (_, Self::Zero) => Self::Zero,
            (Self::One, m) | (m, Self::One) => m,
            (Self::Many, Self::Many) => Self::Many,
        }
    }

    /// Does an observed usage satisfy a declared budget?
    ///
    /// `Many` is a wildcard (any usage fits), but `Zero` and `One` are
    /// exact: an erased variable must go unused and a linear variable must
    /// be used exactly once. This strictness is what rejects both dropping
    /// and duplicating linear resources.
    #[must_use]
    pub fn fits(self, budget: Self) -> bool {
        match budget {
            Self::Many => true,
            other => self == other,
        }
    }
}

impl fmt::Display for Mult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Zero => write!(f, "0"),
            Self::One => write!(f, "1"),
            Self::Many => write!(f, "w"),
        }
    }
}

/// A usage vector: one [`Mult`] per binder currently in scope, recording
/// how many times each was consumed by a term.
#[derive(Clone, Debug)]
pub struct Use(pub Vec<Mult>);

impl Use {
    #[must_use]
    pub fn zeros(n: usize) -> Self {
        Self(vec![Mult::Zero; n])
    }

    /// The unit vector charging a single use to variable `ix` (a de Bruijn
    /// *level* index into the context), zero everywhere else.
    #[must_use]
    pub fn one(n: usize, ix: usize) -> Self {
        let mut v = vec![Mult::Zero; n];
        v[ix] = Mult::One;
        Self(v)
    }

    const fn from_vec(v: Vec<Mult>) -> Self {
        Self(v)
    }

    /// Pointwise semiring addition of two usage vectors of equal length.
    #[must_use]
    pub fn add(&self, other: &Self) -> Self {
        Self::from_vec(
            self.0
                .iter()
                .zip(&other.0)
                .map(|(a, b)| a.add(*b))
                .collect(),
        )
    }

    /// Scale every entry by `m` (the multiplicity at which this term runs).
    #[must_use]
    pub fn scale(&self, m: Mult) -> Self {
        Self::from_vec(self.0.iter().map(|a| a.mul(m)).collect())
    }

    /// Drop the most recently bound variable, returning its usage and the
    /// usage of the remaining outer context.
    #[must_use]
    pub fn pop(mut self) -> (Mult, Self) {
        let m = self.0.pop().unwrap_or(Mult::Zero);
        (m, self)
    }

    /// Drop the two most recently bound variables (a tensor split binds
    /// two at once), returning each one's usage and the outer remainder.
    #[must_use]
    pub fn pop2(self) -> (Mult, Mult, Self) {
        let (snd, rest) = self.pop();
        let (fst, rest) = rest.pop();
        (fst, snd, rest)
    }
}
