use rand::Rng;
use std::collections::hash_map::{HashMap, self};
use std::hash::{Hash, Hasher};
use std::iter::range;
use std::mem;
use std::num::{self, Int, SignedInt, UnsignedInt};

#[cfg(feature = "collect_impls")]
use collect::TrieMap;

/// `Gen` wraps a `rand::Rng` with parameters to control the distribution of
/// random values.
///
/// A value with type satisfying the `Gen` trait can be constructed with the
/// `gen` function in this crate.
pub trait Gen : Rng {
    fn size(&self) -> usize;
}

/// StdGen is the default implementation of `Gen`.
///
/// Values of type `StdGen` can be created with the `gen` function in this
/// crate.
pub struct StdGen<R> {
    rng: R,
    size: usize,
}

/// Returns a `StdGen` with the given configuration using any random number
/// generator.
///
/// The `size` parameter controls the size of random values generated.
/// For example, it specifies the maximum length of a randomly generated vector
/// and also will specify the maximum magnitude of a randomly generated number.
impl<R: Rng> StdGen<R> {
    pub fn new(rng: R, size: usize) -> StdGen<R> {
        StdGen { rng: rng, size: size }
    }
}

impl<R: Rng> Rng for StdGen<R> {
    fn next_u32(&mut self) -> u32 { self.rng.next_u32() }

    // some RNGs implement these more efficiently than the default, so
    // we might as well defer to them.
    fn next_u64(&mut self) -> u64 { self.rng.next_u64() }
    fn fill_bytes(&mut self, dest: &mut [u8]) { self.rng.fill_bytes(dest) }
}

impl<R: Rng> Gen for StdGen<R> {
    fn size(&self) -> usize { self.size }
}

/// `Box<Shrinker>` is an existential type that represents an arbitrary
/// iterator.
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

impl<A> Iterator for Box<Shrinker<A>+'static> {
    type Item = A;
    fn next(&mut self) -> Option<A> { (**self).next_shrink() }
}

impl<T, A: Iterator<Item=T>> Shrinker<T> for A {
    fn next_shrink(&mut self) -> Option<T> { self.next() }
}

struct EmptyShrinker<A>;

impl<A> Iterator for EmptyShrinker<A> {
    type Item = A;
    fn next(&mut self) -> Option<A> { None }
}

/// Creates a shrinker with zero elements.
pub fn empty_shrinker<A>() -> Box<Shrinker<A>+'static> {
    Box::new(EmptyShrinker)
}

struct SingleShrinker<A> {
    value: Option<A>
}

impl<A> Iterator for SingleShrinker<A> {
    type Item = A;
    fn next(&mut self) -> Option<A> { mem::replace(&mut self.value, None) }
}

/// Creates a shrinker with a single element.
pub fn single_shrinker<A: 'static>(value: A) -> Box<Shrinker<A>+'static> {
    Box::new(SingleShrinker { value: Some(value) })
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
                Box::new(chain) as Box<Shrinker<Option<A>>+'static>
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
                let tagged = xs.map(Ok);
                Box::new(tagged) as Box<Shrinker<Result<A, B>>+'static>
            }
            Err(ref x) => {
                let xs: Box<Shrinker<B>+'static> = x.shrink();
                let tagged = xs.map(Err);
                Box::new(tagged) as Box<Shrinker<Result<A, B>>+'static>
            }
        }
    }
}

macro_rules! impl_arb_for_tuple {
    (($var_a:ident, $type_a:ident) $(, ($var_n:ident, $type_n:ident))*) => (
        impl<$type_a: Arbitrary, $($type_n: Arbitrary),*> Arbitrary
                for ($type_a, $($type_n),*) {
            fn arbitrary<GEN: Gen>(g: &mut GEN) -> ($type_a, $($type_n),*) {
                (
                    Arbitrary::arbitrary(g),
                    $({
                        let arb: $type_n = Arbitrary::arbitrary(g);
                        arb
                    },
                    )*
                )
            }

            fn shrink(&self)
                     -> Box<Shrinker<($type_a, $($type_n),*)> + 'static> {
                let (ref $var_a, $(ref $var_n),*) = *self;
                let sa = $var_a.shrink().scan(
                    ($($var_n.clone(),)*),
                    |&mut ($(ref $var_n,)*), $var_a|
                        Some(($var_a, $($var_n.clone(),)*))
                );
                let srest = ($($var_n.clone(),)*).shrink()
                    .scan($var_a.clone(), |$var_a, ($($var_n,)*)|
                        Some(($var_a.clone(), $($var_n,)*))
                    );
                Box::new(sa.chain(srest))
            }
        }
    );
}

impl_arb_for_tuple!((a, A));
impl_arb_for_tuple!((a, A), (b, B));
impl_arb_for_tuple!((a, A), (b, B), (c, C));
impl_arb_for_tuple!((a, A), (b, B), (c, C), (d, D));
impl_arb_for_tuple!((a, A), (b, B), (c, C), (d, D), (e, E));
impl_arb_for_tuple!((a, A), (b, B), (c, C), (d, D), (e, E), (f, F));
impl_arb_for_tuple!((a, A), (b, B), (c, C), (d, D), (e, E), (f, F),
                    (g, G));
impl_arb_for_tuple!((a, A), (b, B), (c, C), (d, D), (e, E), (f, F),
                    (g, G), (h, H));
impl_arb_for_tuple!((a, A), (b, B), (c, C), (d, D), (e, E), (f, F),
                    (g, G), (h, H), (i, I));
impl_arb_for_tuple!((a, A), (b, B), (c, C), (d, D), (e, E), (f, F),
                    (g, G), (h, H), (i, I), (j, J));
impl_arb_for_tuple!((a, A), (b, B), (c, C), (d, D), (e, E), (f, F),
                    (g, G), (h, H), (i, I), (j, J), (k, K));
impl_arb_for_tuple!((a, A), (b, B), (c, C), (d, D), (e, E), (f, F),
                    (g, G), (h, H), (i, I), (j, J), (k, K), (l, L));

impl<A: Arbitrary> Arbitrary for Vec<A> {
    fn arbitrary<G: Gen>(g: &mut G) -> Vec<A> {
        let size = { let s = g.size(); g.gen_range(0, s) };
        range(0, size).map(|_| Arbitrary::arbitrary(g)).collect()
    }

    fn shrink(&self) -> Box<Shrinker<Vec<A>>+'static> {
        if self.len() == 0 {
            return empty_shrinker();
        }

        // Start the shrunk values with an empty vector.
        let mut xs: Vec<Vec<A>> = vec![vec![]];

        // Explore the space of different sized vectors without shrinking
        // any of the elements.
        let mut k = self.len() / 2;
        while k > 0 {
            xs.extend(shuffle_vec(&**self, k).into_iter());
            k = k / 2;
        }

        // Now explore the space of vectors where each element is shrunk
        // in turn. A new vector is generated for each shrunk value of each
        // element.
        for (i, x) in self.iter().enumerate() {
            for sx in x.shrink() {
                let mut change_one = self.clone();
                change_one[i] = sx;
                xs.push(change_one);
            }
        }
        Box::new(xs.into_iter())
    }
}

#[cfg(feature = "collect_impls")]
impl<A: Arbitrary> Arbitrary for TrieMap<A> {
    fn arbitrary<G: Gen>(g: &mut G) -> TrieMap<A> {
        let vec: Vec<(usize, A)> = Arbitrary::arbitrary(g);
        vec.into_iter().collect()
    }

    fn shrink(&self) -> Box<Shrinker<TrieMap<A>>+'static> {
        let vec: Vec<(usize, A)> = self.iter()
                                      .map(|(a, b)| (a, b.clone()))
                                      .collect();
        Box::new(vec.shrink().map(|v| v.into_iter().collect::<TrieMap<A>>()))
    }
}

impl<K: Arbitrary + Eq + Hash<hash_map::Hasher>, V: Arbitrary> Arbitrary for HashMap<K, V> {
    fn arbitrary<G: Gen>(g: &mut G) -> HashMap<K, V> {
        let vec: Vec<(K, V)> = Arbitrary::arbitrary(g);
        vec.into_iter().collect()
    }

    fn shrink(&self) -> Box<Shrinker<HashMap<K, V>>+'static> {
        let vec: Vec<(K, V)> = self.clone().into_iter().collect();
        Box::new(vec.shrink().map(|v| v.into_iter().collect::<HashMap<K, V>>()))
    }
}

impl Arbitrary for String {
    fn arbitrary<G: Gen>(g: &mut G) -> String {
        let size = { let s = g.size(); g.gen_range(0, s) };
        g.gen_ascii_chars().take(size).collect()
    }

    fn shrink(&self) -> Box<Shrinker<String>+'static> {
        // Shrink a string by shrinking a vector of its characters.
        let chars: Vec<char> = self.chars().collect();
        Box::new(chars.shrink().map(|x| x.into_iter().collect::<String>()))
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
    ($($ty:ty),*) => {
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
    isize, i8, i16, i32, i64
}

macro_rules! unsigned_arbitrary {
    ($($ty:ty),*) => {
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
    usize, u8, u16, u32, u64
}

impl Arbitrary for f32 {
    fn arbitrary<G: Gen>(g: &mut G) -> f32 {
        let s = g.size(); g.gen_range(-(s as f32), s as f32)
    }
    fn shrink(&self) -> Box<Shrinker<f32>+'static> {
        let it = SignedShrinker::new(*self as i32);
        Box::new(it.map(|x| x as f32))
    }
}

impl Arbitrary for f64 {
    fn arbitrary<G: Gen>(g: &mut G) -> f64 {
        let s = g.size(); g.gen_range(-(s as f64), s as f64)
    }
    fn shrink(&self) -> Box<Shrinker<f64>+'static> {
        let it = SignedShrinker::new(*self as i64);
        Box::new(it.map(|x| x as f64))
    }
}

/// Returns a sequence of vectors with each contiguous run of elements of
/// length `k` removed.
fn shuffle_vec<A: Clone>(xs: &[A], k: usize) -> Vec<Vec<A>> {
    fn shuffle<A: Clone>(xs: &[A], k: usize, n: usize) -> Vec<Vec<A>> {
        if k > n {
            return vec![];
        }
        let xs1: Vec<A> = xs[..k].iter().map(|x| x.clone()).collect();
        let xs2: Vec<A> = xs[k..].iter().map(|x| x.clone()).collect();
        if xs2.len() == 0 {
            return vec![vec![]];
        }

        let cat = |&mut: x: &Vec<A>| {
            let mut pre = xs1.clone();
            pre.extend(x.clone().into_iter());
            pre
        };
        let shuffled = shuffle(&*xs2, k, n-k);
        let mut more: Vec<Vec<A>> = shuffled.iter().map(cat).collect();
        more.insert(0, xs2);
        more
    }
    shuffle(xs, k, xs.len())
}

fn half<A: Int>(x: A) -> A { x / num::cast(2is).unwrap() }

struct SignedShrinker<A> {
    x: A,
    i: A,
}

impl<A: SignedInt + Send> SignedShrinker<A> {
    fn new(x: A) -> Box<Shrinker<A>+'static> {
        if x == Int::zero() {
            empty_shrinker()
        } else {
            let shrinker = SignedShrinker {
                x: x,
                i: half(x),
            };
            if shrinker.i.is_negative() {
                Box::new(vec![Int::zero(), shrinker.x.abs()].into_iter().chain(shrinker))
            } else {
                Box::new(vec![Int::zero()].into_iter().chain(shrinker))
            }
        }
    }
}

impl<A: SignedInt> Iterator for SignedShrinker<A> {
    type Item = A;
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

impl<A: UnsignedInt + Send> UnsignedShrinker<A> {
    fn new(x: A) -> Box<Shrinker<A>+'static> {
        if x == Int::zero() {
            empty_shrinker::<A>()
        } else {
            Box::new(vec![Int::zero()].into_iter().chain(
                UnsignedShrinker {
                    x: x,
                    i: half(x),
                }
            ))
        }
    }
}

impl<A: UnsignedInt> Iterator for UnsignedShrinker<A> {
    type Item = A;
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
    use rand;
    use std::collections::hash_map;
    use std::collections::{HashMap, HashSet};
    use std::fmt::Debug;
    use std::hash::Hash;
    use std::iter;
    use super::Arbitrary;

    #[cfg(feature = "collect_impls")]
    use collect::TrieMap;

    // Arbitrary testing. (Not much here. What else can I reasonably test?)
    #[test]
    fn arby_unit() {
        assert_eq!(arby::<()>(), ());
    }

    #[test]
    fn arby_int() {
        rep(&mut || { let n: isize = arby(); assert!(n >= -5 && n <= 5); } );
    }

    #[test]
    fn arby_uint() {
        rep(&mut || { let n: usize = arby(); assert!(n <= 5); } );
    }

    fn arby<A: super::Arbitrary>() -> A {
        super::Arbitrary::arbitrary(&mut gen())
    }

    fn gen() -> super::StdGen<rand::ThreadRng> {
        super::StdGen::new(rand::thread_rng(), 5)
    }

    fn rep<F>(f: &mut F) where F : FnMut() -> () {
        for _ in iter::range(0us, 100) {
            f()
        }
    }

    // Shrink testing.
    #[test]
    fn unit() {
        eq((), vec![]);
    }

    #[test]
    fn bools() {
        eq(false, vec![]);
        eq(true, vec![false]);
    }

    #[test]
    fn options() {
        eq(None::<()>, vec![]);
        eq(Some(false), vec![None]);
        eq(Some(true), vec![None, Some(false)]);
    }

    #[test]
    fn results() {
        // Result<A, B> doesn't implement the Hash trait, so these tests
        // depends on the order of shrunk results. Ug.
        // TODO: Fix this.
        ordered_eq(Ok::<bool, ()>(true), vec![Ok(false)]);
        ordered_eq(Err::<(), bool>(true), vec![Err(false)]);
    }

    #[test]
    fn tuples() {
        eq((false, false), vec![]);
        eq((true, false), vec![(false, false)]);
        eq((true, true), vec![(false, true), (true, false)]);
    }

    #[test]
    fn triples() {
        eq((false, false, false), vec![]);
        eq((true, false, false), vec![(false, false, false)]);
        eq((true, true, false),
           vec![(false, true, false), (true, false, false)]);
    }

    #[test]
    fn quads() {
        eq((false, false, false, false), vec![]);
        eq((true, false, false, false), vec![(false, false, false, false)]);
        eq((true, true, false, false),
            vec![(false, true, false, false), (true, false, false, false)]);
    }

    #[test]
    fn ints() {
        // TODO: Test overflow?
        eq(5is, vec![0, 3, 4]);
        eq(-5is, vec![5, 0, -3, -4]);
        eq(0is, vec![]);
    }

    #[test]
    fn ints8() {
        eq(5i8, vec![0, 3, 4]);
        eq(-5i8, vec![5, 0, -3, -4]);
        eq(0i8, vec![]);
    }

    #[test]
    fn ints16() {
        eq(5i16, vec![0, 3, 4]);
        eq(-5i16, vec![5, 0, -3, -4]);
        eq(0i16, vec![]);
    }

    #[test]
    fn ints32() {
        eq(5i32, vec![0, 3, 4]);
        eq(-5i32, vec![5, 0, -3, -4]);
        eq(0i32, vec![]);
    }

    #[test]
    fn ints64() {
        eq(5i64, vec![0, 3, 4]);
        eq(-5i64, vec![5, 0, -3, -4]);
        eq(0i64, vec![]);
    }

    #[test]
    fn uints() {
        eq(5us, vec![0, 3, 4]);
        eq(0us, vec![]);
    }

    #[test]
    fn uints8() {
        eq(5u8, vec![0, 3, 4]);
        eq(0u8, vec![]);
    }

    #[test]
    fn uints16() {
        eq(5u16, vec![0, 3, 4]);
        eq(0u16, vec![]);
    }

    #[test]
    fn uints32() {
        eq(5u32, vec![0, 3, 4]);
        eq(0u32, vec![]);
    }

    #[test]
    fn uints64() {
        eq(5u64, vec![0, 3, 4]);
        eq(0u64, vec![]);
    }

    #[test]
    fn vecs() {
        eq({let it: Vec<isize> = vec![]; it}, vec![]);
        eq({let it: Vec<Vec<isize>> = vec![vec![]]; it}, vec![vec![]]);
        eq(vec![1is], vec![vec![], vec![0]]);
        eq(vec![11is], vec![vec![], vec![0], vec![6], vec![9], vec![10]]);
        eq(
            vec![3is, 5],
            vec![vec![], vec![5], vec![3], vec![0,5], vec![2,5],
                 vec![3,0], vec![3,3], vec![3,4]]
        );
    }

    #[cfg(feature = "collect_impls")]
    #[test]
    fn triemaps() {
        eq({let it: TrieMap<isize> = TrieMap::new(); it}, vec![]);

        {
            let mut map = TrieMap::new();
            map.insert(1, 1i);

            let shrinks = vec![
                {let mut m = TrieMap::new(); m.insert(1, 0is); m},
                {let mut m = TrieMap::new(); m.insert(0, 1is); m},
                TrieMap::new()
            ];

            eq(map, shrinks);
        }
    }

    #[test]
    fn hashmaps() {
        ordered_eq({let it: HashMap<usize, isize> = HashMap::new(); it}, vec![]);

        {
            let mut map = HashMap::new();
            map.insert(1us, 1is);

            let shrinks = vec![
                HashMap::new(),
                {let mut m = HashMap::new(); m.insert(0, 1); m},
                {let mut m = HashMap::new(); m.insert(1, 0); m},
            ];

            ordered_eq(map, shrinks);
        }
    }

    #[test]
    fn chars() {
        eq('a', vec![]);
    }

    #[test]
    fn strs() {
        eq("".to_string(), vec![]);
        eq("A".to_string(), vec!["".to_string()]);
        eq("ABC".to_string(), vec!["".to_string(),
                                   "AB".to_string(),
                                   "BC".to_string(),
                                   "AC".to_string()]);
    }

    // All this jazz is for testing set equality on the results of a shrinker.
    fn eq<A: Arbitrary + Eq + Debug + Hash<hash_map::Hasher>>(s: A, v: Vec<A>) {
        let (left, right) = (shrunk(s), set(v));
        assert_eq!(left, right);
    }
    fn shrunk<A: Arbitrary + Eq + Hash<hash_map::Hasher>>(s: A) -> HashSet<A> {
        set(s.shrink().collect())
    }
    fn set<A: Eq + Hash<hash_map::Hasher>>(xs: Vec<A>) -> HashSet<A> {
        xs.into_iter().collect()
    }

    fn ordered_eq<A: Arbitrary + Eq + Debug>(s: A, v: Vec<A>) {
        let (left, right) = (s.shrink().collect::<Vec<A>>(), v);
        assert_eq!(left, right);
    }
}
