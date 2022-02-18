use std::{thread, time::Duration, io::stdin, process::exit};

use gravity_garbage::{MemoryInterface, Pointer, PinnedPointer, set_field};




struct A{
    a: Pointer<A>
}

pub fn main(){
    let interface = MemoryInterface::initialize_manager();
    
    let mut cur = interface.make_tracked(A{
        a: Pointer::default()
    });

    for _ in 0..1000{
        let mut new = interface.make_tracked(A{
            a: Pointer::default(),
        });
        set_field!(interface, new.a = cur);

        cur = new.clone_pinned();
    }

    println!("Ferdig med lenke");
    let mut out = String::new();
    let _ = stdin().read_line(&mut out);    


    cur = PinnedPointer::default();
    
    println!("Trykk enter igjen for å avslutte");
    let _ = stdin().read_line(&mut out);
}