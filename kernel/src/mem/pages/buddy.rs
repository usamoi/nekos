use crate::mem::utils::integer_partition;
use crate::prelude::*;
use alloc::boxed::Box;
use arrayvec::ArrayVec;
use core::ptr::null_mut;

#[derive(Debug)]
pub enum BuddyError {
    ZeroSize,
    OutOfBounds,
}

#[derive(Debug)]
struct Raw {
    c: u8,
    left: Box<Node>,
    right: Box<Node>,
}

/* 0 <= height <= usize::BITS - 1 */
/* `usize::BITS as height` fallbacks to 2 nodes */

type Node = Result<Raw /* iff height >= 1 */, bool>;

fn merge(height: u8, left: Box<Node>, right: Box<Node>) -> Node {
    assert!(height <= (usize::BITS - 2) as u8);
    match (&*left, &*right) {
        (Err(false), Err(false)) => Err(false),
        (Err(true), Err(true)) => Err(true),
        (Ok(l), Ok(r)) => Ok(Raw {
            c: u8::max(l.c, r.c),
            left,
            right,
        }),
        (_, Err(false)) | (Err(false), _) => Ok(Raw {
            c: height,
            left,
            right,
        }),
        (Ok(x), Err(true)) | (Err(true), Ok(x)) => Ok(Raw {
            c: x.c,
            left,
            right,
        }),
    }
}

fn dfs_get(height: u8, mut u: &Node, addr: usize, query_height: u8) -> Option<bool> {
    assert!(height <= (usize::BITS - 1) as u8 && query_height <= height);
    assert_eq!(addr & ((1usize << query_height) - 1), 0, "bad address");
    for i in (query_height as u8 + 1..=height).rev() {
        if let Err(color) = u {
            return Some(*color);
        }
        let raw = u.as_ref().unwrap();
        if addr & (1usize << (i - 1)) == 0 {
            u = &raw.left;
        } else {
            u = &raw.right;
        }
    }
    if let Err(color) = u {
        Some(*color)
    } else {
        None
    }
}

fn dfs_set(height: u8, mut u: &mut Node, addr: usize, query_height: u8, f: impl FnOnce(&mut Node)) {
    assert!(height <= (usize::BITS - 1) as u8 && query_height <= height);
    assert_eq!(addr & ((1usize << query_height) - 1), 0, "bad address");
    let mut path: [*mut Node; usize::BITS as usize] = [null_mut(); usize::BITS as usize];
    for i in (query_height + 1..=height).rev() {
        path[i as usize] = u as *mut _;
        if let Err(color) = u {
            *u = Ok(Raw {
                c: i,
                left: Box::new(Err(*color)),
                right: Box::new(Err(*color)),
            });
        }
        let raw = u.as_mut().unwrap();
        if addr & (1usize << (i - 1)) == 0 {
            u = &mut raw.left;
        } else {
            u = &mut raw.right;
        }
    }
    f(u);
    for i in query_height + 1..=height {
        let u = unsafe { &mut *path[i as usize] };
        let raw = core::mem::replace(u, Err(false)).unwrap();
        *u = merge(i - 1, raw.left, raw.right);
    }
}

fn dfs_find(height: u8, mut u: &Node, query_height: u8) -> Option<usize> {
    fn continuos_of(height: u8, u: &Node) -> usize {
        assert!(height <= (usize::BITS - 1) as u8);
        match u {
            Ok(raw) => 1usize << raw.c,
            Err(color) => (!color as usize) << height,
        }
    }
    assert!(height <= (usize::BITS - 1) as u8);
    if continuos_of(height, u) < (1usize << query_height) {
        return None;
    }
    let mut addr = 0usize;
    for i in (query_height + 1..=height).rev() {
        if let Err(color) = u {
            assert!(!*color);
            return Some(addr);
        }
        let raw = u.as_ref().unwrap();
        if continuos_of(i, &raw.right) >= 1usize << query_height {
            u = &raw.right;
            addr |= 1 << (i - 1);
        } else {
            u = &raw.left;
        }
    }
    assert!(!*u.as_ref().unwrap_err());
    Some(addr)
}

pub struct Buddy {
    segment: Segment<usize>,
    list: ArrayVec<(usize, u8, Node), { usize::BITS as usize * 2 }>,
}

impl Buddy {
    pub fn new(segment: Segment<usize>) -> Result<Buddy, BuddyError> {
        use BuddyError::*;
        if segment.is_empty() {
            return Err(ZeroSize);
        }
        Ok(Buddy {
            segment,
            list: integer_partition(segment)
                .into_iter()
                .map(|(a, b)| (a, b, Err(false)))
                .collect(),
        })
    }
    #[allow(dead_code)]
    pub fn alloc(&mut self, size: usize) -> Result<usize, BuddyError> {
        use BuddyError::*;
        let addr = self.find(size)?;
        self.set(by_size(addr, size).ok_or(OutOfBounds)?, true)?;
        Ok(addr)
    }
    #[allow(dead_code)]
    pub fn dealloc(&mut self, addr: usize, size: usize) -> Result<(), BuddyError> {
        use BuddyError::*;
        self.set(by_size(addr, size).ok_or(OutOfBounds)?, false)
    }
    pub fn find(&self, size: usize) -> Result<usize, BuddyError> {
        use BuddyError::*;
        if size == 0 {
            return Err(ZeroSize);
        }
        let addr = 'outer: {
            let h = size.checked_next_power_of_two().ok_or(OutOfBounds)?;
            for (xaddr, xheight, xu) in self.list.iter() {
                if let Some(xpos) = dfs_find(*xheight, xu, h.log2() as u8) {
                    break 'outer *xaddr + xpos;
                }
            }
            return Err(OutOfBounds);
        };
        Ok(addr)
    }
    #[allow(dead_code)]
    pub fn get(&mut self, segment: Segment<usize>) -> Result<Option<bool>, BuddyError> {
        use BuddyError::*;
        if segment.is_empty() {
            return Err(ZeroSize);
        }
        if !self.segment.contains(segment) {
            return Err(OutOfBounds);
        }
        let mut iter = self.list.iter_mut();
        let mut this = iter.next().unwrap();
        let mut ans = None;
        for (addr, height) in integer_partition(segment).into_iter() {
            loop {
                let (xaddr, xheight, _) = this;
                assert!(*xaddr <= addr);
                if by_size(*xaddr, 1usize << *xheight)
                    .unwrap()
                    .contains(by_size(addr, 1usize << height).unwrap())
                {
                    break;
                }
                this = iter.next().unwrap();
            }
            let (xaddr, xheight, xu) = this;
            match dfs_get(*xheight, xu, addr - *xaddr, height) {
                Some(false) if ans != Some(true) => ans = Some(false),
                Some(true) if ans != Some(false) => ans = Some(true),
                _ => return Ok(None),
            }
        }
        assert!(ans.is_some());
        Ok(ans)
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
                let (xaddr, xheight, _) = this;
                assert!(*xaddr <= addr);
                if by_size(*xaddr, 1usize << *xheight)
                    .unwrap()
                    .contains(by_size(addr, 1usize << height).unwrap())
                {
                    break;
                }
                this = iter.next().unwrap();
            }
            let (xaddr, xheight, xu) = this;
            assert_eq!(dfs_get(*xheight, xu, addr - *xaddr, height), Some(!val));
            dfs_set(*xheight, xu, addr - *xaddr, height, |u| *u = Err(val));
        }
        Ok(())
    }
}

#[cfg(test)]
#[test_case]
fn alloc_dealloc() {
    let mut s = Buddy::new(Segment::new(0, None).unwrap()).unwrap();
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
    let mut s = Buddy::new(Segment::new(0, None).unwrap()).unwrap();
    let mut saves = Vec::new();
    for _ in 0..10 {
        saves.push(s.alloc(114514).unwrap());
    }
    for _ in 0..10 {
        s.dealloc(saves.pop().unwrap(), 114514).unwrap();
    }
}

#[cfg(test)]
#[test_case]
fn set() {
    let mut s = Buddy::new(by_points(114514, 1919810).unwrap()).unwrap();
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
