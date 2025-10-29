use std::{any::Any, cell::UnsafeCell, collections::HashMap, marker::PhantomData, ops::Range};

use crate::{Container, Method};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ComponentIdInner(u64);
impl ComponentIdInner {
    const fn new(ident: &str) -> Self {
        Self(const_fnv1a_hash::fnv1a_hash_str_64(ident))
    }
}
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct ComponentId<C: Container> {
    inner: ComponentIdInner,
    _marker: PhantomData<C>,
}
impl<C: Container> ComponentId<C> {
    pub const fn new(ident: &str) -> Self {
        Self {
            inner: ComponentIdInner::new(ident),
            _marker: PhantomData,
        }
    }
    pub const fn inner(&self) -> ComponentIdInner {
        self.inner
    }
    pub unsafe fn get_container(&self, components: &Components) -> Option<&C> {
        unsafe { components.get(&self.inner)?.get().as_mut() }?.downcast_ref::<C>()
    }
    pub unsafe fn get_container_mut(&self, components: &Components) -> Option<&mut C> {
        unsafe { components.get(&self.inner)?.get().as_mut() }?.downcast_mut::<C>()
    }
    pub unsafe fn get(&self, components: &Components) -> Option<C::Ref<'_>> {
        Some(
            unsafe { components.get(&self.inner)?.get().as_mut() }?
                .downcast_ref::<C>()?
                .as_ref(),
        )
    }
    pub unsafe fn get_mut(&self, components: &Components) -> Option<C::Mut<'_>> {
        Some(
            unsafe { components.get(&self.inner)?.get().as_mut() }?
                .downcast_mut::<C>()?
                .as_mut(),
        )
    }
}
impl<C: Container> Clone for ComponentId<C> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            _marker: self._marker.clone(),
        }
    }
}
impl<C: Container> Copy for ComponentId<C> {}

pub struct Component<C: Container> {
    pub id: ComponentId<C>,
    pub new: Method<Range<usize>>,
    pub delete: Method<Range<usize>>,
}
impl<C: Container + Default> Component<C> {
    pub fn new(&self) -> ComponentContainer<C> {
        ComponentContainer {
            id: self.id,
            container: Default::default(),
            len: 0,
            new: self.new,
            delete: self.delete,
        }
    }
}
impl<C: Container> Component<C> {
    pub fn with(&self, container: C, len: usize) -> ComponentContainer<C> {
        ComponentContainer {
            id: self.id,
            container,
            len,
            new: self.new,
            delete: self.delete,
        }
    }
}
pub struct ComponentContainer<C: Container> {
    pub id: ComponentId<C>,
    pub container: C,
    pub len: usize,
    pub new: Method<Range<usize>>,
    pub delete: Method<Range<usize>>,
}

pub type Components = HashMap<ComponentIdInner, UnsafeCell<ComponentBox>>;
type ComponentBox = Box<dyn Any>;

#[macro_export]
macro_rules! component {
    (
        $v:vis $name:ident : $t:ty
        $(, new ( $get_i:ident $(, $get_name:ident : $get_new:expr )* $(,)? ) $new:expr )?
        $(, del ( $del_i:ident $(, $del_name:ident : $get_del:ident )* $(,)? ) $del:expr )?
    ) => {
$crate::mident::mident! {
$v const $name : $crate::Component<$t> = {
    const ID: $crate::ComponentId<$t> = $crate::ComponentId::new(stringify!($name));
    const NEW: $crate::Method<std::ops::Range<usize>> = $crate::new!(
        #concat(NEW_ $name),
        ID,
        $( ($get_i, $($get_name : $get_new),*) $new )?
    );
    const DELETE: $crate::Method<std::ops::Range<usize>> = $crate::del!(
        #concat(DEL_ $name),
        ID,
        $( ($del_i, $($del_name : $del_new),*) $del )?
    );
    $crate::Component{
        id: ID,
        new: NEW,
        delete: DELETE
    }
};
}
    };
}
#[macro_export]
macro_rules! del {
    (
        $name:ident,
        $component:expr $(,)?
    ) => {
$crate::Method::new(
    stringify!($name),
    |components, range| {
        unsafe {
        let mut c = $component.get_container_mut(components).unwrap();
        $crate::Container::delete(c, range);
        }
    },
    &[$component.inner()]
)
    };
    (
        $name:ident,
        $component:expr,
        ( $i:ident, $($get_name:ident : $get:expr),* $(,)? )
        $func:expr $(,)?
    ) => {
$crate::Method::new(
    stringify!($name),
    |components, range| {
        unsafe {
        let mut c = $component.get_container_mut(components).unwrap();
        $(
        let mut $get_name = $get.id.get_mut(components).unwrap();
        )*
        for $i in range {
            c.push($func);
        }
        }
    },
    &[$component.inner(), $( $get.id.inner() ),*]
)
    };
}
#[macro_export]
macro_rules! new {
    (
        $name:ident,
        $component:expr $(,)?
    ) => {
$crate::Method::new(
    stringify!($name),
    |components, range| {
        unsafe {
        let mut c = $component.get_container_mut(components).unwrap();
        $crate::ContainerDefault::new(c, range);
        }
    },
    &[$component.inner()]
)
    };
    (
        $name:ident,
        $component:expr,
        ( $i:ident, $($get_name:ident : $get:expr),* $(,)? )
        $func:expr $(,)?
    ) => {
$crate::Method::new(
    stringify!($name),
    |components, range| {
        unsafe {
        let mut c = $component.get_container_mut(components).unwrap();
        $(
        let mut $get_name = $get.id.get_mut(components).unwrap();
        )*
        for $i in range {
            c.push($func);
        }
        }
    },
    &[$component.inner(), $( $get.id.inner() ),*]
)
    };
}
