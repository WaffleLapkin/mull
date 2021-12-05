//! # NOTE TO THE READER
//!
//! This code uses **A LOT** of undocumented and maybe incorrect unsafe code.
//! I do not guarantee that anything below this doc-comment makes any sense or
//! that the code is correct/will be correct in future Rust versions.
//!
//! This is WIP POC (work in progress proof of concept) so don't expect anything from this code.
//!
//! That said, here is a cool demo:
//!
//! ```rust
//! use std::fmt::Display;
//! use mull::Mull;
//!
//! let mut ll = Mull::<dyn Display>::new();
//! ll.push_front_unsized(42);
//! ll.push_front_unsized('!');
//! ll.push_front_unsized("???");
//!
//! assert!(ll.iter().map(<_>::to_string).eq(["???", "!", "42"]))
//! ```

#![feature(ptr_metadata)]
#![feature(unsize)]
#![allow(unused)]

use std::{
    marker::{PhantomData, Unsize},
    mem::size_of_val,
    ptr::{self, DynMetadata, NonNull, Pointee},
};

// TODO: mudll (maybe unsized double linked list)

pub struct Mull<T: ?Sized> {
    head: Option<NonNull<OpaqueNode<<T as Pointee>::Metadata>>>,
    len: usize,
    marker: PhantomData<Box<Node<T>>>,
}

type OpaqueNode<M> = Node<(), M>;

#[repr(C)]
struct Node<T: ?Sized, M = <T as Pointee>::Metadata> {
    next: Option<NonNull<OpaqueNode<M>>>,
    meta: M,
    element: T,
}

pub struct Iter<'a, T: ?Sized> {
    next: Option<&'a Node<T>>,
}

impl<T: ?Sized> Mull<T> {
    pub const fn new() -> Mull<T> {
        Mull {
            head: None,
            len: 0,
            marker: PhantomData,
        }
    }

    pub fn front(&self) -> Option<&T> {
        self.head().map(|h| &h.element)
    }

    pub fn push_front(&mut self, v: T)
    where
        T: Sized,
    {
        // FIXME: This is actually always unit (T: Sized), but compiler can't figure this out
        let meta = ptr::metadata(&v);

        let new_node = Box::new(Node {
            next: self.head.take().map(NonNull::cast),
            meta,
            element: v,
        });

        self.head = Some(unsafe { NonNull::new_unchecked(Box::into_raw(new_node)) }.cast());
        self.len += 1;
    }

    pub fn push_front_unsized<V>(&mut self, v: V)
    where
        V: Unsize<T>,
    {
        // FIXME: Check if this actually stable (i.e. if all `metadata` calls are guaranteed to return the same value independant of unsizing)
        let meta = ptr::metadata::<T>(&v);

        let new_node = Box::new(Node {
            next: self.head.take().map(NonNull::cast),
            meta,
            element: v,
        });

        self.head = Some(unsafe { NonNull::new_unchecked(Box::into_raw(new_node)) }.cast());
        self.len += 1;
    }

    pub fn iter(&self) -> Iter<'_, T> {
        Iter { next: self.head() }
    }

    fn head(&self) -> Option<&Node<T>> {
        self.head
            .as_ref()
            .map(|head| unsafe { head.as_ref().assume_type() })
    }
}

impl<M> OpaqueNode<M> {
    unsafe fn assume_type<T: ?Sized>(&self) -> &Node<T>
    where
        T: Pointee<Metadata = M>,
    {
        let meta = *(&self.meta as *const _ as *const _);
        &*ptr::from_raw_parts(self as *const _ as *const _, meta)
    }

    unsafe fn assume_type_mut<T: ?Sized>(&mut self) -> &mut Node<T> {
        let meta = *(&self.meta as *const _ as *const _);
        &mut *ptr::from_raw_parts_mut(self as *mut _ as *mut _, meta)
    }
}

impl<'a, T: ?Sized> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.next.take()?;
        self.next = next.next.map(|next| unsafe { next.as_ref().assume_type() });
        Some(&next.element)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt::Display;

    #[test]
    fn it_works() {
        let mut ll = Mull::<dyn Display>::new();
        ll.push_front_unsized(42);
        ll.push_front_unsized('!');
        ll.push_front_unsized("???");

        assert!(ll.iter().map(<_>::to_string).eq(["???", "!", "42"]))
    }
}
