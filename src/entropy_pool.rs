extern crate rand;

use std::iter;
use std::mem;
use std::ptr;

use rand::Rng;

#[derive(Debug)]
pub struct EntropyPool<R> {
    pub rng: R,
    pub v: Vec<u8>,
    pub i: usize,
}

impl<R: Rng> EntropyPool<R> {

    pub fn new(rng: R, size: usize) -> EntropyPool<R> {
        EntropyPool {
            rng: rng,
            v: Vec::with_capacity(4 * size),
            i: 0
        }
    }

    pub fn randomize(&mut self) {
        self.rng.fill_bytes(&mut self.v[..self.i]);
        self.i = 0;
    }

    fn reserve(&mut self, n: usize) {
        let j = self.i + n;
        let l = self.v.len();
        if l < j {
            // Grow exponentially so we don't have to constantly call the
            // random number generator.
            self.v.extend(iter::repeat(0).take((j - l) + l));
            self.rng.fill_bytes(&mut self.v[self.i..]);
        }
    }
}

impl<R: Rng> Rng for EntropyPool<R> {
    #[inline]
    fn next_u32(&mut self) -> u32 {
        let width = mem::size_of::<u32>();
        self.reserve(width);
        let ptr = self.v[self.i..].as_ptr();
        self.i += width;
        let mut result = 0;
        unsafe {
            ptr::copy_nonoverlapping(ptr,
                                     &mut result as *mut u32 as *mut u8,
                                     width);
        }
        result
    }

    #[inline]
    fn next_u64(&mut self) -> u64 {
        let width = mem::size_of::<u64>();
        self.reserve(width);
        let ptr = self.v[self.i..].as_ptr();
        self.i += width;
        let mut result = 0;
        unsafe {
            ptr::copy_nonoverlapping(ptr,
                                     &mut result as *mut u64 as *mut u8,
                                     width);
        }
        result
    }

    #[inline]
    fn fill_bytes(&mut self, dest: &mut [u8]) {
        let i = self.i;
        let l = dest.len();
        self.reserve(l);
        let src_ptr = self.v[i..].as_ptr();
        let dest_ptr = dest.as_mut_ptr();
        unsafe {
            ptr::copy_nonoverlapping(src_ptr,
                                     dest_ptr,
                                     l);
        }
        self.i += l;
    }
}
