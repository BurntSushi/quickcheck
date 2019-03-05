#[macro_use]
extern crate quickcheck;
#[macro_use]
extern crate quickcheck_derive;

// Based on code in rust-content-security-policy
#[derive(Arbitrary, Clone, Debug, Eq, PartialEq)]
#[arbitrary(constraint = "self.is_valid()")]
pub struct Directive {
    name: String,
    value: Vec<String>,
}

impl Directive {
    pub fn is_valid(&self) -> bool {
        true
    }
}

quickcheck! {
    fn struct_constraint(t: Directive) -> bool {
        t.is_valid()
    }
}

// Since there's different code triggered for length > 8
#[derive(Arbitrary, Clone, Debug, Eq, PartialEq)]
#[arbitrary(constraint = "self.is_valid()")]
pub struct GiantDirective {
    v1: String,
    v2: String,
    v3: String,
    v4: String,
    v5: String,
    v6: String,
    v7: String,
    v8: String,
    v9: String,
    v10: String,
}

impl GiantDirective {
    pub fn is_valid(&self) -> bool {
        true
    }
}

quickcheck! {
    fn struct_constraint_giant(t: GiantDirective) -> bool {
        t.is_valid()
    }
}
