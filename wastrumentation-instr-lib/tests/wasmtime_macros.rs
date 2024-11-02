#[macro_export]
macro_rules! declare_fns_from_wasm {
    ($instance:ident, $store:ident,
        $($function_name:ident [$($input_type:ty),*] [$($output_type:ty),*]),+ $(,)?) => {
            $(
                let $function_name =
                    $instance.get_typed_func::<
                        declare_fns_from_wasm!(@convert_to_wasmtime_types $($input_type),*),
                        declare_fns_from_wasm!(@convert_to_wasmtime_types $($output_type),*)
                    >(&mut $store, stringify!($function_name)).unwrap();
            )*
        };
    // Below are internal definitions (`@convert_to_wasmtime_types`)
    // to convert rs type into wasmtime expected type
    (@convert_to_wasmtime_types /* nothing */ ) => { () };
    (@convert_to_wasmtime_types $input_type:ty) => { $input_type };
    (@convert_to_wasmtime_types $first_input_type:ty, $($input_type:ty),+) => {
        ($first_input_type, $($input_type),+)
    };
}

#[macro_export]
macro_rules! declare_fns_from_linker {
    ($linker:ident, $store:ident, $module_name:literal,
        $($function_name:ident [$($input_type:ty),*] [$($output_type:ty),*]),+ $(,)?) => {
            $(
                let $function_name =
                    &$linker.get(&mut $store, $module_name, stringify!($function_name))
                        .unwrap()
                        .into_func()
                        .unwrap()
                        .typed::<
                            declare_fns_from_wasm!(@convert_to_wasmtime_types $($input_type),*),
                            declare_fns_from_wasm!(@convert_to_wasmtime_types $($output_type),*)
                        >(&$store)
                        .unwrap();
            )*
        };
    // Below are internal definitions (`@convert_to_wasmtime_types`)
    // to convert rs type into wasmtime expected type
    (@convert_to_wasmtime_types /* nothing */ ) => { () };
    (@convert_to_wasmtime_types $input_type:ty) => { $input_type };
    (@convert_to_wasmtime_types $first_input_type:ty, $($input_type:ty),+) => {
        ($first_input_type, $($input_type),+)
    };
}

#[macro_export]
macro_rules! wasm_call {
    ($store:ident, $func_name:ident /* no argument passed */) => {
        $func_name.call(&mut $store, ()).unwrap()
    };
    ($store:ident, $func_name:ident, $arg:expr /* one argument passed */) => {
        $func_name.call(&mut $store, $arg).unwrap()
    };
    ($store:ident, $func_name:ident, $($args:expr),+ /* multiple argument passed */) => {
        $func_name.call(&mut $store, ($($args),+)).unwrap()
    };
}
