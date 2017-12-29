use std::marker::PhantomData;
use std::ptr;

use StaticFn;

pub struct Boxed<T: ?Sized>(pub Box<T>);
pub struct Static<T>(pub PhantomData<T>);
pub struct Trait<T>(pub T);

// ---------------------------------

pub trait CallRawOnce<A, O>: Sized {
    fn this(this: *const u8) -> Self {
        unsafe { ptr::read(this as *const Self) }
    }

    fn call_raw_once(this: *const u8, args: A) -> O;

    fn drop_raw(this: *const u8) {
        let _ = Self::this(this);
    }
}

impl<A, O, F> CallRawOnce<A, O> for Boxed<F>
where
    F: FnBox<A, O> + ?Sized,
{
    #[inline]
    fn call_raw_once(this: *const u8, args: A) -> O {
        let this: Box<F> = Self::this(this).0;

        this.call_box(args)
    }
}

impl<A, O, F> CallRawOnce<A, O> for Static<F>
where
    F: StaticFn<A, O>,
{
    fn this(_: *const u8) -> Self {
        unreachable!()
    }

    #[inline]
    fn call_raw_once(_: *const u8, args: A) -> O {
        F::call(args)
    }

    fn drop_raw(_: *const u8) {}
}

impl<A, O, F> CallRawOnce<A, O> for Trait<F>
where
    F: FnOnce(A) -> O,
{
    #[inline]
    fn call_raw_once(this: *const u8, args: A) -> O {
        let this = Self::this(this).0;

        this(args)
    }
}

// -------------------------------

pub trait CallRawRef<A, O>: Sized {
    fn this(this: *const u8) -> Self {
        unsafe { ptr::read(this as *const Self) }
    }

    fn this_ref<'a>(this: *const u8) -> &'a Self {
        unsafe { &*(this as *const Self) }
    }

    fn call_raw_ref(this: *const u8, args: A) -> O;

    fn drop_raw(this: *const u8) {
        let _ = Self::this(this);
    }
}

impl<A, O, F> CallRawRef<A, O> for Boxed<F>
where
    F: Fn(A) -> O + ?Sized,
{
    #[inline]
    fn call_raw_ref(this: *const u8, args: A) -> O {
        let this: &F = &*(Self::this_ref(this).0);

        this(args)
    }
}

impl<A, O, F> CallRawRef<A, O> for Static<F>
where
    F: StaticFn<A, O>,
{
    fn this(_: *const u8) -> Self {
        unimplemented!()
    }

    #[inline]
    fn call_raw_ref(_: *const u8, args: A) -> O {
        F::call(args)
    }

    fn drop_raw(_: *const u8) {}
}

impl<A, O, F> CallRawRef<A, O> for Trait<F>
where
    F: Fn(A) -> O,
{
    #[inline]
    fn call_raw_ref(this: *const u8, args: A) -> O {
        let this: &F = &Self::this_ref(this).0;

        this(args)
    }
}

// -------------------------------

pub trait CallRawMut<A, O>: Sized {
    fn this(this: *const u8) -> Self {
        unsafe { ptr::read(this as *const Self) }
    }

    fn this_mut<'a>(this: *mut u8) -> &'a mut Self {
        unsafe { &mut *(this as *mut Self) }
    }

    fn call_raw_mut(this: *mut u8, args: A) -> O;

    fn drop_raw(this: *const u8) {
        let _ = Self::this(this);
    }
}

impl<A, O, F> CallRawMut<A, O> for Boxed<F>
where
    F: FnMut(A) -> O + ?Sized,
{
    #[inline]
    fn call_raw_mut(this: *mut u8, args: A) -> O {
        let this: &mut F = &mut *(Self::this_mut(this).0);

        this(args)
    }
}

impl<A, O, F> CallRawMut<A, O> for Static<F>
where
    F: StaticFn<A, O>,
{
    fn this(_: *const u8) -> Self {
        unimplemented!()
    }

    #[inline]
    fn call_raw_mut(_: *mut u8, args: A) -> O {
        F::call(args)
    }

    fn drop_raw(_: *const u8) {}
}

impl<A, O, F> CallRawMut<A, O> for Trait<F>
where
    F: FnMut(A) -> O,
{
    #[inline]
    fn call_raw_mut(this: *mut u8, args: A) -> O {
        let this: &mut F = &mut Self::this_mut(this).0;

        this(args)
    }
}

// ----------------------------------------------------

pub trait FnBox<A, O> {
    fn call_box(self: Box<Self>, args: A) -> O;
}

impl<A, O, F> FnBox<A, O> for F
where
    F: FnOnce(A) -> O
{
    fn call_box(self: Box<Self>, args: A) -> O {
        let this: F = *self;

        this(args)
    }
}
