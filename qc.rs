use std::iter::Unfold;
use std::rand;
use std::vec;

trait Arbitrary {
    fn arbitrary<R: rand::Rng>(rng: &mut R) -> Self;
}

trait Shrink<T: Iterator<Self>> {
    fn shrink(&self) -> T;
}

impl<T: rand::Rand> Arbitrary for T {
    fn arbitrary<R: rand::Rng>(rng: &mut R) -> T { rng.gen() }
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

fn main() {
    // let mut rng = ~rand::rng(); 
    let r: Option<Result<Option<bool>, ()>> = Some(Ok(Some(true)));
    println!("{}", Some(true).shrink().to_owned_vec());
    println!("{}", r.shrink().to_owned_vec());
}
