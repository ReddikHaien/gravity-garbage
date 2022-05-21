use std::{cell::Cell, sync::atomic::{AtomicU32, Ordering}, any::Any};

use super::pointer::RawPointer;

pub struct GarbageObject{
    pub data: Box<Cell<dyn Traceable>>,
    pub position: usize,
    pins: AtomicU32
}

unsafe impl Send for GarbageObject{}

impl GarbageObject{
    
    pub(crate) unsafe fn new_from<T: Traceable + Sized + 'static>(value: T) -> *mut Self{
        let data = Box::new(Cell::new(value));
        Box::into_raw(Box::new(
            Self{
                data,
                position: 0,
                pins: AtomicU32::new(0)
            }
        )) 
    }
    
    pub(crate) fn pin(&self){
        self.pins.fetch_add(1, Ordering::SeqCst);
    }

    pub(crate) fn unpin(&self){
        self.pins.fetch_sub(1, Ordering::SeqCst);
    }
    
    pub(crate) fn deref_as<T: 'static>(&self) -> &T{
        let d = unsafe { self.data.as_ref().as_ptr().as_ref().unwrap() };
        d.as_any().downcast_ref::<T>().unwrap()
        
    }

    pub(crate) fn deref_as_mut<T: 'static>(&self) -> &mut T{
        let d = unsafe { self.data.as_ref().as_ptr().as_mut().unwrap() };
        d.as_any_mut().downcast_mut::<T>().unwrap()
    }

    pub(crate) fn deref_raw(&self) -> &dyn Traceable{
        unsafe { self.data.as_ref().as_ptr().as_ref().unwrap() }
    }

    pub(crate) fn get_pins(&self) -> u32{
        self.pins.load(Ordering::SeqCst)
    }
}

pub trait Traceable{

    unsafe fn get_pointers(&self) -> Vec<RawPointer>;

    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}