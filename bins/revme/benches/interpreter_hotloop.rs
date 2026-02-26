use criterion::{criterion_group, criterion_main, Criterion};
use revm::{
    bytecode::Bytecode,
    interpreter::{
        instruction_table,
        interpreter::{EthInterpreter, ExtBytecode, InputsImpl, SharedMemory},
        host::DummyHost,
        Interpreter,
    },
    primitives::Bytes,
};

/// Number of PUSH1/POP pairs per run. Each pair = 3 bytes (PUSH1 0x01 POP).
/// 16K pairs = 32K opcodes dispatched per interpreter.run_plain() call.
const PAIRS: usize = 16_384;

/// Build bytecode: repeated [PUSH1 0x01, POP] + STOP
fn make_pushpop_bytecode() -> Bytecode {
    // PUSH1 = 0x60, POP = 0x50, STOP = 0x00
    let mut code = Vec::with_capacity(PAIRS * 3 + 1);
    for _ in 0..PAIRS {
        code.push(0x60); // PUSH1
        code.push(0x01); // immediate
        code.push(0x50); // POP
    }
    code.push(0x00); // STOP
    Bytecode::new_raw(Bytes::from(code))
}

fn interpreter_hotloop(c: &mut Criterion) {
    let bytecode = make_pushpop_bytecode();
    let table = instruction_table::<EthInterpreter, DummyHost>();

    c.bench_function("interpreter_hotloop_pushpop_32k", |b| {
        b.iter_batched(
            || {
                Interpreter::<EthInterpreter>::new(
                    SharedMemory::new(),
                    ExtBytecode::new(bytecode.clone()),
                    InputsImpl::default(),
                    false,
                    Default::default(),
                    u64::MAX,
                )
            },
            |mut interp| {
                let mut host = DummyHost::default();
                interp.run_plain(&table, &mut host)
            },
            criterion::BatchSize::SmallInput,
        );
    });
}

criterion_group!(benches, interpreter_hotloop);
criterion_main!(benches);
