#[macro_export]
macro_rules! components {
    {$($component:ident : $t:ty),+ $(,)?} => {
    ure::_paste!{
        #[derive(Debug, Default)]
        struct Data {
            $(
            pub [<$component:lower>] : Vec<$t>
            ),+
        }
        impl Data {
            pub fn init(size: Offset) -> Self {
                Self {
                    $(
                    [<$component:lower>] : Vec::<$t>::with_capacity(size.[<$component:lower>]),
                    )+
                }
            }
            $(
            pub fn [<init_$component:lower>](&mut self, [<$component:lower>]: $t) {
                self.[<$component:lower>].push([<$component:lower>]);
            }
            )+
        }

        struct Offset {
            $(
            pub [<$component:lower>] : usize
            ),+
        }
        pub trait Entity {
            const SIZE: Offset;
        }

        pub trait Component {
            type ComponentType;
            fn get(self, data: &Data) -> &Self::ComponentType;
            fn get_mut(self, data: &mut Data) -> &mut Self::ComponentType;
        }
        $(
        #[repr(transparent)]
        pub struct $component(Offset);
        impl Entity for $component {
            const SIZE: Offset = Offset {
                [<$component:lower>]: 1,
                ..Offset::ZERO
            };
        }
        impl Component for $component {
            type ComponentType = $t;
            fn get(self, data: &Data) -> &Self::ComponentType {
                &data.[<$component:lower>][self.0.[<$component:lower>]]
            }
            fn get_mut(self, data: &mut Data) -> &mut Self::ComponentType {
                &mut data.[<$component:lower>][self.0.[<$component:lower>]]
            }
        }
        )+

        #[repr(transparent)]
        pub struct Collect<const N: usize, T>(T);
        impl<const N: usize, T:Entity> Entity for Collect<N, T> {
            const SIZE: Offset = T::SIZE.mul(N);
        }

        impl Offset {
            pub const ZERO: Self = Self {$([<$component:lower>]: 0),+};

            pub const fn compose(self, rhs: Self) -> Self {
                $(
                assert!(self.[<$component:lower>] <= 1);
                assert!(rhs.[<$component:lower>] <= 1);
                )+
                Self {
                    $(
                    [<$component:lower>]: self.[<$component:lower>] | rhs.[<$component:lower>]
                    ),+
                }
            }
            pub const fn add(self, rhs: Self) -> Self {
                Self {
                    $(
                    [<$component:lower>]: self.[<$component:lower>] + rhs.[<$component:lower>]
                    ),+
                }
            }
            pub const fn mul(self, rhs: usize) -> Self {
                Self {
                    $(
                    [<$component:lower>]: self.[<$component:lower>] * rhs
                    ),+
                }
            }
        }
    }
    };
}
