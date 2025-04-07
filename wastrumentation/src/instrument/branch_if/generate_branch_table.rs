use thiserror::Error;
use wasabi_wasm::{Code, FunctionType, Instr, Label, Local, LocalOp, UnaryOp, Val, ValType};

#[derive(Error, Debug)]
#[error("Cannot create BrTable from {0}")]
pub struct GenerateBranchConversionError(Instr);

pub trait Reified {
    fn reify(&self) -> Code;
}

#[derive(Debug)]
pub struct BrTable {
    pub table: Vec<Label>,
    pub default: Label,
}

impl Reified for BrTable {
    fn reify(&self) -> Code {
        Code {
            locals: vec![Local {
                type_: ValType::I32,
                name: None,
            }],
            body: self.generate_body(),
        }
    }
}

impl BrTable {
    fn generate_body(&self) -> Vec<(Instr, usize)> {
        let yield_i32 = FunctionType::new(&[], &[ValType::I32]);
        let yield_i32_i32 = FunctionType::empty();
        let mut body = vec![];

        if self.table.is_empty() {
            body.extend_from_slice(&[
                Instr::Local(LocalOp::Get, 0_u32.into()),
                Instr::Const(Val::I32(self.default.to_u32().try_into().unwrap())),
                Instr::End,
            ]);
            return body.into_iter().map(|i| (i, 0)).collect();
        }

        if self.table.len() == 1 {
            body.extend_from_slice(&[
                Instr::Local(LocalOp::Get, 0_u32.into()),
                Instr::Local(LocalOp::Get, 0_u32.into()),
                Instr::Unary(UnaryOp::I32Eqz),
                Instr::If(yield_i32),
                Instr::Const(Val::I32(self.table[0].to_u32().try_into().unwrap())),
                Instr::Else,
                Instr::Const(Val::I32(self.default.to_u32().try_into().unwrap())),
                Instr::End,
                Instr::End,
            ]);
            return body.into_iter().map(|i| (i, 0)).collect();
        }

        let blocks_enter: Vec<Instr> = self
            .table
            .iter()
            .flat_map(|_| vec![Instr::Block(yield_i32_i32)])
            .collect();

        let main_branch: Vec<Instr> = vec![
            Instr::Block(yield_i32_i32),
            Instr::Local(LocalOp::Get, 0_u32.into()),
            self.to_br_table_for_generation(),
            Instr::End,
        ];

        let blocks_bodies: Vec<Instr> = self
            .table
            .iter()
            .flat_map(|index| {
                vec![
                    Instr::Local(LocalOp::Get, 0_u32.into()),
                    Instr::Const(Val::I32(index.to_u32().try_into().unwrap())),
                    Instr::Return,
                    Instr::End,
                ]
            })
            .collect();

        body.extend_from_slice(&blocks_enter);
        body.extend_from_slice(&main_branch);
        body.extend_from_slice(&blocks_bodies);
        body.extend_from_slice(&[
            Instr::Local(LocalOp::Get, 0_u32.into()),
            Instr::Const(Val::I32(self.default.to_u32().try_into().unwrap())),
            Instr::End,
        ]);
        body.into_iter().map(|i| (i, 0)).collect()
    }

    fn to_br_table_for_generation(&self) -> Instr {
        Instr::BrTable {
            table: (0..self.table.len())
                .map(Into::into)
                .collect::<Vec<Label>>()
                .into_boxed_slice(),
            default: self.table.len().into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use assemblyscript_compiler::compiler::Compiler as AssemblyScriptCompiler;
    use assemblyscript_compiler::options::CompilerOptions;
    use wasmtime::{Engine, Instance, Module, Store};

    use super::*;

    impl BrTable {
        pub fn from(table: Vec<u32>, default: u32) -> Self {
            Self {
                table: (table).into_iter().map(Into::into).collect(),
                default: default.into(),
            }
        }
    }

    fn get_as_compiled_body(switch_related_source: &'_ str) -> Vec<(Instr, usize)> {
        let assemblyscript_compiler = AssemblyScriptCompiler::new().unwrap();
        let options = CompilerOptions::default_for(switch_related_source);
        let expected_program_wasm = assemblyscript_compiler.compile(&options).unwrap();

        let (module, _, _) = wasabi_wasm::Module::from_bytes(&expected_program_wasm).unwrap();
        let code = module.function(0_u32.into()).code().unwrap();
        let body = &code.body;
        body.clone()
    }

    fn assert_compiler_expectation_equivalence(
        switch_related_source: &'_ str,
        switch_encoded_body: &[(Instr, usize)],
    ) {
        let body = get_as_compiled_body(switch_related_source);
        assert_eq!(body, switch_encoded_body);
    }

    // #[test]
    // TODO: remove this / correct it / ...
    // There are a many possible better implementations, but we're not building a compiler :)
    #[allow(dead_code)]
    fn test_short_example() {
        let generated_body = BrTable::from(vec![1, 2, 3], 0).reify().body;
        let source = r#"
          export function example_generated(table_target_index: i32): i32 {
            switch (table_target_index) {
              case 0: return 1;
              case 1: return 2;
              case 2: return 3;
              default: return 0;
            }
          }
        "#;

        assert_compiler_expectation_equivalence(source, &generated_body);
    }

    #[test]
    fn test_semi_long_example() {
        let coded_expectation = vec![
            Instr::Block(FunctionType::empty()),
            Instr::Block(FunctionType::empty()),
            Instr::Block(FunctionType::empty()),
            Instr::Block(FunctionType::empty()),
            Instr::Block(FunctionType::empty()),
            Instr::Local(LocalOp::Get, 0_u32.into()),
            Instr::BrTable {
                table: Box::new([0_u32.into(), 1_u32.into(), 2_u32.into(), 3_u32.into()]),
                default: 4_u32.into(),
            },
            Instr::End,
            Instr::Local(LocalOp::Get, 0_u32.into()),
            Instr::Const(Val::I32(4)),
            Instr::Return,
            Instr::End,
            Instr::Local(LocalOp::Get, 0_u32.into()),
            Instr::Const(Val::I32(0)),
            Instr::Return,
            Instr::End,
            Instr::Local(LocalOp::Get, 0_u32.into()),
            Instr::Const(Val::I32(1)),
            Instr::Return,
            Instr::End,
            Instr::Local(LocalOp::Get, 0_u32.into()),
            Instr::Const(Val::I32(2)),
            Instr::Return,
            Instr::End,
            Instr::Local(LocalOp::Get, 0_u32.into()),
            Instr::Const(Val::I32(3)),
            Instr::End,
        ];

        let coded_expectation: Vec<(Instr, usize)> =
            coded_expectation.into_iter().map(|i| (i, 0)).collect();

        let generated_body = BrTable::from(vec![4, 0, 1, 2], 3).reify().body;
        assert_eq!(generated_body, coded_expectation);
    }

    #[test]
    fn test_long_example() {
        let expected_body_hardcoded = vec![
            Instr::Block(FunctionType::empty()),
            Instr::Block(FunctionType::empty()),
            Instr::Block(FunctionType::empty()),
            Instr::Block(FunctionType::empty()),
            Instr::Block(FunctionType::empty()),
            Instr::Block(FunctionType::empty()),
            Instr::Block(FunctionType::empty()),
            Instr::Block(FunctionType::empty()),
            Instr::Local(LocalOp::Get, 0_u32.into()),
            Instr::BrTable {
                table: Box::new([
                    0_u32.into(),
                    1_u32.into(),
                    2_u32.into(),
                    3_u32.into(),
                    4_u32.into(),
                    5_u32.into(),
                    6_u32.into(),
                ]),
                default: 7_u32.into(),
            },
            Instr::End,
            Instr::Local(LocalOp::Get, 0_u32.into()),
            Instr::Const(Val::I32(9)),
            Instr::Return,
            Instr::End,
            Instr::Local(LocalOp::Get, 0_u32.into()),
            Instr::Const(Val::I32(11)),
            Instr::Return,
            Instr::End,
            Instr::Local(LocalOp::Get, 0_u32.into()),
            Instr::Const(Val::I32(2)),
            Instr::Return,
            Instr::End,
            Instr::Local(LocalOp::Get, 0_u32.into()),
            Instr::Const(Val::I32(43)),
            Instr::Return,
            Instr::End,
            Instr::Local(LocalOp::Get, 0_u32.into()),
            Instr::Const(Val::I32(3)),
            Instr::Return,
            Instr::End,
            Instr::Local(LocalOp::Get, 0_u32.into()),
            Instr::Const(Val::I32(5)),
            Instr::Return,
            Instr::End,
            Instr::Local(LocalOp::Get, 0_u32.into()),
            Instr::Const(Val::I32(8)),
            Instr::Return,
            Instr::End,
            Instr::Local(LocalOp::Get, 0_u32.into()),
            Instr::Const(Val::I32(7)),
            Instr::End,
        ];
        let expected_body_hardcoded: Vec<(Instr, usize)> = expected_body_hardcoded
            .into_iter()
            .map(|i| (i, 0))
            .collect();

        // Assert equality to custom implementation
        let generated_body = BrTable::from(vec![9, 11, 2, 43, 3, 5, 8], 7).reify().body;
        assert_eq!(&generated_body, &expected_body_hardcoded);
    }

    #[test]
    fn test_runtime() {
        // Generation
        let generated_body = BrTable::from(vec![4, 0, 1, 2], 3).reify().body;
        let mut module = wasabi_wasm::Module::new();
        module.add_function(
            FunctionType::new(&[ValType::I32], &[ValType::I32, ValType::I32]),
            vec![ValType::I32],
            generated_body,
        );
        module.function_mut(0_u32.into()).export = vec!["idx_to_idx_effective".into()];
        let module_as_wasm = module.to_bytes().unwrap();

        // Runtime
        let engine = Engine::default();
        let module = Module::from_binary(&engine, &module_as_wasm).unwrap();
        let mut store = Store::new(&engine, ());
        let instance = Instance::new(&mut store, &module, &[]).unwrap();
        let func = instance
            .get_typed_func::<i32, (i32, i32)>(&mut store, "idx_to_idx_effective")
            .unwrap();

        for (input, expected_results) in [
            (0, (0, 4)),
            (1, (1, 0)),
            (2, (2, 1)),
            (3, (3, 2)),
            (4, (4, 3)),
            (99, (99, 3)),
        ] {
            assert_eq!(
                func.call(&mut store, input).unwrap(),
                expected_results,
                "Failed for input {input}"
            );
        }
    }
}
