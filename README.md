# gravity-garbage
## a not so good gravity-driven garbage collector written in rust.
This is an attempt at implementing a garbage collector that is powered by gravity.
the premise is as follow:
- the gc is divided into objects and connection, on evey iteration of the gc-thread each object is moved down towards the treshold by a fixed amount, 
if the object is pinned(by a PinnedPointer<T>) it'll be moved back to start(0). 
- Thereafter each connection will be evaluated. If the parent object is closer to start than the child object then the child object will be moved up to the parent.
This prevents an parent from "dragging" down a child that has valid references pointing to it.
- Finally when the object reaches the treshold it'll be freed. This happens by first removing the object from the collection, and then remove every connection referencing it.
If everything is working as expected. The objects that fall to the threshold is no longer reachable, So it should be safe to free its memory.

### Usages
#### Simple example creating an linked list
```rust
use gravity_garbage::{
  MemoryInterface,
  Pointer,
  PinnedPointer,
  set_field
 };


fn main(){
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
}
```
Due to the need to connect objects, the simplest way to do this is just to assign the field to `Pointer<T>::default()`, and use the macro `set_field!(interface, a.b = c)`.
<br>
or you can connect them directly
```rust
interface.connect(obj.make_transfer(),obj.field.make_transfer());
```
> Note: `make_transfer()` is an hidden method, so i would recommend using the `set_field!(...)` macro :)

### Usage Other Places
This is an untestet, ineffectiv and ductaped solution. It's in no shape or form ready for production or debug. But you are free to use it at your own risk :)
