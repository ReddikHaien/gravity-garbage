use std::sync::atomic::{AtomicI32, Ordering};

pub struct Wrapper{
    pub(crate) pins: AtomicI32,
    pub(crate) pos: u64,
    pub(crate) pinned: bool,
    pub(crate) value: *mut (),
}

impl Wrapper{
    pub(crate) unsafe fn from_inner<T>(inner: T) -> *mut Wrapper{
        let ptr = Box::into_raw(Box::new(inner));
        let boxed = Box::new(
            Self{
                value: ptr as *mut  (),
                pins: AtomicI32::new(0),
                pos: 0,
                pinned: false
            }
        );

        Box::into_raw(boxed)
    }

    pub(crate) fn pin(&self){
        self.pins.fetch_add(1, Ordering::SeqCst);
    }

    pub(crate) fn unpin(&self){
        self.pins.fetch_sub(1, Ordering::SeqCst);
    }
    
    pub(crate) fn deref_as<T>(&self) -> &T{
        unsafe{ (self.value as *mut T).as_ref().unwrap() }
    }

    pub(crate) fn deref_as_mut<T>(&self) -> &mut T{
        unsafe{ (self.value as *mut T).as_mut().unwrap() }
    }

    pub(crate) fn get_pins(&self) -> i32{
        self.pins.load(Ordering::SeqCst)
    }
}

impl Drop for Wrapper{
    fn drop(&mut self) {
        if !self.value.is_null(){
            unsafe{
                Box::from_raw(self.value);
            }
        }
    }
}