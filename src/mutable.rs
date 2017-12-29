use std::marker::PhantomData;
use std::mem::{align_of, size_of};
use std::ptr;

use {Array, StaticFn};
use raw::{Boxed, CallRawMut, Static, Trait};

pub struct FnStackMut<'a, A, O, D = [u8; 16]>
where
    D: Array,
{
    data: D,
    drop: fn(*const u8),
    ptr: fn(*mut u8, A) -> O,
    marker: PhantomData<FnMut(A) -> O + 'a>,
}

impl<'a, A, O, D> FnStackMut<'a, A, O, D>
where
    D: Array,
{
    #[inline]
    pub fn new<F: 'a>(f: F) -> Self
    where
        F: FnMut(A) -> O + 'a,
    {
        if size_of::<F>() < D::size() && align_of::<F>() <= D::align() {
            FnStackMut::from_raw(Trait(f))
        } else {
            FnStackMut::from_raw(Boxed(Box::new(f)))
        }
    }

    #[inline]
    pub fn from_static<F>() -> Self
    where
        F: StaticFn<A, O>,
    {
        FnStackMut::from_raw(Static(PhantomData::<F>))
    }

    #[inline]
    fn from_raw<R>(raw: R) -> Self
    where
        R: CallRawMut<A, O>,
    {
        assert!(size_of::<R>() <= D::size(), align_of::<R>() <= D::align());

        unsafe {
            let mut data = D::uninitialized();
            ptr::write(&mut data as *mut D as *mut R, raw);

            FnStackMut {
                data,
                drop: R::drop_raw,
                ptr: R::call_raw_mut,
                marker: PhantomData,
            }
        }
    }

    #[inline]
    pub fn call(&mut self, args: A) -> O {
        (self.ptr)(self.data.as_mut_ptr(), args)
    }
}

impl<'a, A, O, D> Drop for FnStackMut<'a, A, O, D>
where
    D: Array,
{
    fn drop(&mut self) {
        (self.drop)(self.data.as_ptr())
    }
}

impl<'a, A, O, D, F> From<Box<F>> for FnStackMut<'a, A, O, D>
where
    D: Array,
    F: FnMut(A) -> O + ?Sized + 'a,
{
    #[inline]
    fn from(f: Box<F>) -> Self {
        FnStackMut::from_raw(Boxed(f))
    }
}

#[cfg(test)]
mod tests {
    use super::FnStackMut;

    #[test]
    fn test_drop() {
        let mut num_drops = 0;

        struct Droppable<'a>(&'a mut i32);

        impl<'a> Drop for Droppable<'a> {
            fn drop(&mut self) {
                *self.0 += 1;
            }
        }

        {
            let mut droppable = Droppable(&mut num_drops);
            let _closure: FnStackMut<(), ()> = FnStackMut::new(move |()| {
                let _d = &mut droppable;
            });
        }

        assert_eq!(num_drops, 1);

        struct ExceedsLimit<'a>(Droppable<'a>, [u8; 128]);

        {
            let mut obj = ExceedsLimit(Droppable(&mut num_drops), [0; 128]);
            let _closure: FnStackMut<(), (), [u8; 16]> = FnStackMut::new(move |()| {
                let _o = &mut obj;
            });
        }

        assert_eq!(num_drops, 2);
    }

    #[test]
    fn variance_check() {
        fn takes_fn<'a>(mut f: FnStackMut<'a, (), ()>) {
            f.call(());
        }

        const X: usize = 5;
        let x = &X;
        let f: FnStackMut<'static, _, _> = FnStackMut::new(move |()| {
            let _y = x;
        });

        takes_fn(f);
    }
}
