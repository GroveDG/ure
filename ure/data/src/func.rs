use std::{collections::HashMap, fmt::Display};

use nohash_hasher::BuildNoHashHasher;

use crate::{ComponentId, Data, DataBox};

pub type FuncAndImpl = (&'static Func, Impl);

#[derive(Default)]
pub struct Functions {
    funcs: HashMap<FuncId, FuncAndImpl, BuildNoHashHasher<FuncId>>,
}

impl Functions {
    pub fn add(&mut self, data: &Data, func: &'static Func) -> Option<ImplError> {
        let im = match (func.implement)(data) {
            Ok(i) => i,
            Err(e) => return Some(e),
        };
        self.funcs.insert(func.id, (func, im));
        None
    }
    pub fn reimpl(&mut self, data: &Data) -> Option<ImplError> {
        for (func, i) in self.funcs.values_mut() {
            *i = match (func.implement)(data) {
                Ok(i) => i,
                Err(e) => return Some(e),
            };
        }
        None
    }
}

pub type Impl = fn(&[&DataBox], &mut [&mut DataBox]);
pub type Implr = fn(&Data) -> Result<Impl, ImplError>;
pub type FuncId = u64;

pub struct Func {
    pub(crate) name: &'static str,
    pub(crate) id: FuncId,
    pub(crate) implement: Implr,
    pub(crate) data: &'static [ComponentId],
    pub(crate) data_mut: &'static [ComponentId],
}

impl Func {
    pub const fn new(
        name: &'static str,
        data: &'static [ComponentId],
        data_mut: &'static [ComponentId],
        implr: Implr,
    ) -> Self {
        Self {
            name,
            id: const_fnv1a_hash::fnv1a_hash_str_64(name),
            implement: implr,
            data,
            data_mut,
        }
    }
}

#[derive(Debug)]
pub enum ImplError {
    MissingComponent(&'static str),
    NoValidSignature,
}
impl Display for ImplError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ImplError::MissingComponent(name) => write!(f, "Missing component: {name}"),
            ImplError::NoValidSignature => f.write_str("No implementation has a signature valid for this group."),
        }
    }
}
impl std::error::Error for ImplError {}

#[macro_export]
macro_rules! func {
    (
        $name:ident ([$($data:expr),* $(,)?], mut [$($data_mut:expr),* $(,)?])
        $(
            (
                [$($comp:ident : $($cont:ty)? $(=> $data_ty:ty)?),*],
                mut [$($comp_mut:ident : $($cont_mut:ty)? $(=> $data_mut_ty:ty)?),*]
            )
            $body:block
        )+
    ) => {
        const $name: $crate::Func = $crate::Func::new(
            stringify!($name),
            &[
                $($data.id),*
            ],
            &[
                $($data_mut.id),*
            ],
            implr
        );
        const INNER: &'static [std::any::TypeId] = &[
            $($data.inner_type),*
        ];
        const INNER_MUT: &'static [std::any::TypeId] = &[
            $($data_mut.inner_type),*
        ];
        #[allow(unused_variables, unused_assignments, unused_mut)]
        const IMPLS: &'static [$crate::Impl] = &[
            $({
                {
                    const INNER_: &'static [std::any::TypeId] = &[$(std::any::TypeId::of::<$(<$cont as $crate::DataSpecific>::Inner)? $($data_ty)?>()),*];
                    const INNER_MUT_: &'static [std::any::TypeId] = &[$(std::any::TypeId::of::<$(<$cont_mut as $crate::DataSpecific>::Inner)? $($data_mut_ty)?>()),*];
                    const _: () = if ( INNER != INNER_ ) { panic!("Mismatched types in implementation signature. The error is in the immutable segment.") };
                    const _: () = if ( INNER_MUT != INNER_MUT_ ) { panic!("Mismatched types in an implementation's signature. The error is in the mutable segment.") };
                }
                |data, data_mut| {
                    let mut data_iter = data.iter();
                    $(
                        $(
                            let $comp = data_iter.next().unwrap().downcast_ref::<$cont, _>().unwrap();
                        )?
                        $(
                            let $comp = data_iter.next().unwrap().cast_ref::<$data_ty>().unwrap();
                        )?
                    )*
                    let mut data_iter_mut = data_mut.iter_mut();
                    $(
                        $(
                            let $comp_mut = data_iter_mut.next().unwrap().downcast_mut::<$cont_mut, _>().unwrap();
                        )?
                        $(
                            let $comp_mut = data_iter_mut.next().unwrap().cast_mut::<$data_mut_ty>().unwrap();
                        )?
                    )*
                    $body
                }
            },)*
        ];
        #[allow(unused_variables, unused_assignments, unused_mut)]
        fn implr(data: &$crate::Data) -> Result<$crate::Impl, ImplError> {
            let data_any: [&$crate::DataBox; _] = [
                $(data.get(&$data.id).ok_or($crate::ImplError::MissingComponent($data.name))?,)*
            ];
            let data_any_mut: [&$crate::DataBox; _] = [
                $(data.get(&$data_mut.id).ok_or($crate::ImplError::MissingComponent($data_mut.name))?,)*
            ];
            let mut impl_i = 0;
            $(
                let mut typed: bool = true;
                {
                    let mut data_iter = data_any.iter();
                    $(
                        $(
                            typed &= data_iter.next()?.is::<$cont, _>();
                        )?
                        $(
                            typed &= data_iter.next()?.inner_is::<$data_ty>();
                        )?
                    )*
                    let mut data_iter_mut = data_any_mut.iter();
                    $(
                        $(
                            typed &= data_iter.next()?.is::<$cont_mut, _>();
                        )?
                        $(
                            typed &= data_iter.next()?.inner_is::<$data_mut_ty>();
                        )?
                    )*
                }
                if typed {
                    return Some(IMPLS[impl_i]);
                }
                impl_i += 1;
            )+
            Err($crate::ImplError::NoValidSignature)
        }
    };
}
