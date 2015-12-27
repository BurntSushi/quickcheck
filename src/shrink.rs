use std::cmp;
use std::default::Default;
use std::marker::PhantomData;
use std::mem;
use std::ptr;

pub trait Shrinker: Default {
    fn use_shrinker(&mut self, usize, &mut [u8]) -> bool;
}

#[derive(Debug)]
pub struct BlockShrinker<S> {
    recip_size: usize,
    block_index: usize,
    shrinker: S,
}

fn shrink_usize(n: usize, k: usize) -> usize {
    if n > 2 + k {
        n / 2 + 1
    } else {
        n - 1
    }
}

impl <S: Shrinker>Default for BlockShrinker<S> {
    fn default() -> BlockShrinker<S> {
        BlockShrinker {
            recip_size: 1,
            block_index: 0,
            shrinker: S::default(),
        }
    }
}

impl <S: Shrinker>Shrinker for BlockShrinker<S> {
    fn use_shrinker(&mut self, size: usize, pool: &mut [u8]) -> bool {
        let mut i;
        let mut i_max;
        loop {
            let block_size = pool.len() / self.recip_size;
            if block_size == 0 {
                return false;
            }
            loop {
                i = block_size * self.block_index;
                i_max = cmp::min(pool.len(), i + block_size);
                if i >= pool.len() ||
                    self.shrinker.use_shrinker(size, &mut pool[i..i_max]) {
                    break;
                }
                self.block_index += 1
            }
            if i < pool.len() {
                break;
            }
            self.block_index = 0;
            self.recip_size = self.recip_size * 2 + 1;
        }
        self.block_index += 1;
        return true;
    }
}

#[derive(Debug)]
pub struct ZeroOut;

impl Default for ZeroOut {
    fn default() -> ZeroOut { ZeroOut }
}

impl Shrinker for ZeroOut {
    fn use_shrinker(&mut self, _size: usize, pool: &mut [u8]) -> bool {
        if pool.iter().all(|&w| w == 0) {
            return false;
        }
        for ptr in pool.iter_mut() {
            *ptr = 0;
        }
        true
    }
}

fn read<T>(pool: &[u8], i: usize) -> Option<usize> {
    let bytes = mem::size_of::<T>();
    assert!(bytes <= mem::size_of::<usize>());
    if pool.len() - i < bytes {
        return None;
    }
    let mut x = 0;
    unsafe {
        ptr::copy_nonoverlapping(pool[i..].as_ptr(),
                                 &mut x as *mut usize as *mut u8,
                                 bytes);
    }
    Some(x)
}

fn write<T>(x: usize, pool: &mut [u8], i: usize) {
    let bytes = mem::size_of::<T>();
    assert!(bytes <= mem::size_of::<usize>());
    if pool.len() - i < bytes {
        return;
    }
    unsafe {
        ptr::copy_nonoverlapping(
            &x as *const usize as *const u8,
            pool[i..].as_mut_ptr(),
            bytes
        );
    }
}

#[derive(Debug)]
pub struct ModuloSize<T> {
    phantom: PhantomData<T>,
}

impl <T>Default for ModuloSize<T> {
    fn default() -> ModuloSize<T> {
        ModuloSize { phantom: PhantomData }
    }
}

impl <T>Shrinker for ModuloSize<T> {
    fn use_shrinker(&mut self, size: usize, pool: &mut [u8]) -> bool {
        let mut changed = false;
        let mut i = 0;
        while let Some(w) = read::<T>(&pool, i) {
            let x = w % size;
            if x != 0 {
                changed = true;
                write::<T>(x, pool, i);
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

impl <T>Default for DivShrinker<T> {
    fn default() -> DivShrinker<T> {
        DivShrinker {
            i: 0,
            div: 255,
            phantom: PhantomData,
        }
    }
}

impl <T>Shrinker for DivShrinker<T> {
    fn use_shrinker(&mut self, size: usize, pool: &mut [u8]) -> bool {

        if self.div == 0 {
            return false;
        }

        let mut i = self.i;
        let div = self.div as usize;
        while let Some(w) = read::<T>(&pool, i) {
            if w != 0 && w > div {
                write::<T>(w / div, pool, i);
                self.i = i + 1;
                return true;
            }
            i += 1;
        }
        self.i = 0;
        self.div = shrink_usize(div as usize, 16) as u8;
        if self.div == 0 {
            false
        } else {
            self.use_shrinker(size, pool)
        }
    }
}

#[derive(Debug)]
pub struct SubShrinker<T> {
    i: usize,
    sub: u8,
    phantom: PhantomData<T>,
}

impl <T>Default for SubShrinker<T> {
    fn default() -> SubShrinker<T> {
        SubShrinker {
            i: 0,
            sub: 255,
            phantom: PhantomData,
        }
    }
}

impl <T>Shrinker for SubShrinker<T> {
    fn use_shrinker(&mut self, size: usize, pool: &mut [u8]) -> bool {

        if self.sub == 0 {
            return false;
        }

        let mut i = self.i;
        let sub = self.sub as usize;
        while let Some(w) = read::<T>(&pool, i) {
            if w != 0 && w > sub {
                write::<T>(w - sub, pool, i);
                self.i = i + 1;
                return true;
            }
            i += 1;
        }
        self.i = 0;
        self.sub = shrink_usize(sub as usize, 16) as u8;
        if self.sub == 0 {
            false
        } else {
            self.use_shrinker(size, pool)
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
    Div64(DivShrinker<u64>),
    Sub64(SubShrinker<u64>),
    Mod32(BlockShrinker<ModuloSize<u32>>),
    Div32(DivShrinker<u32>),
    Sub32(SubShrinker<u32>),
    Mod8(BlockShrinker<ModuloSize<u8>>),
    Div8(DivShrinker<u8>),
    Sub8(SubShrinker<u8>),
}

#[derive(Debug)]
pub struct StdShrinker {
    body: StdShrinkerBody,
    pass: u8,
}

impl Default for StdShrinker {
    fn default() -> StdShrinker {
        StdShrinker {
            body: StdShrinkerBody::Zero(BlockShrinker::default()),
            pass: 0,
        }
    }
}

impl Shrinker for StdShrinker {
    fn use_shrinker(&mut self, size: usize, pool: &mut [u8]) -> bool {

        let strategy;

        macro_rules! apply_strategy {
            ($shrinker: ident, $strategy: ident) => {{
                if $shrinker.use_shrinker(size, pool) {
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
                return shrinker.use_shrinker(size, pool);
            }
            &mut StdShrinkerBody::Sub8(ref mut shrinker) => {
                self.pass += 1;
                apply_strategy!(shrinker, Sub8);
            }
        }

        macro_rules! switch_strategy {
            ($next: ident, $next_shrinker: ident) => {
                self.body = StdShrinkerBody::$next($next_shrinker::default());
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
            StdStrategy::Sub8  => switch_strategy!(Zero,  BlockShrinker),
        }
        self.use_shrinker(size, pool)
    }
}
