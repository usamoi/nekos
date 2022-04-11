use crate::prelude::*;
use arrayvec::ArrayVec;

pub fn integer_partition(r: Segment<usize>) -> ArrayVec<(usize, u8), { usize::BITS as usize * 2 }> {
    fn lowbit(x: usize) -> usize {
        x & x.wrapping_neg()
    }
    let mut ans = ArrayVec::new();
    if r.is_empty() {
        return ans;
    }
    if r == Segment::new(0, None).unwrap() {
        let height = (usize::BITS - 1) as u8;
        ans.push((0usize, height));
        ans.push((1usize << height, height));
        return ans;
    }
    let end = r.wrapping_end();
    let mut start = r.start();
    if start == 0 {
        let height = lowbit(end).log2() as u8;
        ans.push((start, height));
        start = start.wrapping_add(1usize << height);
    }
    while start != end {
        let guess = start.wrapping_add(lowbit(start));
        if end != 0 && (guess == 0 || guess > end) {
            break;
        }
        ans.push((start, lowbit(start).log2() as u8));
        start = start.wrapping_add(lowbit(start));
    }
    let mut pow = lowbit(start) >> 1;
    while pow != 0 {
        if end & pow != 0 {
            ans.push((start, pow.log2() as u8));
            start = start.wrapping_add(pow);
        }
        pow >>= 1;
    }
    ans
}

#[cfg(test)]
#[test_case]
fn integer_partition_test() {
    fn test(r: Segment<usize>) {
        let mut iter = integer_partition(r).into_iter();
        if r.is_empty() {
            assert_eq!(iter.next(), None);
            return;
        }
        let mut last = iter.next().unwrap();
        assert_eq!(last.0, r.start());
        for (addr, height) in iter {
            assert_eq!(last.0.wrapping_add(1usize << last.1), addr);
            last = (addr, height);
        }
        assert_eq!(last.0.wrapping_add(1usize << last.1), r.wrapping_end());
    }
    test(Segment::new(0, None).unwrap());
    test(Segment::new(usize::MAX, None).unwrap());
    test(by_points(0, 114514).unwrap());
    test(by_points(0, 1919810).unwrap());
    test(Segment::new(114514, None).unwrap());
    test(Segment::new(1919810, None).unwrap());
    test(by_points(114514, 1919810).unwrap());
    test(by_points(666, 888).unwrap());
    test(by_size(777, 1).unwrap());
}
