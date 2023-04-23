# gravity-garbage
## a not so good gravity-driven garbage collector written in rust.
This is an attempt at writing a garbage collector powered by gravity.
The premise is as follows:
- Each object is assigned a number of pins and a position.
- In each iteration, every object falls down one position
- Objects with a positive number of pins will be returned to position 0, including objects referenced by them
- Objects that falls down to position 10 are dropped.

### Usages
#### Simple example creating a linked list
```rust

use gravity_garbage::prelude::*;


struct Node{
    value: i32,
    next: Option<Ptr<Node>>
}

impl Traceable for Node{
    fn trace(&self) {
        if let Some(ref next) = self.next{
            next.trace();
        }
    }
}

fn main(){
    start_gc_manager();


    let mut root = Ptr::new(Node{
        value: 0,
        next: None
    });

    for i in 1..100{
        let n = Ptr::new(Node{
            value: i,
            next: Some(root.clone().into_unpinned())
        });
        root = n;
    }
}
```

### Usage Other Places
This is higly unstable, experimental collector that will find ways to blow up in your face, be warned.