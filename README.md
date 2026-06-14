# tiny-qtt

My first attempt at a basic Rust implementation of Quantitative Type Theory, following McBride's [I Got Plenty o' Nuttin'](https://personal.cis.strath.ac.uk/conor.mcbride/PlentyO-CR.pdf) and Atkey's [Syntax and Semantics of Quantitative Type Theory](https://bentnib.org/quantitative-type-theory.pdf).

QTT is cool because every binder is graded by a multiplicity drawn from the zero-one-many semiring: `0` erases a variable (it exists only for typing), `1` demands it be used exactly once, and `w` allows any number of uses. The type checker tracks a usage vector and checks it against these budgets, so linearity, erasure, and the contrast between multiplicative `*` and additive `&` pairs all fall out of one resource discipline.

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

## What's inside

- `mult.rs` -- the `{0, 1, w}` semiring and usage vectors (add, scale, fits)
- `syntax.rs` / `value.rs` -- surface, core, and semantic (NbE) terms
- `eval.rs` -- evaluation, normalization by evaluation, and conversion
- `elab.rs` -- bidirectional elaboration that counts usage against budgets
- `parser.lalrpop` / `lexer.rs` -- the surface grammar
- a REPL plus `check` for `.qtt` files, with `ariadne` diagnostics

The repl commands:

| command     | effect                                 |
| ----------- | -------------------------------------- |
| `:t <expr>` | infer the type of `<expr>`             |
| `:l <file>` | load definitions from a file           |
| `:?`        | show help                              |
| `:q`        | quit                                   |
| `<decl>`    | run a `def` / `eval` / `check` decl    |
| `<expr>`    | evaluate to normal form and print type |

```bash
cargo run --example demo
cargo run -- check tests/cases/04_tensor_swap.qtt
cargo run -- check tests/cases/21_err_use_linear_twice.qtt
```

The `tests/cases` directory holds twenty-eight worked scenarios (each with a committed snapshot): linear and erased identities, currying between `*` and `->`, additive sharing through `&`, resource-polymorphic `let`, and the rejections (using a linear value twice, dropping it, using an erased one at runtime).

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
