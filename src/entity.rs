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
    ure::_paste!{
        const COMPOSED: Offset = Offset::ZERO$(.compose(<$t>::SIZE))*;
        $(
        pub fn [<get_$t:lower>](self, data: &Data) -> $t {
            $t(self.0)
        }
        )*
    }
    };
}

#[macro_export]
macro_rules! comprise {
    {$($name:ident: $t:ty),* $(,)?} => {
    ure::_paste!{
        comprise!{Self::COMPOSED => $($name, Self::[<$name:upper>].add_const($t::SIZE) =>)* _SIZE}
        $(
        pub fn [<get_$name>](self) -> $t {
            unsafe {
                $t(self.0 + Self::[<$name:upper>])
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