pub use mutable::FnStackMut;
pub use once::FnStackOnce;
pub use reference::FnStackRef;

use std::mem::{align_of, size_of, uninitialized};

mod mutable;
mod once;
mod raw;
mod reference;

mod private {
    pub struct Private;
}

pub trait Array: Sized {
    fn align() -> usize {
        align_of::<Self>()
    }

    fn size() -> usize {
        size_of::<Self>()
    }

    unsafe fn uninitialized() -> Self {
        uninitialized()
    }

    fn as_ptr(&self) -> *const u8;

    fn as_mut_ptr(&mut self) -> *mut u8;

    /// Only `fnstack` may implement this trait.
    fn _private() -> private::Private;
}

macro_rules! impl_array {
    ($($n:expr)*) => {
        $(
            impl Array for [u8; $n] {
                fn as_ptr(&self) -> *const u8 {
                    <[u8]>::as_ptr(self)
                }

                fn as_mut_ptr(&mut self) -> *mut u8 {
                    <[u8]>::as_mut_ptr(self)
                }

                fn _private() -> private::Private {
                    private::Private
                }
            }
        )*
    };
}

impl_array!(
    64 32 24 16 12 8 4 0
);

pub trait StaticFn<A, O> {
    fn call(args: A) -> O;
}
