#![cfg(feature = "proc-macros")]

#[::implied_bounds::implied_bounds]
trait Iter<'r, _Bounds = &'r Self> : 'r
where
    &'r Self : IntoIterator<Item = Self::IterItem>,
{
    type IterItem;

    fn iter(self: &'r Self) -> <&'r Self as IntoIterator>::IntoIter {
        self.into_iter()
    }
}

impl<'r, T : ?Sized> Iter<'r> for T
where
    &'r Self : IntoIterator,
{
    type IterItem = <&'r Self as IntoIterator>::Item;
}

fn debug_twice(it: impl Send + for<'r> Iter<'r, IterItem : ::core::fmt::Debug>)
{
    ::std::thread::scope(|s| _ = s.spawn(move || {
        it.iter().for_each(|it| _ = dbg!(it));
        it.iter().for_each(|it| _ = dbg!(it));
    }));
}

pub fn main() {
    use ::core::cell::Cell;
    debug_twice([Cell::new(42), Cell::new(27)]);
}
