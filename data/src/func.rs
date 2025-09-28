use std::{
    any::{Any, TypeId},
    collections::HashMap,
    hash::Hash,
};

use const_fnv1a_hash::fnv1a_hash_str_64;
use nohash_hasher::BuildNoHashHasher;

use crate::data::{ComponentId, Components};

pub enum ImplError {
	MissingComponent,
	InvalidContainers,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FunctionId {
    inner: u64,
}
impl FunctionId {
    pub const fn new(name: &'static str) -> Self {
        Self {
            inner: fnv1a_hash_str_64(name),
        }
    }
}
impl Hash for FunctionId {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write_u64(self.inner);
    }
}
impl nohash_hasher::IsEnabled for FunctionId {}

pub struct Impl<Func> {
    pub component_types: &'static [fn() -> TypeId],
    pub implementation: Func,
}
pub struct Func<F: 'static> {
    pub(crate) id: FunctionId,
    pub(crate) components: &'static [ComponentId],
    pub(crate) impls: &'static [Impl<F>],
}

trait Implement {
	fn id(&self) -> FunctionId;
    fn implement_any(&self, components: &Components) -> Result<Box<dyn Any>, ImplError>;
}
impl<F: Any + Clone> Implement for Func<F> {
	fn id(&self) -> FunctionId {
		self.id
	}
    fn implement_any(&self, components: &Components) -> Result<Box<dyn Any>, ImplError> {
        Ok(Box::new(self.implement(components)?))
    }
}

impl<F: Clone> Func<F> {
    pub fn implement(&self, data: &Components) -> Result<F, ImplError> {
        let mut components = Vec::with_capacity(self.components.len());
        for component in self.components {
			let Some(component) = data.get(component) else {
				return Err(ImplError::MissingComponent);
			};
            components.push(component.type_id());
        }
        for i in self.impls {
            let i_c = i.component_types.iter().map(|f| (f)());
            if components.iter().copied().eq(i_c) {
                return Ok(i.implementation.clone());
            }
        }
        return Err(ImplError::InvalidContainers);
    }
}

pub type FuncAndImpl = (&'static dyn Implement, Box<dyn Any>);

#[derive(Default)]
pub struct Functions {
    functions: HashMap<FunctionId, FuncAndImpl, BuildNoHashHasher<FunctionId>>,
}
impl Functions {
    pub fn implement<F: Any + Clone>(
        &mut self,
        func: &'static Func<F>,
        components: &Components,
    ) -> Option<ImplError> {
        let f = match func.implement(components) {
			Ok(f) => f,
			Err(e) => return Some(e),
		};
        self.functions.insert(func.id, (func, Box::new(f)));
        None
    }
    pub fn unimplement(&mut self, id: &FunctionId) {
        self.functions.remove(id);
    }
    pub fn reimplement(&mut self, components: &Components) -> Vec<(FunctionId, ImplError)> {
		let mut errors = Vec::new();
        for (id, (func, i)) in self.functions.iter_mut() {
            match func.implement_any(components) {
                Ok(new_impl) => *i = new_impl,
				Err(e) => errors.push((*id, e)),
            }
        }
		errors
    }
	pub fn get<F: Any>(&mut self, func: &'static Func<F>) -> Option<&F> {
		self.functions.get(&func.id)?.1.downcast_ref()
	}
}

#[macro_export]
macro_rules! func {
	(
		$func_vis:vis $func_name:ident :
		($($f_ty:ty $(: $comp_id:expr)?),* $(,)?) $(-> $fr_ty:ty)? =
		$(
			( $( _ $(as &mut $mut_ty:ty)? $(as &$ref_ty:ty)? ),* $(,)? ) $i:expr
		),* $(,)?
	) => {
$func_vis const $func_name: $crate::func::Func<fn ($($f_ty),*) $(-> $fr_ty)?> = crate::mident::mident!($crate::func::Func {
	id: $crate::func::FunctionId::new(stringify!(#downcase $func_name)),
	components: &[$($($comp_id,)?)*],
	impls: &[$(
		$crate::func!(IMPL ( $(#rand $(as &mut $mut_ty)? $(as &$ref_ty)?),* ) $i)
	),*]
});
	};
	(IMPL ( $( $arg_name:ident $(as &mut $mut_ty:ty)? $(as &$ref_ty:ty)? ),* $(,)? ) $i:expr) => {
$crate::func::Impl {
	component_types: &[$($(std::any::TypeId::of::<$mut_ty>,)? $(std::any::TypeId::of::<$ref_ty>,)?)*],
	implementation: |$($arg_name),*| ($i)($(
		$arg_name $(.downcast_mut::<$mut_ty>().unwrap())? $(.downcast_ref::<$ref_ty>().unwrap())?
	),*),
}
	};
}

pub type Method = fn(&mut dyn Any, &[&dyn Any]);

#[macro_export]
macro_rules! method {
	(
		$func_vis:vis $func_name:ident :
		(&mut $mut_comp_id:expr $(, $comp_id:expr)* $(,)?) =
		$(
			( $mut_ty:ty $(, $ref_ty:ty)* $(,)? ) $i:expr
		),* $(,)?
	) => {
$func_vis const $func_name: $crate::func::Func<fn (&mut dyn Any, &[&dyn Any])> = $crate::func::Func {
	id: $crate::func::FunctionId::new(stringify!(#downcase $func_name)),
	components: &[$mut_comp_id $(, $comp_id)*],
	impls: &[$(
		$crate::method!(IMPL ( $mut_ty $(, $ref_ty)* ) $i)
	),*]
};
	};
	(IMPL ( $mut_ty:ty $(, $ref_ty:ty )* ) $i:expr) => {
$crate::func::Impl {
	component_types: &[std::any::TypeId::of::<$mut_ty> $(, std::any::TypeId::of::<$ref_ty>)*],
	#[allow(unused_variables)]
	implementation: |mut_arg, args| {
		let mut args = args.into_iter();
		($i)(
			mut_arg.downcast_mut::<$mut_ty>().unwrap()
			$(, args.next().unwrap().downcast_ref::<$ref_ty>().unwrap())*
		)
	},
}
	};
}

mod example {
    use crate::data::ComponentId;
    use std::any::Any;

    const A: ComponentId = ComponentId::new("example_indices");

    func! {
        pub EXAMPLE: (&mut dyn Any: A, bool) =
        (_ as &mut Vec<usize>, _) example_vec,
    }

    fn example_vec(a: &mut Vec<usize>, b: bool) {}
}
pub use example::EXAMPLE;

mod example2 {
    use crate::data::ComponentId;
    use std::any::Any;

    const A: ComponentId = ComponentId::new("example_indices");
    const B: ComponentId = ComponentId::new("example_indices");
    const C: ComponentId = ComponentId::new("example_indices");

    method! {
        pub EXAMPLE: (&mut A, B, C) =
        (Vec<usize>, Vec<usize>, Vec<usize>) example_vec,
    }

    fn example_vec(a: &mut Vec<usize>, b: &Vec<usize>, c: &Vec<usize>) {}
}
