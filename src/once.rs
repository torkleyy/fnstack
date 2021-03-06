use std::marker::PhantomData;
use std::mem::{align_of, forget, size_of};
use std::ptr;

use raw::{Boxed, CallRawOnce, FnBox, Static, Trait};
use {Array, StaticFn};

pub struct FnStackOnce<'a, A, O, D = [u8; 16]>
where
    D: Array,
{
    data: D,
    drop: fn(*const u8),
    ptr: fn(*const u8, A) -> O,
    marker: PhantomData<FnOnce(A) -> O + 'a>,
}

impl<'a, A, O, D> FnStackOnce<'a, A, O, D>
where
    D: Array,
{
    #[inline]
    pub fn new<F: 'a>(f: F) -> Self
    where
        F: FnOnce(A) -> O + 'a,
    {
        if size_of::<F>() < D::size() && align_of::<F>() <= D::align() {
            FnStackOnce::from_raw(Trait(f))
        } else {
            FnStackOnce::from_raw(Boxed(Box::new(f)))
        }
    }

    #[inline]
    pub fn from_static<F>() -> Self
    where
        F: StaticFn<A, O>,
    {
        FnStackOnce::from_raw(Static(PhantomData::<F>))
    }

    #[inline]
    fn from_raw<R>(raw: R) -> Self
    where
        R: CallRawOnce<A, O>,
    {
        assert!(size_of::<R>() <= D::size(), align_of::<R>() <= D::align());

        unsafe {
            let mut data = D::uninitialized();
            ptr::write(&mut data as *mut D as *mut R, raw);

            FnStackOnce {
                data,
                drop: R::drop_raw,
                ptr: R::call_raw_once,
                marker: PhantomData,
            }
        }
    }

    #[inline]
    pub fn call(self, args: A) -> O {
        let res = (self.ptr)(self.data.as_ptr(), args);

        forget(self);

        res
    }
}

impl<'a, A, O, D> Drop for FnStackOnce<'a, A, O, D>
where
    D: Array,
{
    fn drop(&mut self) {
        (self.drop)(self.data.as_ptr())
    }
}

impl<'a, A, O, D, F> From<Box<F>> for FnStackOnce<'a, A, O, D>
where
    D: Array,
    F: FnBox<A, O> + ?Sized + 'a,
{
    #[inline]
    fn from(f: Box<F>) -> Self {
        FnStackOnce::from_raw(Boxed(f))
    }
}

#[cfg(test)]
mod tests {
    use super::FnStackOnce;

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
            let droppable = Droppable(&mut num_drops);
            let _closure: FnStackOnce<(), ()> = FnStackOnce::new(move |()| {
                let _d = droppable;
            });
        }

        assert_eq!(num_drops, 1);

        struct ExceedsLimit<'a>(Droppable<'a>, [u8; 128]);

        {
            let obj = ExceedsLimit(Droppable(&mut num_drops), [0; 128]);
            let _closure: FnStackOnce<(), (), [u8; 16]> = FnStackOnce::new(move |()| {
                let _o = obj;
            });
        }

        assert_eq!(num_drops, 2);
    }

    #[test]
    fn variance_check() {
        fn takes_fn<'a>(f: FnStackOnce<'a, (), ()>) {
            f.call(());
        }

        const X: usize = 5;
        let x = &X;
        let f: FnStackOnce<'static, _, _> = FnStackOnce::new(move |()| {
            let _y = x;
        });

        takes_fn(f);
    }
}
