use crate::lib_compile::assemblyscript::AssemblyScript;
use wastrumentation::compiler::{LibGeneratable, Library};
use wastrumentation::wasm_constructs::{Signature, WasmType};

// Some sample signatures for testing purposes
fn get_ret_f64_f32_arg_i32_i64() -> Signature {
    Signature {
        return_types: vec![WasmType::F64, WasmType::F32],
        argument_types: vec![WasmType::I32, WasmType::I64],
    }
}

fn get_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64() -> Signature {
    Signature {
        return_types: vec![WasmType::F64, WasmType::F32, WasmType::I32, WasmType::I64],
        argument_types: vec![WasmType::I64, WasmType::I32, WasmType::F32, WasmType::F64],
    }
}

#[test]
fn generating_library_for_signatures() {
    let get_ret_f32_f64_arg_i32_i64 = || Signature {
        return_types: vec![WasmType::F32, WasmType::F64],
        argument_types: vec![WasmType::I32, WasmType::I64],
    };

    let signatures: Vec<Signature> = vec![
        Signature {
            return_types: vec![],
            argument_types: vec![],
        },
        get_ret_f32_f64_arg_i32_i64(),                 // dupe [A]
        get_ret_f64_f32_arg_i32_i64(),                 // dupe [B]
        get_ret_f32_f64_arg_i32_i64(),                 // dupe [A]
        get_ret_f64_f32_arg_i32_i64(),                 // dupe [B]
        get_ret_f64_f32_i32_i64_arg_i64_i32_f32_f64(), // unique
    ];

    let Library { content: lib, .. } = AssemblyScript::generate_lib(&signatures);

    let mut expected = String::from(include_str!("../lib_boilerplate.ts"));
    expected.push_str(include_str!("expected_lib.ts"));

    assert_eq!(lib, expected);
}
