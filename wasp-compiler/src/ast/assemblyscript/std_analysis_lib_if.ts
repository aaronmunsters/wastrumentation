const THEN_KONTN: i32 = 0;
const ELSE_KONTN: i32 = 1;
const SKIP_KONTN: i32 = 1;

class ParameterConditionIfThen {
    readonly continuation: i32;

    readonly is_then: i32;
    readonly is_skip: i32;

    readonly continue_then: i32 = THEN_KONTN;
    readonly continue_skip: i32 = SKIP_KONTN;

    constructor(path_kontinuation: i32) {
        this.continuation = path_kontinuation;

        switch (path_kontinuation) {
            case THEN_KONTN:
                this.is_then = true;
                this.is_skip = false;
                break;
            case SKIP_KONTN:
                this.is_then = false;
                this.is_skip = true;
                break;
            default:
                unreachable();
        }

    }
}

class ParameterConditionIfThenElse {
    readonly continuation: i32;

    readonly is_then: i32;
    readonly is_else: i32;

    readonly continue_then: i32 = THEN_KONTN;
    readonly continue_else: i32 = ELSE_KONTN;

    constructor(path_kontinuation: i32) {
        this.continuation = path_kontinuation;

        switch (path_kontinuation) {
            case THEN_KONTN:
                this.is_then = true;
                this.is_else = false;
                break;
            case ELSE_KONTN:
                this.is_then = false;
                this.is_else = true;
                break;
            default:
                unreachable();
        }

    }
}
