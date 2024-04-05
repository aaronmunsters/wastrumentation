/// Boolean values in WebAssembly are represented as values of type `i32`.
/// In a boolean context, such as a `br_if` condition, any non-zero value
/// is interpreted as true and `0` is interpreted as false.
///
/// [Link to Wasm reference manual source](
/// https://github.com/sunfishcode/wasm-reference-manual/blob/master/WebAssembly.md#booleans
/// )

const THEN_KONTN: i32 = 1;
const ELSE_KONTN: i32 = 0;
const SKIP_KONTN: i32 = ELSE_KONTN;

const BRANCH_FAIL: i32 = 0;

class ParameterConditionIfThen {
    readonly is_then: i32;
    readonly is_skip: i32;

    readonly continue_then: i32 = THEN_KONTN;
    readonly continue_skip: i32 = SKIP_KONTN;

    constructor(path_kontinuation: i32) {
        if (path_kontinuation == BRANCH_FAIL) {
            this.is_then = false;
            this.is_skip = true;
        } else {
            this.is_then = true;
            this.is_skip = false;
        }
    }
}

class ParameterConditionIfThenElse {
    readonly is_then: i32;
    readonly is_else: i32;

    readonly continue_then: i32 = THEN_KONTN;
    readonly continue_else: i32 = ELSE_KONTN;

    constructor(path_kontinuation: i32) {
        if (path_kontinuation == BRANCH_FAIL) {
            this.is_then = false;
            this.is_else = true;
        } else {
            this.is_then = true;
            this.is_else = false;
        }
    }
}

class ParameterConditionBrIf {
    readonly is_branch: i32;
    readonly is_skip: i32;

    readonly continue_branch: i32 = THEN_KONTN;
    readonly continue_skip: i32 = SKIP_KONTN;

    constructor(path_kontinuation: i32) {
        if (path_kontinuation == BRANCH_FAIL) {
            this.is_branch = false;
            this.is_skip = true;
        } else {
            this.is_branch = true;
            this.is_skip = false;
        }
    }
}

class ParameterLabelBrIf {
    readonly label: i32;

    constructor(label: i32) {
        this.label = label;
    }
}