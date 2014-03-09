use std::iter::{Map, Unfold};
use std::num::{one, zero};
use std::str::from_chars;
use std::vec;

pub trait Shrink<T: Iterator<Self>> {
    fn shrink(&self) -> T;
}

impl Shrink<vec::MoveItems<()>> for () {
    fn shrink(&self) -> vec::MoveItems<()> { (~[]).move_iter() }
}

impl Shrink<vec::MoveItems<bool>> for bool {
    fn shrink(&self) -> vec::MoveItems<bool> {
        match *self {
            true => (~[false]).move_iter(),
            false => (~[]).move_iter(),
        }
    }
}

struct OptionState<T> {
    state: Option<T>,
    started: bool,
}

impl<A: Shrink<Ia>, Ia: Iterator<A>>
    Shrink<Unfold<'static, Option<A>, OptionState<Ia>>>
    for Option<A>
{
    fn shrink(&self) -> Unfold<'static, Option<A>, OptionState<Ia>> {
        let init = match *self {
            None => None,
            Some(ref x) => Some(x.shrink()),
        };
        let st = OptionState{
            state: init,
            started: false,
        };
        Unfold::new(st, |st: &mut OptionState<Ia>| -> Option<Option<A>> {
            match st.state {
                None => return None,
                Some(ref mut x) => {
                    if !st.started {
                        st.started = true;
                        return Some(None)
                    }
                    match x.next() {
                        None => None,
                        Some(it) => Some(Some(it)),
                    }
                }
            }
        })
    }
}

impl<A: Shrink<Ia>, B: Shrink<Ib>, Ia: Iterator<A>, Ib: Iterator<B>>
    Shrink<Unfold<'static, Result<A, B>, Result<Ia, Ib>>>
    for Result<A, B>
{
    fn shrink(&self) -> Unfold<'static, Result<A, B>, Result<Ia, Ib>> {
        let init = match *self {
            Ok(ref a) => Ok(a.shrink()),
            Err(ref b) => Err(b.shrink()),
        };
        Unfold::new(init, |st: &mut Result<Ia, Ib>| -> Option<Result<A, B>> {
            match *st {
                Ok(ref mut a) => match a.next() {
                    None => return None,
                    Some(a) => Some(Ok(a)),
                },
                Err(ref mut b) => match b.next() {
                    None => return None,
                    Some(b) => Some(Err(b)),
                },
            }
        })
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
// I wasn't able to figure out how to do this without copying. Maybe there is
// a lifetime parameter lurking somewhere that might help.
impl<A: Shrink<Ia> + Clone, B: Shrink<Ib> + Clone,
     Ia: Iterator<A>, Ib: Iterator<B>>
    Shrink<Unfold<'static, (A, B), (A, B, Ia, Ib)>>
    for (A, B)
{
    fn shrink(&self) -> Unfold<'static, (A, B), (A, B, Ia, Ib)> {
        let (ref a, ref b) = *self;
        let init = (a.clone(), b.clone(), a.shrink(), b.shrink());
        Unfold::new(init, |st: &mut (A, B, Ia, Ib)| -> Option<(A, B)> {
            let (ref a, ref b, ref mut ia, ref mut ib) = *st;
            match ia.next() {
                Some(na) => Some((na, b.clone())),
                None => match ib.next() {
                    Some(nb) => Some((a.clone(), nb)),
                    None => None,
                }
            }
        })
    }
}

impl<Ia: Iterator<A>, A: Shrink<Ia> + Clone>
    Shrink<vec::MoveItems<~[A]>>
    for ~[A] {
    fn shrink(&self) -> vec::MoveItems<~[A]> {
        let mut xs: ~[~[A]] = ~[];
        if self.len() == 0 {
            return xs.move_iter()
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
        xs.move_iter()
    }
}

impl Shrink<vec::MoveItems<~str>> for ~str {
    fn shrink(&self) -> vec::MoveItems<~str> {
        let chars: ~[char] = self.chars().to_owned_vec();
        let mut strs: ~[~str] = ~[];
        for x in chars.shrink() {
            strs.push(from_chars(x));
        }
        strs.move_iter()
    }
}

impl Shrink<vec::MoveItems<char>> for char {
    fn shrink(&self) -> vec::MoveItems<char> { (~[]).move_iter() }
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

impl Shrink<vec::MoveItems<int>> for int {
    fn shrink(&self) -> vec::MoveItems<int> {
        shrink_signed(*self).move_iter()
    }
}

impl Shrink<vec::MoveItems<i8>> for i8 {
    fn shrink(&self) -> vec::MoveItems<i8> {
        shrink_signed(*self).move_iter()
    }
}

impl Shrink<vec::MoveItems<i16>> for i16 {
    fn shrink(&self) -> vec::MoveItems<i16> {
        shrink_signed(*self).move_iter()
    }
}

impl Shrink<vec::MoveItems<i32>> for i32 {
    fn shrink(&self) -> vec::MoveItems<i32> {
        shrink_signed(*self).move_iter()
    }
}

impl Shrink<vec::MoveItems<i64>> for i64 {
    fn shrink(&self) -> vec::MoveItems<i64> {
        shrink_signed(*self).move_iter()
    }
}

impl Shrink<vec::MoveItems<uint>> for uint {
    fn shrink(&self) -> vec::MoveItems<uint> {
        shrink_unsigned(*self).move_iter()
    }
}

impl Shrink<vec::MoveItems<u8>> for u8 {
    fn shrink(&self) -> vec::MoveItems<u8> {
        shrink_unsigned(*self).move_iter()
    }
}

impl Shrink<vec::MoveItems<u16>> for u16 {
    fn shrink(&self) -> vec::MoveItems<u16> {
        shrink_unsigned(*self).move_iter()
    }
}

impl Shrink<vec::MoveItems<u32>> for u32 {
    fn shrink(&self) -> vec::MoveItems<u32> {
        shrink_unsigned(*self).move_iter()
    }
}

impl Shrink<vec::MoveItems<u64>> for u64 {
    fn shrink(&self) -> vec::MoveItems<u64> {
        shrink_unsigned(*self).move_iter()
    }
}

impl Shrink<Map<'static, i32, f32, vec::MoveItems<i32>>> for f32 {
    fn shrink(&self) -> Map<'static, i32, f32, vec::MoveItems<i32>> {
        let it = shrink_signed(self.to_i32().unwrap()).move_iter();
        it.map(|x| x.to_f32().unwrap())
    }
}

impl Shrink<Map<'static, i64, f64, vec::MoveItems<i64>>> for f64 {
    fn shrink(&self) -> Map<'static, i64, f64, vec::MoveItems<i64>> {
        let it = shrink_signed(self.to_i64().unwrap()).move_iter();
        it.map(|x| x.to_f64().unwrap())
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
mod tests {
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
    fn eq<A: Shrink<Ia> + Eq + Show + Hash, Ia: Iterator<A>>(s: A, v: ~[A]) {
        assert_eq!(shrunk(s), set(v))
    }
    fn shrunk<A: Shrink<Ia> + Eq + Hash, Ia: Iterator<A>>(s: A) -> HashSet<A> {
        set(s.shrink().to_owned_vec())
    }
    fn set<A: Eq + Hash>(xs: ~[A]) -> HashSet<A> {
        xs.move_iter().collect()
    }

    fn ordered_eq<A: Shrink<Ia> + Eq + Show, Ia: Iterator<A>>(s: A, v: ~[A]) {
        assert_eq!(s.shrink().to_owned_vec(), v);
    }
}
