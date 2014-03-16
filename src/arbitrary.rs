use std::num::{one, zero};
use std::str::from_chars;
use std::vec_ng::Vec;
use rand::Rng;

/// Returns a `Gen` with the given configuration using any random number
/// generator.
///
/// The `size` parameter controls the size of random values generated.
/// For example, it specifies the maximum length of a randomly generator vector
/// and also will specify the maximum magnitude of a randomly generated number.
pub fn gen<R: Rng>(rng: R, size: uint) -> StdGen<R> {
    StdGen{rng: rng, size: size}
}

/// `Gen` wraps a `rand::Rng` with parameters to control the distribution of
/// random values.
///
/// A value with type satisfying the `Gen` trait can be constructed with the
/// `gen` function in this crate.
pub trait Gen : Rng {
    fn size(&self) -> uint;
}

/// StdGen is the default implementation of `Gen`.
///
/// Values of type `StdGen` can be created with the `gen` function in this
/// crate.
pub struct StdGen<R> {
    priv rng: R,
    priv size: uint,
}

impl<R: Rng> Rng for StdGen<R> {
    fn next_u32(&mut self) -> u32 { self.rng.next_u32() }
}

impl<R: Rng> Gen for StdGen<R> {
    fn size(&self) -> uint { self.size }
}

/// `~ObjIter` is an existential type that represents an arbitrary iterator
/// by satisfying the `Iterator` trait.
///
/// This makes writing shrinkers easier.
/// You should not have to implement this trait directly. By default, all
/// types which implement the `Iterator` trait also implement the `ObjIter`
/// trait.
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

/// `Arbitrary` describes types whose values can be randomly generated and
/// shrunk.
///
/// Aside from shrinking, `Arbitrary` is different from the `std::Rand` trait 
/// in that it uses a `Gen` to control the distribution of random values.
///
/// As of now, all types that implement `Arbitrary` must also implement 
/// `Clone`. (I'm not sure if this is a permanent restriction.)
pub trait Arbitrary : Clone + Send {
    fn arbitrary<G: Gen>(g: &mut G) -> Self;
    fn shrink(&self) -> ~ObjIter:<Self> {
        let zero: Vec<Self> = vec!();
        ~zero.move_iter() as ~ObjIter:<Self>
    }
}

impl Arbitrary for () {
    fn arbitrary<G: Gen>(_: &mut G) -> () { () }
    fn shrink(&self) -> ~ObjIter:<()> {
        let zero: Vec<()> = vec!();
        ~zero.move_iter() as ~ObjIter:<()>
    }
}

impl Arbitrary for bool {
    fn arbitrary<G: Gen>(g: &mut G) -> bool { g.gen() }
    fn shrink(&self) -> ~ObjIter:<bool> {
        ~match *self {
            true => (vec!(false)).move_iter(),
            false => (vec!()).move_iter(),
        } as ~ObjIter:<bool>
    }
}

impl<A: Arbitrary> Arbitrary for Option<A> {
    fn arbitrary<G: Gen>(g: &mut G) -> Option<A> {
        if g.gen() {
            None
        } else {
            Some(Arbitrary::arbitrary(g))
        }
    }

    fn shrink(&self)  -> ~ObjIter:<Option<A>> {
        match *self {
            None => {
                let zero: Vec<Option<A>> = vec!();
                ~zero.move_iter() as ~ObjIter:<Option<A>>
            }
            Some(ref x) => {
                let none: Vec<Option<A>> = vec!(None);
                let tagged = x.shrink().map(Some);
                let chain = none.move_iter().chain(tagged);
                ~chain as ~ObjIter:<Option<A>>
            }
        }
    }
}

impl<A: Arbitrary, B: Arbitrary> Arbitrary for Result<A, B> {
    fn arbitrary<G: Gen>(g: &mut G) -> Result<A, B> {
        if g.gen() {
            Ok(Arbitrary::arbitrary(g))
        } else {
            Err(Arbitrary::arbitrary(g))
        }
    }

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

impl<A: Arbitrary, B: Arbitrary> Arbitrary for (A, B) {
    fn arbitrary<G: Gen>(g: &mut G) -> (A, B) {
        return (Arbitrary::arbitrary(g), Arbitrary::arbitrary(g))
    }

    // Shrinking a tuple is done by shrinking the first element and generated a
    // new tuple with each shrunk element from the first along with a copy of 
    // the given second element. Vice versa for the second element. More 
    // precisely:
    //
    //     shrink((a, b)) =
    //         let (sa, sb) = (a.shrink(), b.shrink());
    //         vec!((sa1, b), ..., (saN, b), (a, sb1), ..., (a, sbN))
    //
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

impl<A: Arbitrary, B: Arbitrary, C: Arbitrary> Arbitrary for (A, B, C) {
    fn arbitrary<G: Gen>(g: &mut G) -> (A, B, C) {
        return (
            Arbitrary::arbitrary(g),
            Arbitrary::arbitrary(g),
            Arbitrary::arbitrary(g),
        )
    }

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

impl<A: Arbitrary> Arbitrary for Vec<A> {
    fn arbitrary<G: Gen>(g: &mut G) -> Vec<A> {
        let size = { let s = g.size(); g.gen_range(0, s) };
        Vec::from_fn(size, |_| Arbitrary::arbitrary(g))
    }

    fn shrink(&self) -> ~ObjIter:<Vec<A>> {
        let mut xs: Vec<Vec<A>> = vec!();
        if self.len() == 0 {
            return ~xs.move_iter() as ~ObjIter:<Vec<A>>
        }
        xs.push(vec!());

        let mut k = self.len() / 2;
        while k > 0 && k <= self.len() {
            xs.push_all_move(shuffle_vec(self, k, self.len()));
            k = k / 2;
        }
        for (i, x) in self.iter().enumerate() {
            for sx in x.shrink() {
                let pre = Vec::from_slice(self.slice_to(i).map(|x| x.clone()));
                let pre = ::std::vec_ng::append_one(pre, sx);
                xs.push(::std::vec_ng::append(pre, self.slice_from(i+1)))
            }
        }
        ~xs.move_iter() as ~ObjIter:<Vec<A>>
    }
}

impl Arbitrary for ~str {
    fn arbitrary<G: Gen>(g: &mut G) -> ~str {
        let size = { let s = g.size(); g.gen_range(0, s) };
        g.gen_ascii_str(size)
    }

    fn shrink(&self) -> ~ObjIter:<~str> {
        let chars: Vec<char> = self.chars().collect();
        let mut strs: Vec<~str> = vec!();
        for x in chars.shrink() {
            strs.push(from_chars(x.as_slice()));
        }
        ~strs.move_iter() as ~ObjIter:<~str>
    }
}

impl Arbitrary for char {
    fn arbitrary<G: Gen>(g: &mut G) -> char { g.gen() }

    fn shrink(&self) -> ~ObjIter:<char> {
        let zero: Vec<char> = vec!();
        ~zero.move_iter() as ~ObjIter:<char>
    }
}

impl Arbitrary for int {
    fn arbitrary<G: Gen>(g: &mut G) -> int {
        let s = g.size(); g.gen_range(-(s as int), s as int)
    }
    fn shrink(&self) -> ~ObjIter:<int> {
        ~shrink_signed(*self).move_iter() as ~ObjIter:<int>
    }
}

impl Arbitrary for i8 {
    fn arbitrary<G: Gen>(g: &mut G) -> i8 {
        let s = g.size(); g.gen_range(-(s as i8), s as i8)
    }
    fn shrink(&self) -> ~ObjIter:<i8> {
        ~shrink_signed(*self).move_iter() as ~ObjIter:<i8>
    }
}

impl Arbitrary for i16 {
    fn arbitrary<G: Gen>(g: &mut G) -> i16 {
        let s = g.size(); g.gen_range(-(s as i16), s as i16)
    }
    fn shrink(&self) -> ~ObjIter:<i16> {
        ~shrink_signed(*self).move_iter() as ~ObjIter:<i16>
    }
}

impl Arbitrary for i32 {
    fn arbitrary<G: Gen>(g: &mut G) -> i32 {
        let s = g.size(); g.gen_range(-(s as i32), s as i32)
    }
    fn shrink(&self) -> ~ObjIter:<i32> {
        ~shrink_signed(*self).move_iter() as ~ObjIter:<i32>
    }
}

impl Arbitrary for i64 {
    fn arbitrary<G: Gen>(g: &mut G) -> i64 {
        let s = g.size(); g.gen_range(-(s as i64), s as i64)
    }
    fn shrink(&self) -> ~ObjIter:<i64> {
        ~shrink_signed(*self).move_iter() as ~ObjIter:<i64>
    }
}

impl Arbitrary for uint {
    fn arbitrary<G: Gen>(g: &mut G) -> uint {
        let s = g.size(); g.gen_range(0, s)
    }
    fn shrink(&self) -> ~ObjIter:<uint> {
        ~shrink_unsigned(*self).move_iter() as ~ObjIter:<uint>
    }
}

impl Arbitrary for u8 {
    fn arbitrary<G: Gen>(g: &mut G) -> u8 {
        let s = g.size(); g.gen_range(0, s as u8)
    }
    fn shrink(&self) -> ~ObjIter:<u8> {
        ~shrink_unsigned(*self).move_iter() as ~ObjIter:<u8>
    }
}

impl Arbitrary for u16 {
    fn arbitrary<G: Gen>(g: &mut G) -> u16 {
        let s = g.size(); g.gen_range(0, s as u16)
    }
    fn shrink(&self) -> ~ObjIter:<u16> {
        ~shrink_unsigned(*self).move_iter() as ~ObjIter:<u16>
    }
}

impl Arbitrary for u32 {
    fn arbitrary<G: Gen>(g: &mut G) -> u32 {
        let s = g.size(); g.gen_range(0, s as u32)
    }
    fn shrink(&self) -> ~ObjIter:<u32> {
        ~shrink_unsigned(*self).move_iter() as ~ObjIter:<u32>
    }
}

impl Arbitrary for u64 {
    fn arbitrary<G: Gen>(g: &mut G) -> u64 {
        let s = g.size(); g.gen_range(0, s as u64)
    }
    fn shrink(&self) -> ~ObjIter:<u64> {
        ~shrink_unsigned(*self).move_iter() as ~ObjIter:<u64>
    }
}

impl Arbitrary for f32 {
    fn arbitrary<G: Gen>(g: &mut G) -> f32 {
        let s = g.size(); g.gen_range(-(s as f32), s as f32)
    }
    fn shrink(&self) -> ~ObjIter:<f32> {
        let it = ~shrink_signed(self.to_i32().unwrap()).move_iter();
        ~it.map(|x| x.to_f32().unwrap()) as ~ObjIter:<f32>
    }
}

impl Arbitrary for f64 {
    fn arbitrary<G: Gen>(g: &mut G) -> f64 {
        let s = g.size(); g.gen_range(-(s as f64), s as f64)
    }
    fn shrink(&self) -> ~ObjIter:<f64> {
        let it = ~shrink_signed(self.to_i64().unwrap()).move_iter();
        ~it.map(|x| x.to_f64().unwrap()) as ~ObjIter:<f64>
    }
}

fn shuffle_vec<A: Clone>(xs: &Vec<A>, k: uint, n: uint) -> Vec<Vec<A>> {
    if k > n {
        return vec!()
    }
    let xs1 = Vec::from_slice(xs.slice_to(k).map(|x| x.clone()));
    let xs2 = Vec::from_slice(xs.slice_from(k).map(|x| x.clone()));
    if xs2.len() == 0 {
        return vec!(vec!())
    }

    let cat = |x: &Vec<A>| {
        let mut pre = xs1.clone();
        pre.push_all_move(x.clone());
        pre
    };
    let shuffled = shuffle_vec(&xs2, k, n-k);
    let mut more: Vec<Vec<A>> = shuffled.iter().map(cat).collect();
    more.unshift(xs2);
    more
}

// This feels incredibly gross. I hacked my way through this one.
// The cloning seems unfortunate, but maybe the compiler is smart enough
// to elide it.
fn shrink_signed<A: Clone + Ord + Signed + Mul<A, A>>(x: A) -> Vec<A> {
    if x.is_zero() {
        return vec!()
    }

    let two: A = one::<A>() + one::<A>();
    let mut xs: Vec<A> = vec!(zero());
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

fn shrink_unsigned<A: Clone + Ord + Unsigned + Mul<A, A>>(x: A) -> Vec<A> {
    if x.is_zero() {
        return vec!()
    }

    let two: A = one::<A>() + one::<A>();
    let mut xs: Vec<A> = vec!(zero());
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
    use std::iter;
    use std::vec_ng::Vec;
    use collections::HashSet;
    use rand;
    use super::Arbitrary;

    // Arbitrary testing. (Not much here. What else can I reasonably test?)
    #[test]
    fn arby_unit() {
        assert_eq!(arby::<()>(), ());
    }

    #[test]
    fn arby_int() {
        rep(|| { let n: int = arby(); assert!(n >= -5 && n <= 5); } );
    }

    #[test]
    fn arby_uint() {
        rep(|| { let n: uint = arby(); assert!(n <= 5); } );
    }

    fn arby<A: super::Arbitrary>() -> A {
        super::Arbitrary::arbitrary(&mut gen())
    }

    fn gen() -> super::StdGen<rand::TaskRng> {
        super::gen(rand::task_rng(), 5)
    }

    fn rep(f: ||) {
        for _ in iter::range(0, 100) {
            f()
        }
    }

    // Shrink testing.
    #[test]
    fn unit() {
        eq((), vec!());
    }

    #[test]
    fn bools() {
        eq(false, vec!());
        eq(true, vec!(false));
    }

    #[test]
    fn options() {
        eq(None::<()>, vec!());
        eq(Some(false), vec!(None));
        eq(Some(true), vec!(None, Some(false)));
    }

    #[test]
    fn results() {
        // Result<A, B> doesn't implement the Hash trait, so these tests
        // depends on the order of shrunk results. Ug.
        // TODO: Fix this.
        ordered_eq(Ok::<bool, ()>(true), vec!(Ok(false)));
        ordered_eq(Err::<(), bool>(true), vec!(Err(false)));
    }

    #[test]
    fn tuples() {
        eq((false, false), vec!());
        eq((true, false), vec!((false, false)));
        eq((true, true), vec!((false, true), (true, false)));
    }

    #[test]
    fn triples() {
        eq((false, false, false), vec!());
        eq((true, false, false), vec!((false, false, false)));
        eq((true, true, false),
           vec!((false, true, false), (true, false, false)));
    }

    #[test]
    fn ints() {
        // TODO: Test overflow?
        eq(5i, vec!(0, 3, 4));
        eq(-5i, vec!(5, 0, -3, -4));
        eq(0i, vec!());
    }

    #[test]
    fn ints8() {
        eq(5i8, vec!(0, 3, 4));
        eq(-5i8, vec!(5, 0, -3, -4));
        eq(0i8, vec!());
    }

    #[test]
    fn ints16() {
        eq(5i16, vec!(0, 3, 4));
        eq(-5i16, vec!(5, 0, -3, -4));
        eq(0i16, vec!());
    }

    #[test]
    fn ints32() {
        eq(5i32, vec!(0, 3, 4));
        eq(-5i32, vec!(5, 0, -3, -4));
        eq(0i32, vec!());
    }

    #[test]
    fn ints64() {
        eq(5i64, vec!(0, 3, 4));
        eq(-5i64, vec!(5, 0, -3, -4));
        eq(0i64, vec!());
    }

    #[test]
    fn uints() {
        eq(5u, vec!(0, 3, 4));
        eq(0u, vec!());
    }

    #[test]
    fn uints8() {
        eq(5u8, vec!(0, 3, 4));
        eq(0u8, vec!());
    }

    #[test]
    fn uints16() {
        eq(5u16, vec!(0, 3, 4));
        eq(0u16, vec!());
    }

    #[test]
    fn uints32() {
        eq(5u32, vec!(0, 3, 4));
        eq(0u32, vec!());
    }

    #[test]
    fn uints64() {
        eq(5u64, vec!(0, 3, 4));
        eq(0u64, vec!());
    }

    #[test]
    fn floats32() {
        ordered_eq(5f32, vec!(0f32, 3f32, 4f32));
        ordered_eq(-5f32, vec!(0f32, 5f32, -3f32, -4f32));
        ordered_eq(0f32, vec!());
    }

    #[test]
    fn floats64() {
        ordered_eq(5f64, vec!(0f64, 3f64, 4f64));
        ordered_eq(-5f64, vec!(0f64, 5f64, -3f64, -4f64));
        ordered_eq(0f64, vec!());
    }

    #[test]
    fn vecs() {
        eq({let it: Vec<int> = vec!(); it}, vec!());
        eq({let it: Vec<Vec<int>> = vec!(vec!()); it}, vec!(vec!()));
        eq(vec!(1), vec!(vec!(), vec!(0)));
        eq(vec!(11), vec!(vec!(), vec!(0), vec!(6), vec!(9), vec!(10)));
        eq(
            vec!(3, 5),
            vec!(vec!(), vec!(5), vec!(3), vec!(0,5), vec!(2,5),
                 vec!(3,0), vec!(3,3), vec!(3,4))
        );
    }

    #[test]
    fn chars() {
        eq('a', vec!());
    }

    #[test]
    fn strs() {
        eq(~"", vec!());
        eq(~"A", vec!(~""));
        eq(~"ABC", vec!(~"", ~"AB", ~"BC", ~"AC"));
    }

    // All this jazz is for testing set equality on the results of a shrinker.
    fn eq<A: Arbitrary + Eq + Show + Hash>(s: A, v: Vec<A>) {
        assert_eq!(shrunk(s), set(v))
    }
    fn shrunk<A: Arbitrary + Eq + Hash>(s: A) -> HashSet<A> {
        set(s.shrink().collect())
    }
    fn set<A: Eq + Hash>(xs: Vec<A>) -> HashSet<A> {
        xs.move_iter().collect()
    }

    fn ordered_eq<A: Arbitrary + Eq + Show>(s: A, v: Vec<A>) {
        assert_eq!(s.shrink().collect::<Vec<A>>(), v);
    }
}
