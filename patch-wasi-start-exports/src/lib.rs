use anyhow::anyhow;
use wasabi_wasm::{
    Function, FunctionType, Idx,
    Instr::{self, End},
    Module,
};

pub fn rename_wasi_export(module: &mut Module, start_append: &str) -> Option<String> {
    let mut new_name = None;
    for (_index, function) in module.functions_mut() {
        function
            .export
            .iter_mut()
            .filter(|export_name| export_name.as_str() == "_start")
            .for_each(|export_name| {
                export_name.push_str(start_append);
                new_name = Some(export_name.clone());
            });
    }

    new_name
}

pub struct ModuleStart {
    pub module_name: String,
    pub start_function: String,
}

pub fn extend_start(module: &mut Module, starts: Vec<ModuleStart>) -> anyhow::Result<()> {
    let mut indices: Vec<Idx<Function>> = Vec::with_capacity(starts.len());

    for module_start in starts {
        let ModuleStart {
            module_name,
            start_function,
        } = module_start;
        let index =
            module.add_function_import(FunctionType::new(&[], &[]), module_name, start_function);
        indices.push(index);
    }

    let mut call_all_starts: Vec<Instr> = indices.iter().map(|index| Instr::Call(*index)).collect();
    call_all_starts.push(End);

    let module_has_start = module.functions().any(|(_index, function)| {
        function
            .export
            .iter()
            .any(|export_name| export_name.as_str() == "_start")
    });

    if module_has_start {
        for (_index, function) in module.functions_mut() {
            if function
                .export
                .iter()
                .any(|export_name| export_name.as_str() == "_start")
            {
                match &mut function.code {
                    wasabi_wasm::ImportOrPresent::Import(_, _) => {
                        return Err(anyhow!("Unexpected '_start' is an import?"))
                    }
                    wasabi_wasm::ImportOrPresent::Present(code) => {
                        let body = &mut code.body;
                        // remove the last `End`, insert calls to all starts

                        match body.remove(body.len() - 1) {
                            End => {}
                            _ => {
                                return Err(anyhow!(
                                    "Unexpected last instruction of '_start' is not `End`"
                                ))
                            }
                        }
                        body.extend_from_slice(&call_all_starts);
                    }
                }
            }
        }
    } else {
        let artificial_start =
            module.add_function(FunctionType::new(&[], &[]), vec![], call_all_starts);
        module
            .function_mut(artificial_start)
            .export
            .push("_start".to_string());
    }

    Ok(())
}
