
use std::ops::{Deref, DerefMut};

pub struct Ptr<T> {
    ptr: *const T,
}

impl<T> Ptr<T> {
    pub fn new(val: &T) -> Self {
        Self {
            ptr: val as *const T,
        }
    }

    pub fn clone(&self) -> Self {
        Self {
            ptr: self.ptr
        }
    }

    pub fn get_mut(&self) -> *mut T {
        self.ptr as *mut T
    }

    pub const fn null() -> Self {
        Self {
            ptr: std::ptr::null()
        }
    }
}

impl<T> Deref for Ptr<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*(self.ptr) }
    }
}

pub struct PtrMut<T> {
    ptr: *mut T,
}

impl<T> PtrMut<T> {
    pub fn new(val: &mut T) -> Self {
        Self {
            ptr: val as *mut T,
        }
    }
}

impl<T> Deref for PtrMut<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*(self.ptr) }
    }
}

impl<T> DerefMut for PtrMut<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *(self.ptr) }
    }
}

pub struct Unsafe<T> {
    val: T,
}

impl<T> Unsafe<T> {
    pub fn new(val: T) -> Self {
        Self {
            val
        }
    }

    pub fn get(&self) -> *mut T {
        unsafe { &self.val as *const T as *mut T }
    }
}

unsafe impl<T> Send for Ptr<T> {}
unsafe impl<T> Sync for Ptr<T> {}