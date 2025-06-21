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
pub
trait ImpliedPredicate<T : ?Sized>
:
    HasAssoc<T, Impls = T> +
{}

impl<T : ?Sized, Self_ : ?Sized> ImpliedPredicate<T> for Self_ {}
