use std::{ops::{Deref, DerefMut}, marker::PhantomData, fmt::Debug};

use crate::wrapper::Wrapper;

macro_rules! common_impl {
    ($name:ident) => {
        impl<T> Default for $name<T>{
            fn default() -> Self {
                Self { 
                    value: std::ptr::null_mut(),
                    marker: PhantomData
                }
            }
        }

        impl<T> Deref for $name<T>{
            type Target = T;

            fn deref(&self) -> &Self::Target {
                unsafe { self.value.as_ref().unwrap() }.deref_as()
            }
        }

        impl<T> DerefMut for $name<T>{
            fn deref_mut(&mut self) -> &mut Self::Target {
                unsafe{ self.value.as_mut().unwrap() }.deref_as_mut()
            }
        }

        impl<T> $name<T>{
            pub fn has_value(&self) -> bool{
                !self.value.is_null()
            }
        
            pub fn try_deref(&self) -> Option<&T>{
                if self.value.is_null(){
                    None
                }
                else {
                    Some(self.deref())
                }
            }
        
            pub fn try_deref_mut(&self) -> Option<&T>{
                if self.value.is_null(){
                    None
                }
                else {
                    Some(self.deref())
                }
            }
        
            /// Creates a new unpinned pointer to the memory adress
            pub fn clone_unpinned(&self) -> Pointer<T>{
                Pointer{
                    value: self.value,
                    marker: PhantomData
                }
            }
        
            /// Creates a new pinned pointer to the memory adress
            pub fn clone_pinned(&self) -> PinnedPointer<T>{
                if !self.value.is_null(){
                    unsafe{ self.value.as_ref().unwrap() }.pin();
                }
                PinnedPointer{
                    value: self.value,
                    marker: PhantomData
                }
            }
        
            #[doc(hidden)]
            pub fn clone_transfer(&self) -> TransferObject{
                TransferObject{
                    value: self.value
                }
            }
        }
    };
}


pub struct Pointer<T>{
    value: *mut Wrapper,
    marker: PhantomData<T>
}

common_impl!(Pointer);

impl<T: Debug> Debug for Pointer<T>{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        
        if self.value.is_null(){
            f.write_str("null")
        }
        else{
            f.write_str("(")?;
            f.write_fmt(format_args!("{}",unsafe{self.value.as_ref().unwrap()}.get_pins()))?;
            f.write_str("): ").unwrap();
            f.write_fmt( format_args!("{:#?}",unsafe{self.value.as_ref().unwrap()}.deref_as::<T>()) )
        }
        
    }
}

pub struct PinnedPointer<T>{
    value: *mut Wrapper,
    marker: PhantomData<T>
}

common_impl!(PinnedPointer);

impl<T> Drop for PinnedPointer<T>{
    fn drop(&mut self) {
        if !self.value.is_null(){
            unsafe{ self.value.as_ref().unwrap() }.unpin();
        }
    }
}

impl<T> PinnedPointer<T>{

    pub(crate) fn new_from(value: *mut Wrapper) -> Self{
        unsafe{value.as_ref().unwrap().pin()};
        Self{
            value,
            marker: PhantomData
        }
    }
}

impl<T: Debug> Debug for PinnedPointer<T>{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        
        if self.value.is_null(){
            f.write_str("pinned: null")
        }
        else{
            f.write_str("pinned(")?;
            f.write_fmt(format_args!("{}",unsafe{self.value.as_ref().unwrap()}.get_pins()))?;
            f.write_str("): ").unwrap();
            f.write_fmt( format_args!("{:#?}",unsafe{self.value.as_ref().unwrap()}.deref_as::<T>()) )
        }
    }
}

#[doc(hidden)]
pub struct TransferObject{
    #[doc(hidden)]
    value: *mut Wrapper
}

impl TransferObject{
    pub(crate) fn unwrap(self) -> *mut Wrapper{
        self.value
    }
}