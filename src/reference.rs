use std::mem::{align_of, size_of};
use std::ptr;

use Array;

pub struct FnStack<A, O, D = [u8; 24]> {
    data: D,
    ptr: fn(*const u8, A) -> O,
}

impl<A, O, D> FnStackOnce<A, O, D>
    where
        D: Array,
{
    pub fn new<F: 'static>(f: F) -> Self
        where
            F: FnOnce(A) -> O,
    {
        if size_of::<F>() < D::size() && align_of::<F>() < D::align() {
            // Store it on the stack
            unsafe {
                let mut data = D::uninitialized();
                ptr::write(&mut data as *mut D as *mut F, f);

                FnStackOnce {
                    data,
                    ptr: call_once_stack::<A, O, F>,
                }
            }

        } else {
            let f = Box::new(f);

            FnStackOnce::from(f as Box<FnBox<A, O>>)
        }
    }

    pub fn call(self, args: A) -> O {
        let this = self.data.as_ptr();

        self.ptr(this, args)
    }
}

impl<A, O, D> From<Box<FnBox<A, O>>> for FnStackOnce<A, O, D>
    where
        D: Array,
{
    fn from(f: Box<FnBox<A, O>>) -> Self {
        assert!(size_of::<Box<FnBox<A, O>>>() <= D::size());

        unsafe {
            let mut data = D::uninitialized();
            ptr::write(&mut data as *mut D as *mut Box<FnBox<A, O>>, f);

            FnStackOnce {
                data,
                ptr: call_once_heap::<A, O>,
            }
        }
    }
}

fn call_once_stack<A, O, F>(ptr: *const u8, args: A) -> O
    where
        F: FnOnce(A) -> O,
{
    let this: F = unsafe { ptr::read(ptr as *const F) };

    this(args)
}

fn call_once_heap<A, O>(ptr: *const u8, args: A) -> O {
    let this: Box<FnBox<A, O>> = unsafe { ptr::read(ptr as *const Box<FnBox<A, O>>) };

    this.call(args)
}

