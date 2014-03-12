#[allow(dead_code)];

use std::num::{one, zero};
use std::str::from_chars;
use std::vec;

/// Implementations of the `Shrink` trait specify how values can be shrunk.
pub trait Shrink : Clone {
    fn shrink(&self) -> ~ObjIter:<Self>;
}

/// `~ObjIter` is an existential type that represents an arbitrary iterator
/// by satisfying the `Iterator` trait.
///
/// This makes writing shrinkers easier.
/// You should not have to implement this trait directly. By default, all
/// types which implement the Iterator trait also implement the ObjIter trait.
///
/// The `A` type variable corresponds to the elements yielded by the iterator.
pub trait ObjIter<A> {
    /// Wraps `<A: Iterator>.next()`.
    fn obj_next(&mut self) -> Option<A>;
}

impl<A> Iterator<A> for ~ObjIter:<A> {
    fn next(&mut self) -> Option<A> { self.obj_next() }
}

impl<T, A: Iterator<T>> ObjIter<T> for A {
    fn obj_next(&mut self) -> Option<T> { self.next() }
}

impl Shrink for () {
    fn shrink(&self) -> ~ObjIter:<()> {
        ~{let zero: ~[()] = ~[]; zero}.move_iter() as ~ObjIter:<()>
    }
}

impl Shrink for bool {
    fn shrink(&self) -> ~ObjIter:<bool> {
        ~match *self {
            true => (~[false]).move_iter(),
            false => (~[]).move_iter(),
        } as ~ObjIter:<bool>
    }
}

impl<A: Shrink> Shrink for Option<A> {
    fn shrink(&self)  -> ~ObjIter:<Option<A>> {
        match *self {
            None => {
                let zero: ~[Option<A>] = ~[];
                ~zero.move_iter() as ~ObjIter:<Option<A>>
            }
            Some(ref x) => {
                let none: ~[Option<A>] = ~[None];
                let tagged = x.shrink().map(Some);
                let chain = none.move_iter().chain(tagged);
                ~chain as ~ObjIter:<Option<A>>
            }
        }
    }
}

impl<A: Shrink, B: Shrink> Shrink for Result<A, B> {
    fn shrink(&self) -> ~ObjIter:<Result<A, B>> {
        match *self {
            // I don't really understand the region type here for Map.
            // I used 'static simply because the compiler let me.
            // I don't know if it is right.
            Ok(ref x) => {
                let xs: ~ObjIter:<A> = x.shrink();
                let tagged = xs.map::<'static, Result<A, B>>(Ok);
                ~tagged as ~ObjIter:<Result<A, B>>
            }
            Err(ref x) => {
                let xs: ~ObjIter:<B> = x.shrink();
                let tagged = xs.map::<'static, Result<A, B>>(Err);
                ~tagged as ~ObjIter:<Result<A, B>>
            }
        }
    }
}

// Shrinking a tuple is done by shrinking the first element and generated a
// new tuple with each shrunk element from the first along with a copy of the
// given second element. Vice versa for the second element. More precisely:
//
//     shrink((a, b)) =
//         let (sa, sb) = (a.shrink(), b.shrink());
//         ~[(sa1, b), ..., (saN, b), (a, sb1), ..., (a, sbN)]
//
// I wasn't able to figure out how to do this without copying.
impl<A: Shrink, B: Shrink> Shrink for (A, B) {
    fn shrink(&self) -> ~ObjIter:<(A, B)> {
        let (ref a, ref b) = *self;

        // I miss real closures.
        let sas = a.shrink().scan(b, |b: &mut &B, x: A| {
            Some((x, b.clone()))
        });
        let sbs = b.shrink().scan(a, |a: &mut &A, x: B| {
            Some((a.clone(), x))
        });
        ~sas.chain(sbs) as ~ObjIter:<(A, B)>
    }
}

impl<A: Shrink, B: Shrink, C: Shrink>
    Shrink for (A, B, C) {
    fn shrink(&self) -> ~ObjIter:<(A, B, C)> {
        let (ref a, ref b, ref c) = *self;

        // Sorry about the unnecessary type annotations, but they're
        // helpful to me.
        let sas = a.shrink().scan((b, c), |&(b, c): &mut (&B, &C), x: A| {
            Some((x, b.clone(), c.clone()))
        });
        let sbs = b.shrink().scan((a, c), |&(a, c): &mut (&A, &C), x: B| {
            Some((a.clone(), x, c.clone()))
        });
        let scs = c.shrink().scan((a, b), |&(a, b): &mut (&A, &B), x: C| {
            Some((a.clone(), b.clone(), x))
        });
        ~sas.chain(sbs).chain(scs) as ~ObjIter:<(A, B, C)>
    }
}

impl<A: Shrink> Shrink for ~[A] {
    fn shrink(&self) -> ~ObjIter:<~[A]> {
        let mut xs: ~[~[A]] = ~[];
        if self.len() == 0 {
            return ~xs.move_iter() as ~ObjIter:<~[A]>
        }
        xs.push(~[]);

        let mut k = self.len() / 2;
        while k > 0 && k <= self.len() {
            xs.push_all_move(shuffle_vec(*self, k, self.len()));
            k = k / 2;
        }
        for (i, x) in self.iter().enumerate() {
            for sx in x.shrink() {
                let pre = self.slice_to(i).map(|x| x.clone());
                let pre = vec::append_one(pre, sx);
                xs.push(vec::append(pre, self.slice_from(i+1)))
            }
        }
        ~xs.move_iter() as ~ObjIter:<~[A]>
    }
}

impl Shrink for ~str {
    fn shrink(&self) -> ~ObjIter:<~str> {
        let chars: ~[char] = self.chars().to_owned_vec();
        let mut strs: ~[~str] = ~[];
        for x in chars.shrink() {
            strs.push(from_chars(x));
        }
        ~strs.move_iter() as ~ObjIter:<~str>
    }
}

impl Shrink for char {
    fn shrink(&self) -> ~ObjIter:<char> {
        let zero: ~[char] = ~[];
        ~zero.move_iter() as ~ObjIter:<char>
    }
}

fn shuffle_vec<A: Clone>(xs: &[A], k: uint, n: uint) -> ~[~[A]] {
    if k > n {
        return ~[]
    }
    let xs1 = xs.slice_to(k).map(|x| x.clone());
    let xs2 = xs.slice_from(k).map(|x| x.clone());
    if xs2.len() == 0 {
        return ~[~[]]
    }

    let cat = |x: &~[A]| {
        let mut pre = xs1.clone();
        pre.push_all_move(x.clone());
        pre
    };
    let mut more = shuffle_vec(xs2, k, n - k).map(cat);
    more.unshift(xs2);
    more
}

impl Shrink for int {
    fn shrink(&self) -> ~ObjIter:<int> {
        ~shrink_signed(*self).move_iter() as ~ObjIter:<int>
    }
}

impl Shrink for i8 {
    fn shrink(&self) -> ~ObjIter:<i8> {
        ~shrink_signed(*self).move_iter() as ~ObjIter:<i8>
    }
}

impl Shrink for i16 {
    fn shrink(&self) -> ~ObjIter:<i16> {
        ~shrink_signed(*self).move_iter() as ~ObjIter:<i16>
    }
}

impl Shrink for i32 {
    fn shrink(&self) -> ~ObjIter:<i32> {
        ~shrink_signed(*self).move_iter() as ~ObjIter:<i32>
    }
}

impl Shrink for i64 {
    fn shrink(&self) -> ~ObjIter:<i64> {
        ~shrink_signed(*self).move_iter() as ~ObjIter:<i64>
    }
}

impl Shrink for uint {
    fn shrink(&self) -> ~ObjIter:<uint> {
        ~shrink_unsigned(*self).move_iter() as ~ObjIter:<uint>
    }
}

impl Shrink for u8 {
    fn shrink(&self) -> ~ObjIter:<u8> {
        ~shrink_unsigned(*self).move_iter() as ~ObjIter:<u8>
    }
}

impl Shrink for u16 {
    fn shrink(&self) -> ~ObjIter:<u16> {
        ~shrink_unsigned(*self).move_iter() as ~ObjIter:<u16>
    }
}

impl Shrink for u32 {
    fn shrink(&self) -> ~ObjIter:<u32> {
        ~shrink_unsigned(*self).move_iter() as ~ObjIter:<u32>
    }
}

impl Shrink for u64 {
    fn shrink(&self) -> ~ObjIter:<u64> {
        ~shrink_unsigned(*self).move_iter() as ~ObjIter:<u64>
    }
}

impl Shrink for f32 {
    fn shrink(&self) -> ~ObjIter:<f32> {
        let it = ~shrink_signed(self.to_i32().unwrap()).move_iter();
        ~it.map(|x| x.to_f32().unwrap()) as ~ObjIter:<f32>
    }
}

impl Shrink for f64 {
    fn shrink(&self) -> ~ObjIter:<f64> {
        let it = ~shrink_signed(self.to_i64().unwrap()).move_iter();
        ~it.map(|x| x.to_f64().unwrap()) as ~ObjIter:<f64>
    }
}

// This feels incredibly gross. I hacked my way through this one.
// The cloning seems unfortunate, but maybe the compiler is smart enough
// to elide it.
fn shrink_signed<A: Clone + Ord + Signed + Mul<A, A>>(x: A) -> ~[A] {
    if x.is_zero() {
        return ~[]
    }

    let two: A = one::<A>() + one::<A>();
    let mut xs: ~[A] = ~[zero()];
    let mut i: A = x.clone() / two;
    if i.is_negative() {
        xs.push(x.clone().abs())
    }
    while (x.clone() - i.clone()).abs() < x.clone().abs() {
        xs.push(x.clone() - i.clone());
        i = i.clone() / two;
    }
    xs
}

fn shrink_unsigned<A: Clone + Ord + Unsigned + Mul<A, A>>(x: A) -> ~[A] {
    if x.is_zero() {
        return ~[]
    }

    let two: A = one::<A>() + one::<A>();
    let mut xs: ~[A] = ~[zero()];
    let mut i: A = x.clone() / two;
    while x.clone() - i.clone() < x.clone() {
        xs.push(x.clone() - i.clone());
        i = i.clone() / two;
    }
    xs
}

#[cfg(test)]
mod test {
    use std::fmt::Show;
    use std::hash::Hash;
    use collections::HashSet;
    use super::Shrink;

    #[test]
    fn unit() {
        eq((), ~[]);
    }

    #[test]
    fn bools() {
        eq(false, ~[]);
        eq(true, ~[false]);
    }

    #[test]
    fn options() {
        eq(None::<()>, ~[]);
        eq(Some(false), ~[None]);
        eq(Some(true), ~[None, Some(false)]);
    }

    #[test]
    fn results() {
        // Result<A, B> doesn't implement the Hash trait, so these tests
        // depends on the order of shrunk results. Ug.
        // TODO: Fix this.
        ordered_eq(Ok::<bool, ()>(true), ~[Ok(false)]);
        ordered_eq(Err::<(), bool>(true), ~[Err(false)]);
    }

    #[test]
    fn tuples() {
        eq((false, false), ~[]);
        eq((true, false), ~[(false, false)]);
        eq((true, true), ~[(false, true), (true, false)]);
    }

    #[test]
    fn triples() {
        eq((false, false, false), ~[]);
        eq((true, false, false), ~[(false, false, false)]);
        eq((true, true, false), ~[(false, true, false), (true, false, false)]);
    }

    #[test]
    fn ints() {
        // TODO: Test overflow?
        eq(5i, ~[0, 3, 4]);
        eq(-5i, ~[5, 0, -3, -4]);
        eq(0i, ~[]);
    }

    #[test]
    fn ints8() {
        eq(5i8, ~[0, 3, 4]);
        eq(-5i8, ~[5, 0, -3, -4]);
        eq(0i8, ~[]);
    }

    #[test]
    fn ints16() {
        eq(5i16, ~[0, 3, 4]);
        eq(-5i16, ~[5, 0, -3, -4]);
        eq(0i16, ~[]);
    }

    #[test]
    fn ints32() {
        eq(5i32, ~[0, 3, 4]);
        eq(-5i32, ~[5, 0, -3, -4]);
        eq(0i32, ~[]);
    }

    #[test]
    fn ints64() {
        eq(5i64, ~[0, 3, 4]);
        eq(-5i64, ~[5, 0, -3, -4]);
        eq(0i64, ~[]);
    }

    #[test]
    fn uints() {
        eq(5u, ~[0, 3, 4]);
        eq(0u, ~[]);
    }

    #[test]
    fn uints8() {
        eq(5u8, ~[0, 3, 4]);
        eq(0u8, ~[]);
    }

    #[test]
    fn uints16() {
        eq(5u16, ~[0, 3, 4]);
        eq(0u16, ~[]);
    }

    #[test]
    fn uints32() {
        eq(5u32, ~[0, 3, 4]);
        eq(0u32, ~[]);
    }

    #[test]
    fn uints64() {
        eq(5u64, ~[0, 3, 4]);
        eq(0u64, ~[]);
    }

    #[test]
    fn floats32() {
        ordered_eq(5f32, ~[0f32, 3f32, 4f32]);
        ordered_eq(-5f32, ~[0f32, 5f32, -3f32, -4f32]);
        ordered_eq(0f32, ~[]);
    }

    #[test]
    fn floats64() {
        ordered_eq(5f64, ~[0f64, 3f64, 4f64]);
        ordered_eq(-5f64, ~[0f64, 5f64, -3f64, -4f64]);
        ordered_eq(0f64, ~[]);
    }

    #[test]
    fn vecs() {
        eq({let it: ~[int] = ~[]; it}, ~[]);
        eq({let it: ~[~[int]] = ~[~[]]; it}, ~[~[]]);
        eq(~[1], ~[~[], ~[0]]);
        eq(~[11], ~[~[], ~[0], ~[6], ~[9], ~[10]]);
        eq(
            ~[3, 5],
            ~[~[], ~[5], ~[3], ~[0,5], ~[2,5], ~[3,0], ~[3,3], ~[3,4]]
        );
    }

    #[test]
    fn chars() {
        eq('a', ~[]);
    }

    #[test]
    fn strs() {
        eq(~"", ~[]);
        eq(~"A", ~[~""]);
        eq(~"ABC", ~[~"", ~"AB", ~"BC", ~"AC"]);
    }

    // All this jazz is for testing set equality on the results of a shrinker.
    fn eq<A: Shrink + Eq + Show + Hash>(s: A, v: ~[A]) {
        assert_eq!(shrunk(s), set(v))
    }
    fn shrunk<A: Shrink + Eq + Hash>(s: A) -> HashSet<A> {
        set(s.shrink().to_owned_vec())
    }
    fn set<A: Eq + Hash>(xs: ~[A]) -> HashSet<A> {
        xs.move_iter().collect()
    }

    fn ordered_eq<A: Shrink + Eq + Show>(s: A, v: ~[A]) {
        assert_eq!(s.shrink().to_owned_vec(), v);
    }
}
