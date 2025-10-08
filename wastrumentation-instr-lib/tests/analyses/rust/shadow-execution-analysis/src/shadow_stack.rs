use wastrumentation_rs_stdlib::WasmType;

use super::WasmValue;
use std::cell::RefCell;

// https://webassembly.github.io/spec/core/exec/conventions.html#prose-notation
// `The execution rules also assume the presence of an implicit stack that is modified by pushing or popping values, labels, and frames.`
// https://webassembly.github.io/spec/core/exec/runtime.html#stack

pub(crate) struct Stack(Vec<StackEntry>);

thread_local! {
    pub(crate) static SHADOW_STACK: RefCell<Stack> = const { RefCell::new(Stack(vec![])) };
}

thread_local! {
    pub(crate) static JUMP_FLAG: RefCell<bool> = const { RefCell::new(false) };
    pub(crate) static CALL_STACK_DEPTH: RefCell<i32> = const { RefCell::new(0) };
}

pub(crate) struct ShadowCallStackDepth;

impl ShadowCallStackDepth {
    pub(crate) fn host_is_caller() -> bool {
        CALL_STACK_DEPTH.with_borrow(|call_stack_depth| {
            debug_assert!(*call_stack_depth >= 0);
            *call_stack_depth == 0
        })
    }

    pub(crate) fn increment_call_stack_depth() {
        CALL_STACK_DEPTH.with_borrow_mut(|call_stack_depth| {
            debug_assert!(*call_stack_depth >= 0);
            *call_stack_depth += 1;
        });
    }
    pub(crate) fn decrement_call_stack_depth() {
        CALL_STACK_DEPTH.with_borrow_mut(|call_stack_depth| {
            debug_assert!(*call_stack_depth >= 0);
            *call_stack_depth -= 1;
        });
    }
}

#[derive(Debug, Clone)]
pub enum StackEntry {
    Value(WasmValue),
    Label(Label),
    Frame(Frame),
}

#[derive(Debug, Clone, PartialEq, Copy)]
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

thread_local! {
    pub(crate) static LABEL_IDENTIFIER: RefCell<usize> = const { RefCell::new(0) };
}

impl Label {
    pub fn new(arity: usize, origin: LabelOrigin) -> Self {
        let identifier = LABEL_IDENTIFIER.with_borrow(|v| *v);
        LABEL_IDENTIFIER.with_borrow_mut(|v| *v += 1);
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
        self.locals[x] = Some(value);
    }

    pub fn get_locals(&mut self, x: usize, type_: WasmType) -> WasmValue {
        if let Some(shadow_local) = &self.locals[x] {
            shadow_local.clone()
        } else {
            let default = WasmValue::default_for(&type_);
            self.locals[x] = Some(default.clone());
            default
        }
    }
}

impl Stack {
    #[must_use]
    pub fn pop_value_from_stack(&mut self) -> WasmValue {
        let Self(shadow_stack_mut) = self;
        match shadow_stack_mut.pop().unwrap() {
            StackEntry::Value(value) => value,
            _ => panic!("Top of stack is not a value"),
        }
    }

    #[must_use]
    pub fn pop_values_from_stack(&mut self, n: usize) -> Vec<WasmValue> {
        let mut values: Vec<WasmValue> = Vec::with_capacity(n);
        for _ in 0..n {
            values.push(self.pop_value_from_stack());
        }
        values.reverse();
        values
    }

    #[must_use]
    pub fn stack_label_count(&self) -> usize {
        let Self(shadow_stack_ref) = self;
        shadow_stack_ref
            .iter()
            .filter(|v| matches!(v, StackEntry::Label(_)))
            .count()
    }

    #[must_use]
    pub fn stack_value_count(&self) -> usize {
        let Self(shadow_stack_ref) = self;
        shadow_stack_ref
            .iter()
            .filter(|v| matches!(v, StackEntry::Value(_)))
            .count()
    }

    #[must_use]
    pub fn lth_label_on_stack_starting_from_top_counting_from_zero(&self, l: usize) -> &Label {
        let Self(shadow_stack_ref) = self;
        shadow_stack_ref
            .iter()
            .filter_map(|stack_entry| match stack_entry {
                StackEntry::Label(label) => Some(label),
                _ => None,
            })
            .rev()
            .nth(l)
            .unwrap()
    }

    pub fn assert_at_least_n_values_on_stack(&self, n: usize) {
        debug_assert!(self.stack_value_count() >= n);
    }

    #[must_use]
    pub fn top_of_stack(&self) -> &StackEntry {
        let Self(shadow_stack_ref) = self;
        shadow_stack_ref.last().unwrap()
    }

    #[must_use]
    pub fn pop_stack(&mut self) -> StackEntry {
        let Self(shadow_stack_mut) = self;
        shadow_stack_mut.pop().unwrap()
    }

    #[must_use]
    pub fn pop_label_from_stack(&mut self) -> Label {
        if let StackEntry::Label(label) = self.pop_stack() {
            label
        } else {
            panic!()
        }
    }

    #[must_use]
    pub fn pop_frame_from_stack(&mut self) -> Frame {
        if let StackEntry::Frame(frame) = self.pop_stack() {
            frame
        } else {
            panic!()
        }
    }

    #[must_use]
    pub fn top_two_values_of_stack(&self) -> (&WasmValue, &WasmValue) {
        let Self(shadow_stack_ref) = self;
        let shadow_stack = shadow_stack_ref;
        let [StackEntry::Value(v2), StackEntry::Value(v1)] =
            &shadow_stack[shadow_stack.len() - 2..]
        else {
            panic!()
        };
        (v2, v1)
    }

    pub fn push_value_on_stack(&mut self, value: WasmValue) {
        let Self(shadow_stack_mut) = self;
        shadow_stack_mut.push(StackEntry::Value(value));
    }

    pub fn push_label_on_stack(&mut self, label: Label) {
        let Self(shadow_stack_mut) = self;
        shadow_stack_mut.push(StackEntry::Label(label));
    }

    pub fn push_activation_on_stack(&mut self, activation: Frame) {
        let Self(shadow_stack_mut) = self;
        shadow_stack_mut.push(StackEntry::Frame(activation));
    }

    #[must_use]
    pub(crate) fn current_frame_mut(&mut self) -> &mut Frame {
        // https://webassembly.github.io/spec/core/exec/conventions.html#prose-notation
        // `Certain rules require the stack to contain at least one frame.
        //   The most recent frame is referred to as the current frame.`
        let Self(shadow_stack_mut) = self;
        shadow_stack_mut
            .iter_mut()
            .filter_map(|stack_entry| match stack_entry {
                StackEntry::Frame(frame) => Some(frame),
                _ => None,
            })
            .last()
            .unwrap()
    }
}
