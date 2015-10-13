use num::traits::NumCast;
use std::cmp;
use std::marker::PhantomData;
use std::mem;
use std::ops::{Div, Rem, Sub};
use std::ptr;

pub trait Shrinker {
    fn new(&[u8]) -> Self;
    fn shrink(&mut self, usize, &mut [u8]) -> bool;
}

#[derive(Debug)]
pub struct BlockShrinker<S> {
    block_size: usize,
    block_index: usize,
    i: usize,
    shrinker: S,
}

fn shrink_usize(n: usize, k: usize) -> usize {
    if n > 2 + k {
        n / 2 + 1
    } else {
        n - 1
    }
}

impl <S: Shrinker>Shrinker for BlockShrinker<S> {

    fn new(pool: &[u8]) -> BlockShrinker<S> {
        let l = pool.len();
        BlockShrinker {
            block_size: l,
            block_index: 0,
            i: 0,
            shrinker: S::new(pool),
        }
    }

    fn shrink(&mut self, size: usize, pool: &mut [u8]) -> bool {
        let mut i;
        let mut i_max;
        loop {
            if self.block_size == 0 {
                return false;
            }
            loop {
                i = self.block_size * self.block_index;
                i_max = cmp::min(pool.len(), i + self.block_size);
                if i >= pool.len() ||
                    self.shrinker.shrink(size, &mut pool[i..i_max]) {
                    break;
                }
                self.block_index += 1
            }
            if i < pool.len() {
                break;
            }
            self.block_index = 0;
            self.block_size  = shrink_usize(self.block_size, 0);
        }
        self.i = i;
        self.block_index += 1;
        return true;
    }
}

#[derive(Debug)]
pub struct ZeroOut;

impl Shrinker for ZeroOut {

    fn new(_: &[u8]) -> ZeroOut {
        ZeroOut
    }

    fn shrink(&mut self, _size: usize, pool: &mut [u8]) -> bool {
        if pool.iter().all(|&w| w == 0) {
            return false;
        }
        for ptr in pool.iter_mut() {
            *ptr = 0;
        }
        true
    }
}

trait AsBytes where Self: Sized {
    fn read(&[u8], usize) -> Option<Self>;
    fn write(self, &mut [u8], usize);
}

macro_rules! impl_as_bytes {
    ($ty: ty) => {
        impl AsBytes for $ty {
            fn read(buf: &[u8], i: usize) -> Option<Self> {
                if buf.len() - i < mem::size_of::<Self>() {
                    return None;
                }
                let mut x = 0;
                unsafe {
                    ptr::copy_nonoverlapping(buf[i..].as_ptr(),
                                             &mut x as *mut Self as *mut u8,
                                             mem::size_of::<Self>());
                }
                Some(x)
            }

            fn write(self, buf: &mut [u8], i: usize) {
                if buf.len() - i < mem::size_of::<Self>() {
                    return;
                }
                let mut _self = self;
                unsafe {
                    ptr::copy_nonoverlapping(&mut _self as *const Self as *const u8,
                                             buf[i..].as_mut_ptr(),
                                             mem::size_of::<Self>());
                }
            }
        }
    }
}

impl_as_bytes!(u8);
impl_as_bytes!(u32);
impl_as_bytes!(u64);

#[derive(Debug)]
pub struct ModuloSize<T> {
    phantom: PhantomData<T>,
}

impl <T: AsBytes + Eq + Rem<Output = T> + NumCast>Shrinker for ModuloSize<T> {
    fn new(_: &[u8]) -> ModuloSize<T> {
        let width = mem::size_of::<T>();
        if width > mem::size_of::<u64>() {
            panic!("ModuloSize<T>: mem::size_of::<T> too large");
        }
        ModuloSize {
            phantom: PhantomData,
        }
    }

    fn shrink(&mut self, size: usize, pool: &mut [u8]) -> bool {
        let mut changed = false;
        let mut i = 0;
        let cast = |x| {NumCast::from(x).unwrap()};
        while let Some(w) = T::read(&pool, i) {
            let x = w.rem(cast(size));
            if x != cast(0) {
                changed = true;
                T::write(x, pool, i);
            }
            i += mem::size_of::<T>();
        }
        changed
    }
}

#[derive(Debug)]
pub struct DivShrinker<T> {
    i: usize,
    div: u8,
    phantom: PhantomData<T>,
}


impl <T: AsBytes + Eq + NumCast + Ord + Div<Output = T>>Shrinker for DivShrinker<T> {
    fn new(_pool: &[u8]) -> DivShrinker<T> {
        let width = mem::size_of::<T>();
        if width > mem::size_of::<u64>() {
            panic!("SubShrinker<T>: mem::size_of::<T> too large");
        }
        DivShrinker {
            i: 0,
            div: 255,
            phantom: PhantomData,
        }
    }

    fn shrink(&mut self, size: usize, pool: &mut [u8]) -> bool {

        let cast = |x| {NumCast::from(x).unwrap()};

        if self.div == 0 {
            return false;
        }

        let mut i = self.i;
        let div = cast(self.div);
        while let Some(w) = T::read(&pool, i) {
            if w != cast(0) && w > div {
                T::write(w / div, pool, i);
                self.i = i + 1;
                return true;
            }
            i += 1;
        }
        self.i = 0;
        self.div = shrink_usize(self.div as usize, 16) as u8;
        if self.div == 0 {
            false
        } else {
            self.shrink(size, pool)
        }
    }
}

#[derive(Debug)]
pub struct SubShrinker<T> {
    i: usize,
    sub: u8,
    phantom: PhantomData<T>,
}

impl <T: AsBytes + Eq + NumCast + Ord + Sub<Output = T>>Shrinker for SubShrinker<T> {
    fn new(_pool: &[u8]) -> SubShrinker<T> {
        let width = mem::size_of::<T>();
        if width > mem::size_of::<u64>() {
            panic!("SubShrinker<T>: mem::size_of::<T> too large");
        }
        SubShrinker {
            i: 0,
            sub: 255,
            phantom: PhantomData,
        }
    }

    fn shrink(&mut self, size: usize, pool: &mut [u8]) -> bool {

        let cast = |x| {NumCast::from(x).unwrap()};

        if self.sub == 0 {
            return false;
        }

        let mut i = self.i;
        let sub = cast(self.sub);
        while let Some(w) = T::read(&pool, i) {
            if w != cast(0) && w > sub {
                T::write(w - sub, pool, i);
                self.i = i + 1;
                return true;
            }
            i += 1;
        }
        self.i = 0;
        self.sub = shrink_usize(self.sub as usize, 16) as u8;
        if self.sub == 0 {
            false
        } else {
            self.shrink(size, pool)
        }
    }
}

#[derive(Debug)]
enum StdStrategy {
    Zero,
    Mod64,
    Div64,
    Sub64,
    Mod32,
    Div32,
    Sub32,
    Mod8,
    Div8,
    Sub8,
}

#[derive(Debug)]
pub enum StdShrinkerBody {
    Zero(BlockShrinker<ZeroOut>),
    Mod64(BlockShrinker<ModuloSize<u64>>),
    Sub64(SubShrinker<u64>),
    Div64(DivShrinker<u64>),
    Mod32(BlockShrinker<ModuloSize<u32>>),
    Sub32(SubShrinker<u32>),
    Div32(DivShrinker<u32>),
    Mod8(BlockShrinker<ModuloSize<u8>>),
    Sub8(SubShrinker<u8>),
    Div8(DivShrinker<u8>),
}

#[derive(Debug)]
pub struct StdShrinker {
    body: StdShrinkerBody,
    pass: u8,
}

impl Shrinker for StdShrinker {
    fn new(pool: &[u8]) -> StdShrinker {
        StdShrinker {
            body: StdShrinkerBody::Zero(BlockShrinker::new(pool)),
            pass: 0,
        }
    }

    fn shrink(&mut self, size: usize, pool: &mut [u8]) -> bool {

        macro_rules! match_strategy {
            ($strategy: ident) => {
                &mut StdShrinkerBody::$strategy(ref mut shrinker)
            }
        }

        let strategy;

        macro_rules! apply_strategy {
            ($shrinker: ident, $strategy: ident) => {{
                if $shrinker.shrink(size, pool) {
                    return true;
                }
                strategy = StdStrategy::$strategy;
            }}
        }

        match &mut self.body {
            &mut StdShrinkerBody::Zero(ref mut shrinker) =>
                apply_strategy!(shrinker, Zero),
            &mut StdShrinkerBody::Mod64(ref mut shrinker) =>
                apply_strategy!(shrinker, Mod64),
            &mut StdShrinkerBody::Div64(ref mut shrinker) =>
                apply_strategy!(shrinker, Div64),
            &mut StdShrinkerBody::Sub64(ref mut shrinker) =>
                apply_strategy!(shrinker, Sub64),
            &mut StdShrinkerBody::Mod32(ref mut shrinker) =>
                apply_strategy!(shrinker, Mod32),
            &mut StdShrinkerBody::Div32(ref mut shrinker) =>
                apply_strategy!(shrinker, Div32),
            &mut StdShrinkerBody::Sub32(ref mut shrinker) =>
                apply_strategy!(shrinker, Sub32),
            &mut StdShrinkerBody::Mod8(ref mut shrinker) =>
                apply_strategy!(shrinker, Mod8),
            &mut StdShrinkerBody::Div8(ref mut shrinker) =>
                apply_strategy!(shrinker, Div8),
            &mut StdShrinkerBody::Sub8(ref mut shrinker) if self.pass >= 4 => {
                    return shrinker.shrink(size, pool);
            }
            &mut StdShrinkerBody::Sub8(ref mut shrinker) =>
                apply_strategy!(shrinker, Sub8),
        }

        macro_rules! switch_strategy {
            ($next: ident, $next_shrinker: ident) => {
                self.body = StdShrinkerBody::$next($next_shrinker::new(pool));
            }
        }

        match strategy {
            StdStrategy::Zero  => switch_strategy!(Mod64, BlockShrinker),
            StdStrategy::Mod64 => switch_strategy!(Div64, DivShrinker),
            StdStrategy::Div64 => switch_strategy!(Sub64, SubShrinker),
            StdStrategy::Sub64 => switch_strategy!(Mod32, BlockShrinker),
            StdStrategy::Mod32 => switch_strategy!(Div32, DivShrinker),
            StdStrategy::Div32 => switch_strategy!(Sub32, SubShrinker),
            StdStrategy::Sub32 => switch_strategy!(Mod8,  BlockShrinker),
            StdStrategy::Mod8  => switch_strategy!(Div8,  DivShrinker),
            StdStrategy::Div8  => switch_strategy!(Sub8,  SubShrinker),
            StdStrategy::Sub8  => {
                self.pass += 1;
                switch_strategy!(Zero,  BlockShrinker)
            }
        }
        self.shrink(size, pool)
    }
}
