use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use grandine_ssz::SszRead;
use grandine_types::{
    combined::SignedBeaconBlock as GrandineBeaconBlock, config::Config, preset::Mainnet,
};
use sigp_types::{ForkName, MainnetEthSpec, SignedBeaconBlock as SigpBeaconBlock};

fn ssz_arena(c: &mut Criterion) {
    let mut group = c.benchmark_group("SSZ Decode");
    let block_bytes: Vec<u8> = std::fs::read("beacon-block.ssz").unwrap();
    for bytes in [block_bytes].iter() {
        #[cfg(feature = "sigp")]
        group.bench_with_input(
            BenchmarkId::new("Lighthouse", "SignedBeaconBlock decode"),
            bytes,
            |b, bytes| {
                b.iter(|| {
                    SigpBeaconBlock::<MainnetEthSpec>::from_ssz_bytes_for_fork(
                        bytes,
                        ForkName::Deneb,
                    )
                })
            },
        );
        #[cfg(feature = "grandine")]
        group.bench_with_input(
            BenchmarkId::new("Grandine", "SignedBeaconBlock decode"),
            bytes,
            |b, bytes| {
                b.iter(|| {
                    GrandineBeaconBlock::<Mainnet>::from_ssz_unchecked(&Config::mainnet(), bytes)
                })
            },
        );
    }
    group.finish();
}

criterion_group!(benches, ssz_arena);
criterion_main!(benches);
