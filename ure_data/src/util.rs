macro_rules! all_the_tuples {
	($macro_name:ident) => {
$macro_name!();
$macro_name!(T0);
$macro_name!(T0, T1);
$macro_name!(T0, T1, T2);
$macro_name!(T0, T1, T2, T3);
$macro_name!(T0, T1, T2, T3, T4);
$macro_name!(T0, T1, T2, T3, T4, T5);
$macro_name!(T0, T1, T2, T3, T4, T5, T6);
$macro_name!(T0, T1, T2, T3, T4, T5, T6, T7);
$macro_name!(T0, T1, T2, T3, T4, T5, T6, T7, T8);
$macro_name!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9);
$macro_name!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10);
$macro_name!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11);
$macro_name!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12);
$macro_name!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13);
$macro_name!(
	T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14
);
$macro_name!(
	T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15
);
$macro_name!(
	T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16
);
	};
}

pub(crate) use all_the_tuples;

pub(crate) const fn hash_combine(a: u64, b: u64) -> u64 {
	a ^ (b + 0x9e3779b9 + (a << 6) + (a >> 2))
}