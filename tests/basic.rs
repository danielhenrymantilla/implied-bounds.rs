#![cfg(feature = "proc-macros")]

pub trait Super { fn super_(&self) {} }

#[::implied_bounds::implied_bounds]
pub trait Foo<T : Clone>
where
    Self : Super,
    Self::Gat<true> : Send,
{
    type Gat<const IS_SEND: bool>;
}

pub enum Bar {}

impl<T : Clone> Foo<T> for Bar {
    type Gat<const __: bool> = ();
}
impl Super for Bar {}

// Here we test that `Self : Super` not get transformed into
// `: ImpliedPredicate<Self, Impls : Super>`, since that would break the `dyn`ability of the trait.
//
// But if the attribute is smart enough not to do that, then it's doing nothing, hence the `allow`.
#[::implied_bounds::implied_bounds(allow_none)]
pub trait Baz
where
    Self : Super,
{
    fn m(&self);
}

pub fn demo(it: &dyn Baz) {
    it.super_();
    it.m();
}

pub fn foo<T, X : Foo<T>>(it: T, x: X) {
    _ = it.clone();
    x.super_();
    let _it = None::<X::Gat<true>>;
    let _: &dyn Send = &_it;
    let _it = None::<X::Gat<false>>;
    // let _: &dyn Send = &it;
}
