# `::implied-bounds`

<img
    src="https://github.com/danielhenrymantilla/implied-bounds.rs/blob/c1244e3dbdc2a263ea5fa752c58d718da833f636/457381166-528cd8ea-f954-434c-a7f2-6147e82cc10b.png"
    height="100px"
/>


Trait trick and helper convenience macro to make **all** of the bounds of a trait definition be
properly _implied/entailed_, as expected, avoiding "non-entailment" bugs which the current trait
system has.

[![Repository](https://img.shields.io/badge/repository-GitHub-brightgreen.svg)](
https://github.com/danielhenrymantilla/implied-bounds.rs)
[![Latest version](https://img.shields.io/crates/v/implied-bounds.svg)](
https://crates.io/crates/implied-bounds)
[![Documentation](https://docs.rs/implied-bounds/badge.svg)](
https://docs.rs/implied-bounds)
[![MSRV](https://img.shields.io/badge/MSRV-1.79.0-white)](
https://gist.github.com/danielhenrymantilla/9b59de4db8e5f2467ed008b3c450527b)
[![unsafe forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg)](
https://github.com/rust-secure-code/safety-dance/)
[![License](https://img.shields.io/crates/l/implied-bounds.svg)](
https://github.com/danielhenrymantilla/implied-bounds.rs/blob/master/LICENSE-ZLIB)
[![CI](https://github.com/danielhenrymantilla/implied-bounds.rs/workflows/CI/badge.svg)](
https://github.com/danielhenrymantilla/implied-bounds.rs/actions)
[![no_std compatible](https://img.shields.io/badge/no__std-compatible-success.svg)](
https://github.com/rust-secure-code/safety-dance/)

<!-- Templated by `cargo-generate` using https://github.com/danielhenrymantilla/proc-macro-template -->

# Prior Context

<img
    src="https://github.com/danielhenrymantilla/implied-bounds.rs/blob/c1244e3dbdc2a263ea5fa752c58d718da833f636/457462208-0e3dc973-57e3-4fcd-9e21-a56e3dff8ffb.png"
    height="300px"
/>

<details class="custom"><summary><span class="summary-box"><span>Click to show</span></span></summary>

In Rust, when defining a `trait` or some helper type, such as a `struct`, certain bounds are
**not** _implied/entailed_, which is probably contrary to the user/human expectation, and thus,
surprising, unintuitive, and being honest, annoying, since it then requires the user of such `trait`
and `struct` to be repeating the bounds.

```rust ,compile_fail
struct Typical<T: Clone>(T);

fn demo<T>(_: Typical<T>) {} // Error, missing `T: Clone`…
```

  - Error message:

    <details class="custom"><summary><span class="summary-box"><span>Click to show</span></span></summary>

    ```rust ,ignore
    # /*
    error[E0277]: the trait bound `T: Clone` is not satisfied
     --> src/lib.rs:3:15
      |
    3 | fn demo<T>(_: Typical<T>) {}
      |               ^^^^^^^^^^ the trait `Clone` is not implemented for `T`
      |
    note: required by a bound in `Typical`
     --> src/lib.rs:1:19
      |
    1 | struct Typical<T: Clone>(T);
      |                   ^^^^^ required by this bound in `Typical`
    help: consider restricting type parameter `T` with trait `Clone`
      |
    3 | fn demo<T: std::clone::Clone>(_: Typical<T>) {}
      |          +++++++++++++++++++
    # */
    ```

    </details>

And likewise for a `trait`:

```rust ,compile_fail
trait MyTrait<U: Clone> {}

fn demo<T: MyTrait<U>, U>() {} // Error, missing `U: Clone`…
```

  - Error message:

    <details class="custom"><summary><span class="summary-box"><span>Click to show</span></span></summary>

    ```rust ,ignore
    # /*
    error[E0277]: the trait bound `U: Clone` is not satisfied
     --> src/lib.rs:3:12
      |
    3 | fn demo<T: MyTrait<U>, U>() {}
      |            ^^^^^^^^^^ the trait `Clone` is not implemented for `U`
      |
    note: required by a bound in `MyTrait`
     --> src/lib.rs:1:18
      |
    1 | trait MyTrait<U: Clone> {}
      |                  ^^^^^ required by this bound in `MyTrait`
    help: consider restricting type parameter `U` with trait `Clone`
      |
    3 | fn demo<T: MyTrait<U>, U: std::clone::Clone>() {}
      |                         +++++++++++++++++++
    # */
    ```

    </details>

Enter this crate.

</details>

# What this crate offers

are tools to alleviate the `trait` case (currently, there is no special tooling for the `struct`
case, but some may be added in the future).

Mainly:

  - the maximally convenient [`#[implied_bounds]`][`implied_bounds`] attribute:

    ```rust
    #[::implied_bounds::implied_bounds] // 👈
    trait MyTrait<U: Clone> {}

    fn demo<T: MyTrait<U>, U>() {} // ✅
    ```

  - Otherwise, if you are wary of magical macros and prefer magical type-system stuff (🤷), if
    anything, because they do not impact from-scratch compile-time, you can directly use
    [`ImpliedPredicate`] (which is the helper trait used by the macro, under the hood, in its
    expansion).

    ```rust
    //              no longer needed, since it's implied by the added clause
    //              (but it can generally still be kept if you so wish)
    //              vvvvvvvvvvvv
    trait MyTrait<U /*: Clone */>
    :
        ::implied_bounds::ImpliedPredicate<U, Impls: Clone> + // 👈
    {}

    fn demo<T: MyTrait<U>, U>() {} // ✅
    ```

      - Note: you can disable the macro API —and thus, its compilation— by disabling the
        otherwise-enabled-by-`"default"` `"proc-macros"` Cargo feature.

        Do note, however, that:

          - the macro involves a couple of extra knowledge-savy heuristics so as to maximize the
            quality of the diagnostics should implementors fail to abide by the clauses of trait;

          - the compile-time impact of `syn` with `"full"` features (the only noticeable thing) is:

              - of about 1 full second, tops;

              - only encountered when compiling _from scratch_, since it's otherwise cached;

              - and only avoidable if you curate every transitive dependency of your dependency tree
                to avoid it, even though inevitably some crate somewhere shall pull it in,
                eventually.

                > _Let he who is without `syn` cast the first `build`._


## Which predicates are currently not implied/entailed?

<details class="custom"><summary><span class="summary-box"><span>Click to show</span></span></summary>

  - Bounds on generic parameters:

    ```rust
    trait Example<U: Clone> {}
    ```

  - `where` clauses wherein the left hand side ("bounded type") of the predicate is neither
    `Self` nor a simple associated type:

    ```rust
    trait Example
    where
        Self : Sized, // entailed, since equivalent to `trait Example : Sized {`,
        Self::SimpleAssoc : Send, // entailed, since equivalent to `type SimpleAssoc : Send`

        // But none of the following clauses are entailed/implied:
        String : Into<Self>,
        for<'r> &'r Self : IntoIterator,
        Self::Gat<true> : Send,
    {
        type SimpleAssoc;

        type Gat<const IS_SEND: bool>;
    }
    ```

      - Note that that last `Self::Gat<true> : Send` clause can be rewritten as a `where` clause on
        the GAT itself:

        ```rust
        trait Example {
            type Gat<const IS_SEND: bool> where Self::Gat<true> : Send;
        }
        ```

        This is both equivalent to the other syntax, and not detected/handled by the
        [`#[implied_bounds]`][`implied_bounds`] attribute, so be aware you may need to hoist such
        clauses to the top-level/`trait`-level `where` clauses in order for the attribute to pick
        them up and make them correctly implied/entailed.

</details>

# Inspiration / Credit be given where it is due

The ideas behind the design of the [`ImpliedPredicate`] trait were brought to my attention courtesy
of the [`::imply-hack`](https://docs.rs/imply-hack/0.1.0) crate, from
[`@gtsiam`](https://github.com/gtsiam). Thank you!

I have gone with my own take on the matter, mainly:

  - to have the convenience macro;

  - to slightly rename the `Is` assoc type as `Impls`, and pay special attention to the
    erroring diagnostics to make them as pretty as possible (mostly done in the macro).

[`implied_bounds`]: https://docs.rs/implied-bounds/*/implied_bounds/attr.implied_bounds.html
[`ImpliedPredicate`]: https://docs.rs/implied-bounds/*/ImpliedPredicate/trait.ImpliedPredicate.html
