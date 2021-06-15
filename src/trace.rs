use crate::rc::Rc;

pub trait Trace: Sized {
    fn yield_owned_rcs<F>(&self, mark: F)
    where
        F: for<'a> FnMut(&'a mut Rc<Self>);
}
