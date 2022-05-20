use super::object::{GarbageObject, Traceable};

pub(crate) struct Collector{
    objects: Vec<*mut GarbageObject>
}

impl Collector{
    fn new() -> Self{
        Self{
            objects: vec![]
        }
    }

    fn add_new_object(&mut self, object: *mut GarbageObject){
        self.objects.push(object);
    }

    fn run(&mut self){
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
            let children = unsafe { node.deref_raw().get_pointers() };

            let parent = node.position;
            children.iter().for_each(|x|{
                let child = unsafe { x.data_mut() };
                child.position = child.position.min(parent);
            });

            max = max.max(parent);
        });

        println!("max height {}",max);
    }


}

pub enum MemoryMessage{
    Track(*mut GarbageObject),
}

pub struct MemoryInterface{
       
}