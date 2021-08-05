use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

pub struct LinkNode<T> {
    pub val: Option<T>,
    next: Option<Rc<RefCell<LinkNode<T>>>>,
    prev: Option<Weak<RefCell<LinkNode<T>>>>,
}

pub struct LinkList<T> {
    len: usize,
    head: Rc<RefCell<LinkNode<T>>>,
    tail: Weak<RefCell<LinkNode<T>>>,
}

impl<T> LinkList<T> {
    fn insert_before(at: Rc<RefCell<LinkNode<T>>>, item: Rc<RefCell<LinkNode<T>>>) {
        let prev = (*at)
            .borrow_mut()
            .prev
            .replace(Rc::downgrade(&item))
            .unwrap();
        let prev_strong = Weak::upgrade(&prev).unwrap();
        {
            let mut item = (*item).borrow_mut();
            item.next = Some(at);
            item.prev = Some(prev);
        }
        (*prev_strong).borrow_mut().next = Some(item);
    }

    fn insert_after(at: Rc<RefCell<LinkNode<T>>>, item: Rc<RefCell<LinkNode<T>>>) {
        let next = (*at).borrow_mut().next.replace(Rc::clone(&item)).unwrap();
        (*next).borrow_mut().prev = Some(Rc::downgrade(&item));
        {
            let mut item = (*item).borrow_mut();
            item.next = Some(next);
            item.prev = Some(Rc::downgrade(&at));
        }
    }
}

impl<T> LinkList<T> {
    pub fn new() -> Self {
        let head = Rc::new(RefCell::new(LinkNode {
            val: None,
            next: None,
            prev: None,
        }));
        let tail = Rc::new(RefCell::new(LinkNode {
            val: None,
            next: None,
            prev: None,
        }));
        (*tail).borrow_mut().prev = Some(Rc::downgrade(&head));
        let weak_tail = Rc::downgrade(&tail);
        (*head).borrow_mut().next = Some(tail);
        Self {
            len: 0,
            head,
            tail: weak_tail,
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    pub fn push_back(&mut self, item: T) -> Weak<RefCell<LinkNode<T>>> {
        self.len += 1;
        let item = Rc::new(RefCell::new(LinkNode {
            val: Some(item),
            next: None,
            prev: None,
        }));
        let item_weak = Rc::downgrade(&item);
        Self::insert_before(Weak::upgrade(&self.tail).unwrap(), item);
        item_weak
    }

    pub fn push_front(&mut self, item: T) -> Weak<RefCell<LinkNode<T>>> {
        self.len += 1;
        let item = Rc::new(RefCell::new(LinkNode {
            val: Some(item),
            next: None,
            prev: None,
        }));
        let item_weak = Rc::downgrade(&item);
        Self::insert_after(Rc::clone(&self.head), item);
        item_weak
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn check_list(list: LinkList<i32>, v: Vec<i32>) {
        assert_eq!(list.len(), v.len());
        let mut prev = list.head.clone();
        for x in v.iter() {
            let n = (*prev).borrow().next.clone().unwrap();
            assert_eq!((*n).borrow().val.unwrap(), *x);
            prev = n;
        }
        assert_eq!(
            Rc::downgrade((*prev).borrow().next.as_ref().unwrap()).as_ptr(),
            list.tail.as_ptr()
        );
        let mut next = Weak::upgrade(&list.tail).unwrap();
        for x in v.into_iter().rev() {
            let n = (*next).borrow().prev.clone().unwrap();
            let n = Weak::upgrade(&n).unwrap();
            assert_eq!((*n).borrow().val.unwrap(), x);
            next = n;
        }
        assert_eq!(
            (*next).borrow().prev.as_ref().unwrap().as_ptr(),
            Rc::downgrade(&list.head).as_ptr()
        );
    }

    #[test]
    fn test_push_back() {
        let mut list = LinkList::new();
        list.push_back(1);
        list.push_back(2);
        list.push_back(3);
        check_list(list, vec![1, 2, 3]);
    }

    #[test]
    fn test_push_front() {
        let mut list = LinkList::new();
        list.push_front(3);
        list.push_front(2);
        list.push_front(1);
        check_list(list, vec![1, 2, 3]);
    }

    #[test]
    fn test_push() {
        let mut list = LinkList::new();
        list.push_front(2);
        list.push_back(3);
        list.push_back(4);
        list.push_front(1);
        check_list(list, vec![1, 2, 3, 4]);
    }
}