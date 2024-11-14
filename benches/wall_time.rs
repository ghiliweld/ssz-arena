use criterion::{
    black_box, criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion, SamplingMode,
    Throughput,
};

#[cfg(feature = "sszb")]
use sszb::{SszDecode, SszEncode};

#[cfg(all(feature = "sigp", feature = "block"))]
use sigp_types::{
    ssz_tagged_signed_beacon_block::encode::as_ssz_bytes as sigp_block_encode, ForkName,
    MainnetEthSpec, SignedBeaconBlock as SigpBeaconBlock,
};
#[cfg(feature = "sigp")]
use ssz::{ssz_encode, Decode, Encode};

#[cfg(feature = "grandine")]
use grandine_ssz::{PersistentList, SszRead, SszWrite};
#[cfg(all(feature = "grandine", feature = "state"))]
use grandine_types::combined::BeaconState as GrandineBeaconState;
#[cfg(all(feature = "grandine", feature = "block"))]
use grandine_types::combined::SignedBeaconBlock as GrandineBeaconBlock;
#[cfg(feature = "grandine")]
use grandine_types::{config::Config, preset::Mainnet};
#[cfg(feature = "grandine")]
use try_from_iter::TryFromIterator;

fn basic_types(c: &mut Criterion) {
    use milhouse::List;

    type C = typenum::U1099511627776;
    const N: u64 = 1_000_000;

    let mut group = c.benchmark_group("Milhouse List");

    // basic test case
    let size = N;
    let list = List::<u64, C>::try_from_iter(0..size).unwrap();

    let list_bytes = list.to_ssz();

    group.throughput(Throughput::Bytes(list_bytes.len() as u64));

    #[cfg(feature = "sszb")]
    group.bench_with_input(
        BenchmarkId::new("Sszb", "decode"),
        list_bytes.as_slice(),
        |b, bytes| b.iter(|| <List<u64, C> as SszDecode>::from_ssz_bytes(bytes).unwrap()),
    );
    #[cfg(feature = "sszb")]
    group.bench_with_input(BenchmarkId::new("Sszb", "encode"), &list, |b, list| {
        b.iter(|| list.to_ssz())
    });
    #[cfg(feature = "sszb")]
    group.bench_with_input(
        BenchmarkId::new("Sszb", "encode with slice"),
        &list,
        |b, list| {
            let len = SszEncode::ssz_bytes_len(list);
            let mut buf: Vec<u8> = vec![0u8; len];
            b.iter(|| list.ssz_write(&mut buf.as_mut_slice()))
        },
    );

    #[cfg(feature = "sigp")]
    group.bench_with_input(
        BenchmarkId::new("Sigp", "decode"),
        list_bytes.as_slice(),
        |b, bytes| b.iter(|| <List<u64, C> as Decode>::from_ssz_bytes(bytes).unwrap()),
    );
    #[cfg(feature = "sigp")]
    group.bench_with_input(BenchmarkId::new("Sigp", "encode"), &list, |b, list| {
        b.iter(|| list.as_ssz_bytes())
    });

    group.finish();
}

fn container_types(c: &mut Criterion) {
    use ghilhouse::List;
    use ssz_arena::ExecutionPayloadHeader;

    type C = typenum::U1000000;
    const N: u64 = 1_000;

    let mut group = c.benchmark_group("Ghilhouse ExecutionPayloadHeader List");

    // basic test case
    let size: usize = 1000;
    let iter = vec![ExecutionPayloadHeader::default(); size];
    let list = List::<ExecutionPayloadHeader, C>::try_from_iter(iter).unwrap();

    let list_bytes = list.to_ssz();

    group.throughput(Throughput::Bytes(list_bytes.len() as u64));
    group.sample_size(10);
    group.sampling_mode(SamplingMode::Flat);

    #[cfg(feature = "sszb")]
    group.bench_with_input(
        BenchmarkId::new("Sszb", "decode"),
        list_bytes.as_slice(),
        |b, bytes| {
            b.iter(|| {
                <List<ExecutionPayloadHeader, C> as SszDecode>::from_ssz_bytes(bytes).unwrap()
            })
        },
    );
    #[cfg(feature = "sszb")]
    group.bench_with_input(BenchmarkId::new("Sszb", "encode"), &list, |b, list| {
        b.iter(|| list.to_ssz())
    });
    #[cfg(feature = "sszb")]
    group.bench_with_input(
        BenchmarkId::new("Sszb", "encode with slice"),
        &list,
        |b, list| {
            let len = SszEncode::ssz_bytes_len(list);
            let mut buf: Vec<u8> = vec![0u8; len];
            b.iter(|| list.ssz_write(&mut buf.as_mut_slice()))
        },
    );

    #[cfg(feature = "sigp")]
    group.bench_with_input(
        BenchmarkId::new("Sigp", "decode"),
        list_bytes.as_slice(),
        |b, bytes| {
            b.iter(|| <List<ExecutionPayloadHeader, C> as Decode>::from_ssz_bytes(bytes).unwrap())
        },
    );

    group.finish();
}

#[cfg(feature = "block")]
fn beacon_block(c: &mut Criterion) {
    use ssz_arena::{get_block_bytes, SignedBeaconBlock};

    let mut group = c.benchmark_group("SignedBeaconBlock");
    let block_bytes: Vec<u8> =
        std::fs::read("beacon-block.ssz").unwrap_or_else(|_| get_block_bytes().unwrap());
    group.throughput(Throughput::Bytes(block_bytes.len() as u64));

    #[cfg(feature = "sszb")]
    group.bench_with_input(
        BenchmarkId::new("Sszb", "decode"),
        block_bytes.as_slice(),
        |b, bytes| b.iter(|| <SignedBeaconBlock as SszDecode>::from_ssz_bytes(bytes).unwrap()),
    );
    #[cfg(feature = "sszb")]
    let beacon_block =
        <SignedBeaconBlock as SszDecode>::from_ssz_bytes(block_bytes.as_slice()).unwrap();
    #[cfg(feature = "sszb")]
    group.bench_with_input(
        BenchmarkId::new("Sszb", "encode"),
        &beacon_block,
        |b, block| b.iter(|| block.to_ssz()),
    );
    #[cfg(feature = "sszb")]
    let beacon_block =
        <SignedBeaconBlock as SszDecode>::from_ssz_bytes(block_bytes.as_slice()).unwrap();
    #[cfg(feature = "sszb")]
    group.bench_with_input(
        BenchmarkId::new("Sszb", "encode to slice"),
        &beacon_block,
        |b, block| {
            let len = SszEncode::ssz_bytes_len(block);
            let mut buf: Vec<u8> = vec![0u8; len];
            b.iter(|| block.ssz_write(&mut buf.as_mut_slice()))
        },
    );

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

    #[cfg(feature = "sigp")]
    let beacon_block = SigpBeaconBlock::<MainnetEthSpec>::from_ssz_bytes_for_fork(
        block_bytes.as_slice(),
        ForkName::Deneb,
    )
    .unwrap();

    #[cfg(feature = "sigp")]
    group.bench_with_input(
        BenchmarkId::new("Lighthouse", "encode"),
        &beacon_block,
        |b, block| b.iter(|| sigp_block_encode(block)),
    );

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

    #[cfg(feature = "grandine")]
    let beacon_block = GrandineBeaconBlock::<Mainnet>::from_ssz_unchecked(
        &Config::mainnet(),
        block_bytes.as_slice(),
    )
    .unwrap();
    #[cfg(feature = "grandine")]
    group.bench_with_input(
        BenchmarkId::new("Grandine", "encode"),
        &beacon_block,
        |b, bytes| b.iter(|| beacon_block.to_ssz()),
    );

    group.finish();
}

#[cfg(feature = "state")]
fn beacon_state(c: &mut Criterion) {
    use ssz_arena::{get_state_bytes, BeaconState};

    #[cfg(feature = "sigp")]
    use sigp_types::{BeaconState as SigpBeaconState, ChainSpec, MainnetEthSpec};

    let mut group = c.benchmark_group("BeaconState");
    let state_bytes: Vec<u8> =
        std::fs::read("beacon-state.ssz").unwrap_or_else(|_| get_state_bytes().unwrap());
    let beacon_state = <BeaconState as SszDecode>::from_ssz_bytes(state_bytes.as_slice()).unwrap();
    group.throughput(Throughput::Bytes(state_bytes.len() as u64));
    group.sample_size(10);
    group.sampling_mode(SamplingMode::Flat);

    #[cfg(feature = "sszb")]
    group.bench_with_input(
        BenchmarkId::new("Sszb", "encode"),
        &beacon_state,
        |b, state: &BeaconState| {
            let len = SszEncode::ssz_bytes_len(state);
            let mut buf: Vec<u8> = vec![0u8; len];
            b.iter(|| state.ssz_write(&mut buf.as_mut_slice()))
        },
    );

    #[cfg(feature = "sszb")]
    group.bench_with_input(
        BenchmarkId::new("Sszb", "decode"),
        state_bytes.as_slice(),
        |b, bytes| {
            // b.iter(|| <BeaconState as SszDecode>::from_ssz_bytes(bytes).unwrap())
            b.iter_batched_ref(
                || bytes.clone(),
                |bytes| {
                    let state =
                        <BeaconState as SszDecode>::from_ssz_bytes(bytes).expect("should decode");
                    black_box(state)
                },
                BatchSize::SmallInput,
            )
        },
    );

    // #[cfg(feature = "sigp")]
    // group.bench_with_input(
    //     BenchmarkId::new("Lighthouse", "encode"),
    //     state_bytes.as_slice(),
    //     |b, bytes| {
    //         let state: SigpBeaconState<MainnetEthSpec> =
    //             SigpBeaconState::from_ssz_bytes(bytes, &ChainSpec::default()).unwrap();
    //         b.iter(|| ssz_encode(&state))
    //     },
    // );

    #[cfg(feature = "sigp")]
    group.bench_with_input(
        BenchmarkId::new("Lighthouse", "decode"),
        state_bytes.as_slice(),
        |b, bytes| {
            // b.iter(|| {
            //     SigpBeaconState::<MainnetEthSpec>::from_ssz_bytes(bytes, &ChainSpec::default())
            //         .unwrap()
            // })
            b.iter_batched_ref(
                || bytes.clone(),
                |bytes| {
                    let state: SigpBeaconState<MainnetEthSpec> =
                        SigpBeaconState::from_ssz_bytes(&bytes, &ChainSpec::default())
                            .expect("should decode");
                    black_box(state)
                },
                BatchSize::SmallInput,
            )
        },
    );

    #[cfg(feature = "grandine")]
    group.bench_with_input(
        BenchmarkId::new("Grandine", "decode"),
        state_bytes.as_slice(),
        |b, bytes| {
            b.iter(|| {
                GrandineBeaconState::<Mainnet>::from_ssz_unchecked(&Config::mainnet(), bytes)
                    .unwrap()
            })
        },
    );

    group.finish();
}

#[cfg(feature = "block")]
criterion_group!(light_benches, basic_types, container_types, beacon_block);

#[cfg(not(feature = "block"))]
criterion_group!(light_benches, basic_types, container_types);

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
