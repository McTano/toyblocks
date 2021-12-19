// use std::iter::Step;

// fn pair_range<A, B>((l1, l2): (A, B), (u1, u2): (A, B)) -> impl Iterator<Item = (A, B)>
// where
//     A: Step + PartialOrd + Sized,
//     B: Step + PartialOrd + Sized,
// {
//     let a = (l1..u1).zip((l2..u2));
//     a
// }

// where
//     A: PartialOrd,
//     B: PartialOrd,
// {
//     (l1..=u1).zip((l2..=u2));
// }
