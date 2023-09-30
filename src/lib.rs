extern crate proc_macros;
use std::marker::PhantomData;
use core::ptr::NonNull;
// a lot of this is taken from https://www.youtube.com/watch?v=wU8hQvU8aKM

// TODO: impl drop

/// An implementation which holds the vtable inside of itself(wasting memory) rather than pointing to it
pub struct ThickDyn<'a, Table>{
    _p: PhantomData<&'a ()>,
    pub data: NonNull<()>,
    pub vtable: Table
}
impl <'a, T, Table> From<(T,Table)> for ThickDyn<'a, Table> {
    fn from(value: (T,Table)) -> Self {
        ThickDyn{
            _p: PhantomData::default(),
            data: NonNull::from(&value.0).cast(),
            vtable: value.1
        }
    }
}
/// Similar to rust's dyn object this is a wide pointer to the vtable
pub struct WideDyn<'a, Table:'static>{
    _p: PhantomData<&'a ()>,
    pub data: NonNull<()>,
    pub vtable: &'static Table
}
impl <'a, T, Table:'static> From<(T,&'static Table)> for WideDyn<'a, Table> {
    fn from(value: (T,&'static Table)) -> Self {
        WideDyn{
            _p: PhantomData::default(),
            data: NonNull::from(&value.0).cast(),
            vtable: value.1
        }
    }
}

#[macro_export]
macro_rules! dyn_call {
    ($ident:ident.$method:ident($($args:expr),*)) => {
        unsafe{($ident.vtable.$method)($ident.data, $($args),*)};
    };
}

/// makes the vtable for you
#[macro_export]
macro_rules! gen_vtable_type {
    // fml
    ($table_name:ident, $($methods:ident:fn$fn_args:tt),+) => {
        struct $table_name {
            $($methods:my_dyn::transform_self_arg!($fn_args)),+
        }
    }
}

#[macro_export]
macro_rules! gen_vtable_value {
    ($struct_name:ident, $table_name:ident, $($methods:ident),+)=>{
        $table_name{
            $($methods:|data gen_vtable_closure_args!($($meth_args),*)|$struct_name::$methods(data.gen_vtable_helper_2!($self_arg) gen_vtable_closure_args!($($meth_args),*))),+
        }
    };
}

pub use proc_macros::transform_self_arg;
pub use proc_macros::construct_closure_body_args;
