use std::{ptr::{NonNull, drop_in_place}, mem::MaybeUninit, thread, sync::{Mutex, Once}, alloc::{Global, Allocator, Layout}, time::{Instant, Duration}};

use crate::{mem_block::Ptr, prelude::Traceable};

const NUM_GENS: usize = 3;


pub struct Manager{
    objects: Vec<Ptr<dyn Traceable>>
}

impl Manager{
    pub fn new() -> Self{
        Self{
            objects: vec![]
        }
    }

    pub fn run(mut self){

        thread::spawn(move ||{
            
            loop {
                
                thread::sleep(Duration::from_millis(20));

                //Collecting new objects
                with_sink(|sink|{
                    let l = sink.len();
                    for _ in 0..l{
                        self.objects.push(sink.pop().expect("Should be elements"));
                    }
                });

                self.objects.iter().for_each(|ptr|{
                    ptr.move_up();
                });

                //Mark
                self.objects.iter().for_each(|ptr|{
                    if ptr.get_pins() > 0{
                        ptr.trace();
                    }
                });

                //Sweep
                self.objects.retain(|ptr|{

                    //If an object has been moved off the cliff(10 steps), then it is no longer referenced by the program and can be dropped
                    //It shouldn't matter if the object is part of a cyclic reference, since it's pointers are no longer going to be used by the program
                    if ptr.get_pos() > 10{
                        unsafe{
                            drop_in_place(ptr.ptr_mut());
                            Global.deallocate(ptr.raw().cast(), Layout::for_value_raw(ptr.raw().as_ptr()));
                        }
                        false
                    }
                    else {
                        true
                    }
                });
            }
        });
    }
}

///
/// Starts a new instance of Manager
pub fn start_gc_manager() {
    Manager::new().run();
}

pub(crate) fn with_sink<F, Q>(f: F) -> Q
    where F: FnOnce(&mut Vec<Ptr<dyn Traceable>>) -> Q
{
    static mut SINK: MaybeUninit<Mutex<Vec<Ptr<dyn Traceable>>>> = MaybeUninit::uninit();
    static ONCE: Once = Once::new();

    ONCE.call_once(||{
        unsafe{
            SINK = MaybeUninit::new(Mutex::new(vec![]));
        }
    });
    unsafe{
        let sink = SINK.assume_init_mut();
        let mut locked = sink.lock().unwrap();
        f(&mut locked)
    }
}


