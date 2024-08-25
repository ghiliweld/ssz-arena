use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use grandine_ssz::SszRead;
use grandine_types::{
    combined::{BeaconState as GrandineBeaconState, SignedBeaconBlock as GrandineBeaconBlock},
    config::Config,
    preset::Mainnet,
};
use sigp_types::{
    BeaconState as SigpBeaconState, ChainSpec, ForkName, MainnetEthSpec,
    SignedBeaconBlock as SigpBeaconBlock,
};

fn ssz_arena(c: &mut Criterion) {
    let mut group = c.benchmark_group("SSZ Decode");
    group.sample_size(10);

    let block_bytes: Vec<u8> = std::fs::read("beacon-block.ssz").unwrap();
    let state_bytes: Vec<u8> = std::fs::read("beacon-state.ssz").unwrap();

    for bytes in [block_bytes].iter() {
        group.throughput(Throughput::Bytes(bytes.len() as u64));

        #[cfg(feature = "sigp")]
        group.bench_with_input(
            BenchmarkId::new("Lighthouse", "SignedBeaconBlock"),
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
            BenchmarkId::new("Grandine", "SignedBeaconBlock"),
            bytes,
            |b, bytes| {
                b.iter(|| {
                    GrandineBeaconBlock::<Mainnet>::from_ssz_unchecked(&Config::mainnet(), bytes)
                })
            },
        );
    }

    for bytes in [state_bytes].iter() {
        group.throughput(Throughput::Bytes(bytes.len() as u64));

        #[cfg(feature = "sigp")]
        group.bench_with_input(
            BenchmarkId::new("Lighthouse", "BeaconState"),
            bytes,
            |b, bytes| {
                b.iter(|| {
                    SigpBeaconState::<MainnetEthSpec>::from_ssz_bytes(bytes, &ChainSpec::default())
                })
            },
        );
        #[cfg(feature = "grandine")]
        group.bench_with_input(
            BenchmarkId::new("Grandine", "BeaconState"),
            bytes,
            |b, bytes| {
                b.iter(|| {
                    GrandineBeaconState::<Mainnet>::from_ssz_unchecked(&Config::mainnet(), bytes)
                })
            },
        );
    }

    group.finish();
}

criterion_group!(benches, ssz_arena);
criterion_main!(benches);
