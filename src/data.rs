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
            fn get_ref(self, data: &Data) -> &Self::ComponentType;
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
            fn get_ref(self, data: &Data) -> &Self::ComponentType {
                &data.[<$component:lower>][self.0.[<$component:lower>]]
            }
            fn get_mut(self, data: &mut Data) -> &mut Self::ComponentType {
                &mut data.[<$component:lower>][self.0.[<$component:lower>]]
            }
        }
        )+

        impl Offset {
            pub const ZERO: Self = Self {$([<$component:lower>]: 0),+};

            // Hopefully one day this can be const BitOr
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
            pub const fn add_const(self, rhs: Self) -> Self {
                Self {
                    $(
                    [<$component:lower>]: self.[<$component:lower>] + rhs.[<$component:lower>]
                    ),+
                }
            }
        }
        impl std::ops::Add<Offset> for Offset {
            type Output = Self;

            fn add(self, rhs: Self) -> Self::Output {
                Self {
                    $(
                    [<$component:lower>]: self.[<$component:lower>] + rhs.[<$component:lower>]
                    ),+
                }
            }
        }
    }
    };
}
