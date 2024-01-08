export function execute_tests(): void {
  assert_for<i32>();
  assert_for<f32>();
  assert_for<i64>();
  assert_for<f64>();
}

// The Computer Language Shootout
// http://shootout.alioth.debian.org/
// contributed by Isaac Gouy

function ack<T>(m: T, n: T): T {
  if (m == 0) {
    return n + 1;
  }
  if (n == 0) {
    return ack(m - 1, 1);
  }
  return ack(m - 1, ack(m, n - 1));
}

function fib<T>(n: T): T {
  if (n < 2) {
    return 1;
  }
  return fib(n - 2) + fib(n - 1);
}

function tak<T>(x: T, y: T, z: T): T {
  if (y >= x) return z;
  return tak(tak(x - 1, y, z), tak(y - 1, z, x), tak(z - 1, x, y));
}

function assert_for<T>(): void {
  let result: T = 0;

  for (let i: T = 3; i <= 5; i++) {
    result += ack<T>(3 as T, i);
    result += fib<T>((17.0 as T) + i);
    result += tak<T>(
      ((3 as T) * i + 3) as T,
      ((2 as T) * i + 2) as T,
      (i + 1) as T,
    );
  }

  let expected: T = 57775;
  assert(result === expected);
}

function assert(condition: bool): void {
  if (!condition) unreachable(); // WASM TRAP
}

// Source:
// sunspider-benchmarks/sunspider/controlflow-recursive.js
