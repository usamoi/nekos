mod errors;
pub use self::errors::*;

use crate::prelude::*;
use alloc::collections::LinkedList;
use spin::Mutex;

struct SocketInner {
    head: Box<[u8; 4096]>,
    head_p: usize,
    body: LinkedList<Box<[u8; 4096]>>,
    tail: Box<[u8; 4096]>,
    tail_p: usize,
}

pub struct Socket {
    inner: Mutex<SocketInner>,
}

impl Socket {
    pub fn create() -> Arc<Socket> {
        Arc::new(Socket {
            inner: Mutex::new(SocketInner {
                head: Box::new([0; 4096]),
                head_p: 0,
                body: LinkedList::new(),
                tail: Box::new([0; 4096]),
                tail_p: 0,
            }),
        })
    }
    pub fn read(&self, buf: &mut [u8]) -> usize {
        let mut inner = self.inner.lock();
        let inner = &mut *inner;
        if buf.len() <= 4096 - inner.head_p {
            let d = buf.len();
            buf[0..d].copy_from_slice(&inner.head[inner.head_p..inner.head_p + d]);
            inner.head_p += d;
            return d;
        }
        let mut x = 4096 - inner.head_p;
        buf[0..x].copy_from_slice(&inner.head[inner.head_p..inner.head_p + x]);
        inner.head_p += x;
        //
        while let Some(page) = inner.body.pop_front() {
            let d = core::cmp::min(4096, buf.len() - x);
            buf[x..x + d].copy_from_slice(&page[0..d]);
            x += d;
            if d != 4096 {
                inner.head = page;
                inner.head_p = d;
            }
            if x == buf.len() {
                return x;
            }
        }
        //
        let d = core::cmp::min(inner.tail_p, buf.len() - x);
        buf[x..x + d].copy_from_slice(&inner.tail[0..d]);
        inner.tail_p += d;
        x += d;
        //
        x
    }
    pub fn write(&self, buf: &[u8]) {
        let mut inner = self.inner.lock();
        let inner = &mut *inner;
        if buf.len() <= 4096 - inner.tail_p {
            let d = buf.len();
            inner.tail[inner.tail_p..inner.tail_p + d].copy_from_slice(&buf[0..d]);
            return;
        }
        let mut x = 4096 - inner.tail_p;
        inner.tail[inner.tail_p..inner.tail_p + x].copy_from_slice(&buf[0..x]);
        inner.body.push_back(inner.tail.clone());
        while buf.len() - x >= 4096 {
            inner.body.push_back(Box::new(
                TryInto::<&[u8; 4096]>::try_into(&buf[x..x + 4096])
                    .unwrap()
                    .clone(),
            ));
            x += 4096;
        }
        inner.tail[0..buf.len() - x].copy_from_slice(&buf[x..]);
        inner.tail_p = buf.len() - x;
    }
}
