# tiny-qtt

A small Rust type checker for Quantitative Type Theory, written mostly while I was on holiday after a few too many glasses of wine. So this is just for fun and lolz.

Based on McBride's [I Got Plenty o' Nuttin'](https://personal.cis.strath.ac.uk/conor.mcbride/PlentyO-CR.pdf) and Atkey's [Syntax and Semantics of Quantitative Type Theory](https://bentnib.org/quantitative-type-theory.pdf).

The neat trick is that every binder carries a multiplicity, a little budget drawn from the zero-one-many semiring:

- `0` means erased. The variable is real to the type checker and invisible at runtime, perfect for the phantom type arguments you never wanted to pay for.
- `1` means linear. Use it exactly once. Not zero times, not twice, once, like a good bottle.
- `w` means whatever. Use it as much as you like.

The checker drags a usage vector around and reconciles it against those budgets. The pleasant surprise is how much falls out for free: linearity, erasure, and the difference between multiplicative pairs `*` (you get both) and additive pairs `&` (you pick one) are all just the same accounting done honestly.

```bash
cargo build
cargo run -- repl
```

```text
tiny-qtt REPL. Type :? for help, :q to quit.
> def id : (0 A : Type) -> (1 x : A) -> A := \A x => x
def id
  : (0 A : Type) -> (1 x : A) -> A
  := \A => \x => x
> :t id Bool
id Bool : (1 x : Bool) -> Bool
> id Bool true
true
  : Bool
> def dup : (0 A : Type) -> (w x : A) -> A * A := \A x => (x, x)
> dup Bool true
(true, true)
  : Bool * Bool
> :q
```

Note that `dup` needs `w` on its argument. Ask for `1` there and the checker will, quite correctly, refuse to let you have your value and copy it too.

```bash
cargo run --example demo
cargo run -- check tests/cases/04_tensor_swap.qtt
cargo run -- check tests/cases/21_err_use_linear_twice.qtt
```

## License

MIT. See [LICENSE](LICENSE).
