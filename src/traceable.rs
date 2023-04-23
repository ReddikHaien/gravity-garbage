use std::{sync::{RwLock, Mutex}, ops::Deref};

pub trait Traceable: Send + Sync{
    fn trace(&self);
}

impl<Q: Traceable> Traceable for Option<Q>{
    fn trace(&self) {
        if let Some(ref inner) = self{
            inner.trace();
        }
    }
}

impl<Q: Traceable> Traceable for Box<Q>{
    fn trace(&self) {
        self.as_ref().trace();
    }
}

impl<Q: Traceable> Traceable for [Q]{
    fn trace(&self) {
        for v in self{
            v.trace();
        }
    }
}

impl<Q: Traceable> Traceable for RwLock<Q>{
    fn trace(&self) {
        if let Ok(lock) = self.read(){
            lock.trace();
        }
    }
}

impl<Q: Traceable> Traceable for Mutex<Q>{
    fn trace(&self) {
        if let Ok(lock) = self.lock(){
            lock.trace();
        }
    }
}
