//! Criterion benchmark for the decode hot path.
//!
//! Compares serial single-instruction decoding against the parallel
//! [`decode_batch`] path across a synthetic mixed-protocol workload.

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};

use soldecoders_core::InstructionDecoder;
use soldecoders_core::{decode_batch, DecoderRegistry, SystemDecoder};
use soldecoders_types::{Pubkey, RawInstruction};

fn system_transfer(lamports: u64, program: &Pubkey) -> RawInstruction {
    let mut data = 2u32.to_le_bytes().to_vec();
    data.extend_from_slice(&lamports.to_le_bytes());
    RawInstruction::new(
        program.clone(),
        vec![Pubkey::new([1; 32]), Pubkey::new([2; 32])],
        data,
    )
}

fn bench_decode(c: &mut Criterion) {
    let registry = DecoderRegistry::builtin();
    let program = SystemDecoder::new().program_id().clone();

    let mut group = c.benchmark_group("decode");
    for size in [1_000usize, 100_000] {
        let items: Vec<_> = (0..size)
            .map(|i| system_transfer(i as u64, &program))
            .collect();
        group.throughput(Throughput::Elements(size as u64));

        group.bench_with_input(BenchmarkId::new("serial", size), &items, |b, items| {
            b.iter(|| {
                let mut n = 0u64;
                for ix in items {
                    if registry.decode_instruction(ix).is_ok() {
                        n += 1;
                    }
                }
                n
            });
        });

        group.bench_with_input(BenchmarkId::new("parallel", size), &items, |b, items| {
            b.iter(|| decode_batch(&registry, items).len());
        });
    }
    group.finish();
}

criterion_group!(benches, bench_decode);
criterion_main!(benches);
