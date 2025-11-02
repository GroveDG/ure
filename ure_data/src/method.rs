use crate::{
	components::{ComponentDependency, ComponentId},
	group::Group,
};

// This is inspired by Axum's extractor system

#[derive(Debug)]
pub struct Method<Args, Return = ()> {
	fn_ptr: fn(),
	call_ptr: fn(fn(), &Group, Args) -> Option<Return>,
	dependencies: fn() -> Vec<ComponentId>,
}
impl<Args, Return> Clone for Method<Args, Return> {
	fn clone(&self) -> Self {
		Self {
			fn_ptr: self.fn_ptr.clone(),
			call_ptr: self.call_ptr.clone(),
			dependencies: self.dependencies.clone(),
		}
	}
}
impl<Args, Return> Copy for Method<Args, Return> {}
impl<Args, Return> Method<Args, Return> {
	pub const fn new<'a, F: Copy + MethodTrait<'a, Args, Return>>(fn_ptr: F) -> Self {
		// assert_eq!(size_of::<F>(), size_of::<fn()>()); // Double check that F is a fn pointer.
		unsafe {
			Self {
				fn_ptr: *(&fn_ptr as *const F as *const fn()), // Erase the fn type.
				call_ptr: std::mem::transmute(F::call_method as fn(F, &'a Group, Args) -> Option<Return>), // Erase the fn type.
				dependencies: F::dependencies,
			}
		}
	}
	pub fn call(&self, group: &Group, args: Args) -> Option<Return> {
		(self.call_ptr)(self.fn_ptr, group, args)
	}
	pub fn dependencies(&self) -> Vec<ComponentId> {
		(self.dependencies)()
	}
	pub(crate) unsafe fn erase(self) -> Method<()> {
		unsafe { std::mem::transmute(self) }
	}
}
impl<'a, Args, F: MethodTrait<'a, Args> + Copy> From<F> for Method<Args> {
	fn from(value: F) -> Self {
		Method::new(value)
	}
}

pub trait MethodTrait<'a, Args, Return = ()>: ComponentDependency {
	fn call_method(self, group: &'a Group, args: Args) -> Option<Return>;
}
pub trait FromGroup<'a>: ComponentDependency {
	fn from_group(group: &'a Group) -> Option<Self>
	where
		Self: 'a + Sized;
}

#[macro_export]
macro_rules! impl_method {
	($($C:ident),*) => {
impl<$($C,)* Args, Return> ComponentDependency for fn($($C,)* Args) -> Return
where
	$(
	$C: $crate::method::ComponentDependency,
	)*
{
	#[allow(unused_mut)]
	fn dependencies() -> Vec<ComponentId> {
		let mut dependencies = Vec::new();
		$(
		dependencies.append(&mut <$C as $crate::method::ComponentDependency>::dependencies());
		)*
		dependencies
	}
}
#[allow(non_snake_case)]
impl<'a, $($C,)* Args, Return> MethodTrait<'a, Args, Return> for fn($($C,)* Args) -> Return
where
	$(
	$C: 'a + FromGroup<'a>,
	)*
{
	#[allow(unused_variables)]
	fn call_method(self, group: &'a Group, args: Args) -> Option<Return> {
		$(
		let $C = <$C as $crate::method::FromGroup>::from_group(group)?;
		)*
		Some((self)($($C,)* args))
	}
}
	};
}

crate::util::all_the_tuples!(impl_method);

// pub const trait UniqueFnCoercion<A> {
// 	type FnPtr: Copy;
// 	fn coerce(&self) -> Self::FnPtr {
// 		unsafe { *(self as *const Self as *const Self::FnPtr) }
// 	}
// }

// macro_rules! impl_fn_coercion {
// 	($($A:ident),*) => {
// impl<$($A,)* Return, F: FnOnce($($A),*) -> Return> UniqueFnCoercion<($($A,)*)> for F {
// 	type FnPtr = fn($($A),*) -> Return;
// }
// 	};
// }

// crate::util::all_the_tuples!(impl_fn_coercion);
