use std::{error::Error, marker::PhantomData};

use crate::{
	components::{ComponentDependency, ComponentId},
	glob::GlobItemRef,
	util::all_the_tuples,
};

pub trait TryFromGlob<'a>: Sized + ComponentDependency {
	fn from_glob(glob: GlobItemRef<'a>) -> Result<Self, Box<dyn Error>>;
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
impl<'a, $($T),*> TryFromGlob<'a> for ($($T,)*)
where $(
	Self: ComponentDependency,
	$T: TryFrom<GlobItemRef<'a>>,
	<$T as TryFrom<GlobItemRef<'a>>>::Error: Error + 'static
),*
{
	#[allow(unused_variables)]
	fn from_glob(glob: GlobItemRef<'a>) -> Result<Self, Box<dyn Error>> {
		Ok(($(
			<$T as TryFrom<GlobItemRef<'a>>>::try_from(glob)?,
		)*))
	}
}
	};
}

all_the_tuples!(impl_try_from_glob);

pub trait MethodTrait<T, Args, Return> {
	fn call_method<'a>(self, glob: GlobItemRef<'a>, args: Args) -> Result<Return, Box<dyn Error>>
	where
		T: TryFromGlob<'a>;
}

macro_rules! impl_method {
	($($T:ident),*) => {
impl<'a, $($T,)* Args, Return> ComponentDependency for &'a dyn MethodTrait<($($T,)*), Args, Return>
where
	($($T,)*): TryFromGlob<'a>
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
	fn call_method<'a>(self, glob: GlobItemRef<'a>, args: Args) -> Result<Return, Box<dyn Error>>
	where
		($($T,)*): TryFromGlob<'a>
	{
		let ($($T,)*) = <($($T,)*) as TryFromGlob<'a>>::from_glob(glob)?;
		Ok((self)($($T,)* args))
	}
}
	};
}

all_the_tuples!(impl_method);

pub type Method<Args, Return = ()> =
	dyn for<'a, 'b> Fn(GlobItemRef<'a>, &'b mut Args) -> Result<Return, Box<dyn Error>>;
