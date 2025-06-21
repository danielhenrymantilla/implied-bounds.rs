pub
trait HasAssoc<T : ?Sized> {
    type Impls : ?Sized;
}

impl<T : ?Sized, Self_ : ?Sized> HasAssoc<T> for Self_ {
    type Impls = T;
}

/// Helper trait for the implied/entailed bounds trick.
///
/// # Usage:
///
/// When you have:
///
/// ```rust
/// //        this clause is not entailed
/// //               vvvvvvvv
/// trait SomeTrait<T : Clone>
/// where
///     // And neither is this one.
///     Self::SomeGat<true> : Send,
/// {
///     type SomeGat<const IS_SEND: bool>;
///     // …
/// }
/// ```
///
/// instead, write:
///
/// ```rust
/// use ::implied_bounds::ImpliedPredicate;
///
/// trait SomeTrait<T>
/// :
///     ImpliedPredicate<T, Impls : Clone> +
///     ImpliedPredicate<Self::SomeGat<true>, Impls : Send> +
/// {
///     type SomeGat<const IS_SEND: bool>;
///     // …
/// }
/// ```
///
/// # Convenience macro
///
/// Since this usage is not only not the most obvious to write, but more importantly, not very
/// readable afterwards, this crate exposes a helper convenience macro which shall do this
/// mechanical transformation in your stead; in an automated, reliable, and predictable manner.
///
/// ```rust
/// use ::implied_bounds::implied_bounds;
///
/// #[implied_bounds] // 👈
/// trait SomeTrait<T : Clone>
/// where
///     Self::SomeGat<true> : Send,
/// {
///     type SomeGat<const IS_SEND: bool>;
///     // …
/// }
/// ```
///
/// And _voilà_ 😙👌
pub
trait ImpliedPredicate<T : ?Sized>
:
    HasAssoc<T, Impls = T> +
{}

impl<T : ?Sized, Self_ : ?Sized> ImpliedPredicate<T> for Self_ {}
