function simply_return<T>(arg: T): T {
    return arg
}

export function main(): i32 {
    return simply_return<i32>(1)
    + (simply_return<f32>(2) as i32)
    + (simply_return<i64>(3) as i32)
    + (simply_return<f64>(4) as i32);
}