mod v2_test{
    use std::{thread, time::Duration, mem::size_of};

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

        println!("size of        pointer to          A: {}", size_of::<Pointer<A>>());
        println!("size of pinned pointer to          A: {}", size_of::<PinnedPointer<A>>());
        println!("size of    raw pointer to          A: {}", size_of::<RawPointer>());
        println!("size of        pointer to [usize;16]: {}", size_of::<Pointer<[usize;16]>>());
        println!("size of pinned pointer to [usize;16]: {}", size_of::<PinnedPointer<[usize;16]>>());
        println!("size of    raw pointer to [usize;16]: {}", size_of::<RawPointer>());


        for _ in 0..2{
            let new = interface.track(A{
                next: cur.clone_unpinned()
            });
            

            //This object will be unpinned at the end of the loop
            //Inefficient, just for demostration purposes
            let _ = interface.track(A{
                next: cur.clone_unpinned()
            });

            cur = new.clone_pinned();
        }
        thread::sleep(Duration::from_secs(15));
        drop(cur);
        thread::sleep(Duration::from_secs(60));
    }
}

pub fn main(){

    v2_test::run();
}