use std::collections::{BTreeMap, HashMap};

pub unsafe trait Trace {}

#[macro_export]
macro_rules! unsafe_empty_trace {
    ($type:ident) => {
        unsafe impl Trace for $type {}
    };
}

unsafe_empty_trace!(bool);
unsafe_empty_trace!(i8);
unsafe_empty_trace!(i16);
unsafe_empty_trace!(i32);
unsafe_empty_trace!(i64);
unsafe_empty_trace!(i128);
unsafe_empty_trace!(u8);
unsafe_empty_trace!(u16);
unsafe_empty_trace!(u32);
unsafe_empty_trace!(u64);
unsafe_empty_trace!(f32);
unsafe_empty_trace!(f64);
unsafe_empty_trace!(String);

unsafe impl<K: Trace, V: Trace> Trace for BTreeMap<K, V> {}

unsafe impl<K: Trace, V: Trace> Trace for HashMap<K, V> {}

unsafe impl<T: Trace> Trace for Vec<T> {}

unsafe impl<T: Trace> Trace for Option<T> {}
