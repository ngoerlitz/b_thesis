#[macro_export]
macro_rules! getter {
    ($name:ident: $t:ty as $fn_name:ident) => {
        #[inline]
        pub fn $fn_name(&self) -> &$t {
            &self.$name
        }
    };

    ($name:ident: $t:ty) => {
        crate::getter!($name: $t as $name);
    };
}

#[macro_export]
macro_rules! getter_assoc {
    ($name:ident) => {
        paste::paste! {
            #[inline]
            fn $name(&self) -> &Self::[<$name:camel>] {
                &self.$name
            }
        }
    };
}

#[macro_export]
macro_rules! getter_assoc_mut {
    ($name:ident) => {
        paste::paste! {
            #[inline]
            fn $name(&self) -> &mut Self::[<$name:camel>] {
                &mut self.$name
            }
        }
    };
}
