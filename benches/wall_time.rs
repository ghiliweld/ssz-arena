use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};

#[cfg(feature = "grandine")]
use grandine_ssz::{SszRead, SszWrite};
#[cfg(all(feature = "grandine", feature = "state"))]
use grandine_types::combined::BeaconState as GrandineBeaconState;
#[cfg(all(feature = "grandine", feature = "block"))]
use grandine_types::combined::SignedBeaconBlock as GrandineBeaconBlock;
#[cfg(feature = "grandine")]
use grandine_types::{config::Config, preset::Mainnet};
#[cfg(all(feature = "sigp", feature = "state"))]
use sigp_types::{BeaconState as SigpBeaconState, ChainSpec, MainnetEthSpec};
#[cfg(all(feature = "sigp", feature = "block"))]
use sigp_types::{ForkName, MainnetEthSpec, SignedBeaconBlock as SigpBeaconBlock};

use ssz::{Decode, Encode};

fn basic_types(c: &mut Criterion) {
    use milhouse::List;

    type C = typenum::U1099511627776;
    const N: u64 = 1_000_000;

    let mut group = c.benchmark_group("List");

    // basic test case
    let size = N;
    let list = List::<u64, C>::try_from_iter(0..size).unwrap();
    let list_bytes = list.as_ssz_bytes();

    group.throughput(Throughput::Bytes(list_bytes.len() as u64));
    group.bench_with_input(
        BenchmarkId::new("Milhouse", "decode"),
        list_bytes.as_slice(),
        |b, bytes| b.iter(|| <List<u64, C> as Decode>::from_ssz_bytes(bytes).unwrap()),
    );
    group.bench_with_input(BenchmarkId::new("Milhouse", "encode"), &list, |b, list| {
        b.iter(|| list.as_ssz_bytes())
    });

    group.finish();
}

#[cfg(feature = "block")]
fn beacon_block(c: &mut Criterion) {
    use ssz_arena::get_block_bytes;

    let mut group = c.benchmark_group("SignedBeaconBlock");
    let block_bytes: Vec<u8> =
        std::fs::read("beacon-block.ssz").unwrap_or(get_block_bytes().unwrap());
    group.throughput(Throughput::Bytes(block_bytes.len() as u64));

    #[cfg(feature = "sigp")]
    group.bench_with_input(
        BenchmarkId::new("Lighthouse", "decode"),
        block_bytes.as_slice(),
        |b, bytes| {
            b.iter(|| {
                SigpBeaconBlock::<MainnetEthSpec>::from_ssz_bytes_for_fork(bytes, ForkName::Deneb)
                    .unwrap()
            })
        },
    );

    // #[cfg(feature = "sigp")]
    // let beacon_block = SigpBeaconBlock::<MainnetEthSpec>::from_ssz_bytes_for_fork(
    //     block_bytes.as_slice(),
    //     ForkName::Deneb,
    // )
    // .unwrap();
    // #[cfg(feature = "sigp")]
    // group.bench_with_input(
    //     BenchmarkId::new("Lighthouse", "encode"),
    //     &beacon_block,
    //     |b, beacon_block| b.iter(|| *beacon_block.as_ssz_bytes()),
    // );

    #[cfg(feature = "grandine")]
    group.bench_with_input(
        BenchmarkId::new("Grandine", "decode"),
        block_bytes.as_slice(),
        |b, bytes| {
            b.iter(|| {
                GrandineBeaconBlock::<Mainnet>::from_ssz_unchecked(&Config::mainnet(), bytes)
                    .unwrap()
            })
        },
    );

    // //#[cfg(feature = "grandine")]
    // let beacon_block = GrandineBeaconBlock::<Mainnet>::from_ssz_unchecked(
    //     &Config::mainnet(),
    //     block_bytes.as_slice(),
    // )
    // .unwrap();
    // #[cfg(feature = "grandine")]
    // group.bench_with_input(
    //     BenchmarkId::new("Grandine", "encode"),
    //     &beacon_block,
    //     |b, bytes| b.iter(|| beacon_block.to_ssz()),
    // );

    group.finish();
}

#[cfg(feature = "state")]
fn beacon_state(c: &mut Criterion) {
    use ssz_arena::get_state_bytes;

    let mut group = c.benchmark_group("BeaconState");
    let state_bytes: Vec<u8> =
        std::fs::read("beacon-state.ssz").unwrap_or(get_state_bytes().unwrap());
    group.throughput(Throughput::Bytes(state_bytes.len() as u64));

    #[cfg(feature = "sigp")]
    group.bench_with_input(
        BenchmarkId::new("Lighthouse", "decode"),
        state_bytes.as_slice(),
        |b, bytes| {
            b.iter(|| {
                SigpBeaconState::<MainnetEthSpec>::from_ssz_bytes(bytes, &ChainSpec::default())
                    .unwrap()
            })
        },
    );

    #[cfg(feature = "grandine")]
    group.bench_with_input(
        BenchmarkId::new("Grandine", "decode"),
        state_bytes.as_slice(),
        |b, bytes| {
            b.iter(|| GrandineBeaconState::<Mainnet>::from_ssz_unchecked(&Config::mainnet(), bytes))
                .unwrap()
        },
    );

    group.finish();
}

#[cfg(feature = "block")]
criterion_group!(light_benches, basic_types, beacon_block);

#[cfg(not(feature = "block"))]
criterion_group!(light_benches, basic_types);

// so-called heavy bench because BeaconState requires more time to benchmark than basic types and beacon blocks
// we use a different Criterion setup to account for this

#[cfg(feature = "state")]
criterion_group! {
    name = heavy_benches;
    config = Criterion::default(); // TODO: add more time
    targets = beacon_state
}

#[cfg(not(feature = "state"))]
criterion_main!(light_benches);

#[cfg(feature = "state")]
criterion_main!(light_benches, heavy_benches);
