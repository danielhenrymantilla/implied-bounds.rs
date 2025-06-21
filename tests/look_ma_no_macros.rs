pub
trait Demo<U : Clone>
:
    ::implied_bounds::ImpliedPredicate<U, Impls : Clone> +
    ::implied_bounds::ImpliedPredicate<Self::Gat<true>, Impls : Send> +
where
    Self::Gat<true> : Send,
{
    type Gat<const IS_SEND: bool>;
}

pub
fn demo<T: ?Sized + Demo<U>, U>() {}
