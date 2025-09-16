use std::{collections::HashMap, fmt::Display};

use nohash_hasher::BuildNoHashHasher;

use crate::{Component, Data};

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
    pub fn call(&self, data: &mut Data, func: &'static Func) {
        let Some((_, im)) = self.funcs.get(&func.id) else {
            return
        };
        (im)(data);
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

pub type Impl = fn(&mut Data);
pub type Implr = fn(&Data) -> Result<Impl, ImplError>;
pub type FuncId = u64;

pub struct Func {
    pub(crate) name: &'static str,
    pub(crate) id: FuncId,
    pub(crate) implement: Implr,
    pub(crate) components: &'static [&'static Component],
}

impl Func {
    pub const fn new(
        name: &'static str,
        components: &'static [&'static Component],
        implr: Implr,
    ) -> Self {
        Self {
            name,
            id: const_fnv1a_hash::fnv1a_hash_str_64(name),
            implement: implr,
            components,
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
            ImplError::NoValidSignature => {
                f.write_str("No implementation has a signature valid for this group.")
            }
        }
    }
}
impl std::error::Error for ImplError {}

#[macro_export]
macro_rules! func {
    (
        $name:ident ($($components:expr),+ $(,)?)
        $(
            ( $($comp:ident : $($cont:ty)? $(=> $data_ty:ty)?),+ )
            $body:block
        )+
    ) => {
        const COMPONENTS: &'static [&'static $crate::Component] = &[
            $($components),+
        ];
        const COMPONENTS_IDS: [&'static $crate::ComponentId; COMPONENTS.len()] = [
            $(&$components.id),+
        ];
        #[allow(unused_variables, unused_assignments, unused_mut)]
        const IMPLS: &'static [$crate::Impl] = &[
            $({
                |data| {
                    let mut components = data.get_mut_disjoint(COMPONENTS_IDS);
                    let mut i = 0;
                    $(
                        let $comp = components[i].as_mut().unwrap()
                        $(
                            .downcast_mut::<$cont, _>().unwrap();
                        )?
                        $(
                            .cast_mut::<$data_ty>().unwrap();
                        )?
                        i += 1;
                    )+
                    $body
                }
            },)+
        ];
        fn implr(data: &$crate::Data) -> Result<$crate::Impl, ImplError> {
            let data_boxes: [&$crate::DataBox; _] = [
                $(data.get(&$components.id).ok_or(
                    $crate::ImplError::MissingComponent($components.name)
                )?,)+
            ];
            let mut impl_i = 0;
            $(
                let mut typed: bool = true;
                $(
                    let mut i = 0;
                    typed &= data_boxes[i]
                    $(
                        .is::<$cont, _>();
                    )?
                    $(
                        .inner_is::<$data_ty>();
                    )?
                    i += 1;
                )+
                if typed {
                    return Ok(IMPLS[impl_i]);
                }
                impl_i += 1;
            )+
            Err($crate::ImplError::NoValidSignature)
        }


        const $name: $crate::Func = $crate::Func::new(
            stringify!($name),
            COMPONENTS,
            implr
        );
    };
}

const C1: Component = Component::new::<usize>("indices");
use crate::DataRef;

func!{
    EXAMPLE (&C1)
    (indices: => usize)
    {
        indices.read(0);
    }
}