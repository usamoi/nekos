use crate::mem::utils::integer_partition;
use crate::prelude::*;
use arrayvec::ArrayVec;

#[derive(Debug)]
pub enum BuddyError {
    ZeroSize,
    OutOfBounds,
}

type Node = i8;

const TOTAL_FALSE: Node = -1;
const TOTAL_TRUE: Node = -2;

type Nodes<'a> = (u8, &'a mut [Node]);

fn dfs_set(nodes: &mut Nodes, addr: usize, query_height: u8, f: impl FnOnce(&mut Node)) {
    assert!(nodes.0 <= (usize::BITS - 1) as u8 && query_height <= nodes.0);
    assert_eq!(addr & ((1usize << query_height) - 1), 0, "bad address");
    let mut index = 1usize;
    for i in (query_height + 1..=nodes.0).rev() {
        if let TOTAL_FALSE = nodes.1[index] {
            nodes.1[index << 1 | 0] = TOTAL_FALSE;
            nodes.1[index << 1 | 1] = TOTAL_FALSE;
        }
        if let TOTAL_TRUE = nodes.1[index] {
            nodes.1[index << 1 | 0] = TOTAL_TRUE;
            nodes.1[index << 1 | 1] = TOTAL_TRUE;
        }
        if addr & (1usize << (i - 1)) == 0 {
            index = index << 1 | 0;
        } else {
            index = index << 1 | 1;
        }
    }
    f(&mut nodes.1[index]);
    for _ in query_height + 1..=nodes.0 {
        index >>= 1;
        match (nodes.1[index << 1 | 0], nodes.1[index << 1 | 1]) {
            (TOTAL_FALSE, TOTAL_FALSE) => nodes.1[index] = TOTAL_FALSE,
            (TOTAL_TRUE, TOTAL_TRUE) => nodes.1[index] = TOTAL_TRUE,
            (l, r) if (l >= 0 && r >= 0) => nodes.1[index] = i8::max(l, r),
            (_, TOTAL_FALSE) => nodes.1[index] = nodes.0 as i8 - 1 - (index).log2() as i8,
            (TOTAL_FALSE, _) => nodes.1[index] = nodes.0 as i8 - 1 - (index).log2() as i8,
            (x, TOTAL_TRUE) if x >= 0 => nodes.1[index] = x,
            (TOTAL_TRUE, x) if x >= 0 => nodes.1[index] = x,
            _ => unreachable!(),
        }
    }
}

fn dfs_get(nodes: &Nodes, addr: usize, query_height: u8) -> Option<bool> {
    assert!(nodes.0 <= (usize::BITS - 1) as u8 && query_height <= nodes.0);
    assert_eq!(addr & ((1usize << query_height) - 1), 0, "bad address");
    let mut index = 1usize;
    for i in (query_height + 1..=nodes.0).rev() {
        if let TOTAL_FALSE = nodes.1[index] {
            return Some(false);
        }
        if let TOTAL_TRUE = nodes.1[index] {
            return Some(true);
        }
        if addr & (1usize << (i - 1)) == 0 {
            index = index << 1 | 0;
        } else {
            index = index << 1 | 1;
        }
    }
    if let TOTAL_FALSE = nodes.1[index] {
        return Some(false);
    }
    if let TOTAL_TRUE = nodes.1[index] {
        return Some(true);
    }
    None
}

fn dfs_find(nodes: &Nodes, query_height: u8) -> Option<usize> {
    fn continuos_of(nodes: &Nodes, i: usize) -> usize {
        match nodes.1[i] {
            TOTAL_FALSE => 1usize << (nodes.0 - i.log2() as u8),
            TOTAL_TRUE => 0,
            x => 1usize << x,
        }
    }
    assert!(nodes.0 <= (usize::BITS - 1) as u8);
    if continuos_of(nodes, 1) < (1usize << query_height) {
        return None;
    }
    let mut addr = 0usize;
    let mut index = 1usize;
    for i in (query_height + 1..=nodes.0).rev() {
        assert_ne!(TOTAL_TRUE, nodes.1[index]);
        if let TOTAL_FALSE = nodes.1[index] {
            return Some(addr);
        }
        let t = 1usize << query_height;
        let l = continuos_of(nodes, index << 1 | 0);
        let r = continuos_of(nodes, index << 1 | 1);
        if r < t || (t <= l && l <= r) {
            index = index << 1 | 0;
        } else {
            index = index << 1 | 1;
            addr |= 1 << (i - 1);
        }
    }
    assert_eq!(nodes.1[index], TOTAL_FALSE);
    Some(addr)
}

pub struct Buddy<'a> {
    segment: Segment<usize>,
    list: ArrayVec<(usize, Nodes<'a>), { usize::BITS as usize * 2 }>,
}

impl<'a> Buddy<'a> {
    pub fn new(segment: Segment<usize>, buffer: &'a mut [i8]) -> Result<Buddy<'a>, BuddyError> {
        use BuddyError::*;
        if segment.is_empty() {
            return Err(ZeroSize);
        }
        let len = segment.wrapping_end().wrapping_sub(segment.start());
        if len * 2 > buffer.len() {
            return Err(OutOfBounds);
        }
        let mut buffer = &mut buffer[..len * 2];
        buffer.fill(TOTAL_FALSE);
        let mut list = ArrayVec::new();
        for (addr, height) in integer_partition(segment).into_iter() {
            let (data, rest) = buffer.split_at_mut(2usize << height);
            buffer = rest;
            list.push((addr, (height, data)));
        }
        Ok(Buddy { segment, list })
    }
    pub fn alloc(&mut self, size: usize) -> Result<usize, BuddyError> {
        use BuddyError::*;
        let addr = self.find(size)?;
        self.set(by_size(addr, size).ok_or(OutOfBounds)?, true)?;
        Ok(addr)
    }
    pub fn dealloc(&mut self, addr: usize, size: usize) -> Result<(), BuddyError> {
        use BuddyError::*;
        self.set(by_size(addr, size).ok_or(OutOfBounds)?, false)?;
        Ok(())
    }
    pub fn find(&self, size: usize) -> Result<usize, BuddyError> {
        use BuddyError::*;
        if size == 0 {
            return Err(ZeroSize);
        }
        let addr = 'outer: {
            let h = size.checked_next_power_of_two().ok_or(OutOfBounds)?;
            for (xaddr, xnodes) in self.list.iter() {
                if let Some(xpos) = dfs_find(xnodes, h.log2() as u8) {
                    break 'outer *xaddr + xpos;
                }
            }
            return Err(OutOfBounds);
        };
        Ok(addr)
    }
    pub fn _get(&mut self, segment: Segment<usize>) -> Result<Option<bool>, BuddyError> {
        use BuddyError::*;
        if segment.is_empty() {
            return Err(ZeroSize);
        }
        if !self.segment.contains(segment) {
            return Err(OutOfBounds);
        }
        let mut iter = self.list.iter_mut();
        let mut this = iter.next().unwrap();
        let mut valid = None;
        for (addr, height) in integer_partition(segment).into_iter() {
            loop {
                let (xaddr, xelem) = this;
                assert!(*xaddr <= addr);
                if by_size(*xaddr, 1usize << xelem.0)
                    .unwrap()
                    .contains(by_size(addr, 1usize << height).unwrap())
                {
                    break;
                }
                this = iter.next().unwrap();
            }
            let (xaddr, xnodes) = this;
            match dfs_get(xnodes, addr - *xaddr, height) {
                Some(false) if valid != Some(true) => valid = Some(false),
                Some(true) if valid != Some(false) => valid = Some(true),
                _ => return Ok(None),
            }
        }
        assert!(valid.is_some());
        Ok(valid)
    }
    pub fn set(&mut self, segment: Segment<usize>, val: bool) -> Result<(), BuddyError> {
        use BuddyError::*;
        if segment.is_empty() {
            return Err(ZeroSize);
        }
        if !self.segment.contains(segment) {
            return Err(OutOfBounds);
        }
        let mut iter = self.list.iter_mut();
        let mut this = iter.next().unwrap();
        for (addr, height) in integer_partition(segment).into_iter() {
            loop {
                let (xaddr, xnodes) = this;
                assert!(*xaddr <= addr);
                if by_size(*xaddr, 1usize << xnodes.0)
                    .unwrap()
                    .contains(by_size(addr, 1usize << height).unwrap())
                {
                    break;
                }
                this = iter.next().unwrap();
            }
            let (xaddr, xnodes) = this;
            assert_eq!(dfs_get(xnodes, addr - *xaddr, height), Some(!val));
            dfs_set(xnodes, addr - *xaddr, height, |u| {
                *u = if val { TOTAL_TRUE } else { TOTAL_FALSE }
            });
        }
        Ok(())
    }
}

#[cfg(test)]
#[test_case]
fn alloc_dealloc() {
    let mut buffer = vec![0i8; (1145140 - 233) * 2].into_boxed_slice();
    let mut s = Buddy::new(by_points(233, 1145140).unwrap(), buffer.as_mut()).unwrap();
    for _ in 0..10 {
        let addr1 = s.alloc(114514).unwrap();
        println!("buddy_alloc = {}", addr1);
        let addr2 = s.alloc(114514).unwrap();
        println!("buddy_alloc = {}", addr2);
        println!("buddy_dealloc = {}", addr1);
        s.dealloc(addr1, 114514).unwrap();
        println!("buddy_dealloc = {}", addr2);
        s.dealloc(addr2, 114514).unwrap();
    }
}

#[cfg(test)]
#[test_case]
fn saves() {
    let mut buffer = vec![0i8; 48001 * 2].into_boxed_slice();
    let mut s = Buddy::new(by_points(0, 48000).unwrap(), buffer.as_mut()).unwrap();
    let mut saves = Vec::new();
    for _ in 0..1000 {
        saves.push(s.alloc(11).unwrap());
    }
    for _ in 0..1000 {
        s.dealloc(saves.pop().unwrap(), 11).unwrap();
    }
}

#[cfg(test)]
#[test_case]
fn set() {
    let mut buffer = vec![0i8; (1919810 - 114514) * 2].into_boxed_slice();
    let mut s = Buddy::new(by_points(114514, 1919810).unwrap(), buffer.as_mut()).unwrap();
    s.set(by_points(114514, 115000).unwrap(), true).unwrap();
    s.set(by_points(115000, 116000).unwrap(), true).unwrap();
    s.set(by_points(114555, 115110).unwrap(), false).unwrap();
    let seg = by_points(114554, 114555).unwrap();
    assert_eq!(s.get(seg).unwrap(), Some(true));
    let seg = by_points(114555, 114556).unwrap();
    assert_eq!(s.get(seg).unwrap(), Some(false));
    s.set(by_points(114555, 115110).unwrap(), true).unwrap();
    s.set(by_points(114514, 115000).unwrap(), false).unwrap();
    s.set(by_points(115000, 116000).unwrap(), false).unwrap();
    s.set(by_points(114514, 1919810).unwrap(), true).unwrap();
    s.set(by_points(114514, 1919810).unwrap(), false).unwrap();
}
