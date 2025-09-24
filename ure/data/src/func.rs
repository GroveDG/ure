use std::{
    any::{Any, TypeId},
    marker::PhantomData,
};

use crate::data::{ComponentId, Components, Container};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ConstantId {
    inner: u64,
}
pub struct Constant<T: Any> {
    id: ConstantId,
    _marker: PhantomData<T>,
}

pub struct Function<T: Any> {
    inner: Constant<T>,
}

pub struct Impl<Func> {
    pub component_types: &'static [fn() -> TypeId],
    pub implementation: Func,
}
pub struct Func<F: 'static> {
    components: &'static [ComponentId],
    impls: &'static [Impl<F>],
}
impl<F: Copy> Func<F> {
    pub fn implement(&self, data: &Components) -> Option<F> {
        let mut components = Vec::with_capacity(self.components.len());
        for component in self.components {
            components.push(data.get(component)?.type_id());
        }
        for i in self.impls {
            let i_c = i.component_types.iter().map(|f| (f)());
            if components.iter().copied().eq(i_c) {
                return Some(i.implementation);
            }
        }
        None
    }
}

const A: ComponentId = ComponentId::new("example");
const B: ComponentId = ComponentId::new("example");

const EXAMPLE: Func<fn(&mut dyn Container, &dyn Container, bool) -> f32> = Func {
    components: &[A, B],
    impls: &[Impl {
        component_types: &[TypeId::of::<Vec<usize>>, TypeId::of::<Vec<usize>>],
        implementation: |a, b, c| {
            (|a: &mut Vec<usize>, b: &Vec<usize>, c: bool| -> f32 { 0.0 })(
                a.downcast_mut().unwrap(),
                b.downcast_ref().unwrap(),
                c,
            )
        },
    }],
};

/*
(...) (-> T)? [...]
|...| [...]
*/

#[macro_export]
macro_rules! func {
    (
        ($($f_ty:ty $(: $comp_id:expr)?),* $(,)?) $(-> $fr_ty:ty)?;
        $(
            ($($(as mut $comp_mut_ty:ty)? $(as ref $comp_ref_ty:ty)? $(as $arg_ty:ty)?),* $(,)?) $i:expr;
        )*
    ) => {
const EXAMPLES: $crate::func::Func<fn ($($f_ty),*) $(-> $fr_ty)?> = $crate::func::Func {
    components: &[$($($comp_id,)?)*],
    impls: &[$crate::mident::mident!($(
        $crate::func!(IMPL ($(#rand : $(as mut $comp_mut_ty)? $(as ref $comp_ref_ty)? $(as $arg_ty)?),*) $i)
    ),*)]
};
    };
    (IMPL ($($arg_name:ident : $(as mut $comp_mut_ty:ty)? $(as ref $comp_ref_ty:ty)? $(as $arg_ty:ty)?),* $(,)?) $i:expr) => {
Impl {
    component_types: &[$($(TypeId::of::<$comp_mut_ty>,)? $(TypeId::of::<$comp_ref_ty>,)?)*],
    implementation: |$($arg_name),*| ($i as fn($($(&mut $comp_mut_ty)? $(&$comp_ref_ty)? $($arg_ty)?),*))($(
        $arg_name $(.downcast_mut::<$comp_mut_ty>().unwrap())? $(.downcast_ref::<$comp_ref_ty>().unwrap())?
    ),*),
}
    };
}

func!{
    (&mut dyn Container: A, bool);
    (as mut Vec<usize>, as bool) |a, b| {
        a.reverse();
    };
}