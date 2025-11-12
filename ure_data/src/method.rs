use std::error::Error;

use crate::{
	components::{ComponentDependency, ComponentId},
	glob::GlobuleRef,
	util::all_the_tuples,
};

pub trait TryFromGlob<'a, 'b>: Sized + ComponentDependency {
	fn from_glob(glob: GlobuleRef<'a, 'b>) -> Result<Self, Box<dyn Error>>;
}

macro_rules! impl_try_from_glob {
	($($T:ident),*) => {
impl<$($T: ComponentDependency),*> ComponentDependency for ($($T,)*) {
	#[allow(unused_mut)]
	fn dependencies() -> Vec<ComponentId> {
		let mut dependencies = Vec::new();
		$(
		dependencies.extend(<$T as ComponentDependency>::dependencies());
		)*
		dependencies
	}
}
impl<'a, 'b, $($T),*> TryFromGlob<'a, 'b> for ($($T,)*)
where $(
	Self: ComponentDependency,
	$T: TryFrom<GlobuleRef<'a, 'b>>,
	<$T as TryFrom<GlobuleRef<'a, 'b>>>::Error: Error + 'static
),*
{
	#[allow(unused_variables)]
	fn from_glob(glob: GlobuleRef<'a, 'b>) -> Result<Self, Box<dyn Error>> {
		Ok(($(
			<$T as TryFrom<GlobuleRef<'a, 'b>>>::try_from(glob.clone())?,
		)*))
	}
}
	};
}

all_the_tuples!(impl_try_from_glob);

pub trait MethodTrait<T, Args, Return> {
	fn call_method<'a, 'b>(self, glob: GlobuleRef<'a, 'b>, args: Args) -> Result<Return, Box<dyn Error>>
	where
		T: TryFromGlob<'a, 'b>;
}

macro_rules! impl_method {
	($($T:ident),*) => {
impl<'a, 'b, $($T,)* Args, Return> ComponentDependency for &'a dyn MethodTrait<($($T,)*), Args, Return>
where
	($($T,)*): TryFromGlob<'a, 'b>
{
	fn dependencies() -> Vec<ComponentId> {
		<($($T,)*)>::dependencies()
	}
}
#[allow(unused_parens)]
#[allow(non_snake_case)]
impl<$($T,)* Args, Return, F: FnOnce($($T,)* Args) -> Return> MethodTrait<($($T,)*), Args, Return>
	for F
{
	#[allow(unused_variables)]
	fn call_method<'a, 'b>(self, glob: GlobuleRef<'a, 'b>, args: Args) -> Result<Return, Box<dyn Error>>
	where
		($($T,)*): TryFromGlob<'a, 'b>
	{
		let ($($T,)*) = <($($T,)*) as TryFromGlob<'a, 'b>>::from_glob(glob)?;
		Ok((self)($($T,)* args))
	}
}
	};
}

all_the_tuples!(impl_method);

pub type Method<Args, Return = ()> =
	dyn for<'a, 'b> Fn(GlobuleRef<'a, 'b>, &'b mut Args) -> Result<Return, Box<dyn Error>>;
