use std::{ptr::NonNull, sync::atomic::{AtomicU32, Ordering, fence, AtomicBool, AtomicU8}, marker::{PhantomData, Unsize}, ops::{DispatchFromDyn, CoerceUnsized, Deref, DerefMut}, thread};

use crate::prelude::{Traceable, with_sink, TracingContext};

#[repr(C)]
pub struct Data<Q: ?Sized>{
    pins: AtomicU32,
    pos: AtomicU8,
    trace_lock: AtomicBool,
    payload: Q   
}

impl<Q: Sized> Data<Q>{
    pub fn new(data: Q) -> Self{
        Self{
            pins: AtomicU32::new(1),
            pos: AtomicU8::new(0),
            trace_lock: AtomicBool::new(false),
            payload: data
        }
    }
}

pub struct Ptr<Q: ?Sized>{
    ptr: NonNull<Data<Q>>,
    marker: PhantomData<Data<Q>>
}

unsafe impl<Q: ?Sized + Send> Send for Ptr<Q> {}
unsafe impl<Q: ?Sized + Sync> Sync for Ptr<Q> {}

impl<T: ?Sized + Unsize<U>, U: ?Sized> CoerceUnsized<Ptr<U>> for Ptr<T> {}

impl<T: ?Sized + Unsize<U>, U: ?Sized> DispatchFromDyn<Ptr<U>> for Ptr<T> {}

impl<Q: ?Sized> Clone for Ptr<Q>{
    fn clone(&self) -> Self {
        Self { ptr: self.ptr.clone(), marker: PhantomData }
    }
}


impl<Q: Sized> Ptr<Q>{
    fn new_pinned(data: Q) -> Ptr<Q> {
        Self{
            ptr: Box::leak(
                Box::new(
                    Data::new(data)
                )
            ).into(),
            marker: PhantomData
        }
    }
}

impl<Q: Sized + Traceable + 'static> Ptr<Q>{
    

    pub fn new(data: Q) -> PinPtr<Q>{
        let v = PinPtr::new(data);
        let dynned = v.0.clone();
        with_sink(|s| s.push(dynned));
        v
    }
}

impl<Q: ?Sized> Ptr<Q>{
    pub(crate) fn inner(&self) -> &Data<Q>{
        unsafe{
            self.ptr.as_ref()
        }
    }

    pub fn into_pinned(self) -> PinPtr<Q>{
        self.pin();
        PinPtr(self)
    }

    pub fn get_pins(&self) -> u32{
        self.inner().pins.load(Ordering::Acquire)
    }

    ///
    /// Increases the number of pins on this object
    pub fn pin(&self){
        self.inner().pins.fetch_add(1, Ordering::Relaxed);
    }

    ///
    /// Decreases the number of pins on this object
    pub fn unpin(&self){
        self.inner().pins.fetch_sub(1, Ordering::Release);
        fence(Ordering::Acquire);
    }

    pub fn move_up(&self) -> u8{
        self.inner().pos.fetch_add(1, Ordering::Relaxed)
    }

    pub fn get_pos(&self) -> u8{
        self.inner().pos.load(Ordering::Acquire)
    }

    pub fn ptr_mut(&self) -> &mut Q{
        unsafe{
            &mut (*self.ptr.as_ptr()).payload
        }
    }

    pub fn raw(&self) -> NonNull<Data<Q>>{
        self.ptr
    }
}


impl<Q: ?Sized> Deref for Ptr<Q>{
    type Target = Q;

    fn deref(&self) -> &Self::Target {

        //Wait for the lock to become false
        while self.inner().trace_lock.load(Ordering::Acquire) {
            thread::yield_now();
        }

        &self.inner().payload
    }
}

impl<Q: ?Sized> DerefMut for Ptr<Q>{
    fn deref_mut(&mut self) -> &mut Self::Target {
        while self.inner().trace_lock.load(Ordering::Acquire) {
            thread::yield_now();
        }

        self.ptr_mut()
    }
}

impl<Q: ?Sized + Traceable> Traceable for Ptr<Q>{
    fn trace(&self, ctx: &mut TracingContext) {
        //If this object is not returned to position 0, call trace on the payload.
        if self.inner().pos.swap(0, Ordering::Relaxed) > 0{
            if !self.inner().trace_lock.swap(true, Ordering::Acquire){
                self.inner().payload.trace(ctx);                
                self.inner().trace_lock.store(false, Ordering::Relaxed);
            }
        }
    }
}


pub struct PinPtr<Q: ?Sized>(Ptr<Q>);

impl<Q: ?Sized> Drop for PinPtr<Q>{
    fn drop(&mut self) {
        self.0.unpin();
    }
}

impl<Q: ?Sized> Clone for PinPtr<Q>{
    fn clone(&self) -> Self {
        self.0.pin();
        Self(self.0.clone())
    }
}

impl<Q: ?Sized> PinPtr<Q>{
    pub fn into_unpinned(self) -> Ptr<Q>{
        self.0.clone()
    }
}

impl<Q: Sized> PinPtr<Q>{
    pub fn new(data: Q) -> Self{
        Self(Ptr::new_pinned(data))
    }
}

impl<Q: ?Sized> Deref for PinPtr<Q>{
    type Target = Q;

    fn deref(&self) -> &Self::Target {

        //Wait for the lock to become false
        while self.0.inner().trace_lock.load(Ordering::Acquire) {
            thread::yield_now();
        }

        &self.0.inner().payload
    }
}

impl<Q: ?Sized> DerefMut for PinPtr<Q>{
    fn deref_mut(&mut self) -> &mut Self::Target {
        while self.0.inner().trace_lock.load(Ordering::Acquire) {
            thread::yield_now();
        }

        self.0.ptr_mut()
    }
}
