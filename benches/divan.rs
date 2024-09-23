use divan::{AllocProfiler, Divan};

pub mod checkpointz;
pub mod test_struct;

#[global_allocator]
static ALLOC: AllocProfiler = AllocProfiler::system();

fn main() {
    // Run registered benchmarks.
    Divan::default().color(true).main();
}

#[divan::bench_group]
mod default {
    use crate::test_struct::Foo;
    use divan::Bencher;
    use milhouse::List;
    use ssz::{Decode, Encode};

    type C = typenum::U1099511627776;
    const N: u64 = 1_000_000;

    #[divan::bench]
    fn encode_list(bencher: Bencher) {
        bencher
            .with_inputs(|| {
                let list = List::<u64, C>::try_from_iter(0..N).unwrap();
                list
            })
            .bench_values(|list| list.as_ssz_bytes());
    }

    #[divan::bench]
    fn decode_list(bencher: Bencher) {
        let list = List::<u64, C>::try_from_iter(0..N).unwrap();

        bencher
            .with_inputs(|| list.as_ssz_bytes())
            .bench_values(|bytes: Vec<u8>| {
                List::<u64, C>::from_ssz_bytes(bytes.as_slice()).unwrap()
            });
    }

    #[divan::bench]
    fn encode_list_of_structs(bencher: Bencher) {
        let size: usize = N as usize;

        bencher
            .with_inputs(|| {
                let my_foo = Foo {
                    a: 42,
                    b: vec![0, 1, 2, 3],
                    c: 11,
                };
                let vec = vec![my_foo; size];
                vec
            })
            .bench_values(|vec: Vec<Foo>| vec.as_ssz_bytes());
    }

    #[divan::bench]
    fn decode_list_of_structs(bencher: Bencher) {
        let size: usize = N as usize;
        let my_foo = Foo {
            a: 42,
            b: vec![0, 1, 2, 3],
            c: 11,
        };
        let vec = vec![my_foo; size];

        bencher
            .with_inputs(|| vec.as_ssz_bytes())
            .bench_values(|bytes: Vec<u8>| Vec::<Foo>::from_ssz_bytes(bytes.as_slice()).unwrap());
    }
}

#[cfg(feature = "sigp")]
#[divan::bench_group]
mod sigp {
    use crate::checkpointz::{get_block_bytes, get_state_bytes};
    use divan::Bencher;
    use sigp_types::{
        BeaconState as SigpBeaconState, ChainSpec, ForkName, MainnetEthSpec,
        SignedBeaconBlock as SigpBeaconBlock,
    };
    use ssz::ssz_encode;

    #[cfg(feature = "block")]
    #[divan::bench]
    fn decode_sigp_beacon_block(bencher: Bencher) {
        let bytes = get_block_bytes().unwrap();

        bencher.bench_local(move || {
            SigpBeaconBlock::<MainnetEthSpec>::from_ssz_bytes_for_fork(
                bytes.as_slice(),
                ForkName::Deneb,
            )
        });
    }

    #[cfg(feature = "block")]
    #[divan::bench]
    fn encode_sigp_beacon_block(bencher: Bencher) {
        let bytes = get_block_bytes().unwrap();

        bencher
            .with_inputs(move || {
                let block = SigpBeaconBlock::<MainnetEthSpec>::from_ssz_bytes_for_fork(
                    bytes.as_slice(),
                    ForkName::Deneb,
                )
                .unwrap();
                block
            })
            .bench_values(|block| block.as_ssz_bytes());
    }

    #[cfg(feature = "state")]
    #[divan::bench]
    fn decode_sigp_beacon_state(bencher: Bencher) {
        let bytes = get_state_bytes().unwrap();

        bencher.bench_local(move || {
            SigpBeaconState::<MainnetEthSpec>::from_ssz_bytes(
                bytes.as_slice(),
                &ChainSpec::default(),
            )
        });
    }

    #[cfg(feature = "state")]
    #[divan::bench]
    fn encode_sigp_beacon_state(bencher: Bencher) {
        let bytes = get_state_bytes().unwrap();

        bencher
            .with_inputs(move || {
                let state = SigpBeaconState::<MainnetEthSpec>::from_ssz_bytes(
                    bytes.as_slice(),
                    &ChainSpec::default(),
                )
                .unwrap();
                state
            })
            .bench_values(|state| state.as_ssz_bytes());
    }
}

#[cfg(feature = "grandine")]
#[divan::bench_group]
mod grandine {
    use crate::checkpointz::{get_block_bytes, get_state_bytes};
    use divan::Bencher;
    use grandine_ssz::SszRead;
    use grandine_types::{
        combined::{BeaconState as GrandineBeaconState, SignedBeaconBlock as GrandineBeaconBlock},
        config::Config,
        preset::Mainnet,
    };

    #[cfg(feature = "block")]
    #[divan::bench]
    fn decode_grandine_beacon_block(bencher: Bencher) {
        let bytes = get_block_bytes().unwrap();

        bencher.bench_local(move || {
            GrandineBeaconBlock::<Mainnet>::from_ssz_unchecked(&Config::mainnet(), bytes.as_slice())
        });
    }

    #[cfg(feature = "state")]
    #[divan::bench]
    fn decode_grandine_beacon_state(bencher: Bencher) {
        let bytes = get_state_bytes().unwrap();

        bencher.bench_local(move || {
            GrandineBeaconState::<Mainnet>::from_ssz_unchecked(&Config::mainnet(), bytes.as_slice())
        });
    }
}
