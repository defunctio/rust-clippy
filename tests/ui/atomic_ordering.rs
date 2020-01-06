#![warn(clippy::atomic_ordering)]
use std::sync::atomic::{
    AtomicBool, AtomicI16, AtomicI32, AtomicI64, AtomicI8, AtomicIsize, AtomicPtr, AtomicU16, AtomicU32, AtomicU64,
    AtomicU8, AtomicUsize, Ordering,
};

macro_rules! order_test {
    ( [ $($typ:ty),+ ], $op:ident, $ord:expr ) => (
        $(
            let g = <$typ>::new(1);
            g.$op(10, $ord);
        )+
    );
    ( $typ:ty, $op:ident, $ord:expr, $val:expr ) => {
            let g = <$typ>::new($val);
            g.$op($val, $ord);
    };
}

fn main() {
    // Test invalid signed/unsigned
    order_test!(
        [AtomicI16, AtomicI32, AtomicI64, AtomicI8, AtomicIsize],
        store,
        Ordering::Acquire
    );
    order_test!(
        [AtomicI16, AtomicI32, AtomicI64, AtomicI8, AtomicIsize],
        store,
        Ordering::AcqRel
    );
    order_test!(
        [AtomicU16, AtomicU32, AtomicU64, AtomicU8, AtomicUsize],
        store,
        Ordering::Acquire
    );
    order_test!(
        [AtomicU16, AtomicU32, AtomicU64, AtomicU8, AtomicUsize],
        store,
        Ordering::AcqRel
    );

    //AtomicBool test
    order_test!(AtomicBool, store, Ordering::Acquire, true);
    order_test!(AtomicBool, store, Ordering::AcqRel, true);

    //AtomicPtr test
}
