use std::{collections::VecDeque, net::SocketAddr};

use crate::{kademlia::IdType, msg::NearNode};

pub type TestFun = dyn Fn(IdType, Vec<SocketAddr>) -> Vec<bool> + Send;

fn log2_floor(mut x: u32) -> u32 {
    if x == 0 {
        return 0;
    }
    x >>= 1;
    let mut ret = 0;
    while x != 0 {
        x >>= 1;
        ret += 1;
    }
    ret
}

pub struct KBuckets {
    my_id: IdType,
    bucket_k: usize,
    data: Vec<VecDeque<(IdType, SocketAddr)>>,
    test: Box<TestFun>,
}

impl KBuckets {
    pub fn new(my_id: IdType, bucket_k: usize, test: Box<TestFun>) -> Self {
        Self {
            my_id,
            bucket_k,
            data: vec![VecDeque::new(); std::mem::size_of::<IdType>() * 8],
            test,
        }
    }

    pub fn add(&mut self, adds: Vec<(IdType, SocketAddr)>) {
        let mut test_idx = Vec::new();
        let mut test_addr = Vec::new();
        for (id, addr) in adds {
            let dist = (id ^ self.my_id) as u32;
            let idx = log2_floor(dist) as usize;
            self.data[idx].push_back((id, addr));
            if self.data[idx].len() == self.bucket_k + 1 {
                test_idx.push(idx);
                test_addr.push(addr);
            }
        }
        let test_results = (self.test)(self.my_id, test_addr);
        let mut idx = 0;
        while idx < test_results.len() {
            let i = test_idx[idx];
            if test_results[idx] {
                self.data[i].pop_back().unwrap();
                let test = self.data[i].pop_front().unwrap();
                self.data[i].push_back(test);
            } else {
                self.data[i].pop_front();
            }
            idx += 1;
        }
    }

    pub fn find_k_nearest(&self, target: IdType) -> Vec<NearNode> {
        let dist = (self.my_id ^ target) as usize;
        let s_idx = log2_floor(dist as u32) as usize;
        let mut idx = s_idx;
        let mut ret = Vec::new();
        let mut flag = false;
        while ret.len() < self.bucket_k {
            let bucket = &self.data[idx];
            let mut i = 0;
            while i < bucket.len() && ret.len() < self.bucket_k as usize {
                ret.push(NearNode {
                    dist: bucket[i].0 ^ target,
                    id: bucket[i].0,
                    addr: bucket[i].1,
                });
                i += 1;
            }
            if idx == 0 {
                idx = s_idx;
                flag = true;
            }
            if idx == self.data.len() - 1 {
                break;
            }
            if flag {
                idx += 1;
            } else {
                idx -= 1;
            }
        }
        ret
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::net::IpAddr;
    use std::str::FromStr;

    #[test]
    fn k_buckets() {
        let test = |_id: IdType, addrs: Vec<SocketAddr>| vec![true; addrs.len()];
        let mut kbuckets = KBuckets::new(0b101000, 3, Box::new(test));
        let addr = SocketAddr::new(IpAddr::from_str("127.0.0.1").unwrap(), 12200);
        let adds = vec![
            (0b101001, addr.clone()),
            (0b101010, addr.clone()),
            (0b101100, addr.clone()),
            (0b100111, addr.clone()),
            (0b100000, addr.clone()),
            (0b110000, addr.clone()),
            (0b111111, addr.clone()),
            (0b010001, addr.clone()),
            (0b000001, addr.clone()),
            (0b000000, addr.clone()),
        ];
        kbuckets.add(adds);

        assert_eq!(kbuckets.data[0].len(), 1);
        assert_eq!(kbuckets.data[1].len(), 1);
        assert_eq!(kbuckets.data[2].len(), 1);
        assert_eq!(kbuckets.data[3].len(), 2);
        assert_eq!(kbuckets.data[4].len(), 2);
        assert_eq!(kbuckets.data[5].len(), 3);

        let ret = kbuckets.find_k_nearest(0b101010);
        assert_eq!(ret.len(), 3);
        assert_eq!(ret[0].id, 0b101010);
        assert_eq!(ret[1].id, 0b101001);
        assert_eq!(ret[2].id, 0b101100);

        let ret = kbuckets.find_k_nearest(0b101001);
        assert_eq!(ret.len(), 3);
        assert_eq!(ret[0].id, 0b101001);
        assert_eq!(ret[1].id, 0b101010);
        assert_eq!(ret[2].id, 0b101100);

        let ret = kbuckets.find_k_nearest(0b110001);
        assert_eq!(ret.len(), 3);
        assert_eq!(ret[0].id, 0b110000);
        assert_eq!(ret[1].id, 0b111111);
        assert_eq!(ret[2].id, 0b100111);
    }
    // K 3
    // id 101000

    // 101001
    // 101010
    // 101100
    // 100111 100000
    // 110000 111111
    // 000000 010001 000001
}
