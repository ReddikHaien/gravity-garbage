use std::{sync::{atomic::Ordering, mpsc::{Sender, self}}, f32::consts::E, thread, alloc::Layout, mem::{size_of, align_of}};

use pointer::TransferObject;
pub use pointer::{PinnedPointer,Pointer};

use wrapper::Wrapper;

mod pointer;
mod wrapper;
pub mod macros;

//Helper macros
macro_rules! read {
    ($x:expr) => {
        unsafe{$x.as_ref().unwrap()}
    };
}

macro_rules! write {
    ($x:expr) => {
        unsafe{$x.as_mut().unwrap()}
    };
}

macro_rules! to_usize {
    ($x:expr) => {
        unsafe{ std::mem::transmute::<_,usize>($x) }
    };
}


struct MemoryManager{
    objects: Vec<*mut Wrapper>,
    connections: Vec<(*mut Wrapper, *mut Wrapper)>,
    dropped: u32,
}

impl MemoryManager{

    fn new() -> Self{
        Self{
            objects: Vec::new(),
            connections: Vec::new(),
            dropped: 0,
        }
    }

    fn make_tracked(&mut self, value: *mut Wrapper){
        if value.is_null() || read!(value).value.is_null(){
            return;
        }
        //println!("New {:?}",to_usize!(value));
        self.objects.push(value)
    }

    fn create_connection(&mut self, from: *mut Wrapper, to: *mut Wrapper){
        if from.is_null() || read!(from).value.is_null() || to.is_null() || read!(to).value.is_null(){
            return;
        }
        //println!("Connection {} <-> {}",to_usize!(from),to_usize!(to));
        self.connections.push((from,to));
    }

    fn remove_connection(&mut self, from: *mut Wrapper, to: *mut Wrapper){
        self.connections.retain(|x|{
            to_usize!(x.0) != to_usize!(from) || to_usize!(x.1) != to_usize!(to)
        });
    }

    fn update(&mut self){
        let mut max_pos: f32 = 0.0;
        let mut min_pos: f32 = f32::MAX;
        let mut pinned = 0;
        //Gravity
        for x in self.objects.iter(){
            let x = *x;
            if read!(x).get_pins() > 0{
                write!(x).pos = 0.0;
                write!(x).pinned = true;
                pinned+= 1;
            }
            else{
                let old = read!(x).pos;
                write!(x).pos = old + 1.0;
                write!(x).pinned = false;
                max_pos = max_pos.max(old);
                min_pos = min_pos.min(old);
                //println!("obj: {}, pos: {}",to_usize!(x), read!(x).pos);
            }
        }

        if self.objects.len() > 0{
            //println!("range [{} <-> {}], pinned: {}, total: {}, total dropped: {}",max_pos,min_pos,pinned,self.objects.len(), self.dropped);
        }
        
        self.connections.sort_by(|a,b|{
            match read!(b.0).pinned.cmp(&read!(a.0).pinned){
                std::cmp::Ordering::Equal => {
                    read!(b.1).pinned.cmp(&read!(a.1).pinned)
                },
                x => x
            }
        });

        for _ in 0..6{
            //connections
            for (from,to) in self.connections.iter(){
                let a = read!(from).pos;
                let b = read!(to).pos;
                
                //move to the highest point
                let mov = a.min(b);

                //println!("[{}]: {},[{}]: {}, connection count: {}",to_usize!(from),a,to_usize!(to),b,self.connections.len());

                if read!(from).pinned{
                    write!(from).pos = 0.0;
                    write!(from).pinned = true;
                    write!(to).pos = 0.0;
                    write!(to).pinned = true;
                }
                else if !read!(to).pinned{
                    //Koblingen er mellom to som ikke er pinned, om to er pinned så gjør vi ingenting med den siden den trolig er endret allerede
                    write!(to).pos = mov;
                }
                else {
                    write!(to).pos = 0.0;
                }
            }
        }
        
        let l = self.objects.len();
        let death_zone = (l.max(100) / 6) as f32;
        for x in (0..l).rev(){
            if read!(self.objects[x]).pos > death_zone{
                let v = self.objects.swap_remove(x);
                self.connections.retain(|(a,b)|{
                    to_usize!(a) != to_usize!(v) && to_usize!(b) != to_usize!(v)
                });
                //println!("Object Killed {:?}, {}",to_usize!(v),read!(v).pos);
                
                unsafe{ 
                    Box::from_raw(v);
                };
                self.dropped += 1;
            }
        }

    }
}

enum MemoryMessages{
    Track(*mut Wrapper),
    Connect(*mut Wrapper, *mut Wrapper),
    DisConnect(*mut Wrapper, *mut Wrapper)
}

unsafe impl Send for MemoryMessages{}

pub struct MemoryInterface{
    tsc: Sender<MemoryMessages>
}

impl MemoryInterface{
    pub fn initialize_manager() -> Self{
        let (tsc, rcv) = mpsc::channel();

        thread::spawn(move ||{
            let mut manager = MemoryManager::new();

            loop{
                let mut new_objs = 0;
                let mut new_connections = 0;
                let mut removed_connections = 0;
                while let Ok(message) = rcv.try_recv() {
                    match message {
                        MemoryMessages::Track(object) => {
                            manager.make_tracked(object);
                            new_objs += 1;
                        }
                        MemoryMessages::Connect(from, to) => {
                            manager.create_connection(from, to);
                            new_connections += 1;
                        }
                        MemoryMessages::DisConnect(from, to) => {
                            manager.remove_connection(from, to);
                            removed_connections += 1;
                        }
                    }
                }

                if new_objs != 0 && new_connections != 0 && removed_connections != 0{
                    println!("new objs: {}, new connections: {}, removed connections: {}, death zone: {}",new_objs,new_connections,removed_connections,new_connections.max(100) / 6);
                }

                manager.update();
            }
        });

        Self{
            tsc,
        }
    }

    pub fn connect(&self, from: TransferObject, to: TransferObject){
        self.tsc.send(MemoryMessages::Connect(from.unwrap(),to.unwrap())).unwrap();
    }

    pub fn make_tracked<T>(&self, value: T) -> PinnedPointer<T>{
        let ptr = unsafe { Wrapper::from_inner(value) };
        self.tsc.send(MemoryMessages::Track(ptr)).unwrap();
        PinnedPointer::new_from(ptr)
    }

    pub fn disconnect(&self, from: TransferObject, to: TransferObject){
        self.tsc.send(MemoryMessages::DisConnect(from.unwrap(),to.unwrap())).unwrap();
    }
}