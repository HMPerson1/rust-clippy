#![deny(os_c_str_conv)]
#![allow(needless_pass_by_value)]

use std::ffi::{CStr, CString, OsStr, OsString};

trait TraitAll {
    type T;
    fn t(&self) -> Self::T;
}

impl TraitAll for String {
    type T = u16;
    fn t(&self) -> Self::T { 0 }
}

impl<'a> TraitAll for &'a str {
    type T = u32;
    fn t(&self) -> Self::T { 1 }
}

impl<'a> TraitAll for &'a OsStr {
    type T = u8;
    fn t(&self) -> Self::T { 2 }
}

fn foo1<A: TraitAll, B: TraitAll>(_a1: A, _a2: A, _b: B) {}
fn foo2<A: TraitAll>(a: A) -> A::T { a.t() }

fn bad() {
    {
        let mut files = Vec::new();
        for arg in std::env::args() {
            files.push(std::fs::File::open(arg));
        }
    }

    let _ = std::fs::File::open(std::env::var("foo").unwrap());

    {
        if let Ok(v) = std::env::var("foo") {
            std::fs::File::open(v);
        }
    }

    let _cabt_o: &'static str = "hi";
    std::fs::File::open(_cabt_o);
    foo1("", "", _cabt_o);
    foo1("", _cabt_o, _cabt_o);
}

fn ok() {
    let _ = std::fs::File::open(std::env::var_os("foo").unwrap());
    let _cabt_o: &'static str = "hi";
    foo1(_cabt_o, "aaa", <&std::ffi::OsStr>::default());
    foo2(_cabt_o);
    {
        let mut v = Vec::new();
        v.push(_cabt_o);
        Vec::push(&mut v, _cabt_o);
    }
}

fn main() {
    let _cabt_o: &'static str = "hi";
    bad();
    foo1("", "", _cabt_o);
    foo1("", _cabt_o, _cabt_o);
    ok();
    foo1(_cabt_o, "aaa", <&std::ffi::OsStr>::default());
    foo2(_cabt_o);
}
