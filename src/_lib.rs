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
