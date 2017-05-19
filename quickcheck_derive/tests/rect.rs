#[macro_use]
extern crate quickcheck;
#[macro_use]
extern crate quickcheck_derive;

#[derive(Arbitrary, Clone, Debug)]
#[arbitrary(constraint = "this.left.checked_add(this.width).is_some()")]
#[arbitrary(constraint = "this.top.checked_add(this.height).is_some()")]
pub struct Rect {
    left: u8,
    top: u8,
    width: u8,
    height: u8,
}

impl Rect {
    /// Returns the area of the rectangle, or None if it overflows a u8.
    pub fn area(&self) -> Option<u8> {
        self.width.checked_mul(self.height)
    }
}

quickcheck! {
    fn zero_area_iff_zero_dim(r: Rect) -> bool {
        (r.area() == Some(0)) == (r.height == 0 || r.width == 0)
    }
}

quickcheck! {
    fn first_constraint_holds(r: Rect) -> bool {
        r.left.checked_add(r.width).is_some()
    }
}

quickcheck! {
    fn second_constraint_holds(r: Rect) -> bool {
        r.top.checked_add(r.height).is_some()
    }
}
