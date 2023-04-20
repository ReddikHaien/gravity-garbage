use std::{ptr::NonNull, sync::atomic::AtomicU32};


pub struct Data<Q: ?Sized>{
    pins: AtomicU32,
    pos: AtomicU32,
    payload: Q   
}

impl<Q: Sized> Data<Q>{
    pub fn new(data: Q) -> Self{
        Self{
            pins: AtomicU32::new(0),
            pos: AtomicU32::new(0),
            payload: data
        }
    }
}

pub struct InnerPtr<Q: ?Sized>{
    ptr: NonNull<Data<Q>>,
}

impl<Q: Sized> InnerPtr<Q>{
    pub fn new(data: Q) -> InnerPtr<Q> {
        Self{
            ptr: Box::leak(
                Box::new(
                    Data::new(data)
                )
            ).into()
        }
    }
}

impl<Q: ?Sized> InnerPtr<Q>{
    pub fn 
}