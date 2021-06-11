use alloc::alloc::{dealloc, Layout};
use core::cell::{Cell, RefCell};
use core::mem::{self, MaybeUninit};
use core::ptr::{self, NonNull};
use std::process::abort;

use crate::link::Links;
use crate::Rc;

#[allow(clippy::module_name_repetitions)]
pub trait RcBoxPtr<T> {
    fn inner(&self) -> &RcBox<T>;

    #[inline]
    fn strong(&self) -> usize {
        self.inner().strong.get()
    }

    #[inline]
    fn inc_strong(&self) {
        // We want to abort on overflow instead of dropping the value.
        // nevertheless, we insert an abort here to hint LLVM at
        // an otherwise missed optimization.
        let strong_count = self.strong();
        if strong_count == 0 || strong_count == usize::MAX || strong_count + 1 == usize::MAX {
            abort();
        }
        // guaranteed not to overflow by the abort above.
        self.inner().strong.set(strong_count + 1);
    }

    #[inline]
    fn dec_strong(&self) {
        self.inner().strong.set(self.strong().saturating_sub(1));
    }

    #[inline]
    fn weak(&self) -> usize {
        self.inner().weak.get()
    }

    #[inline]
    fn inc_weak(&self) {
        // We want to abort on overflow instead of dropping the value.
        // The reference count will never be zero when this is called;
        // nevertheless, we insert an abort here to hint LLVM at
        // an otherwise missed optimization.
        let weak_count = self.weak();
        if weak_count == 0 || weak_count == usize::MAX || weak_count + 1 == usize::MAX {
            abort();
        }
        // guaranteed not to overflow by the abort above.
        self.inner().weak.set(weak_count + 1);
    }

    #[inline]
    fn dec_weak(&self) {
        self.inner().weak.set(self.weak().saturating_sub(1));
    }

    #[inline]
    fn kill(&self) {
        self.inner().strong.set(0);
    }

    #[inline]
    fn is_dead(&self) -> bool {
        self.strong() == 0 || self.strong() == usize::MAX
    }

    #[inline]
    fn is_initialized(&self) -> bool {
        self.strong() == usize::MAX
    }

    #[inline]
    fn make_uninit(&self) {
        self.inner().strong.set(usize::MAX);
    }
}

impl<T> RcBoxPtr<T> for Rc<T> {
    fn inner(&self) -> &RcBox<T> {
        unsafe { self.ptr.as_ref() }
    }
}

impl<T> RcBoxPtr<T> for RcBox<T> {
    fn inner(&self) -> &Self {
        self
    }
}

pub struct RcBox<T> {
    pub strong: Cell<usize>,
    pub weak: Cell<usize>,
    pub links: RefCell<Links<T>>,
    pub value: MaybeUninit<T>,
}

pub fn is_dangling<T>(ptr: NonNull<T>) -> bool {
    let address = ptr.as_ptr() as *mut () as usize;
    address == usize::max_value()
}

// duplicated from a crate-private function in std
// <https://github.com/rust-lang/rust/blob/baab1914/src/liballoc/alloc.rs#L212-L223>
#[inline]
pub unsafe fn box_free<T>(ptr: NonNull<T>) {
    let ptr = ptr.as_ptr();
    let size = mem::size_of_val(&*ptr);
    let align = mem::align_of_val(&*ptr);
    // We do not allocate for Box<T> when T is ZST, so deallocation is also not necessary.
    if size != 0 {
        let layout = Layout::from_size_align_unchecked(size, align);
        dealloc(ptr as *mut u8, layout);
    }
}

// Sets the data pointer of a `?Sized` raw pointer.
//
// For a slice/trait object, this sets the `data` field and leaves the rest
// unchanged. For a sized raw pointer, this simply sets the pointer.
#[allow(clippy::module_name_repetitions)]
pub unsafe fn set_data_ptr<T: ?Sized, U>(mut ptr: *mut T, data: *mut U) -> *mut T {
    ptr::write(&mut ptr as *mut _ as *mut *mut u8, data as *mut u8);
    ptr
}

#[allow(clippy::cast_possible_wrap)]
pub unsafe fn data_offset<T: ?Sized>(ptr: *const T) -> isize {
    // Align the unsized value to the end of the RcBox.
    // Because it is ?Sized, it will always be the last field in memory.
    let align = mem::align_of_val(&*ptr);
    let layout = Layout::new::<RcBox<()>>();
    (layout.size() + layout.padding_needed_for(align)) as isize
}

/// Computes the offset of the data field within `RcBox`.
///
/// Unlike [`data_offset`], this doesn't need the pointer, but it works only on `T: Sized`.
#[allow(clippy::cast_possible_wrap)]
pub fn data_offset_sized<T>() -> isize {
    let align = mem::align_of::<T>();
    let layout = Layout::new::<RcBox<()>>();
    (layout.size() + layout.padding_needed_for(align)) as isize
}

#[cfg(test)]
mod tests {
    use core::mem::size_of;

    use super::RcBox;

    #[test]
    fn sizeof_rcbox() {
        assert_eq!(size_of::<RcBox<()>>(), 88);
    }
}
