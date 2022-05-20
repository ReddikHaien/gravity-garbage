use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use super::object::GarbageObject;

macro_rules! common_impl {
    ($name:ident) => {
        impl<T> Default for $name<T>{
            fn default() -> Self {
                Self { 
                    inner: RawPointer{
                        data: std::ptr::null_mut()
                    },
                    marker: PhantomData
                }
            }
        }

        impl<T> Deref for $name<T>{
            type Target = T;

            fn deref(&self) -> &Self::Target {
                unsafe{
                    self.inner.get_ref()
                }
            }
        }

        impl<T> DerefMut for $name<T>{
            fn deref_mut(&mut self) -> &mut Self::Target {
                unsafe{
                    self.inner.get_mut()
                }
            }
        }

        impl<T: 'static> $name<T>{
            pub fn has_value(&self) -> bool{
                !self.inner.is_null()
            }
        
            pub fn try_deref(&self) -> Option<&T>{
                if self.inner.is_null(){
                    None
                }
                else {
                    Some(self.deref())
                }
            }
        
            pub fn try_deref_mut(&self) -> Option<&T>{
                if self.inner.is_null(){
                    None
                }
                else {
                    Some(self.deref())
                }
            }
        
            /// Creates a new unpinned pointer to the memory adress
            pub fn clone_unpinned(&self) -> Pointer<T>{
                Pointer{
                    inner: self.inner,
                    marker: PhantomData
                }
            }
        
            /// Creates a new pinned pointer to the memory adress
            pub fn clone_pinned(&self) -> PinnedPointer<T>{
                if !self.inner.is_null(){
                    unsafe{
                        self.inner.pin();
                    }
                }
                PinnedPointer{
                    inner: self.inner,
                    marker: PhantomData
                }
            }
        }
    };
}


pub struct Pointer<T: 'static>{
    inner: RawPointer,
    marker: PhantomData<T>
}

common_impl!(Pointer);

pub struct PinnedPointer<T: 'static>{
    inner: RawPointer,
    marker: PhantomData<T>    
}

common_impl!(PinnedPointer);

#[derive(Clone, Copy)]
pub struct RawPointer{
    data: *mut GarbageObject
}

impl RawPointer{

    fn is_null(&self) -> bool{
        self.data.is_null()
    }

    unsafe fn data(&self) -> &GarbageObject{
        self.data.as_ref().unwrap()
    }
    pub(crate) unsafe fn data_mut(&self) -> &mut GarbageObject{
        self.data.as_mut().unwrap()
    }
    unsafe fn get_ref<T: 'static>(&self) -> &T{
        self.data().deref_as()
    }
    unsafe fn get_mut<T: 'static>(&mut self) -> &mut T{
        self.data().deref_as_mut()
    }

    unsafe fn pin(&self){
        self.data_mut().pin();
    }
}