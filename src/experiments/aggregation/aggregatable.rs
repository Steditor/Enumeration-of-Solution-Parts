use num::{traits::AsPrimitive, Bounded};
use serde::Serialize;

// Aggregatable data can be averaged as f64, has a maximum and minimum value
// and provides max/min functions to find the larger/smaller of two values
pub trait Aggregatable: Copy + Bounded + PartialOrd + Serialize {
    fn to_aggregatable(&self) -> f64;
}

macro_rules! impl_forward_asPrimitiveF64 {
    ($($t:ty)*) => ($(
        impl Aggregatable for $t {
            #[inline]
            fn to_aggregatable(&self) -> f64 {
                self.as_()
            }
        }
    )*)
}
impl_forward_asPrimitiveF64!(u8 i8 u16 i16 u32 i32 f32 u64 i64 f64 u128 i128);

impl Aggregatable for () {
    fn to_aggregatable(&self) -> f64 {
        0.0 // neutral element for addition
    }
}
