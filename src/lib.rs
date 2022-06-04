use std::{
    alloc::Layout,
    f32::consts::E,
    mem::{align_of, size_of},
    sync::{
        atomic::Ordering,
        mpsc::{self, Sender},
    },
    thread,
};

use pointer::TransferObject;
pub use pointer::{PinnedPointer, Pointer};

use wrapper::Wrapper;

pub mod macros;
mod pointer;
mod wrapper;

pub mod v2;

//Helper macros
macro_rules! read {
    ($x:expr) => {
        unsafe { $x.as_ref().unwrap() }
    };
}

macro_rules! write {
    ($x:expr) => {
        unsafe { $x.as_mut().unwrap() }
    };
}

macro_rules! to_usize {
    ($x:expr) => {
        unsafe { std::mem::transmute::<_, usize>($x) }
    };
}

struct MemoryManager {
    objects: Vec<*mut Wrapper>,
    connections: Vec<(*mut Wrapper, *mut Wrapper)>,
}

impl MemoryManager {
    fn new() -> Self {
        Self {
            objects: Vec::new(),
            connections: Vec::new(),
        }
    }

    fn make_tracked(&mut self, value: *mut Wrapper) {
        if value.is_null() || read!(value).value.is_null() {
            return;
        }
        self.objects.push(value)
    }

    fn create_connection(&mut self, from: *mut Wrapper, to: *mut Wrapper) {
        if from.is_null()
            || read!(from).value.is_null()
            || to.is_null()
            || read!(to).value.is_null()
        {
            return;
        }
        self.connections.push((from, to));
    }

    fn remove_connection(&mut self, from: *mut Wrapper, to: *mut Wrapper) {
        self.connections
            .retain(|x| to_usize!(x.0) != to_usize!(from) || to_usize!(x.1) != to_usize!(to));
    }

    fn update(&mut self) {
        let mut pinned = 0;

        //Apply the gravity
        for x in self.objects.iter() {
            //Pinned objects are teleported up to 0
            if read!(x).get_pins() > 0 {
                write!(x).pos = 0;
                write!(x).pinned = true;
                pinned += 1;
            }
            //Everything else is moved down by 1
            else {
                let old = read!(x).pos;
                write!(x).pos = old + 1;
                write!(x).pinned = false;
            }
        }

        //If everything is pinned, don't bother with checking the connections
        if pinned == self.objects.len() {
            return;
        }

        //Sorting the array based on wether a connection is pinned or not
        self.connections
            .sort_by(|a, b| read!(a.1).pos.cmp(&read!(b.1).pos));

        //connections
        for (from, to) in self.connections.iter() {
            //Pick the shortest distance between the referee: from, and the referent: to
            let a = read!(from).pos;
            let b = read!(to).pos;

            //move to the highest point
            let mov = a.min(b);

            //If the parent is pinned, both objects are moved up to the top
            if read!(from).pinned {
                write!(to).pos = 0;
            } else if !read!(to).pinned {
                //if neither are pinned, move to up to the shortest distance, this means that it won't be moved if its already higher up than the referee
                write!(to).pos = mov;
            }
        }

        //The garbage collection

        let l = self.objects.len();
        let death_zone = l.max(100) as u64 + 2;

        for x in (0..l).rev() {
            if read!(self.objects[x]).pos > death_zone {
                let v = self.objects.swap_remove(x);

                //Removes every connection that the dead object contains in the list
                //We do this, beacuse, if the object if there is other referenses to it, their not referenced by the program any more

                self.connections.retain(|(a, b)| {
                    to_usize!(*a) != to_usize!(v) && to_usize!(*b) != to_usize!(v)
                });

                unsafe {
                    Box::from_raw(v);
                };
            }
        }
    }
}

enum MemoryMessages {
    Track(*mut Wrapper),
    Connect(*mut Wrapper, *mut Wrapper),
    DisConnect(*mut Wrapper, *mut Wrapper),
    Exit,
}

unsafe impl Send for MemoryMessages {}

pub struct MemoryInterface {
    tsc: Sender<MemoryMessages>,
}

impl MemoryInterface {
    pub fn initialize_manager() -> Self {
        let (tsc, rcv) = mpsc::channel();

        thread::spawn(move || {
            let mut manager = MemoryManager::new();

            'outer: loop {
                while let Ok(message) = rcv.try_recv() {
                    match message {
                        MemoryMessages::Track(object) => {
                            manager.make_tracked(object);
                        }
                        MemoryMessages::Connect(from, to) => {
                            manager.create_connection(from, to);
                        }
                        MemoryMessages::DisConnect(from, to) => {
                            manager.remove_connection(from, to);
                        }
                        MemoryMessages::Exit => {
                            break 'outer;
                        }
                    }
                }

                manager.update();
            }

            println!("garbage thread stopped");
        });

        Self { tsc }
    }

    pub fn connect(&self, from: TransferObject, to: TransferObject) {
        self.tsc
            .send(MemoryMessages::Connect(from.unwrap(), to.unwrap()))
            .unwrap();
    }

    pub fn make_tracked<T>(&self, value: T) -> PinnedPointer<T> {
        let ptr = unsafe { Wrapper::from_inner(value) };
        self.tsc.send(MemoryMessages::Track(ptr)).unwrap();
        PinnedPointer::new_from(ptr)
    }

    pub fn disconnect(&self, from: TransferObject, to: TransferObject) {
        self.tsc
            .send(MemoryMessages::DisConnect(from.unwrap(), to.unwrap()))
            .unwrap();
    }

    pub unsafe fn terminate(self) {
        self.tsc.send(MemoryMessages::Exit).unwrap();
    }
}
