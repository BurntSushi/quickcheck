use std::char;
use std::collections::{
    BTreeMap, BTreeSet, BinaryHeap, HashMap, HashSet, LinkedList, VecDeque,
};
use std::env;
use std::ffi::{CString, OsString};
use std::hash::{BuildHasher, Hash};
use std::iter::{empty, once};
use std::net::{
    IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6,
};
use std::num::Wrapping;
use std::num::{
    NonZeroU128, NonZeroU16, NonZeroU32, NonZeroU64, NonZeroU8, NonZeroUsize,
};
use std::ops::{
    Bound, Range, RangeFrom, RangeFull, RangeInclusive, RangeTo,
    RangeToInclusive,
};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use rand::seq::SliceRandom;
use rand::{self, Rng, SeedableRng};

/// Gen represents a PRNG.
///
/// It is the source of randomness from which QuickCheck will generate
/// values. An instance of `Gen` is passed to every invocation of
/// `Arbitrary::arbitrary`, which permits callers to use lower level RNG
/// routines to generate values.
///
/// It is unspecified whether this is a secure RNG or not. Therefore, callers
/// should assume it is insecure.
pub struct Gen {
    rng: rand::rngs::SmallRng,
    size: usize,
}

impl Gen {
    /// Returns a `Gen` with the given size configuration.
    ///
    /// The `size` parameter controls the size of random values generated.
    /// For example, it specifies the maximum length of a randomly generated
    /// vector, but is and should not be used to control the range of a
    /// randomly generated number. (Unless that number is used to control the
    /// size of a data structure.)
    pub fn new(size: usize) -> Gen {
        Gen { rng: rand::rngs::SmallRng::from_entropy(), size: size }
    }

    /// Returns the size configured with this generator.
    pub fn size(&self) -> usize {
        self.size
    }

    /// Choose among the possible alternatives in the slice given. If the slice
    /// is empty, then `None` is returned. Otherwise, a non-`None` value is
    /// guaranteed to be returned.
    pub fn choose<'a, T>(&mut self, slice: &'a [T]) -> Option<&'a T> {
        slice.choose(&mut self.rng)
    }

    fn gen<T>(&mut self) -> T
    where
        rand::distributions::Standard: rand::distributions::Distribution<T>,
    {
        self.rng.gen()
    }

    fn gen_range<T, R>(&mut self, range: R) -> T
    where
        T: rand::distributions::uniform::SampleUniform,
        R: rand::distributions::uniform::SampleRange<T>,
    {
        self.rng.gen_range(range)
    }
}

/// Creates a shrinker with zero elements.
pub fn empty_shrinker<A: 'static>() -> Box<dyn Iterator<Item = A>> {
    Box::new(empty())
}

/// Creates a shrinker with a single element.
pub fn single_shrinker<A: 'static>(value: A) -> Box<dyn Iterator<Item = A>> {
    Box::new(once(value))
}

/// `Arbitrary` describes types whose values can be randomly generated and
/// shrunk.
///
/// Aside from shrinking, `Arbitrary` is different from typical RNGs in that
/// it respects `Gen::size()` for controlling how much memory a particular
/// value uses, for practical purposes. For example, `Vec::arbitrary()`
/// respects `Gen::size()` to decide the maximum `len()` of the vector.
/// This behavior is necessary due to practical speed and size limitations.
/// Conversely, `i32::arbitrary()` ignores `size()` since all `i32` values
/// require `O(1)` memory and operations between `i32`s require `O(1)` time
/// (with the exception of exponentiation).
///
/// Additionally, all types that implement `Arbitrary` must also implement
/// `Clone`.
pub trait Arbitrary: Clone + 'static {
    /// Return an arbitrary value.
    ///
    /// Implementations should respect `Gen::size()` when decisions about how
    /// big a particular value should be. Implementations should generally
    /// defer to other `Arbitrary` implementations to generate other random
    /// values when necessary. The `Gen` type also offers a few RNG helper
    /// routines.
    fn arbitrary(g: &mut Gen) -> Self;

    /// Return an iterator of values that are smaller than itself.
    ///
    /// The way in which a value is "smaller" is implementation defined. In
    /// some cases, the interpretation is obvious: shrinking an integer should
    /// produce integers smaller than itself. Others are more complex, for
    /// example, shrinking a `Vec` should both shrink its size and shrink its
    /// component values.
    ///
    /// The iterator returned should be bounded to some reasonable size.
    ///
    /// It is always correct to return an empty iterator, and indeed, this
    /// is the default implementation. The downside of this approach is that
    /// witnesses to failures in properties will be more inscrutable.
    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        empty_shrinker()
    }
}

impl Arbitrary for () {
    fn arbitrary(_: &mut Gen) -> () {
        ()
    }
}

impl Arbitrary for bool {
    fn arbitrary(g: &mut Gen) -> bool {
        g.gen()
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = bool>> {
        if *self {
            single_shrinker(false)
        } else {
            empty_shrinker()
        }
    }
}

impl<A: Arbitrary> Arbitrary for Option<A> {
    fn arbitrary(g: &mut Gen) -> Option<A> {
        if g.gen() {
            None
        } else {
            Some(Arbitrary::arbitrary(g))
        }
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Option<A>>> {
        match *self {
            None => empty_shrinker(),
            Some(ref x) => {
                let chain = single_shrinker(None).chain(x.shrink().map(Some));
                Box::new(chain)
            }
        }
    }
}

impl<A: Arbitrary, B: Arbitrary> Arbitrary for Result<A, B> {
    fn arbitrary(g: &mut Gen) -> Result<A, B> {
        if g.gen() {
            Ok(Arbitrary::arbitrary(g))
        } else {
            Err(Arbitrary::arbitrary(g))
        }
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Result<A, B>>> {
        match *self {
            Ok(ref x) => {
                let xs = x.shrink();
                let tagged = xs.map(Ok);
                Box::new(tagged)
            }
            Err(ref x) => {
                let xs = x.shrink();
                let tagged = xs.map(Err);
                Box::new(tagged)
            }
        }
    }
}

macro_rules! impl_arb_for_single_tuple {
    ($(($type_param:ident, $tuple_index:tt),)*) => {
        impl<$($type_param),*> Arbitrary for ($($type_param,)*)
            where $($type_param: Arbitrary,)*
        {
            fn arbitrary(g: &mut Gen) -> ($($type_param,)*) {
                (
                    $(
                        $type_param::arbitrary(g),
                    )*
                )
            }

            fn shrink(&self) -> Box<dyn Iterator<Item=($($type_param,)*)>> {
                let iter = ::std::iter::empty();
                $(
                    let cloned = self.clone();
                    let iter = iter.chain(
                        self.$tuple_index.shrink().map(move |shr_value| {
                            let mut result = cloned.clone();
                            result.$tuple_index = shr_value;
                            result
                        })
                    );
                )*
                Box::new(iter)
            }
        }
    };
}

macro_rules! impl_arb_for_tuples {
    (@internal [$($acc:tt,)*]) => { };
    (@internal [$($acc:tt,)*] ($type_param:ident, $tuple_index:tt), $($rest:tt,)*) => {
        impl_arb_for_single_tuple!($($acc,)* ($type_param, $tuple_index),);
        impl_arb_for_tuples!(@internal [$($acc,)* ($type_param, $tuple_index),] $($rest,)*);
    };
    ($(($type_param:ident, $tuple_index:tt),)*) => {
        impl_arb_for_tuples!(@internal [] $(($type_param, $tuple_index),)*);
    };
}

impl_arb_for_tuples! {
    (A, 0),
    (B, 1),
    (C, 2),
    (D, 3),
    (E, 4),
    (F, 5),
    (G, 6),
    (H, 7),
}

impl<A: Arbitrary> Arbitrary for Vec<A> {
    fn arbitrary(g: &mut Gen) -> Vec<A> {
        let size = {
            let s = g.size();
            g.gen_range(0..s)
        };
        (0..size).map(|_| A::arbitrary(g)).collect()
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Vec<A>>> {
        VecShrinker::new(self.clone())
    }
}

///Iterator which returns successive attempts to shrink the vector `seed`
struct VecShrinker<A> {
    seed: Vec<A>,
    /// How much which is removed when trying with smaller vectors
    size: usize,
    /// The end of the removed elements
    offset: usize,
    /// The shrinker for the element at `offset` once shrinking of individual
    /// elements are attempted
    element_shrinker: Box<dyn Iterator<Item = A>>,
}

impl<A: Arbitrary> VecShrinker<A> {
    fn new(seed: Vec<A>) -> Box<dyn Iterator<Item = Vec<A>>> {
        let es = match seed.get(0) {
            Some(e) => e.shrink(),
            None => return empty_shrinker(),
        };
        let size = seed.len();
        Box::new(VecShrinker {
            seed: seed,
            size: size,
            offset: size,
            element_shrinker: es,
        })
    }

    /// Returns the next shrunk element if any, `offset` points to the index
    /// after the returned element after the function returns
    fn next_element(&mut self) -> Option<A> {
        loop {
            match self.element_shrinker.next() {
                Some(e) => return Some(e),
                None => match self.seed.get(self.offset) {
                    Some(e) => {
                        self.element_shrinker = e.shrink();
                        self.offset += 1;
                    }
                    None => return None,
                },
            }
        }
    }
}

impl<A> Iterator for VecShrinker<A>
where
    A: Arbitrary,
{
    type Item = Vec<A>;
    fn next(&mut self) -> Option<Vec<A>> {
        // Try with an empty vector first
        if self.size == self.seed.len() {
            self.size /= 2;
            self.offset = self.size;
            return Some(vec![]);
        }
        if self.size != 0 {
            // Generate a smaller vector by removing the elements between
            // (offset - size) and offset
            let xs1 = self.seed[..(self.offset - self.size)]
                .iter()
                .chain(&self.seed[self.offset..])
                .cloned()
                .collect();
            self.offset += self.size;
            // Try to reduce the amount removed from the vector once all
            // previous sizes tried
            if self.offset > self.seed.len() {
                self.size /= 2;
                self.offset = self.size;
            }
            Some(xs1)
        } else {
            // A smaller vector did not work so try to shrink each element of
            // the vector instead Reuse `offset` as the index determining which
            // element to shrink

            // The first element shrinker is already created so skip the first
            // offset (self.offset == 0 only on first entry to this part of the
            // iterator)
            if self.offset == 0 {
                self.offset = 1
            }

            match self.next_element() {
                Some(e) => Some(
                    self.seed[..self.offset - 1]
                        .iter()
                        .cloned()
                        .chain(Some(e).into_iter())
                        .chain(self.seed[self.offset..].iter().cloned())
                        .collect(),
                ),
                None => None,
            }
        }
    }
}

impl<K: Arbitrary + Ord, V: Arbitrary> Arbitrary for BTreeMap<K, V> {
    fn arbitrary(g: &mut Gen) -> BTreeMap<K, V> {
        let vec: Vec<(K, V)> = Arbitrary::arbitrary(g);
        vec.into_iter().collect()
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = BTreeMap<K, V>>> {
        let vec: Vec<(K, V)> = self.clone().into_iter().collect();
        Box::new(
            vec.shrink().map(|v| v.into_iter().collect::<BTreeMap<K, V>>()),
        )
    }
}

impl<
        K: Arbitrary + Eq + Hash,
        V: Arbitrary,
        S: BuildHasher + Default + Clone + 'static,
    > Arbitrary for HashMap<K, V, S>
{
    fn arbitrary(g: &mut Gen) -> Self {
        let vec: Vec<(K, V)> = Arbitrary::arbitrary(g);
        vec.into_iter().collect()
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        let vec: Vec<(K, V)> = self.clone().into_iter().collect();
        Box::new(vec.shrink().map(|v| v.into_iter().collect::<Self>()))
    }
}

impl<T: Arbitrary + Ord> Arbitrary for BTreeSet<T> {
    fn arbitrary(g: &mut Gen) -> BTreeSet<T> {
        let vec: Vec<T> = Arbitrary::arbitrary(g);
        vec.into_iter().collect()
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = BTreeSet<T>>> {
        let vec: Vec<T> = self.clone().into_iter().collect();
        Box::new(vec.shrink().map(|v| v.into_iter().collect::<BTreeSet<T>>()))
    }
}

impl<T: Arbitrary + Ord> Arbitrary for BinaryHeap<T> {
    fn arbitrary(g: &mut Gen) -> BinaryHeap<T> {
        let vec: Vec<T> = Arbitrary::arbitrary(g);
        vec.into_iter().collect()
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = BinaryHeap<T>>> {
        let vec: Vec<T> = self.clone().into_iter().collect();
        Box::new(
            vec.shrink().map(|v| v.into_iter().collect::<BinaryHeap<T>>()),
        )
    }
}

impl<T: Arbitrary + Eq + Hash, S: BuildHasher + Default + Clone + 'static>
    Arbitrary for HashSet<T, S>
{
    fn arbitrary(g: &mut Gen) -> Self {
        let vec: Vec<T> = Arbitrary::arbitrary(g);
        vec.into_iter().collect()
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        let vec: Vec<T> = self.clone().into_iter().collect();
        Box::new(vec.shrink().map(|v| v.into_iter().collect::<Self>()))
    }
}

impl<T: Arbitrary> Arbitrary for LinkedList<T> {
    fn arbitrary(g: &mut Gen) -> LinkedList<T> {
        let vec: Vec<T> = Arbitrary::arbitrary(g);
        vec.into_iter().collect()
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = LinkedList<T>>> {
        let vec: Vec<T> = self.clone().into_iter().collect();
        Box::new(
            vec.shrink().map(|v| v.into_iter().collect::<LinkedList<T>>()),
        )
    }
}

impl<T: Arbitrary> Arbitrary for VecDeque<T> {
    fn arbitrary(g: &mut Gen) -> VecDeque<T> {
        let vec: Vec<T> = Arbitrary::arbitrary(g);
        vec.into_iter().collect()
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = VecDeque<T>>> {
        let vec: Vec<T> = self.clone().into_iter().collect();
        Box::new(vec.shrink().map(|v| v.into_iter().collect::<VecDeque<T>>()))
    }
}

impl Arbitrary for IpAddr {
    fn arbitrary(g: &mut Gen) -> IpAddr {
        let ipv4: bool = g.gen();
        if ipv4 {
            IpAddr::V4(Arbitrary::arbitrary(g))
        } else {
            IpAddr::V6(Arbitrary::arbitrary(g))
        }
    }
}

impl Arbitrary for Ipv4Addr {
    fn arbitrary(g: &mut Gen) -> Ipv4Addr {
        Ipv4Addr::new(g.gen(), g.gen(), g.gen(), g.gen())
    }
}

impl Arbitrary for Ipv6Addr {
    fn arbitrary(g: &mut Gen) -> Ipv6Addr {
        Ipv6Addr::new(
            g.gen(),
            g.gen(),
            g.gen(),
            g.gen(),
            g.gen(),
            g.gen(),
            g.gen(),
            g.gen(),
        )
    }
}

impl Arbitrary for SocketAddr {
    fn arbitrary(g: &mut Gen) -> SocketAddr {
        SocketAddr::new(Arbitrary::arbitrary(g), g.gen())
    }
}

impl Arbitrary for SocketAddrV4 {
    fn arbitrary(g: &mut Gen) -> SocketAddrV4 {
        SocketAddrV4::new(Arbitrary::arbitrary(g), g.gen())
    }
}

impl Arbitrary for SocketAddrV6 {
    fn arbitrary(g: &mut Gen) -> SocketAddrV6 {
        SocketAddrV6::new(Arbitrary::arbitrary(g), g.gen(), g.gen(), g.gen())
    }
}

impl Arbitrary for PathBuf {
    fn arbitrary(g: &mut Gen) -> PathBuf {
        // use some real directories as guesses, so we may end up with
        // actual working directories in case that is relevant.
        let here =
            env::current_dir().unwrap_or(PathBuf::from("/test/directory"));
        let temp = env::temp_dir();
        #[allow(deprecated)]
        let home = env::home_dir().unwrap_or(PathBuf::from("/home/user"));
        let mut p = g
            .choose(&[
                here,
                temp,
                home,
                PathBuf::from("."),
                PathBuf::from(".."),
                PathBuf::from("../../.."),
                PathBuf::new(),
            ])
            .unwrap()
            .to_owned();
        p.extend(Vec::<OsString>::arbitrary(g).iter());
        p
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = PathBuf>> {
        let mut shrunk = vec![];
        let mut popped = self.clone();
        if popped.pop() {
            shrunk.push(popped);
        }

        // Iterating over a Path performs a small amount of normalization.
        let normalized = self.iter().collect::<PathBuf>();
        if normalized.as_os_str() != self.as_os_str() {
            shrunk.push(normalized);
        }

        // Add the canonicalized variant only if canonicalizing the path
        // actually does something, making it (hopefully) smaller. Also, ignore
        // canonicalization if canonicalization errors.
        if let Ok(canonicalized) = self.canonicalize() {
            if canonicalized.as_os_str() != self.as_os_str() {
                shrunk.push(canonicalized);
            }
        }

        Box::new(shrunk.into_iter())
    }
}

impl Arbitrary for OsString {
    fn arbitrary(g: &mut Gen) -> OsString {
        OsString::from(String::arbitrary(g))
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = OsString>> {
        let mystring: String = self.clone().into_string().unwrap();
        Box::new(mystring.shrink().map(|s| OsString::from(s)))
    }
}

impl Arbitrary for String {
    fn arbitrary(g: &mut Gen) -> String {
        let size = {
            let s = g.size();
            g.gen_range(0..s)
        };
        (0..size).map(|_| char::arbitrary(g)).collect()
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = String>> {
        // Shrink a string by shrinking a vector of its characters.
        let chars: Vec<char> = self.chars().collect();
        Box::new(chars.shrink().map(|x| x.into_iter().collect::<String>()))
    }
}

impl Arbitrary for CString {
    fn arbitrary(g: &mut Gen) -> Self {
        let size = {
            let s = g.size();
            g.gen_range(0..s)
        };
        // Use either random bytes or random UTF-8 encoded codepoints.
        let utf8: bool = g.gen();
        if utf8 {
            CString::new(
                (0..)
                    .map(|_| char::arbitrary(g))
                    .filter(|&c| c != '\0')
                    .take(size)
                    .collect::<String>(),
            )
        } else {
            CString::new(
                (0..)
                    .map(|_| u8::arbitrary(g))
                    .filter(|&c| c != b'\0')
                    .take(size)
                    .collect::<Vec<u8>>(),
            )
        }
        .expect("null characters should have been filtered out")
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = CString>> {
        // Use the implementation for a vec here, but make sure null characters
        // are filtered out.
        Box::new(VecShrinker::new(self.as_bytes().to_vec()).map(|bytes| {
            CString::new(
                bytes.into_iter().filter(|&c| c != 0).collect::<Vec<u8>>(),
            )
            .expect("null characters should have been filtered out")
        }))
    }
}

impl Arbitrary for char {
    fn arbitrary(g: &mut Gen) -> char {
        let mode = g.gen_range(0..100);
        match mode {
            0..=49 => {
                // ASCII + some control characters
                g.gen_range(0..0xB0) as u8 as char
            }
            50..=59 => {
                // Unicode BMP characters
                loop {
                    if let Some(x) = char::from_u32(g.gen_range(0..0x10000)) {
                        return x;
                    }
                    // ignore surrogate pairs
                }
            }
            60..=84 => {
                // Characters often used in programming languages
                g.choose(&[
                    ' ', ' ', ' ', '\t', '\n', '~', '`', '!', '@', '#', '$',
                    '%', '^', '&', '*', '(', ')', '_', '-', '=', '+', '[',
                    ']', '{', '}', ':', ';', '\'', '"', '\\', '|', ',', '<',
                    '>', '.', '/', '?', '0', '1', '2', '3', '4', '5', '6',
                    '7', '8', '9',
                ])
                .unwrap()
                .to_owned()
            }
            85..=89 => {
                // Tricky Unicode, part 1
                g.choose(&[
                    '\u{0149}', // a deprecated character
                    '\u{fff0}', // some of "Other, format" category:
                    '\u{fff1}',
                    '\u{fff2}',
                    '\u{fff3}',
                    '\u{fff4}',
                    '\u{fff5}',
                    '\u{fff6}',
                    '\u{fff7}',
                    '\u{fff8}',
                    '\u{fff9}',
                    '\u{fffA}',
                    '\u{fffB}',
                    '\u{fffC}',
                    '\u{fffD}',
                    '\u{fffE}',
                    '\u{fffF}',
                    '\u{0600}',
                    '\u{0601}',
                    '\u{0602}',
                    '\u{0603}',
                    '\u{0604}',
                    '\u{0605}',
                    '\u{061C}',
                    '\u{06DD}',
                    '\u{070F}',
                    '\u{180E}',
                    '\u{110BD}',
                    '\u{1D173}',
                    '\u{e0001}', // tag
                    '\u{e0020}', //  tag space
                    '\u{e000}',
                    '\u{e001}',
                    '\u{ef8ff}', // private use
                    '\u{f0000}',
                    '\u{ffffd}',
                    '\u{ffffe}',
                    '\u{fffff}',
                    '\u{100000}',
                    '\u{10FFFD}',
                    '\u{10FFFE}',
                    '\u{10FFFF}',
                    // "Other, surrogate" characters are so that very special
                    // that they are not even allowed in safe Rust,
                    //so omitted here
                    '\u{3000}', // ideographic space
                    '\u{1680}',
                    // other space characters are already covered by two next
                    // branches
                ])
                .unwrap()
                .to_owned()
            }
            90..=94 => {
                // Tricky unicode, part 2
                char::from_u32(g.gen_range(0x2000..0x2070)).unwrap()
            }
            95..=99 => {
                // Completely arbitrary characters
                g.gen()
            }
            _ => unreachable!(),
        }
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = char>> {
        Box::new((*self as u32).shrink().filter_map(char::from_u32))
    }
}

macro_rules! unsigned_shrinker {
    ($ty:ty) => {
        mod shrinker {
            pub struct UnsignedShrinker {
                x: $ty,
                i: $ty,
            }

            impl UnsignedShrinker {
                pub fn new(x: $ty) -> Box<dyn Iterator<Item = $ty>> {
                    if x == 0 {
                        super::empty_shrinker()
                    } else {
                        Box::new(
                            vec![0]
                                .into_iter()
                                .chain(UnsignedShrinker { x: x, i: x / 2 }),
                        )
                    }
                }
            }

            impl Iterator for UnsignedShrinker {
                type Item = $ty;
                fn next(&mut self) -> Option<$ty> {
                    if self.x - self.i < self.x {
                        let result = Some(self.x - self.i);
                        self.i = self.i / 2;
                        result
                    } else {
                        None
                    }
                }
            }
        }
    };
}

macro_rules! unsigned_problem_values {
    ($t:ty) => {
        &[<$t>::min_value(), 1, <$t>::max_value()]
    };
}

macro_rules! unsigned_arbitrary {
    ($($ty:tt),*) => {
        $(
            impl Arbitrary for $ty {
                fn arbitrary(g: &mut Gen) -> $ty {
                    match g.gen_range(0..10) {
                        0 => {
                            *g.choose(unsigned_problem_values!($ty)).unwrap()
                        },
                        _ => g.gen()
                    }
                }
                fn shrink(&self) -> Box<dyn Iterator<Item=$ty>> {
                    unsigned_shrinker!($ty);
                    shrinker::UnsignedShrinker::new(*self)
                }
            }
        )*
    }
}

unsigned_arbitrary! {
    usize, u8, u16, u32, u64, u128
}

macro_rules! signed_shrinker {
    ($ty:ty) => {
        mod shrinker {
            pub struct SignedShrinker {
                x: $ty,
                i: $ty,
            }

            impl SignedShrinker {
                pub fn new(x: $ty) -> Box<dyn Iterator<Item = $ty>> {
                    if x == 0 {
                        super::empty_shrinker()
                    } else {
                        let shrinker = SignedShrinker { x: x, i: x / 2 };
                        let mut items = vec![0];
                        if shrinker.i < 0 && shrinker.x != <$ty>::MIN {
                            items.push(shrinker.x.abs());
                        }
                        Box::new(items.into_iter().chain(shrinker))
                    }
                }
            }

            impl Iterator for SignedShrinker {
                type Item = $ty;
                fn next(&mut self) -> Option<$ty> {
                    if self.x == <$ty>::MIN
                        || (self.x - self.i).abs() < self.x.abs()
                    {
                        let result = Some(self.x - self.i);
                        self.i = self.i / 2;
                        result
                    } else {
                        None
                    }
                }
            }
        }
    };
}

macro_rules! signed_problem_values {
    ($t:ty) => {
        &[<$t>::min_value(), 0, <$t>::max_value()]
    };
}

macro_rules! signed_arbitrary {
    ($($ty:tt),*) => {
        $(
            impl Arbitrary for $ty {
                fn arbitrary(g: &mut Gen) -> $ty {
                    match g.gen_range(0..10) {
                        0 => {
                            *g.choose(signed_problem_values!($ty)).unwrap()
                        },
                        _ => g.gen()
                    }
                }
                fn shrink(&self) -> Box<dyn Iterator<Item=$ty>> {
                    signed_shrinker!($ty);
                    shrinker::SignedShrinker::new(*self)
                }
            }
        )*
    }
}

signed_arbitrary! {
    isize, i8, i16, i32, i64, i128
}

macro_rules! float_problem_values {
    ($path:path) => {{
        // hack. see: https://github.com/rust-lang/rust/issues/48067
        use $path as p;
        &[p::NAN, p::NEG_INFINITY, p::MIN, -0., 0., p::MAX, p::INFINITY]
    }};
}

macro_rules! float_arbitrary {
    ($($t:ty, $path:path, $shrinkable:ty),+) => {$(
        impl Arbitrary for $t {
            fn arbitrary(g: &mut Gen) -> $t {
                match g.gen_range(0..10) {
                    0 => *g.choose(float_problem_values!($path)).unwrap(),
                    _ => {
                        use $path as p;
                        let exp = g.gen_range((0.)..p::MAX_EXP as i16 as $t);
                        let mantissa = g.gen_range((1.)..2.);
                        let sign = *g.choose(&[-1., 1.]).unwrap();
                        sign * mantissa * exp.exp2()
                    }
                }
            }
            fn shrink(&self) -> Box<dyn Iterator<Item = $t>> {
                signed_shrinker!($shrinkable);
                let it = shrinker::SignedShrinker::new(*self as $shrinkable);
                Box::new(it.map(|x| x as $t))
            }
        }
    )*};
}

float_arbitrary!(f32, std::f32, i32, f64, std::f64, i64);

macro_rules! unsigned_non_zero_shrinker {
    ($ty:tt) => {
        mod shrinker {
            pub struct UnsignedNonZeroShrinker {
                x: $ty,
                i: $ty,
            }

            impl UnsignedNonZeroShrinker {
                pub fn new(x: $ty) -> Box<dyn Iterator<Item = $ty>> {
                    debug_assert!(x > 0);

                    if x == 1 {
                        super::empty_shrinker()
                    } else {
                        Box::new(
                            std::iter::once(1).chain(
                                UnsignedNonZeroShrinker { x: x, i: x / 2 },
                            ),
                        )
                    }
                }
            }

            impl Iterator for UnsignedNonZeroShrinker {
                type Item = $ty;

                fn next(&mut self) -> Option<$ty> {
                    if self.x - self.i < self.x {
                        let result = Some(self.x - self.i);
                        self.i = self.i / 2;
                        result
                    } else {
                        None
                    }
                }
            }
        }
    };
}

macro_rules! unsigned_non_zero_arbitrary {
    ($($ty:tt => $inner:tt),*) => {
        $(
            impl Arbitrary for $ty {
                fn arbitrary(g: &mut Gen) -> $ty {
                    let mut v: $inner = g.gen();
                    if v == 0 {
                        v += 1;
                    }
                    $ty::new(v).expect("non-zero value contsturction failed")
                }

                fn shrink(&self) -> Box<dyn Iterator<Item = $ty>> {
                    unsigned_non_zero_shrinker!($inner);
                    Box::new(shrinker::UnsignedNonZeroShrinker::new(self.get())
                        .map($ty::new)
                        .map(Option::unwrap))
                }
            }
        )*
    }
}

unsigned_non_zero_arbitrary! {
    NonZeroUsize => usize,
    NonZeroU8    => u8,
    NonZeroU16   => u16,
    NonZeroU32   => u32,
    NonZeroU64   => u64,
    NonZeroU128  => u128
}

impl<T: Arbitrary> Arbitrary for Wrapping<T> {
    fn arbitrary(g: &mut Gen) -> Wrapping<T> {
        Wrapping(T::arbitrary(g))
    }
    fn shrink(&self) -> Box<dyn Iterator<Item = Wrapping<T>>> {
        Box::new(self.0.shrink().map(|inner| Wrapping(inner)))
    }
}

impl<T: Arbitrary> Arbitrary for Bound<T> {
    fn arbitrary(g: &mut Gen) -> Bound<T> {
        match g.gen_range(0..3) {
            0 => Bound::Included(T::arbitrary(g)),
            1 => Bound::Excluded(T::arbitrary(g)),
            _ => Bound::Unbounded,
        }
    }
    fn shrink(&self) -> Box<dyn Iterator<Item = Bound<T>>> {
        match *self {
            Bound::Included(ref x) => {
                Box::new(x.shrink().map(Bound::Included))
            }
            Bound::Excluded(ref x) => {
                Box::new(x.shrink().map(Bound::Excluded))
            }
            Bound::Unbounded => empty_shrinker(),
        }
    }
}

impl<T: Arbitrary + Clone + PartialOrd> Arbitrary for Range<T> {
    fn arbitrary(g: &mut Gen) -> Range<T> {
        Arbitrary::arbitrary(g)..Arbitrary::arbitrary(g)
    }
    fn shrink(&self) -> Box<dyn Iterator<Item = Range<T>>> {
        Box::new(
            (self.start.clone(), self.end.clone()).shrink().map(|(s, e)| s..e),
        )
    }
}

impl<T: Arbitrary + Clone + PartialOrd> Arbitrary for RangeInclusive<T> {
    fn arbitrary(g: &mut Gen) -> RangeInclusive<T> {
        Arbitrary::arbitrary(g)..=Arbitrary::arbitrary(g)
    }
    fn shrink(&self) -> Box<dyn Iterator<Item = RangeInclusive<T>>> {
        Box::new(
            (self.start().clone(), self.end().clone())
                .shrink()
                .map(|(s, e)| s..=e),
        )
    }
}

impl<T: Arbitrary + Clone + PartialOrd> Arbitrary for RangeFrom<T> {
    fn arbitrary(g: &mut Gen) -> RangeFrom<T> {
        Arbitrary::arbitrary(g)..
    }
    fn shrink(&self) -> Box<dyn Iterator<Item = RangeFrom<T>>> {
        Box::new(self.start.clone().shrink().map(|start| start..))
    }
}

impl<T: Arbitrary + Clone + PartialOrd> Arbitrary for RangeTo<T> {
    fn arbitrary(g: &mut Gen) -> RangeTo<T> {
        ..Arbitrary::arbitrary(g)
    }
    fn shrink(&self) -> Box<dyn Iterator<Item = RangeTo<T>>> {
        Box::new(self.end.clone().shrink().map(|end| ..end))
    }
}

impl<T: Arbitrary + Clone + PartialOrd> Arbitrary for RangeToInclusive<T> {
    fn arbitrary(g: &mut Gen) -> RangeToInclusive<T> {
        ..=Arbitrary::arbitrary(g)
    }
    fn shrink(&self) -> Box<dyn Iterator<Item = RangeToInclusive<T>>> {
        Box::new(self.end.clone().shrink().map(|end| ..=end))
    }
}

impl Arbitrary for RangeFull {
    fn arbitrary(_: &mut Gen) -> RangeFull {
        ..
    }
}

impl Arbitrary for Duration {
    fn arbitrary(gen: &mut Gen) -> Self {
        let seconds = gen.gen_range(0..gen.size() as u64);
        let nanoseconds = gen.gen_range(0..1_000_000);
        Duration::new(seconds, nanoseconds)
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        Box::new(
            (self.as_secs(), self.subsec_nanos())
                .shrink()
                .map(|(secs, nanos)| Duration::new(secs, nanos % 1_000_000)),
        )
    }
}

impl<A: Arbitrary> Arbitrary for Box<A> {
    fn arbitrary(g: &mut Gen) -> Box<A> {
        Box::new(A::arbitrary(g))
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Box<A>>> {
        Box::new((**self).shrink().map(Box::new))
    }
}

impl<A: Arbitrary + Sync> Arbitrary for Arc<A> {
    fn arbitrary(g: &mut Gen) -> Arc<A> {
        Arc::new(A::arbitrary(g))
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Arc<A>>> {
        Box::new((**self).shrink().map(Arc::new))
    }
}

impl Arbitrary for SystemTime {
    fn arbitrary(gen: &mut Gen) -> Self {
        let after_epoch = bool::arbitrary(gen);
        let duration = Duration::arbitrary(gen);
        if after_epoch {
            UNIX_EPOCH + duration
        } else {
            UNIX_EPOCH - duration
        }
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        let duration = match self.duration_since(UNIX_EPOCH) {
            Ok(duration) => duration,
            Err(e) => e.duration(),
        };
        Box::new(
            duration
                .shrink()
                .flat_map(|d| vec![UNIX_EPOCH + d, UNIX_EPOCH - d]),
        )
    }
}

#[cfg(test)]
mod test {
    use std::collections::{
        BTreeMap, BTreeSet, BinaryHeap, HashMap, HashSet, LinkedList, VecDeque,
    };
    use std::fmt::Debug;
    use std::hash::Hash;
    use std::num::Wrapping;
    use std::path::PathBuf;

    use super::{Arbitrary, Gen};

    #[test]
    fn arby_unit() {
        assert_eq!(arby::<()>(), ());
    }

    macro_rules! arby_int {
        ( $signed:expr, $($t:ty),+) => {$(
            let mut arbys = (0..1_000_000).map(|_| arby::<$t>());
            let mut problems = if $signed {
                    signed_problem_values!($t).iter()
                } else {
                    unsigned_problem_values!($t).iter()
                };
            assert!(problems.all(|p| arbys.any(|arby| arby == *p)),
                "Arbitrary does not generate all problematic values");
            let max = <$t>::max_value();
            let mid = (max + <$t>::min_value()) / 2;
            // split full range of $t into chunks
            // Arbitrary must return some value in each chunk
            let double_chunks: $t = 9;
            let chunks = double_chunks * 2;  // chunks must be even
            let lim: Box<dyn Iterator<Item=$t>> = if $signed {
                Box::new((0..=chunks)
                        .map(|idx| idx - chunks / 2)
                        .map(|x| mid + max / (chunks / 2) * x))
            } else {
                Box::new((0..=chunks).map(|idx| max / chunks * idx))
            };
            let mut lim = lim.peekable();
            while let (Some(low), Some(&high)) = (lim.next(), lim.peek()) {
                assert!(arbys.any(|arby| low <= arby && arby <= high),
                    "Arbitrary doesn't generate numbers in {}..={}", low, high)
            }
        )*};
    }

    #[test]
    fn arby_int() {
        arby_int!(true, i8, i16, i32, i64, isize, i128);
    }

    #[test]
    fn arby_uint() {
        arby_int!(false, u8, u16, u32, u64, usize, u128);
    }

    macro_rules! arby_float {
        ($($t:ty, $path:path),+) => {$({
            use $path as p;
            let mut arbys = (0..1_000_000).map(|_| arby::<$t>());
            //NaN != NaN
            assert!(arbys.any(|f| f.is_nan()),
                "Arbitrary does not generate the problematic value NaN"
            );
            for p in float_problem_values!($path).iter().filter(|f| !f.is_nan()) {
                assert!(arbys.any(|arby| arby == *p),
                    "Arbitrary does not generate the problematic value {}",
                    p
                );
            }
            // split full range of $t into chunks
            // Arbitrary must return some value in each chunk
            let double_chunks: i8 = 9;
            let chunks = double_chunks * 2;  // chunks must be even
            let lim = (-double_chunks..=double_chunks)
                        .map(|idx| <$t>::from(idx))
                        .map(|idx| p::MAX/(<$t>::from(chunks/2)) * idx);
            let mut lim = lim.peekable();
            while let (Some(low), Some(&high)) = (lim.next(), lim.peek()) {
                assert!(
                    arbys.any(|arby| low <= arby && arby <= high),
                    "Arbitrary doesn't generate numbers in {:e}..={:e}",
                    low,
                    high,
                )
            }
        })*};
    }

    #[test]
    fn arby_float() {
        arby_float!(f32, std::f32, f64, std::f64);
    }

    fn arby<A: Arbitrary>() -> A {
        Arbitrary::arbitrary(&mut Gen::new(5))
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
        eq(
            (true, true, false),
            vec![(false, true, false), (true, false, false)],
        );
    }

    #[test]
    fn quads() {
        eq((false, false, false, false), vec![]);
        eq((true, false, false, false), vec![(false, false, false, false)]);
        eq(
            (true, true, false, false),
            vec![(false, true, false, false), (true, false, false, false)],
        );
    }

    #[test]
    fn ints() {
        // TODO: Test overflow?
        eq(5isize, vec![0, 3, 4]);
        eq(-5isize, vec![5, 0, -3, -4]);
        eq(0isize, vec![]);
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
    fn ints128() {
        eq(5i128, vec![0, 3, 4]);
        eq(-5i128, vec![5, 0, -3, -4]);
        eq(0i128, vec![]);
    }

    #[test]
    fn uints() {
        eq(5usize, vec![0, 3, 4]);
        eq(0usize, vec![]);
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
    fn uints128() {
        eq(5u128, vec![0, 3, 4]);
        eq(0u128, vec![]);
    }

    macro_rules! define_float_eq {
        ($ty:ty) => {
            fn eq(s: $ty, v: Vec<$ty>) {
                let shrunk: Vec<$ty> = s.shrink().collect();
                for n in v {
                    let found = shrunk.iter().any(|&i| i == n);
                    if !found {
                        panic!(format!(
                            "Element {:?} was not found \
                             in shrink results {:?}",
                            n, shrunk
                        ));
                    }
                }
            }
        };
    }

    #[test]
    fn floats32() {
        define_float_eq!(f32);

        eq(0.0, vec![]);
        eq(-0.0, vec![]);
        eq(1.0, vec![0.0]);
        eq(2.0, vec![0.0, 1.0]);
        eq(-2.0, vec![0.0, 2.0, -1.0]);
        eq(1.5, vec![0.0]);
    }

    #[test]
    fn floats64() {
        define_float_eq!(f64);

        eq(0.0, vec![]);
        eq(-0.0, vec![]);
        eq(1.0, vec![0.0]);
        eq(2.0, vec![0.0, 1.0]);
        eq(-2.0, vec![0.0, 2.0, -1.0]);
        eq(1.5, vec![0.0]);
    }

    #[test]
    fn wrapping_ints32() {
        eq(Wrapping(5i32), vec![Wrapping(0), Wrapping(3), Wrapping(4)]);
        eq(
            Wrapping(-5i32),
            vec![Wrapping(5), Wrapping(0), Wrapping(-3), Wrapping(-4)],
        );
        eq(Wrapping(0i32), vec![]);
    }

    #[test]
    fn vecs() {
        eq(
            {
                let it: Vec<isize> = vec![];
                it
            },
            vec![],
        );
        eq(
            {
                let it: Vec<Vec<isize>> = vec![vec![]];
                it
            },
            vec![vec![]],
        );
        eq(vec![1isize], vec![vec![], vec![0]]);
        eq(vec![11isize], vec![vec![], vec![0], vec![6], vec![9], vec![10]]);
        eq(
            vec![3isize, 5],
            vec![
                vec![],
                vec![5],
                vec![3],
                vec![0, 5],
                vec![2, 5],
                vec![3, 0],
                vec![3, 3],
                vec![3, 4],
            ],
        );
    }

    macro_rules! map_tests {
        ($name:ident, $ctor:expr) => {
            #[test]
            fn $name() {
                ordered_eq($ctor, vec![]);

                {
                    let mut map = $ctor;
                    map.insert(1usize, 1isize);

                    let shrinks = vec![
                        $ctor,
                        {
                            let mut m = $ctor;
                            m.insert(0, 1);
                            m
                        },
                        {
                            let mut m = $ctor;
                            m.insert(1, 0);
                            m
                        },
                    ];

                    ordered_eq(map, shrinks);
                }
            }
        };
    }

    map_tests!(btreemap, BTreeMap::<usize, isize>::new());
    map_tests!(hashmap, HashMap::<usize, isize>::new());

    macro_rules! list_tests {
        ($name:ident, $ctor:expr, $push:ident) => {
            #[test]
            fn $name() {
                ordered_eq($ctor, vec![]);

                {
                    let mut list = $ctor;
                    list.$push(2usize);

                    let shrinks = vec![
                        $ctor,
                        {
                            let mut m = $ctor;
                            m.$push(0);
                            m
                        },
                        {
                            let mut m = $ctor;
                            m.$push(1);
                            m
                        },
                    ];

                    ordered_eq(list, shrinks);
                }
            }
        };
    }

    list_tests!(btreesets, BTreeSet::<usize>::new(), insert);
    list_tests!(hashsets, HashSet::<usize>::new(), insert);
    list_tests!(linkedlists, LinkedList::<usize>::new(), push_back);
    list_tests!(vecdeques, VecDeque::<usize>::new(), push_back);

    #[test]
    fn binaryheaps() {
        ordered_eq(
            BinaryHeap::<usize>::new().into_iter().collect::<Vec<_>>(),
            vec![],
        );

        {
            let mut heap = BinaryHeap::<usize>::new();
            heap.push(2usize);

            let shrinks = vec![vec![], vec![0], vec![1]];

            ordered_eq(heap.into_iter().collect::<Vec<_>>(), shrinks);
        }
    }

    #[test]
    fn chars() {
        eq('\x00', vec![]);
    }

    // All this jazz is for testing set equality on the results of a shrinker.
    fn eq<A: Arbitrary + Eq + Debug + Hash>(s: A, v: Vec<A>) {
        let (left, right) = (shrunk(s), set(v));
        assert_eq!(left, right);
    }
    fn shrunk<A: Arbitrary + Eq + Hash>(s: A) -> HashSet<A> {
        set(s.shrink())
    }
    fn set<A: Hash + Eq, I: IntoIterator<Item = A>>(xs: I) -> HashSet<A> {
        xs.into_iter().collect()
    }

    fn ordered_eq<A: Arbitrary + Eq + Debug>(s: A, v: Vec<A>) {
        let (left, right) = (s.shrink().collect::<Vec<A>>(), v);
        assert_eq!(left, right);
    }

    #[test]
    fn bounds() {
        use std::ops::Bound::*;
        for i in -5..=5 {
            ordered_eq(Included(i), i.shrink().map(Included).collect());
            ordered_eq(Excluded(i), i.shrink().map(Excluded).collect());
        }
        eq(Unbounded::<i32>, vec![]);
    }

    #[test]
    fn ranges() {
        ordered_eq(0..0, vec![]);
        ordered_eq(1..1, vec![0..1, 1..0]);
        ordered_eq(3..5, vec![0..5, 2..5, 3..0, 3..3, 3..4]);
        ordered_eq(5..3, vec![0..3, 3..3, 4..3, 5..0, 5..2]);
        ordered_eq(3.., vec![0.., 2..]);
        ordered_eq(..3, vec![..0, ..2]);
        ordered_eq(.., vec![]);
        ordered_eq(3..=5, vec![0..=5, 2..=5, 3..=0, 3..=3, 3..=4]);
        ordered_eq(..=3, vec![..=0, ..=2]);
    }

    #[test]
    fn pathbuf() {
        ordered_eq(
            PathBuf::from("/home/foo//.././bar"),
            vec![
                PathBuf::from("/home/foo//.."),
                PathBuf::from("/home/foo/../bar"),
            ],
        );
    }
}
