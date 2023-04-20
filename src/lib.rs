mod mem_block;

use std::{sync::{Arc, atomic::{AtomicU32, AtomicU8, Ordering, fence}, Weak, Once}, ops::Deref, mem::MaybeUninit, time::Duration};

pub trait Traceable{
    fn trace(&self);
}

#[repr(C)]
struct Data<Q: Traceable + ?Sized>{
    pins: AtomicU32,
    pos: AtomicU8,
    payload: Q
}

impl<Q: Traceable + ?Sized> Drop for Data<Q>{
    fn drop(&mut self) {
        println!("Dropping value");
    }
}

struct InnerPtr<Q: Traceable + ?Sized>(Arc<Data<Q>>);

impl<Q: Traceable + Sized + 'static> InnerPtr<Q> {
    pub fn new(data: Q) -> Self{
        Self(Arc::new(Data{
            pins: AtomicU32::new(0),
            pos: AtomicU8::new(0),
            payload: data
        }))
    }

    fn to_dyn(self) -> InnerPtr<dyn Traceable>{
        let inner: Arc<Data<dyn Traceable>> = self.0;
        InnerPtr(inner)
    }
}

impl<Q: Traceable + ?Sized> Clone for InnerPtr<Q>{
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<Q: Traceable + ?Sized> InnerPtr<Q> {

    fn get_pins(&self) -> u32{
        self.0.pins.load(Ordering::Acquire)
    }

    fn get_pos(&self) -> u8{
        self.0.pos.load(Ordering::Acquire)
    }

    fn tag(&self){
        if self.0.pos.swap(0, Ordering::Release) > 0{
            fence(Ordering::Acquire);
            self.0.payload.trace();
        }
    }

    fn move_up(&self){
        self.0.pos.fetch_add(1, Ordering::SeqCst);
    }

    fn pin(&self){
        self.0.pins.fetch_add(1, Ordering::Relaxed);
    }

    fn unpin(&self){
        self.0.pins.fetch_sub(1, Ordering::Relaxed);
    }

    fn payload(&self) -> &Q{
        &self.0.payload
    }
}

pub struct Ptr<Q: Traceable + ?Sized>(InnerPtr<Q>);

impl<Q: Traceable + ?Sized> Ptr<Q>{
    fn from_inner(inner: InnerPtr<Q>) -> Self{
        Self(inner)
    }
    pub fn into_pinned(self) -> PinPtr<Q> {
        PinPtr::from_inner(self.0)
    }
}

impl<Q: Traceable + ?Sized> Clone for Ptr<Q>{
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<Q: Traceable + ?Sized> Deref for Ptr<Q>{
    type Target = Q;

    fn deref(&self) -> &Self::Target {
        self.0.payload()
    }
}

pub struct PinPtr<Q: Traceable + ?Sized>(InnerPtr<Q>);

impl<Q: Traceable + Sized + 'static> PinPtr<Q>{
    pub fn new(data: Q) -> Self{
        with_sink(move |sink|{
            let inner = InnerPtr::new(data);
            let p = Self::from_inner(inner.clone());    
            sink.push(inner.to_dyn());
            p
        })
    }
}

impl<Q: Traceable + ?Sized> PinPtr<Q>{
    fn from_inner(inner: InnerPtr<Q>) -> Self{
        inner.pin();
        Self(inner)
    }

    pub fn reference(&self) -> Ptr<Q>{
        Ptr::from_inner(self.0.clone())
    }
}

impl<Q: Traceable + ?Sized> Deref for PinPtr<Q>{
    type Target = Q;

    fn deref(&self) -> &Self::Target {
        self.0.payload()
    }
}

impl<Q: Traceable + ?Sized> Clone for PinPtr<Q> {
    fn clone(&self) -> Self {
        let copy = self.0.clone();
        copy.pin();
        Self(copy)
    }
}

impl<Q: Traceable + ?Sized> Drop for PinPtr<Q> {
    fn drop(&mut self) {
        self.0.unpin();
    }
}

fn with_sink<F, Q>(f: F) -> Q
    where F: FnOnce(&mut Vec<InnerPtr<dyn Traceable>>) -> Q
{
    
    static mut SINK: MaybeUninit<Vec<InnerPtr<dyn Traceable>>> = MaybeUninit::uninit();
    static ONCE: Once = Once::new();
    unsafe{
        ONCE.call_once(||{
            SINK = MaybeUninit::new(vec![]);
        });

        let sink = SINK.assume_init_mut();
        f(sink)
    }
}

pub fn start_memory_manager() -> std::thread::JoinHandle<()> {
    std::thread::spawn(||{
        let mut objects = Vec::new();
        loop {
            std::thread::sleep(Duration::from_millis(20));
            
            //Collect new objects
            let new = with_sink(|sink|{
                if !sink.is_empty()
                {
                    Some(std::mem::take(sink))
                }
                else{
                    None
                }
            });

            if let Some(new) = new{
                objects.extend(new);
            }

            //Tag referenced objects
            objects.iter().for_each(|x|{
                let pin = x.get_pins();
                if pin > 0{
                    x.tag();
                }
                else{
                    x.move_up()
                }
            });

            //Kill objects past treshold
            objects.retain_mut(|object|{
                object.get_pos() < 10
            });

        }
    })
}