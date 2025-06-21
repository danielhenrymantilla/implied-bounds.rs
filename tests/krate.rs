#![cfg(feature = "proc-macros")]

pub extern crate implied_bounds as renamed;
extern crate core as implied_bounds;

#[::renamed::implied_bounds(crate = ::renamed)]
trait _Foo<T : Clone> {}

macro_rules! in_macro {() => (
    #[$crate::macro_internals::renamed::implied_bounds(
        crate = $crate::macro_internals::renamed,
    )]
    trait _Bar<T : Clone> {}
)}
in_macro!();

pub mod macro_internals {
    pub extern crate implied_bounds as renamed;
}
