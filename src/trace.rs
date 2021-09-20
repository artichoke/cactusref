use crate::rc::Rc;

/// TODO: document me!
pub trait Trace: Sized {
    /// TODO: document me!
    fn yield_owned_rcs<F>(&self, mark: F)
    where
        F: FnMut(&mut Rc<Self>);
}
