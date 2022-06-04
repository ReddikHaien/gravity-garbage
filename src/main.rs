mod v2_test {
    use std::{mem::size_of, thread, time::Duration};

    use gravity_garbage::v2::{MemoryInterface, PinnedPointer, Pointer, RawPointer, Traceable};
    struct A {
        next: Pointer<A>,
    }

    impl Drop for A {
        fn drop(&mut self) {
            println!("dropped!!");
        }
    }

    impl Traceable for A {
        unsafe fn get_pointers(&self) -> Vec<RawPointer> {
            vec![self.next.clone_raw()]
        }

        fn as_any(&self) -> &dyn std::any::Any {
            self
        }

        fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
            self
        }
    }

    pub fn run() {
        let interface = MemoryInterface::create_collector();
        let mut cur = interface.track(A {
            next: Pointer::default(),
        });

        for _ in 0..5 {
            let new = interface.track(A {
                next: cur.clone_unpinned(),
            });
            let _ = interface.track(A {
                next: Pointer::default(),
            });

            cur = new.clone_pinned();
        }
        thread::sleep(Duration::from_secs(10));
        drop(cur);
        thread::sleep(Duration::from_secs(20));
    }
}

pub fn main() {
    v2_test::run();
}
