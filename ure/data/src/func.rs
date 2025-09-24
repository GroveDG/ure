use std::any::TypeId;

use crate::data::{ComponentId, Components};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FunctionId {
    inner: u64,
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

#[macro_export]
macro_rules! func {
    (
        $func_vis:vis $func_name:ident :
        ($($f_ty:ty $(: $comp_id:expr)?),* $(,)?) $(-> $fr_ty:ty)? =
        $(
            ( $( _ $(as &mut $mut_ty:ty)? $(as &$ref_ty:ty)? ),* $(,)? ) $i:expr
        ),* $(,)?
    ) => {
$func_vis const $func_name: $crate::func::Func<fn ($($f_ty),*) $(-> $fr_ty)?> = $crate::func::Func {
    components: &[$($($comp_id,)?)*],
    impls: &[$($crate::mident::mident!(
        $crate::func!(IMPL ( $(#rand $(as &mut $mut_ty)? $(as &$ref_ty)?),* ) $i)
    )),*]
};
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

// mod example {
//     use crate::data::{ComponentId, Container};

//     const A: ComponentId = ComponentId::new("example_indices");

//     func! {
//         pub EXAMPLE: (&mut dyn Container: A, bool) =
//         (_ as &mut Vec<usize>, _) example_vec,
//     }

//     fn example_vec(a: &mut Vec<usize>, b: bool) {}
// }
// pub use example::EXAMPLE;
