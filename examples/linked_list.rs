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


    let mut p = root.clone();
    loop{
        println!("{}",p.value);

        if let Some(ref l) = p.next{
            p = l.clone().into_pinned();
        }
        else{
            break;
        }
    }
}