use std::marker::PhantomData;
use core::ptr::NonNull;
// a lot of this is taken from https://www.youtube.com/watch?v=wU8hQvU8aKM

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
    ($table_name:ident, $($methods:ident:fn($self_arg:tt, $($meth_args:ident:$meth_types:ty),*)),+) => {
        struct $table_name {
            $($methods:fn(gen_vtable_helper!($self_arg)$($meth_args:$meth_types),*)),+
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

/// Ignore this macro it's used in gen_vtable for convenience
#[macro_export]
macro_rules! gen_vtable_helper {
    (&mut self) => {NonNull<()>,};
    (&self) => {NonNull<()>,};
    (self) => {NonNull<()>,};
    () => {};
}

/// Ignore this macro it's used in gen_vtable for convenience
// expecting to use this in the impl
#[macro_export]
macro_rules! gen_vtable_helper_2 {
    ()=>{}
}

/// Ignore this macro it's used in gen_vtable for convenience
#[macro_export]
macro_rules! gen_vtable_closure_args {
    () => {};
    ($($args:ident),+) => {, $($args),+};
}
