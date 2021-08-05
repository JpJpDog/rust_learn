use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

pub type RcNode<T> = Rc<RefCell<LinkNode<T>>>;

pub type WeakNode<T> = Weak<RefCell<LinkNode<T>>>;

pub struct LinkNode<T> {
    pub val: Option<T>,
    next: Option<RcNode<T>>,
    prev: Option<WeakNode<T>>,
}

impl<T> LinkNode<T> {
    #[inline]
    fn wrap(x: T) -> RcNode<T> {
        Rc::new(RefCell::new(Self {
            val: Some(x),
            next: None,
            prev: None,
        }))
    }
}

#[derive(Clone)]
pub struct LinkList<T> {
    len: usize,
    head: RcNode<T>,
    tail: WeakNode<T>,
}

impl<T> LinkList<T> {
    fn insert_after(at: RcNode<T>, item: RcNode<T>) {
        let next = (*at).borrow_mut().next.replace(Rc::clone(&item)).unwrap();
        (*next).borrow_mut().prev = Some(Rc::downgrade(&item));
        {
            let mut item = (*item).borrow_mut();
            item.next = Some(next);
            item.prev = Some(Rc::downgrade(&at));
        }
    }

    fn remove(at: RcNode<T>) -> RcNode<T> {
        let prev = Weak::upgrade(&(*at).borrow_mut().prev.take().unwrap()).unwrap();
        let next = (*at).borrow_mut().next.take().unwrap();
        (*next).borrow_mut().prev = Some(Rc::downgrade(&prev));
        (*prev).borrow_mut().next = Some(next);
        at
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

    pub fn push_back_node(&mut self, item: RcNode<T>) -> WeakNode<T> {
        self.len += 1;
        let item_weak = Rc::downgrade(&item);
        let at = Weak::upgrade(&self.tail).unwrap();
        let at = Weak::upgrade((*at).borrow_mut().prev.as_mut().unwrap()).unwrap();
        Self::insert_after(at, item);
        item_weak
    }

    pub fn push_front_node(&mut self, item: RcNode<T>) -> WeakNode<T> {
        self.len += 1;
        let item_weak = Rc::downgrade(&item);
        Self::insert_after(Rc::clone(&self.head), item);
        item_weak
    }

    pub fn push_back(&mut self, item: T) -> WeakNode<T> {
        Self::push_back_node(self, LinkNode::wrap(item))
    }

    pub fn push_front(&mut self, item: T) -> WeakNode<T> {
        Self::push_front_node(self, LinkNode::wrap(item))
    }

    pub fn pop_back_node(&mut self) -> Option<RcNode<T>> {
        if self.len == 0 {
            return None;
        }
        self.len -= 1;
        let at = Weak::upgrade(&self.tail).unwrap();
        let at = Weak::upgrade((*at).borrow_mut().prev.as_mut().unwrap()).unwrap();
        Some(Self::remove(at))
    }

    pub fn pop_front_node(&mut self) -> Option<RcNode<T>> {
        if self.len == 0 {
            return None;
        }
        self.len -= 1;
        let at = (*self.head).borrow_mut().next.clone().unwrap();
        Some(Self::remove(at))
    }

    pub fn pop_back(&mut self) -> Option<T> {
        if let Some(back) = self.pop_back_node() {
            Some(Rc::try_unwrap(back).ok().unwrap().into_inner().val.unwrap())
        } else {
            None
        }
    }

    pub fn pop_front(&mut self) -> Option<T> {
        if let Some(front) = self.pop_front_node() {
            Some(
                Rc::try_unwrap(front)
                    .ok()
                    .unwrap()
                    .into_inner()
                    .val
                    .unwrap(),
            )
        } else {
            None
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn check_list(list: &LinkList<i32>, v: Vec<i32>) {
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
        check_list(&list, vec![1, 2, 3]);
    }

    #[test]
    fn test_push_front() {
        let mut list = LinkList::new();
        list.push_front(3);
        list.push_front(2);
        list.push_front(1);
        check_list(&list, vec![1, 2, 3]);
    }

    #[test]
    fn test_push() {
        let mut list = LinkList::new();
        list.push_front(2);
        list.push_back(3);
        list.push_back(4);
        list.push_front(1);
        check_list(&list, vec![1, 2, 3, 4]);
    }

    fn init_list() -> LinkList<i32> {
        let mut list = LinkList::new();
        list.push_back(1);
        list.push_back(2);
        list.push_back(3);
        list
    }

    #[test]
    fn test_pop_back() {
        let mut list = init_list();
        assert_eq!(list.pop_back().unwrap(), 3);
        check_list(&list, vec![1, 2]);
        assert_eq!(list.pop_back().unwrap(), 2);
        check_list(&list, vec![1]);
        assert_eq!(list.pop_back().unwrap(), 1);
        check_list(&list, vec![]);
        assert!(list.pop_back().is_none());
    }

    #[test]
    fn test_pop_front() {
        let mut list = init_list();
        assert_eq!(list.pop_front().unwrap(), 1);
        check_list(&list, vec![2, 3]);
        assert_eq!(list.pop_front().unwrap(), 2);
        check_list(&list, vec![3]);
        assert_eq!(list.pop_front().unwrap(), 3);
        check_list(&list, vec![]);
        assert!(list.pop_back().is_none());
    }
}
