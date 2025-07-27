#[macro_export]
macro_rules! entity {
    ($name:ident) => {
        struct $name(Offset);

        impl Entity for $name {
            const SIZE: Offset = $name::_SIZE;
        }
    }
}

#[macro_export]
macro_rules! compose {
    ($($t:ty),* $(,)?) => {
        const COMPOSED: Offset = Offset::ZERO$(.compose(<$t>::SIZE))*;
        ure::_paste!{
        $(
        pub fn [<get_$t:lower>](self, data: &Data) -> &<$t as Component>::ComponentType {
            $t(self.0).get_ref(data)
        }
        pub fn [<get_mut_$t:lower>](self, data: &mut Data) -> &mut <$t as Component>::ComponentType {
            $t(self.0).get_mut(data)
        }
        )*
        }
    };
}

#[macro_export]
macro_rules! comprise {
    ($($comprise:ident: $t:ty),* $(,)?) => {
    ure::_paste!{
        comprise!{Self::COMPOSED => $($comprise, Self::[<$comprise:upper>].add_const($t::SIZE) =>)* _SIZE}
        $(
        pub fn [<get_$comprise>](self) -> $t {
            unsafe {
                $t(self.0 + Self::[<$comprise:upper>])
            }
        }
        )*
    }
    };
    {$($value:expr => $name:ident),+} => {
    ure::_paste!{
        $(const [<$name:upper>]: Offset = $value;)+
    }
    };
}