use std::{any::Any, collections::HashMap};

use crate::data::{Component, Components, Container};

pub trait Interface<'a> {
    fn of(components: &'a mut Components) -> impl Iterator<Item = &'a mut dyn Container>;
    fn implement(components: &Components) -> Option<Intr<'a, Self>>;
}

pub const INDEX_EX: Component = Component::new("Index");

pub trait InterfaceExample {
    fn call(&mut self);
}

pub type Intr<'a, S> = fn(&'a mut Components) -> Box<S>;

macro_rules! interface {
    {
        $trait:ident
        [Component; $num:literal] = [$($comp:expr),* $(,)?];
        $(
            $impl_vis:vis $impl_name:ident ($($field:ident : $comp_ty:ty),* $(,)?);
        )*
    } => {

$(
    $impl_vis struct $impl_name<'a> {$(
        $field: &'a mut $comp_ty
    ),*}
    impl<'a> $impl_name<'a> {
        fn new(components: &'a mut Components) -> Box<dyn $trait + 'a> {
            let mut components = <dyn $trait>::of(components);
            Box::new(Self {$(
                $field: components.next().unwrap().downcast_mut().unwrap(),
            ),*})
        }
    }
)*

#[allow(unused)]
const COMPONENTS: [&'static $crate::data::Component; $num] = [&$($comp),*];
const COMPONENT_IDS: [&'static $crate::data::ComponentId; $num] = [$(&$comp.id()),*];

impl<'a> $crate::func::Interface<'a> for dyn $trait + 'a {
    fn of(components: &'a mut $crate::data::Components) -> impl Iterator<Item = &'a mut dyn $crate::data::Container> {
        components
            .get_disjoint_mut(COMPONENT_IDS)
            .map(|c| c.unwrap())
            .into_iter()
    }
    fn implement(components: &Components) -> Option<Intr<'a, Self>> {
        let components = COMPONENT_IDS.map(|c| components.get(c).unwrap());
        $({
            let mut components = components.into_iter();
            let mut should_impl = true;
            $(
                should_impl &= (components.next().unwrap() as &dyn Any).is::<$comp_ty>();
            )*
            if should_impl {
                return Some(<$impl_name>::new)
            }
        })*
        None
    }
}

    };
}

interface! {
    InterfaceExample
    [Component; 1] = [INDEX_EX];
    VecImpl (indices: Vec<usize>);
}
impl InterfaceExample for VecImpl<'_> {
    fn call(&mut self) {
        self.indices[0];
    }
}
