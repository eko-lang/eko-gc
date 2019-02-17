use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;

pub struct Arena<'gc> {
    marker: PhantomData<&'gc ()>,
}

impl<'gc> Arena<'gc> {
    pub fn new() -> Arena<'gc> {
        Arena {
            marker: PhantomData,
        }
    }
}

pub unsafe trait Trace {}

pub struct Gc<'gc, T: Trace + 'gc> {
    data: Rc<T>,
    marker: PhantomData<&'gc ()>,
}

impl<'gc, T: Trace + 'gc> Gc<'gc, T> {
    fn new(_arena: &Arena<'gc>, data: T) -> Gc<'gc, T> {
        Gc {
            data: Rc::new(data),
            marker: PhantomData,
        }
    }
}

unsafe impl<'gc, T: Trace + 'gc> Trace for Gc<'gc, T> {}

pub struct RefCell<'gc, T: Trace + ?Sized + 'gc> {
    marker: PhantomData<&'gc ()>,
    data: std::cell::RefCell<T>,
}

impl<'gc, T: Trace + 'gc> RefCell<'gc, T> {
    fn new(arena: &Arena<'gc>, data: T) -> RefCell<'gc, T> {
        RefCell {
            data: std::cell::RefCell::new(data),
            marker: PhantomData,
        }
    }
}

impl<'gc, T: Trace + ?Sized + 'gc> RefCell<'gc, T> {
    fn borrow<'a>(&'a self) -> Ref<'a, 'gc, T> {
        Ref {
            data: self.data.borrow(),
            marker: PhantomData,
        }
    }

    fn borrow_mut<'a>(&'a self) -> RefMut<'a, 'gc, T> {
        RefMut {
            data: self.data.borrow_mut(),
            marker: PhantomData,
        }
    }
}

unsafe impl<'gc, T: Trace + ?Sized + 'gc> Trace for RefCell<'gc, T> {}

pub struct Ref<'a, 'gc, T: Trace + ?Sized + 'gc> {
    data: std::cell::Ref<'a, T>,
    marker: PhantomData<&'gc ()>,
}

impl<'a, 'gc, T: Trace + ?Sized + 'gc> Deref for Ref<'a, 'gc, T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.data.deref()
    }
}

pub struct RefMut<'a, 'gc, T: Trace + ?Sized + 'gc> {
    data: std::cell::RefMut<'a, T>,
    marker: PhantomData<&'gc ()>,
}

impl<'a, 'gc, T: Trace + ?Sized + 'gc> Deref for RefMut<'a, 'gc, T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.data.deref()
    }
}

impl<'a, 'gc, T: Trace + ?Sized + 'gc> DerefMut for RefMut<'a, 'gc, T> {
    fn deref_mut(&mut self) -> &mut T {
        self.data.deref_mut()
    }
}
