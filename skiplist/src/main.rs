use skiplist::SkipList;

mod skiplist;

fn main() {
    let mut list = SkipList::new();
    list.insert(1);
    list.insert(2);
    list.insert(2);
    list.insert(11);
    list.insert(22);
    list.insert(21);
    list.insert(31);
    list.insert(12);
    list.insert(3);
    println!("{}", list.get(&1).is_some());
    list.remove(&2);
    println!("{}", list.get(&2).is_some());
    list.remove(&2);
    println!("{}", list.get(&2).is_some());
}
