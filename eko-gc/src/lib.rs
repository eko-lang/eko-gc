use std::fmt;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;

pub use self::trace::Trace;

mod trace;

pub struct Arena<'gc> {
    phantom: PhantomData<&'gc ()>,
}

impl<'gc> Arena<'gc> {
    pub fn new() -> Arena<'gc> {
        Arena {
            phantom: PhantomData,
        }
    }
}

pub struct Gc<'gc, T: Trace + ?Sized + 'gc> {
    data: Rc<T>,
    phantom: PhantomData<&'gc ()>,
}

impl<'gc, T: Trace + 'gc> Gc<'gc, T> {
    pub fn new(_arena: &Arena<'gc>, data: T) -> Gc<'gc, T> {
        Gc {
            data: Rc::new(data),
            phantom: PhantomData,
        }
    }
}

impl<'gc, T: Trace + ?Sized + 'gc> Clone for Gc<'gc, T> {
    fn clone(&self) -> Gc<'gc, T> {
        Gc {
            data: self.data.clone(),
            phantom: PhantomData,
        }
    }
}

impl<'gc, T: fmt::Debug + Trace + ?Sized + 'gc> fmt::Debug for Gc<'gc, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

impl<'gc, T: Trace + ?Sized + 'gc> Deref for Gc<'gc, T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.data.deref()
    }
}

unsafe impl<'gc, T: Trace + 'gc> Trace for Gc<'gc, T> {}

pub struct RefCell<'gc, T: Trace + ?Sized + 'gc> {
    phantom: PhantomData<&'gc ()>,
    data: std::cell::RefCell<T>,
}

impl<'gc, T: Trace + 'gc> RefCell<'gc, T> {
    pub fn new(_arena: &Arena<'gc>, data: T) -> RefCell<'gc, T> {
        RefCell {
            data: std::cell::RefCell::new(data),
            phantom: PhantomData,
        }
    }
}

impl<'gc, T: fmt::Debug + Trace + ?Sized + 'gc> fmt::Debug for RefCell<'gc, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.data.try_borrow() {
            Ok(borrow) => f.debug_struct("RefCell").field("value", &borrow).finish(),
            Err(_) => {
                // The RefCell is mutably borrowed so we can't look at its value
                // here. Show a placeholder instead.
                struct BorrowedPlaceholder;

                impl fmt::Debug for BorrowedPlaceholder {
                    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                        f.write_str("<borrowed>")
                    }
                }

                f.debug_struct("RefCell")
                    .field("value", &BorrowedPlaceholder)
                    .finish()
            }
        }
    }
}

impl<'gc, T: Trace + ?Sized + 'gc> RefCell<'gc, T> {
    pub fn borrow<'a>(&'a self) -> Ref<'a, 'gc, T> {
        Ref {
            data: self.data.borrow(),
            phantom: PhantomData,
        }
    }

    pub fn borrow_mut<'a>(&'a self) -> RefMut<'a, 'gc, T> {
        RefMut {
            data: self.data.borrow_mut(),
            phantom: PhantomData,
        }
    }
}

unsafe impl<'gc, T: Trace + ?Sized + 'gc> Trace for RefCell<'gc, T> {}

pub struct Ref<'a, 'gc, T: Trace + ?Sized + 'gc> {
    data: std::cell::Ref<'a, T>,
    phantom: PhantomData<&'gc ()>,
}

impl<'a, 'gc, T: Trace + ?Sized + 'gc> Deref for Ref<'a, 'gc, T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.data.deref()
    }
}

pub struct RefMut<'a, 'gc, T: Trace + ?Sized + 'gc> {
    data: std::cell::RefMut<'a, T>,
    phantom: PhantomData<&'gc ()>,
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

#[cfg(test)]
mod tests {
    use super::{Arena, Gc, RefCell};

    #[test]
    fn gc() {
        let arena = Arena::new();
        let gc = Gc::new(&arena, 0);
        assert_eq!(*gc, 0);
    }

    #[test]
    fn gc_clone() {
        let arena = Arena::new();
        let gc = Gc::new(&arena, 0);
        let gc_clone = Gc::clone(&gc);
        assert_eq!(*gc_clone, 0);
    }

    #[test]
    fn ref_cell() {
        let arena = Arena::new();
        let ref_cell = RefCell::new(&arena, 0);
        assert_eq!(*ref_cell.borrow(), 0);
        *ref_cell.borrow_mut() = 1;
        assert_eq!(*ref_cell.borrow(), 1);
    }

    #[test]
    fn gc_ref_cell() {
        let arena = Arena::new();
        let gc = Gc::new(&arena, RefCell::new(&arena, 0));
        assert_eq!(*gc.borrow(), 0);
        *gc.borrow_mut() = 1;
        assert_eq!(*gc.borrow(), 1);
    }

    #[test]
    fn gc_ref_cell_clone() {
        let arena = Arena::new();
        let gc = Gc::new(&arena, RefCell::new(&arena, 0));
        assert_eq!(*gc.borrow(), 0);
        let gc_clone = Gc::clone(&gc);
        *gc_clone.borrow_mut() = 1;
        assert_eq!(*gc.borrow(), 1);
    }
}
