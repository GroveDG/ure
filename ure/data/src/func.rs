use std::{collections::HashMap, fmt::Display};

use nohash_hasher::BuildNoHashHasher;

use crate::data::{Component, Data, ValidRange};

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
    pub fn get(&self, func: &'static Func) -> Option<Impl> {
        Some(self.funcs.get(&func.id)?.1)
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

pub type Impl = fn(&mut Data, ValidRange);
pub type Implr = fn(&Data) -> Result<Impl, ImplError>;
pub type FuncId = u64;

#[derive(Debug)]
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
        ($range:ident, $($components:expr),+ $(,)?)
        $(
            ( $($comp:ident : $($cont:ty)? $(=> $data_ty:ty)?),+ )
            $body:block
        )+
    ) => {
    {
        const COMPONENTS: &'static [&'static $crate::data::Component] = &[
            $($components),+
        ];
        const COMPONENTS_IDS: [&'static $crate::data::ComponentId; COMPONENTS.len()] = [
            $(&$components.id),+
        ];
        #[allow(unused_variables, unused_assignments, unused_mut)]
        const IMPLS: &'static [$crate::func::Impl] = &[
            $({
                |data, $range| {
                    let mut components = data.get_mut_disjoint(COMPONENTS_IDS);
                    let mut i = 0;
                    $(
                        let (mooring, $comp) = components[i].as_mut().unwrap()
                        $(
                            .downcast_mut::<$cont, _>().unwrap();
                        )?
                        $(
                            .slice_ref::<$data_ty>().unwrap();
                        )?
                        i += 1;
                    )+
                    $body
                }
            },)+
        ];
        #[allow(unused_assignments)]
        fn implr(data: &$crate::data::Data) -> Result<$crate::func::Impl, ImplError> {
            let data_boxes: [&$crate::data::DataBox; _] = [
                $(data.get(&$components.id).ok_or(
                    $crate::func::ImplError::MissingComponent($components.name)
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
            Err($crate::func::ImplError::NoValidSignature)
        }


        $crate::func::Func::new(
            stringify!($name),
            COMPONENTS,
            implr
        )
    }
    };
}

const C1: Component = Component::new::<usize>("indices");

pub const EXAMPLE: Func = {
    const COMPONENTS: &'static[&'static crate::data::Component] =  &[(&C1)];
    const COMPONENTS_IDS:[&'static crate::data::ComponentId;
    COMPONENTS.len()] = [&(&C1).id];
    #[allow(unused_variables,unused_assignments,unused_mut)]
    const IMPLS: &'static[crate::func::Impl] =  &[{
        |data,range|{
            let mut components = data.get_mut_disjoint(COMPONENTS_IDS);
            let mut i = 0;
            let(mooring,indices) = components[i].as_mut().unwrap().slice_ref::<usize>().unwrap();
            i+=1;
            {
                for i in range {
                    indices.get_data(i);
                }
            }
        }
    },];
    #[allow(unused_assignments)]
    fn implr(data: &crate::data::Data) -> Result<crate::func::Impl,ImplError>{
        let data_boxes:[&crate::data::DataBox;
        _] = [data.get(&(&C1).id).ok_or(crate::func::ImplError::MissingComponent((&C1).name))?,];
        let mut impl_i = 0;
        let mut typed:bool = true;
        let mut i = 0;
        typed&=data_boxes[i].inner_is::<usize>();
        i+=1;
        if typed {
            return Ok(IMPLS[impl_i]);
        }impl_i+=1;
        Err(crate::func::ImplError::NoValidSignature)
    }
    crate::func::Func::new(stringify!($name),COMPONENTS,implr)
};
