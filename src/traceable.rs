use std::{sync::{RwLock, Mutex}};

use crate::prelude::{TracingContext};

pub trait Traceable: Send + Sync{
    fn trace(&self, ctx: &mut TracingContext);
}

impl<Q: Traceable> Traceable for Option<Q>{
    fn trace(&self, ctx: &mut TracingContext) {
        if let Some(ref inner) = self{
            inner.trace(ctx);
        }
    }
}

impl<Q: Traceable> Traceable for Box<Q>{
    fn trace(&self, ctx: &mut TracingContext) {
        self.as_ref().trace(ctx);
    }
}

impl<Q: Traceable> Traceable for [Q]{
    fn trace(&self, ctx: &mut TracingContext) {
        for v in self{
            v.trace(ctx);
        }
    }
}

impl<Q: Traceable> Traceable for RwLock<Q>{
    fn trace(&self, ctx: &mut TracingContext) {
        if let Ok(lock) = self.read(){
            lock.trace(ctx);
        }
    }
}

impl<Q: Traceable> Traceable for Mutex<Q>{
    fn trace(&self, ctx: &mut TracingContext) {
        if let Ok(lock) = self.lock(){
            lock.trace(ctx);
        }
    }
}
