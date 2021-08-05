use iter::Iterator;

use crate::linklist::LinkList;

mod iter;
mod linklist;

fn main() {
    let mut list = LinkList::new();
    list.push_back(1);
    list.push_front(2);
    list.pop_front();
    list.pop_back();
    list.len();
    let mut iter = list.iter();
    iter.next();
    let mut iter = list.rev_iter();
    iter.prev();
}
