use criterion::{criterion_group, criterion_main, Criterion};
use iridium::vm::VM;

fn execute_add() {
    let mut test_vm = VM::get_test_vm();
    test_vm.program = vec![1, 0, 1, 2];
    test_vm.run_once();
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("execute_add", |b| b.iter(|| execute_add()));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
