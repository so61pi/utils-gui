/// Use `clone!(a, mut b, c)` to quickly clone variables reusing the same name.
#[macro_export]
macro_rules! clone {
    () => {};
    ($name:ident) => {
        let $name = $name.clone();
    };
    (mut $name:ident) => {
        let mut $name = $name.clone();
    };
    ($name:ident, $($tail:tt)*) => {
        clone!($name);
        clone!($($tail)*);
    };
    (mut $name:ident, $($tail:tt)*) => {
        clone!(mut $name);
        clone!($($tail)*);
    };
}

/// Use `idgen!{ AaaId }` to create `AaaId` and `AaaIdGen`.
#[macro_export]
macro_rules! idgen {
    ($($name:ident),+) => {
        $(
            paste::paste! {
                #[allow(non_snake_case)]
                mod [<$name Gen_mod>] {
                    use std::{cell::Cell, rc::Rc};

                    #[allow(dead_code)]
                    #[derive(Debug, Clone, Copy, PartialEq, Hash, derive_more::Display)]
                    pub struct $name(u64);

                    impl $name {
                        #[allow(dead_code)]
                        pub fn gnext() -> Self {
                            [<$name Gen>]::gnext()
                        }
                    }

                    #[doc = concat!(stringify!([<$name Gen>]), " is id generator for ", stringify!($name))]
                    #[allow(dead_code)]
                    #[derive(Debug, Clone)]
                    pub struct [<$name Gen>] {
                        id: Rc<Cell<u64>>,
                    }

                    impl [<$name Gen>] {
                        #[allow(dead_code)]
                        pub fn gnext() -> $name {
                            thread_local!(static IDGEN: [<$name Gen>] = [<$name Gen>]::new(1_000_000_u64));
                            IDGEN.with(|f| f.next())
                        }

                        #[allow(dead_code)]
                        pub fn new(init: u64) -> Self {
                            Self { id: Rc::new(Cell::new(init)) }
                        }

                        #[allow(dead_code)]
                        pub fn next(&self) -> $name {
                            self.id.replace(self.id.get().checked_add(1).expect("id overflown"));
                            $name(self.id.get())
                        }
                    }
                }

                #[allow(unused_imports)]
                pub use [<$name Gen_mod>]::[<$name Gen>];

                #[allow(unused_imports)]
                pub use [<$name Gen_mod>]::$name;
            }
        )+
    };
}
