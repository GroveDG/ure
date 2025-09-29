use std::{any::Any, collections::HashMap, ops::Range};

use bitvec::{slice::BitSlice, vec::BitVec};
use indexmap::IndexSet;
use one_or_many::OneOrMany;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ComponentId(u64);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FunctionId(u64);

pub trait Container: Any {
    type Ref<'a>;
    type Mut<'a>;

    fn container_ref(&self) -> Self::Ref<'_>;
    fn container_mut(&mut self) -> Self::Mut<'_>;
}

pub trait Component {
    const ID: ComponentId = ComponentId(const_fnv1a_hash::fnv1a_hash_str_64(Self::IDENT));
    const IDENT: &'static str;
    type Container: Container;

    fn new(container: &mut Self::Container, num: usize);
    fn delete(container: &mut Self::Container, range: Range<usize>);
}

pub struct ExampleIndex;
impl Component for ExampleIndex {
    const IDENT: &'static str = stringify!(ExampleIndex);
    type Container = Vec<usize>;

    fn new(container: &mut Self::Container, num: usize) {
        container.resize(container.len() + num, 0);
    }

    fn delete(container: &mut Self::Container, range: Range<usize>) {
        for i in range.rev() {
            container.swap_remove(i);
        }
    }
}

type Components = HashMap<ComponentId, Box<dyn Any>>;

#[derive(Default)]
pub struct Group {
    len: usize,
    pub(crate) components: Components,
    // pub(crate) functions: HashMap<FunctionId>,
}

impl Group {
    pub fn len(&self) -> usize {
        self.len
    }
    pub fn add_component<C: Component>(&mut self, init: impl FnOnce(&Self) -> C::Container) {
        self.components.insert(C::ID, Box::new((init)(self)));
    }
    pub fn get_components<C: ComponentRetrieve>(
        &self,
    ) -> Option<<C::Containers as Container>::Ref<'_>> {
        C::retrieve(&self.components)
    }
    pub fn get_components_mut<C: ComponentRetrieve>(
        &mut self,
    ) -> Option<<C::Containers as Container>::Mut<'_>> {
        C::retrieve_mut(&mut self.components)
    }
    pub fn remove_component<C: Component>(&mut self) {
        self.components.remove(&C::ID);
    }
}

pub trait ComponentRetrieve {
    type Containers: Container;

    fn retrieve(components: &Components) -> Option<<Self::Containers as Container>::Ref<'_>>;
    fn retrieve_mut(
        components: &mut Components,
    ) -> Option<<Self::Containers as Container>::Mut<'_>>;
}
impl<C: Component> ComponentRetrieve for C {
    type Containers = C::Container;

    fn retrieve(components: &Components) -> Option<<Self::Containers as Container>::Ref<'_>> {
        Some(
            components
                .get(&C::ID)?
                .downcast_ref::<C::Container>()?
                .container_ref(),
        )
    }
    fn retrieve_mut(
        components: &mut Components,
    ) -> Option<<Self::Containers as Container>::Mut<'_>> {
        Some(
            components
                .get_mut(&C::ID)?
                .downcast_mut::<C::Container>()?
                .container_mut(),
        )
    }
}

impl<T: 'static> Container for Vec<T> {
    type Ref<'a> = &'a [T];
    type Mut<'a> = &'a mut [T];

    fn container_ref(&self) -> Self::Ref<'_> {
        self
    }
    fn container_mut(&mut self) -> Self::Mut<'_> {
        self
    }
}
impl<T: 'static> Container for IndexSet<T> {
    type Ref<'a> = &'a IndexSet<T>; // TODO: prevent structural modifications
    type Mut<'a> = &'a mut IndexSet<T>;

    fn container_ref(&self) -> Self::Ref<'_> {
        self
    }
    fn container_mut(&mut self) -> Self::Mut<'_> {
        self
    }
}
impl<T: 'static> Container for OneOrMany<T> {
    type Ref<'a> = &'a [T];
    type Mut<'a> = &'a mut [T];

    fn container_ref(&self) -> Self::Ref<'_> {
        self.as_slice()
    }
    fn container_mut(&mut self) -> Self::Mut<'_> {
        self.as_mut_slice()
    }
}
impl Container for BitVec {
    type Ref<'a> = &'a BitSlice;
    type Mut<'a> = &'a mut BitSlice;

    fn container_ref(&self) -> Self::Ref<'_> {
        self
    }
    fn container_mut(&mut self) -> Self::Mut<'_> {
        self
    }
}
// GPU Buffer container defined in the URE GPU crate.

macro_rules! container_tuples {
    ($($T:ident),*) => {
#[allow(non_snake_case)]
impl<$($T: Container),*> Container for ($($T),*) {
    type Ref<'a> = ($($T::Ref<'a>),*);
    type Mut<'a> = ($($T::Mut<'a>),*);

    fn container_ref(&self) -> Self::Ref<'_> {
        let ($($T),*) = self;
        ($($T.container_ref()),*)
    }
    fn container_mut(&mut self) -> Self::Mut<'_> {
        let ($($T),*) = self;
        ($($T.container_mut()),*)
    }
}
#[allow(non_snake_case)]
impl<$($T: Component),*> ComponentRetrieve for ($($T),*) {
    type Containers = ($($T::Container),*);

    fn retrieve(components: &Components) -> Option<<Self::Containers as Container>::Ref<'_>> {
        $(
            let $T = components
            .get(&$T::ID)?
            .downcast_ref::<$T::Container>()?
            .container_ref();
        )*
        Some(($($T),*))
    }
    fn retrieve_mut(
        components: &mut Components,
    ) -> Option<<Self::Containers as Container>::Mut<'_>> {
        let [$($T),*] = components.get_disjoint_mut([$(&$T::ID),*]);
        $(
            let $T = $T?.downcast_mut::<$T::Container>()?.container_mut();
        )*
        Some(($($T),*))
    }
}
    };
}
container_tuples!(A, B);
container_tuples!(A, B, C);
container_tuples!(A, B, C, D);
container_tuples!(A, B, C, D, E);
container_tuples!(A, B, C, D, E, F);
container_tuples!(A, B, C, D, E, F, G);
container_tuples!(A, B, C, D, E, F, G, H);
container_tuples!(A, B, C, D, E, F, G, H, I);
container_tuples!(A, B, C, D, E, F, G, H, I, J);
