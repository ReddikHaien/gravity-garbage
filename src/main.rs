use std::{thread, time::Duration, io::stdin, process::exit};

use gravity_garbage::{MemoryInterface, Pointer, PinnedPointer, set_field};

mod v2_test{
    use std::{thread, time::Duration};

    use gravity_garbage::v2::{PinnedPointer, Pointer, MemoryInterface, Traceable, RawPointer};
    struct A{
        next: Pointer<A>
    }

    impl Drop for A{
        fn drop(&mut self) {
            println!("dropped!!");
        }
    }

    impl Traceable for A {
        unsafe fn get_pointers(&self) -> Vec<RawPointer> {
            vec![
                self.next.clone_raw()
            ]
        }

        fn as_any(&self) -> &dyn std::any::Any {
            self
        }

        fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
            self
        }
    }

    pub fn run(){
        let interface = MemoryInterface::create_collector();
        let mut cur = interface.track(A{
            next: Pointer::default()
        });

        for _ in 0..2{
            let new = interface.track(A{
                next: cur.clone_unpinned()
            });
            let new2 = interface.track(A{
                next: cur.clone_unpinned()
            });
            cur = new.clone_pinned();
        }
        thread::sleep(Duration::from_secs(15));
        drop(cur);
        thread::sleep(Duration::from_secs(60));
    }
}


struct A{
    a: Pointer<A>
}

pub fn main(){

    v2_test::run();

    // let interface = MemoryInterface::initialize_manager();
    
    // let mut cur = interface.make_tracked(A{
    //     a: Pointer::default()
    // });

    // for _ in 0..10_000{
    //     let mut new = interface.make_tracked(A{
    //         a: Pointer::default(),
    //     });
    //     set_field!(interface, new.a = cur);

    //     cur = new.clone_pinned();
    // }

    // println!("Ferdig med lenke");
    // let mut out = String::new();
    // let _ = stdin().read_line(&mut out);    


    // cur = PinnedPointer::default();
    
    // println!("Trykk enter igjen for å avslutte");
    // let _ = stdin().read_line(&mut out);

    // unsafe{
    //     interface.terminate();
    // }
}