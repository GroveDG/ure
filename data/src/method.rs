use crate::group::Group;

// This is inspired by Axum's extractor system

pub struct Method<Args, Return = ()> {
	fn_ptr: fn(),
	call_ptr: fn(fn(), Args, &Group) -> Option<Return>,
}
impl<Args, Return> Method<Args, Return> {
	pub fn new<F: MethodTrait<Args, Return>>(fn_ptr: F) -> Self {
		assert_eq!(size_of::<F>(), size_of::<fn()>()); // Double check that F is a fn pointer.
		unsafe {
			Self {
				fn_ptr: *(&fn_ptr as *const F as *const fn()), // Erase the fn type.
				call_ptr: *(F::call as *const fn(fn(), Args, &Group) -> Option<Return>), // Erase the fn type.
			}
		}
	}
	pub fn call(self, args: Args, group: &Group) -> Option<Return> {
		(self.call_ptr)(self.fn_ptr, args, group)
	}
}

pub trait MethodTrait<Args, Return = ()> {
	fn call(self, args: Args, group: &Group) -> Option<Return>;
}

pub trait FromGroup<'a> {
	fn from_group(group: &'a Group) -> Option<Self>
	where
		Self: 'a + Sized;
}

#[macro_export]
macro_rules! impl_method {
	($($C:ident),*) => {
#[allow(unused_parens, non_snake_case)]
impl<$($C,)* Args, Return> MethodTrait<Args, Return> for fn($($C,)* Args) -> Return
where
	$(
	for<'a> $C: 'a + FromGroup<'a>,
	)*
{
	#[allow(unused_variables)]
	fn call<'a>(self, args: Args, group: &'a Group) -> Option<Return> {
		$(
		let $C = <$C as $crate::method::FromGroup<'a>>::from_group(group)?;
		)*
		Some((self)($($C,)* args))
	}
}
	};
}

impl_method!();
impl_method!(C0);
impl_method!(C0, C1);
impl_method!(C0, C1, C2);
impl_method!(C0, C1, C2, C3);
impl_method!(C0, C1, C2, C3, C4);
impl_method!(C0, C1, C2, C3, C4, C5);
impl_method!(C0, C1, C2, C3, C4, C5, C6);
impl_method!(C0, C1, C2, C3, C4, C5, C6, C7);
impl_method!(C0, C1, C2, C3, C4, C5, C6, C7, C8);
impl_method!(C0, C1, C2, C3, C4, C5, C6, C7, C8, C9);
impl_method!(C0, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10);
impl_method!(C0, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11);
impl_method!(C0, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11, C12);
impl_method!(C0, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11, C12, C13);
impl_method!(
	C0, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11, C12, C13, C14
);
impl_method!(
	C0, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11, C12, C13, C14, C15
);
impl_method!(
	C0, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11, C12, C13, C14, C15, C16
);
