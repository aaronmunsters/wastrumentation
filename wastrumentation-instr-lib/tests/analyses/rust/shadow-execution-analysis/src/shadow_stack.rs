use wastrumentation_rs_stdlib::WasmType;

use super::WasmValue;
use core::ptr::addr_of;
use core::ptr::addr_of_mut;

// https://webassembly.github.io/spec/core/exec/conventions.html#prose-notation
// `The execution rules also assume the presence of an implicit stack that is modified by pushing or popping values, labels, and frames.`
// https://webassembly.github.io/spec/core/exec/runtime.html#stack

type Stack = Vec<StackEntry>;
static mut SHADOW_STACK: Stack = vec![];

#[derive(Debug, Clone)]
pub enum StackEntry {
    Value(WasmValue),
    Label(Label),
    Frame(Frame),
}

#[derive(Debug, Clone, PartialEq)]
pub enum LabelOrigin {
    Function(usize),
    Block,
    Loop,
    If,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Label {
    identifier: usize,
    origin: LabelOrigin,
    arity: usize,
}

static mut LABEL_IDENTIFIER: usize = 0;

impl Label {
    pub fn new(arity: usize, origin: LabelOrigin) -> Self {
        let identifier = unsafe { LABEL_IDENTIFIER };
        let identifier_incremented = identifier.wrapping_add(1);
        unsafe { LABEL_IDENTIFIER = identifier_incremented };
        Self {
            identifier,
            origin,
            arity,
        }
    }

    #[must_use]
    pub fn arity(&self) -> usize {
        self.arity
    }

    #[must_use]
    pub fn origin(&self) -> &LabelOrigin {
        &self.origin
    }
}

#[derive(Debug, Clone)]
pub struct Frame {
    arity: usize,
    function_index: usize,
    locals: Vec<Option<WasmValue>>,
}

impl Frame {
    pub fn new(arity: usize, function_index: usize, arguments: Vec<WasmValue>) -> Self {
        Self {
            arity,
            function_index,
            locals: arguments.into_iter().map(Some).collect(),
        }
    }

    #[must_use]
    pub fn arity(&self) -> usize {
        self.arity
    }

    #[must_use]
    pub fn function_index(&self) -> usize {
        self.function_index
    }

    pub fn assert_local_exists(&mut self, x: usize) {
        let default = || None;
        if self.locals.len() <= x {
            self.locals.resize_with(x + 1, default);
        }
    }

    pub fn replace_local_with(&mut self, x: usize, value: WasmValue) {
        self.locals[x] = Some(value)
    }

    pub fn get_locals(&mut self, x: usize, type_: &WasmType) -> WasmValue {
        if let Some(shadow_local) = &self.locals[x] {
            shadow_local.clone()
        } else {
            let default = WasmValue::default_for(type_);
            self.locals[x] = Some(default.clone());
            default
        }
    }
}

fn print_shadow_stack(prefix: Option<&str>) {
    if let Some(prefix) = prefix {
        print!("🧐 Inspection called for: {prefix:15}\n=| ");
    }

    for stack_entry in shadow_stack_ref() {
        let s = match stack_entry {
            StackEntry::Value(wasm_value) => {
                format!("[VALUE({})]", wasm_value.bytes_string())
            }
            StackEntry::Label(label) => {
                format!("[LBL-arity:{}-origin:{:?}]", label.arity(), label.origin)
            }
            StackEntry::Frame(frame) => {
                format!(
                    "[FRAME| idx:{idx} - arity:{arity} - 👉 {locals}]",
                    idx = frame.function_index(),
                    arity = frame.arity(),
                    locals = frame
                        .locals
                        .iter()
                        .map(|l| l.as_ref().map_or("🌥️".into(), |l| l.bytes_string()))
                        .collect::<Vec<String>>()
                        .join(","),
                )
            }
        };
        print!("{s}")
    }
    println!();
}

#[must_use]
fn shadow_stack_ref() -> &'static Stack {
    unsafe { addr_of!(SHADOW_STACK).as_ref().unwrap() }
}

#[must_use]
fn shadow_stack_mut() -> &'static mut Stack {
    unsafe { addr_of_mut!(SHADOW_STACK).as_mut().unwrap() }
}

#[must_use]
pub fn pop_value_from_stack() -> WasmValue {
    print_shadow_stack(Some("POP_VLU_PRE"));
    match shadow_stack_mut().pop().unwrap() {
        StackEntry::Value(value) => {
            print_shadow_stack(Some("POP_VLU_PST"));
            value
        }
        _ => panic!("Top of stack is not a value"),
    }
}

#[must_use]
pub fn stack_label_count() -> usize {
    print_shadow_stack(Some("stack_label_count"));
    shadow_stack_ref()
        .iter()
        .filter(|v| matches!(v, StackEntry::Label(_)))
        .count()
}

#[must_use]
pub fn stack_value_count() -> usize {
    shadow_stack_ref()
        .iter()
        .filter(|v| matches!(v, StackEntry::Value(_)))
        .count()
}

#[must_use]
pub fn lth_label_on_stack_starting_from_top_counting_from_zero(l: usize) -> &'static mut Label {
    shadow_stack_mut()
        .iter_mut()
        .filter_map(|stack_entry| match stack_entry {
            StackEntry::Label(label) => Some(label),
            _ => None,
        })
        .rev()
        .nth(l)
        .unwrap()
}

pub fn assert_at_least_n_values_on_stack(n: usize) {
    assert!(stack_value_count() >= n);
}

#[must_use]
pub fn top_of_stack() -> &'static StackEntry {
    shadow_stack_ref().last().unwrap()
}

#[must_use]
pub fn pop_stack() -> StackEntry {
    shadow_stack_mut().pop().unwrap()
}

#[must_use]
pub fn pop_label_from_stack() -> Label {
    if let StackEntry::Label(label) = pop_stack() {
        label
    } else {
        panic!()
    }
}

#[must_use]
pub fn pop_frame_from_stack() -> Frame {
    if let StackEntry::Frame(frame) = pop_stack() {
        frame
    } else {
        panic!()
    }
}

#[must_use]
pub fn top_two_values_of_stack() -> (&'static WasmValue, &'static WasmValue) {
    let shadow_stack = shadow_stack_ref();
    print_shadow_stack(Some("top_two_values_of_stack"));
    let [StackEntry::Value(v2), StackEntry::Value(v1)] = &shadow_stack[shadow_stack.len() - 2..]
    else {
        panic!()
    };
    (v2, v1)
}

pub fn push_value_on_stack(value: WasmValue) {
    print_shadow_stack(Some("PSH_VLU_PRE"));
    shadow_stack_mut().push(StackEntry::Value(value));
    print_shadow_stack(Some("PSH_VLU_PST"));
}

pub fn push_label_on_stack(label: Label) {
    shadow_stack_mut().push(StackEntry::Label(label));
}

pub fn push_activation_on_stack(activation: Frame) {
    shadow_stack_mut().push(StackEntry::Frame(activation));
}

#[must_use]
pub fn current_frame() -> &'static mut Frame {
    // https://webassembly.github.io/spec/core/exec/conventions.html#prose-notation
    // `Certain rules require the stack to contain at least one frame.
    //   The most recent frame is referred to as the current frame.`
    shadow_stack_mut()
        .iter_mut()
        .filter_map(|stack_entry| match stack_entry {
            StackEntry::Frame(frame) => Some(frame),
            _ => None,
        })
        .last()
        .unwrap()
}
