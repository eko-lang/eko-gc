use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};
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

pub struct Gc<'gc, T: ?Sized + Trace + 'gc> {
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

    pub fn ptr_eq(&self, other: &Gc<'gc, T>) -> bool {
        Rc::ptr_eq(&self.data, &other.data)
    }
}

impl<'gc, T: ?Sized + Trace + 'gc> Clone for Gc<'gc, T> {
    fn clone(&self) -> Gc<'gc, T> {
        Gc {
            data: self.data.clone(),
            phantom: PhantomData,
        }
    }
}

impl<'gc, T: ?Sized + Trace + PartialEq + 'gc> PartialEq for Gc<'gc, T> {
    fn eq(&self, other: &Gc<'gc, T>) -> bool {
        self.data == other.data
    }
}

impl<'gc, T: ?Sized + Eq + Trace + 'gc> Eq for Gc<'gc, T> {}

impl<'gc, T: ?Sized + Trace + PartialOrd + 'gc> PartialOrd for Gc<'gc, T> {
    fn partial_cmp(&self, other: &Gc<'gc, T>) -> Option<Ordering> {
        self.data.partial_cmp(&other.data)
    }
}

impl<'gc, T: ?Sized + Trace + Ord + 'gc> Ord for Gc<'gc, T> {
    fn cmp(&self, other: &Gc<'gc, T>) -> Ordering {
        self.data.cmp(&other.data)
    }
}

impl<'gc, T: ?Sized + Hash + Trace + 'gc> Hash for Gc<'gc, T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.data.hash(state);
    }
}

impl<'gc, T: ?Sized + fmt::Debug + Trace + 'gc> fmt::Debug for Gc<'gc, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

impl<'gc, T: ?Sized + Trace + 'gc> Deref for Gc<'gc, T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.data.deref()
    }
}

unsafe impl<'gc, T: Trace + 'gc> Trace for Gc<'gc, T> {}

pub struct RefCell<'gc, T: ?Sized + Trace + 'gc> {
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

impl<'gc, T: ?Sized + fmt::Debug + Trace + 'gc> fmt::Debug for RefCell<'gc, T> {
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

impl<'gc, T: ?Sized + Trace + 'gc> RefCell<'gc, T> {
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

unsafe impl<'gc, T: ?Sized + Trace + 'gc> Trace for RefCell<'gc, T> {}

pub struct Ref<'a, 'gc, T: ?Sized + Trace + 'gc> {
    data: std::cell::Ref<'a, T>,
    phantom: PhantomData<&'gc ()>,
}

impl<'a, 'gc, T: ?Sized + Trace + 'gc> Deref for Ref<'a, 'gc, T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.data.deref()
    }
}

pub struct RefMut<'a, 'gc, T: ?Sized + Trace + 'gc> {
    data: std::cell::RefMut<'a, T>,
    phantom: PhantomData<&'gc ()>,
}

impl<'a, 'gc, T: ?Sized + Trace + 'gc> Deref for RefMut<'a, 'gc, T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.data.deref()
    }
}

impl<'a, 'gc, T: ?Sized + Trace + 'gc> DerefMut for RefMut<'a, 'gc, T> {
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
