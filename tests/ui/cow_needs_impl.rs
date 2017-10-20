#![deny(cow_needs_impl)]
#![allow(dead_code)]

use std::borrow::Cow;

struct Type1;

trait Trait1 {}

impl Trait1 for Type1 {}

trait Trait2 {
    fn do_trait_2(&self);
}

impl<'a, T> Trait2 for Cow<'a, T>
    where T: ToOwned + Trait2,
          <T as ToOwned>::Owned: Trait2
{
    fn do_trait_2(&self) {
        match *self {
            Cow::Owned(ref o) => o.do_trait_2(),
            Cow::Borrowed(b) => b.do_trait_2(),
        }
    }
}

fn main() {}
