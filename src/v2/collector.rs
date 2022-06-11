use std::{
    collections::{HashMap, HashSet, VecDeque},
    mem::swap,
    sync::mpsc::{self, Sender},
    thread,
    time::Instant,
};

use super::{
    object::{GarbageObject, Traceable},
    pointer::PinnedPointer,
};

pub(crate) struct Collector {
    new: Vec<*mut GarbageObject>,
    objects: Vec<*mut GarbageObject>,
    dead: HashMap<*mut GarbageObject, usize>,
}

impl Collector {
    fn new() -> Self {
        Self {
            new: vec![],
            objects: vec![],
            dead: HashMap::new(),
        }
    }

    fn add_new_object(&mut self, object: *mut GarbageObject) {
        self.new.push(object);
    }

    fn run(&mut self) {
        let timer = Instant::now();

        //The trace queue to be used
        let mut trace_queue = VecDeque::new();
        //Start by applying the gravity to every node, and resetting the positions of the pinned objects to 0
        self.objects.iter().for_each(|pnt| {
            let node = unsafe { pnt.as_mut().unwrap() };

            if node.get_pins() > 0 {
                trace_queue.push_back(*pnt);
                node.position = 0;
            } else {
                node.position += 1;
            }
        });

        //Temporarily pin new objects to prevent unwanted deletion
        let mut new = vec![];
        swap(&mut self.new, &mut new);
        for x in new {
            let node = unsafe { x.as_mut().unwrap() };
            node.position = 0;
            trace_queue.push_back(x);
            self.objects.push(x);
        }

        let mut max = 0;

        while let Some(pnt) = trace_queue.pop_front() {
            let node = unsafe { pnt.as_ref().unwrap() };
            let children = unsafe { node.deref_raw().get_pointers() };

            for child in children {
                unsafe {
                    if !child.data_pnt().is_null() {
                        match self.dead.get_mut(&child.data_pnt()) {
                            Some(ref_count) => {
                                *ref_count += 1;
                            }
                            _ => {
                                let child_node = child.data_mut();
                                if child_node.position != 0 {
                                    child_node.position = 0;
                                    trace_queue.push_back(child.data_pnt());
                                }
                            }
                        }
                    }
                }
            }
        }

        max = self
            .objects
            .iter()
            .map(|x| unsafe { x.as_ref().unwrap().position })
            .max()
            .unwrap_or(0);

        //Remove and drop garbage objects that don't have any references anymore
        self.dead.retain(|pnt, refs| match *refs == 0 {
            true => {
                unsafe { Box::from_raw(*pnt) };
                false
            }
            _ => {
                *refs = 0;
                true
            }
        });

        let treshold = 100;
        //Iterate the objects and mark the dead ones
        for i in (0..self.objects.len()).rev() {
            let x = &self.objects[i];
            let node = unsafe { x.as_ref().unwrap() };
            if node.position > treshold {
                match self.dead.get_mut(x) {
                    None => {
                        self.dead.insert(*x, 0);
                        self.objects.swap_remove(i);
                    }
                    _ => (),
                }
            }
        }
    }
}

pub enum MemoryMessage {
    Track(*mut GarbageObject),
    Exit,
}

unsafe impl Send for MemoryMessage {}
unsafe impl Sync for MemoryMessage {}

#[derive(Clone)]
pub struct MemoryInterface {
    sender: Sender<MemoryMessage>,
}

impl MemoryInterface {
    pub fn create_collector() -> Self {
        let (sender, receiver) = mpsc::channel();

        thread::spawn(move || {
            let mut collector = Collector::new();
            let receiver = receiver;
            'outer: loop {
                while let Ok(message) = receiver.try_recv() {
                    match message {
                        MemoryMessage::Track(object) => collector.add_new_object(object),
                        MemoryMessage::Exit => break 'outer,
                    }
                }

                collector.run();
            }
        });

        Self { sender }
    }

    pub fn track<T: Traceable + Sized + 'static>(&self, value: T) -> PinnedPointer<T> {
        let object = unsafe { GarbageObject::new_from(value) };
        self.sender.send(MemoryMessage::Track(object)).unwrap();
        unsafe { PinnedPointer::from_raw(object) }
    }
}
