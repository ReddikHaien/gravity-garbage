use std::{collections::{HashSet, HashMap}, thread, sync::mpsc::{self, Sender}, time::Instant};


use super::{object::{GarbageObject, Traceable}, pointer::PinnedPointer};

pub(crate) struct Collector{
    objects: Vec<*mut GarbageObject>,
    dead: HashMap<*mut GarbageObject,usize>
}

impl Collector{
    fn new() -> Self{
        Self{
            objects: vec![],
            dead: HashMap::new()
        }
    }

    fn add_new_object(&mut self, object: *mut GarbageObject){
        self.objects.push(object);
    }

    fn run(&mut self){
        let timer = Instant::now();
        //Start by applying the gravity to every node, and resetting the positions of the pinned objects to 0
        self.objects.iter().for_each(|pnt|{
            let node = unsafe { pnt.as_mut().unwrap() };

            if node.get_pins() > 0 {
                node.position = 0;
            }
            else{
                node.position += 1;
            }
        });

        let mut max = 0;

        //Iterate again and move children up to the parent closest to 0
        self.objects.iter().for_each(|pnt|{
            let node = unsafe { pnt.as_ref().unwrap() };
            let mut children = unsafe { node.deref_raw().get_pointers() };
            let parent = node.position;
            children.iter_mut().for_each(|x|{

                if !unsafe {x.data_pnt().is_null()}{
                    match self.dead.get_mut(&unsafe{x.data_pnt()}) {
                        Some(ref_count) => {
                            *ref_count += 1;
                        },
                        _ => {
                            let child = unsafe { x.data_mut() };
    
                            child.position = child.position.min(parent);
                        }
                    }
                }

                
            });

            max = max.max(parent);
        });
        //Remove and drop garbage objects that don't have any references anymore
        self.dead.retain(|pnt,refs|{
            match *refs == 0{
                true => {
                    unsafe { 
                        Box::from_raw(*pnt) 
                    };
                    false
                },
                _ => {
                    *refs = 0;
                    true
                }
            }
        });

        let treshold = self.objects.len()+2;
        //Iterate the objects and mark the dead ones
        for i in (0..self.objects.len()).rev(){
            let x = &self.objects[i];
            let node = unsafe { x.as_ref().unwrap() };
            if node.position > treshold{
                match self.dead.get_mut(x) {
                    None => {
                        self.dead.insert(*x, 0);
                        self.objects.swap_remove(i);
                    },
                    _ => ()
                }
            }
        }
        


        //println!("max height {}, objects {}, time used {}ms",max,self.objects.len(),timer.elapsed().as_millis());
    }


}

pub enum MemoryMessage{
    Track(*mut GarbageObject),
    Exit
}

unsafe impl Send for MemoryMessage{}

pub struct MemoryInterface{
    sender: Sender<MemoryMessage>
}

impl MemoryInterface{
    pub fn create_collector() -> Self{
        let (sender, receiver) = mpsc::channel();
        
        thread::spawn(move ||{
            let mut collector = Collector::new();
            let receiver = receiver;
            'outer: loop {
                let mut count = 0;
                while let Ok(message) = receiver.try_recv() {
                    match message {
                        MemoryMessage::Track(object) => collector.add_new_object(object),
                        MemoryMessage::Exit => break 'outer
                    }
                    // count+= 1;
                    // if count > 1024{
                    //     break;
                    // }
                }

                collector.run();
            }
        });

        Self{
            sender
        }
    }

    pub fn track<T: Traceable + Sized + 'static>(&self, value: T) -> PinnedPointer<T>{
        let object = unsafe { GarbageObject::new_from(value) };
        self.sender.send(MemoryMessage::Track(object)).unwrap();
        unsafe { PinnedPointer::from_raw(object) }
    }
}