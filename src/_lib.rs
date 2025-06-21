#![doc = include_str!("../README.md")]
#![no_std]
#![forbid(unsafe_code)]
#![allow(unused_braces)]

pub use helper_trait::ImpliedPredicate;
mod helper_trait;

/// Convenience attribute macro to help one rewrite a `trait` definition as per the rules described
/// in the documentation of [`ImpliedPredicate`].
///
/// Indeed, that trait is very handy, but its usage is neither very obvious to write, nor very
/// readable afterwards.
///
/// But it is actually a very mechanical operation, hence being a good fit for macro automation 🙂
///
/// ## Example
///
/// The following fails to compile:
///
/// ```rust ,compile_fail
/// trait Trait<U: Clone>
/// where
///     Self::Gat<true>: Send,
/// {
///     type Gat<const IS_SEND: bool>;
/// }
///
/// fn demo<T: Trait<U>, U>()
/// where
///     // ❌ Error, missing:
///     // U: Clone,
///     // T::Gat<true>: Send,
/// {}
/// ```
///
///   - Error message:
///
///     <details class="custom"><summary><span class="summary-box"><span>Click to show</span></span></summary>
///
///     ```rust ,ignore
///     # /*
///     error[E0277]: the trait bound `U: Clone` is not satisfied
///       --> src/_lib.rs:29:12
///        |
///     10 | fn demo<T: Trait<U>, U>()
///        |            ^^^^^^^^ the trait `Clone` is not implemented for `U`
///        |
///     note: required by a bound in `Trait`
///       --> src/_lib.rs:22:16
///        |
///     3  | trait Trait<U: Clone>
///        |                ^^^^^ required by this bound in `Trait`
///     help: consider restricting type parameter `U`
///        |
///     10 | fn demo<T: Trait<U>, U: std::clone::Clone>()
///        |                       +++++++++++++++++++
///
///     error[E0277]: `<T as Trait<U>>::Gat<true>` cannot be sent between threads safely
///       --> src/_lib.rs:29:12
///        |
///     10 | fn demo<T: Trait<U>, U>()
///        |            ^^^^^^^^ `<T as Trait<U>>::Gat<true>` cannot be sent between threads safely
///        |
///        = help: the trait `Send` is not implemented for `<T as Trait<U>>::Gat<true>`
///     note: required by a bound in `Trait`
///       --> src/_lib.rs:24:22
///        |
///     3  | trait Trait<U: Clone>
///        |       ----- required by a bound in this trait
///     4  | where
///     5  |     Self::Gat<true>: Send,
///        |                      ^^^^ required by this bound in `Trait`
///     help: consider further restricting the associated type
///        |
///     11 | where <T as Trait<U>>::Gat<true>: Send
///        |       ++++++++++++++++++++++++++++++++
///     # */
///     ```
///
///     </details>
///
/// You can easily fix this by slapping the <code>[#\[implied_bounds\]][`implied_bounds`]</code>
/// attribute on it:
///
/// ```rust
/// #[::implied_bounds::implied_bounds] // 👈
/// trait Trait<U: Clone>
/// where
///     Self::Gat<true>: Send,
/// {
///     type Gat<const IS_SEND: bool>;
/// }
///
/// fn demo<T: Trait<U>, U>()
/// where
///     // OK ✅
/// {}
/// ```
///
/// This shall not change anything for implementors (they have to abide by the provided
/// bounds/clauses/predicates no matter whether the
/// <code>[#\[implied_bounds\]][`implied_bounds`]</code> attribute is used or not, and it shall
/// suffice).
///
/// ## How does the macro work
///
///   - Tip: you can provide the `debug` arg to the attribute for it to highlight the non-implied
///     clauses it shall rewrite:
///
///     ```rust
///     # /*
///     //                👇
///     #[implied_bounds(debug)]
///     trait ...
///     # */
///     ```
///
/// The attribute identifies the non-implied clauses (bounds on generic type parameters, as well
/// as `where` clauses where the left-hand-side (bounded type) is not `Self`), and rewrites them
/// using [`ImpliedPredicate`], like this:
///
/// ```rust
/// #[::implied_bounds::implied_bounds] // 👈
/// trait Trait<U: Clone>
/// where
///     Self::Gat<true>: Send,
/// {
///     type Gat<const IS_SEND: bool>;
/// }
/// ```
///
/// becomes:
///
/// ```rust
/// trait Trait<U>
/// :
///     ::implied_bounds::ImpliedPredicate<U, Impls: Clone> +
///     ::implied_bounds::ImpliedPredicate<Self::Gat<true>, Impls: Send> +
/// {
///     type Gat<const IS_SEND: bool>;
/// }
/// ```
///
/// where [`ImpliedPredicate`] is trivially-true / always-holding trait clause, on condition
/// that it be well-formed, _i.e._, on condition that the bounds on its `Impls` associated type
/// do hold; where its `Impls` associated type is defined to always be the same as the generic arg fed
/// to it:
///
/// ```rust ,ignore
/// X: Bounds
/// <=>
/// ImpliedPredicate<X, Impls: Bounds>
/// ```
pub use ::implied_bounds_proc_macros::implied_bounds;

// macro internals
#[doc(hidden)] /** Not part of the public API */ pub
mod ඞ {
    pub use ::core; // or `std`

    /// We reëxport this, and rename it, merely so the diagnostics read a bit more nicely:
    ///
    /// That way we get:
    ///
    /// > which is required by `<Bar as implied_bounds::ඞ::ImpliedPredicate<…>`
    ///
    /// instead of:
    ///
    /// > which is required by `<Bar as implied_bounds::helper_trait::HasAssoc<…>`
    pub use crate::helper_trait::HasAssoc as ImpliedPredicate;
}

#[doc = include_str!("compile_fail_tests.md")]
mod _compile_fail_tests {}
