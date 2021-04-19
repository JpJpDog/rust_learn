### lifetime annotation ###
All struct with ref member must have explicit lifetime annotation('la').  
Here it means how long the ref member is legal.  
```rust
struct TT<'a> {
    tt1: &'a u32,
    tt2: &'a u32,
}
```
This means both tt1 and tt2's lifetime is the same as the shorter lifetime of which tt1 and tt2 ref to (and off course no more than the instance of TT).  
```rust
struct TT<'a, 'b> {
    tt1: &'a u32,
    tt2: &'b u32,
}
```
If we make seperate 'la' like this, tt1 and tt2 will seperately live as long as the one they ref to (and also off course no more than TT instance).  
  
Function also needs explicit 'la' when it returns a ref.  
```rust
fn tt<'a>(t1: &'a u32, t2: &'a u32) -> &'a u32 {
    if t1 > t2 {
        t1
    } else {
        t2
    }
}
```
This example means return value's lifetime is the smaller one of t1 and t2.  
Function's 'la' can be implicit in the following 3 cases:  
1. If we not provide parameters's 'la', Rust will consider they are all different. (however, because return ref must be from param ref, there must be one param with explicit 'la'. This also gives the 2nd case) like this:  
   ```rust
   fn tt<'a>(t1: &'a mut u32, t2: &u32) -> &'a u32 {
        *t1 += t2;
        t1
    }
   ```
2. Only one ref param so the ret value's life time is known.  
3. The lifetime of return ref of method can be omitted if it is the same as `&self` or `&mut self`
  
### thread param in spawn ###
New thread may outlive the variable scope inevitably, so move closure is a must. Move closure just 'move' the closure capture param so no lifetime problem will happen. (primitive type just impl 'move' as copy implicitly)  
However, when we need to share between threads, `Arc` (atomic reference count) is needed.  
```rust
fn routine(data: Arc<Vec<i32>>) {
    let data = &*data;
    println!("{}", data[0]);
}
```
```rust
let data = vec![1, 2, 3];
let data = Arc::new(data);
for _ in 0..THREAD_N {
    let data = data.clone();
    handlers.push(thread::spawn(move || routine(data)));
}
```
`Arc` impliments `Deref` trait, so `&*data` can get the ref, or `data` can be use directly as T thanks to Rust's deducation. data will drop when refcnt is 0.  
  
And if we want to get mut ref in multiple thread, the `Mutex` or `RwLock` can be used.  
Mutex:  
```rust
let data = data.lock().unwrap();
drop(data);
```
```rust
let data = Arc::new(Mutex::new(data));
```
RwLock:  
```rust
let data = data.read().unwrap();
// or let mut data = data.write().unwrap();
drop(data);
```
`Mutex` and `RwLock` also implements `Deref` trait, so just call method of T and Rust will deduce it. To unlock it, the return type also implements `Drop` trait which release the lock when go out the scope or explicitly drop it by `std::mem::drop`  
  
### cell and inner mutability ###
`RefCell` can borrow mut ref of the data it owns when itself is not mut ref. The safety is guaranteed runtime, by testing whether a `MutRef` and a `Ref` are in the same scope.  
```rust
fn append_and_add(data: &RefCell<Vec<u32>>) {
    let mut data1 = data.borrow_mut();
    data1.push(1);
    for dat in &mut *data1 {
        *dat += 1;
    }
    drop(data1);
}
```
in this example, we can also find that data1 with type `RefMut<Vec<i32>>` can use `push` directly but cannot iterate in "for clause" directly. the `dat` with type `& mut u32` also cannot "+= 1" directly. I guess Rust only automatically dereference when calling method whose struct implements `deref` trait but can do nothing with primitive type or other syntaxic sugar.  
  
`Cell` can only used when T implements `Copy` trait. By method `get()`, it return a copied value. And method `set()` can mutate the value with not mut ref.  
  
`Cell` and `RefCell` all implements by `UnsafeCell`. `Mutex` and `RwLock` also owns data by the latter.  
  
### Sync and Send Trait ###

### smart pointer ###
