use crate::linklist::LinkList;

mod linklist;
mod iter;

fn main() {
    let mut list = LinkList::new();
    list.push_back(1);
    list.push_front(2);
    list.pop_front();
    list.pop_back();
    list.len();
    println!("Hello, world!");
}
