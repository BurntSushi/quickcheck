use std::mem;
use std::num;
use std::rand::Rng;

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
    rng: R,
    size: uint,
}

impl<R: Rng> Rng for StdGen<R> {
    fn next_u32(&mut self) -> u32 { self.rng.next_u32() }

    // some RNGs implement these more efficiently than the default, so
    // we might as well defer to them.
    fn next_u64(&mut self) -> u64 { self.rng.next_u64() }
    fn fill_bytes(&mut self, dest: &mut [u8]) { self.rng.fill_bytes(dest) }
}

impl<R: Rng> Gen for StdGen<R> {
    fn size(&self) -> uint { self.size }
}

/// `~Shrinker` is an existential type that represents an arbitrary iterator
/// by satisfying the `Iterator` trait.
///
/// This makes writing shrinkers easier.
/// You should not have to implement this trait directly. By default, all
/// types which implement the `Iterator` trait also implement the `Shrinker`
/// trait.
///
/// The `A` type variable corresponds to the elements yielded by the iterator.
pub trait Shrinker<A> {
    /// Wraps `<A: Iterator>.next()`.
    fn next_shrink(&mut self) -> Option<A>;
}

impl<A> Iterator<A> for Box<Shrinker<A>+'static> {
    fn next(&mut self) -> Option<A> { self.next_shrink() }
}

impl<T, A: Iterator<T>> Shrinker<T> for A {
    fn next_shrink(&mut self) -> Option<T> { self.next() }
}

struct EmptyShrinker<A>;

impl<A> Iterator<A> for EmptyShrinker<A> {
    fn next(&mut self) -> Option<A> { None }
}

/// Creates a shrinker with zero elements.
pub fn empty_shrinker<A>() -> Box<Shrinker<A>+'static> {
    let zero: EmptyShrinker<A> = EmptyShrinker;
    box zero as Box<Shrinker<A>+'static>
}

struct SingleShrinker<A> {
    value: Option<A>
}

impl<A> Iterator<A> for SingleShrinker<A> {
    fn next(&mut self) -> Option<A> { mem::replace(&mut self.value, None) }
}

/// Creates a shrinker with a single element.
pub fn single_shrinker<A: 'static>(value: A) -> Box<Shrinker<A>+'static> {
    let one: SingleShrinker<A> = SingleShrinker { value: Some(value) };
    box one as Box<Shrinker<A>+'static>
}

/// `Arbitrary` describes types whose values can be randomly generated and
/// shrunk.
///
/// Aside from shrinking, `Arbitrary` is different from the `std::Rand` trait 
/// in that it uses a `Gen` to control the distribution of random values.
///
/// As of now, all types that implement `Arbitrary` must also implement 
/// `Clone`. (I'm not sure if this is a permanent restriction.)
///
/// They must also be sendable since every test is run inside its own task.
/// (This permits failures to include task failures.)
pub trait Arbitrary : Clone + Send {
    fn arbitrary<G: Gen>(g: &mut G) -> Self;
    fn shrink(&self) -> Box<Shrinker<Self>+'static> {
        empty_shrinker()
    }
}

impl Arbitrary for () {
    fn arbitrary<G: Gen>(_: &mut G) -> () { () }
}

impl Arbitrary for bool {
    fn arbitrary<G: Gen>(g: &mut G) -> bool { g.gen() }
    fn shrink(&self) -> Box<Shrinker<bool>+'static> {
        match *self {
            true => single_shrinker(false),
            false => empty_shrinker(),
        }
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

    fn shrink(&self)  -> Box<Shrinker<Option<A>>+'static> {
        match *self {
            None => {
                empty_shrinker()
            }
            Some(ref x) => {
                let chain = single_shrinker(None).chain(x.shrink().map(Some));
                box chain as Box<Shrinker<Option<A>>+'static>
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

    fn shrink(&self) -> Box<Shrinker<Result<A, B>>+'static> {
        match *self {
            Ok(ref x) => {
                let xs: Box<Shrinker<A>+'static> = x.shrink();
                let tagged = xs.map::<'static, Result<A, B>>(Ok);
                box tagged as Box<Shrinker<Result<A, B>>+'static>
            }
            Err(ref x) => {
                let xs: Box<Shrinker<B>+'static> = x.shrink();
                let tagged = xs.map::<'static, Result<A, B>>(Err);
                box tagged as Box<Shrinker<Result<A, B>>+'static>
            }
        }
    }
}

impl<A: Arbitrary, B: Arbitrary> Arbitrary for (A, B) {
    fn arbitrary<G: Gen>(g: &mut G) -> (A, B) {
        return (Arbitrary::arbitrary(g), Arbitrary::arbitrary(g))
    }

    // Shrinking a tuple is done by shrinking the first element and generating 
    // a new tuple with each shrunk element from the first along with a copy of 
    // the given second element. Vice versa for the second element. More 
    // precisely:
    //
    //     shrink((a, b)) =
    //         let (sa, sb) = (a.shrink(), b.shrink());
    //         vec!((sa1, b), ..., (saN, b), (a, sb1), ..., (a, sbN))
    //
    fn shrink(&self) -> Box<Shrinker<(A, B)>+'static> {
        let (ref a, ref b) = *self;
        let sas = a.shrink().scan(b.clone(), |b, a| {
            Some((a, b.clone()))
        });
        let sbs = b.shrink().scan(a.clone(), |a, b| {
            Some((a.clone(), b))
        });
        box sas.chain(sbs) as Box<Shrinker<(A, B)>+'static>
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

    fn shrink(&self) -> Box<Shrinker<(A, B, C)>+'static> {
        let (ref a, ref b, ref c) = *self;
        let sas = a.shrink().scan((b.clone(), c.clone()), |&(ref b, ref c), a| {
            Some((a, b.clone(), c.clone()))
        });
        let sbs = b.shrink().scan((a.clone(), c.clone()), |&(ref a, ref c), b| {
            Some((a.clone(), b, c.clone()))
        });
        let scs = c.shrink().scan((a.clone(), b.clone()), |&(ref a, ref b), c| {
            Some((a.clone(), b.clone(), c))
        });
        box sas.chain(sbs).chain(scs) as Box<Shrinker<(A, B, C)>+'static>
    }
}

impl<A: Arbitrary> Arbitrary for Vec<A> {
    fn arbitrary<G: Gen>(g: &mut G) -> Vec<A> {
        let size = { let s = g.size(); g.gen_range(0, s) };
        Vec::from_fn(size, |_| Arbitrary::arbitrary(g))
    }

    fn shrink(&self) -> Box<Shrinker<Vec<A>>+'static> {
        if self.len() == 0 {
            return empty_shrinker()
        }

        // Start the shrunk values with an empty vector.
        let mut xs: Vec<Vec<A>> = vec!(vec!());

        // Explore the space of different sized vectors without shrinking
        // any of the elements.
        let mut k = self.len() / 2;
        while k > 0 {
            xs.push_all_move(shuffle_vec(self.as_slice(), k));
            k = k / 2;
        }

        // Now explore the space of vectors where each element is shrunk
        // in turn. A new vector is generated for each shrunk value of each
        // element.
        for (i, x) in self.iter().enumerate() {
            for sx in x.shrink() {
                let mut change_one = self.clone();
                *change_one.get_mut(i) = sx;
                xs.push(change_one);
            }
        }
        box xs.move_iter() as Box<Shrinker<Vec<A>>+'static>
    }
}

impl Arbitrary for String {
    fn arbitrary<G: Gen>(g: &mut G) -> String {
        let size = { let s = g.size(); g.gen_range(0, s) };
        g.gen_ascii_chars().take(size).collect()
    }

    fn shrink(&self) -> Box<Shrinker<String>+'static> {
        // Shrink a string by shrinking a vector of its characters.
        let chars: Vec<char> = self.as_slice().chars().collect();
        let strs = chars.shrink().map(|x| x.move_iter().collect::<String>());
        box strs as Box<Shrinker<String>+'static>
    }
}

impl Arbitrary for char {
    fn arbitrary<G: Gen>(g: &mut G) -> char { g.gen() }

    fn shrink(&self) -> Box<Shrinker<char>+'static> {
        // No char shrinking for now.
        empty_shrinker()
    }
}

macro_rules! signed_arbitrary {
    ($($ty: ty),*) => {
        $(
            impl Arbitrary for $ty {
                fn arbitrary<G: Gen>(g: &mut G) -> $ty {
                    let s = g.size(); g.gen_range(-(s as $ty), s as $ty)
                }
                fn shrink(&self) -> Box<Shrinker<$ty>+'static> {
                    SignedShrinker::new(*self)
                }
            }
            )*
    }
}
signed_arbitrary! {
    int, i8, i16, i32, i64
}

macro_rules! unsigned_arbitrary {
    ($($ty: ty),*) => {
        $(
            impl Arbitrary for $ty {
                fn arbitrary<G: Gen>(g: &mut G) -> $ty {
                    let s = g.size(); g.gen_range(0, s as $ty)
                }
                fn shrink(&self) -> Box<Shrinker<$ty>+'static> {
                    UnsignedShrinker::new(*self)
                }
            }
            )*
    }
}
unsigned_arbitrary! {
    uint, u8, u16, u32, u64
}

impl Arbitrary for f32 {
    fn arbitrary<G: Gen>(g: &mut G) -> f32 {
        let s = g.size(); g.gen_range(-(s as f32), s as f32)
    }
    fn shrink(&self) -> Box<Shrinker<f32>+'static> {
        let it = SignedShrinker::new(self.to_i32().unwrap());
        box it.map(|x| x.to_f32().unwrap()) as Box<Shrinker<f32>+'static>
    }
}

impl Arbitrary for f64 {
    fn arbitrary<G: Gen>(g: &mut G) -> f64 {
        let s = g.size(); g.gen_range(-(s as f64), s as f64)
    }
    fn shrink(&self) -> Box<Shrinker<f64>+'static> {
        let it = SignedShrinker::new(self.to_i64().unwrap());
        box it.map(|x| x.to_f64().unwrap()) as Box<Shrinker<f64>+'static>
    }
}

/// Returns a sequence of vectors with each contiguous run of elements of
/// length `k` removed. 
fn shuffle_vec<A: Clone>(xs: &[A], k: uint) -> Vec<Vec<A>> {
    fn shuffle<A: Clone>(xs: &[A], k: uint, n: uint) -> Vec<Vec<A>> {
        if k > n {
            return vec!()
        }
        let xs1: Vec<A> = xs.slice_to(k).iter().map(|x| x.clone()).collect();
        let xs2: Vec<A> = xs.slice_from(k).iter().map(|x| x.clone()).collect();
        if xs2.len() == 0 {
            return vec!(vec!())
        }

        let cat = |x: &Vec<A>| {
            let mut pre = xs1.clone();
            pre.push_all_move(x.clone());
            pre
        };
        let shuffled = shuffle(xs2.as_slice(), k, n-k);
        let mut more: Vec<Vec<A>> = shuffled.iter().map(cat).collect();
        more.insert(0, xs2);
        more
    }
    shuffle(xs, k, xs.len())
}

fn half<A: Primitive>(x: A) -> A { x / num::cast(2i).unwrap() }

struct SignedShrinker<A> {
    x: A,
    i: A,
}

impl<A: Primitive + Signed + Send> SignedShrinker<A> {
    fn new(x: A) -> Box<Shrinker<A>+'static> {
        if x.is_zero() {
            empty_shrinker::<A>()
        } else {
            let shrinker = SignedShrinker {
                x: x,
                i: half(x),
            };
            if shrinker.i.is_negative() {
                box {
                    vec![num::zero(), shrinker.x.abs()]
                }.move_iter().chain(shrinker) as Box<Shrinker<A>+'static>
            } else {
                box {
                    vec![num::zero()]
                }.move_iter().chain(shrinker) as Box<Shrinker<A>+'static>
            }
        }
    }
}

impl<A: Primitive + Signed> Iterator<A> for SignedShrinker<A> {
    fn next(&mut self) -> Option<A> {
        if (self.x - self.i).abs() < self.x.abs() {
            let result = Some(self.x - self.i);
            self.i = half(self.i);
            result
        } else {
            None
        }
    }
}

struct UnsignedShrinker<A> {
    x: A,
    i: A,
}

impl<A: Primitive + Unsigned + Send> UnsignedShrinker<A> {
    fn new(x: A) -> Box<Shrinker<A>+'static> {
        if x.is_zero() {
            empty_shrinker::<A>()
        } else {
            box { vec![num::zero()] }.move_iter().chain(
                UnsignedShrinker {
                    x: x,
                    i: half(x),
                }
            ) as Box<Shrinker<A>+'static>
        }
    }
}

impl<A: Primitive + Unsigned> Iterator<A> for UnsignedShrinker<A> {
    fn next(&mut self) -> Option<A> {
        if self.x - self.i < self.x {
            let result = Some(self.x - self.i);
            self.i = half(self.i);
            result
        } else {
            None
        }
    }
}

#[cfg(test)]
mod test {
    use std::fmt::Show;
    use std::hash::Hash;
    use std::iter;
    use std::collections::HashSet;
    use std::rand;
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
        for _ in iter::range(0u, 100) {
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
    fn vecs() {
        eq({let it: Vec<int> = vec!(); it}, vec!());
        eq({let it: Vec<Vec<int>> = vec!(vec!()); it}, vec!(vec!()));
        eq(vec!(1i), vec!(vec!(), vec!(0)));
        eq(vec!(11i), vec!(vec!(), vec!(0), vec!(6), vec!(9), vec!(10)));
        eq(
            vec!(3i, 5),
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
        eq("".to_string(), vec!());
        eq("A".to_string(), vec!("".to_string()));
        eq("ABC".to_string(), vec!("".to_string(),
                                   "AB".to_string(),
                                   "BC".to_string(),
                                   "AC".to_string()));
    }

    // All this jazz is for testing set equality on the results of a shrinker.
    fn eq<A: Arbitrary + Eq + Show + Hash>(s: A, v: Vec<A>) {
        let (left, right) = (shrunk(s), set(v));
        assert_eq!(left, right);
    }
    fn shrunk<A: Arbitrary + Eq + Hash>(s: A) -> HashSet<A> {
        set(s.shrink().collect())
    }
    fn set<A: Eq + Hash>(xs: Vec<A>) -> HashSet<A> {
        xs.move_iter().collect()
    }

    fn ordered_eq<A: Arbitrary + Eq + Show>(s: A, v: Vec<A>) {
        let (left, right) = (s.shrink().collect::<Vec<A>>(), v);
        assert_eq!(left, right);
    }
}
