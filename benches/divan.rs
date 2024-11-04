use divan::{AllocProfiler, Divan};

#[global_allocator]
static ALLOC: AllocProfiler = AllocProfiler::system();
fn main() {
    // Run registered benchmarks.
    Divan::default().color(true).main();
}

#[divan::bench_group]
mod sszb {
    use divan::Bencher;
    use milhouse::List;
    use ssz_arena::{get_block_bytes, SignedBeaconBlock};
    use sszb::{SszDecode, SszEncode};

    type C = typenum::U1099511627776;
    const N: u64 = 1_000_000;

    #[divan::bench]
    fn encode_list_naive(bencher: Bencher) {
        bencher
            .with_inputs(|| {
                let list = List::<u64, C>::try_from_iter(0..N).unwrap();
                list
            })
            .bench_values(|list| list.to_ssz());
    }

    #[divan::bench]
    fn encode_list_fast(bencher: Bencher) {
        bencher
            .with_inputs(|| {
                let list = List::<u64, C>::try_from_iter(0..N).unwrap();
                let len = SszEncode::ssz_bytes_len(&list);
                let buf: Vec<u8> = vec![0u8; len];
                (list, buf)
            })
            .bench_values(|(list, mut buf)| list.ssz_write(&mut buf.as_mut_slice()));
    }

    #[divan::bench]
    fn decode_list(bencher: Bencher) {
        let list = List::<u64, C>::try_from_iter(0..N).unwrap();
        bencher
            .with_inputs(|| list.to_ssz())
            .bench_values(|bytes: Vec<u8>| {
                List::<u64, C>::from_ssz_bytes(bytes.as_slice()).unwrap()
            });
    }

    #[cfg(feature = "block")]
    #[divan::bench]
    fn encode_beacon_block_naive(bencher: Bencher) {
        let bytes = get_block_bytes().unwrap();
        bencher
            .with_inputs(|| {
                let beacon_block =
                    <SignedBeaconBlock as SszDecode>::from_ssz_bytes(bytes.as_slice()).unwrap();
                beacon_block
            })
            .bench_values(|block| block.to_ssz());
    }

    #[cfg(feature = "block")]
    #[divan::bench]
    fn encode_beacon_block_fast(bencher: Bencher) {
        let bytes = get_block_bytes().unwrap();
        bencher
            .with_inputs(|| {
                let beacon_block =
                    <SignedBeaconBlock as SszDecode>::from_ssz_bytes(bytes.as_slice()).unwrap();
                let len = SszEncode::ssz_bytes_len(&beacon_block);
                let buf: Vec<u8> = vec![0u8; len];
                (beacon_block, buf)
            })
            .bench_values(move |(block, mut buf)| block.ssz_write(&mut buf.as_mut_slice()));
    }

    #[cfg(feature = "block")]
    #[divan::bench]
    fn decode_beacon_block(bencher: Bencher) {
        let bytes = std::fs::read("beacon-block.ssz").unwrap_or(get_block_bytes().unwrap());
        bencher.bench_local(move || {
            <SignedBeaconBlock as SszDecode>::from_ssz_bytes(bytes.as_slice()).unwrap()
        });
    }
}

#[cfg(feature = "sigp")]
#[divan::bench_group]
mod sigp {
    use divan::Bencher;
    use milhouse::List;
    use sigp_types::{
        BeaconState as SigpBeaconState, ChainSpec, ForkName, MainnetEthSpec,
        SignedBeaconBlock as SigpBeaconBlock,
    };
    use ssz::{Decode, Encode};
    use ssz_arena::{get_block_bytes, get_state_bytes};

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
                <List<u64, C> as Decode>::from_ssz_bytes(bytes.as_slice()).unwrap()
            });
    }

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

    impl<T: SszSize + milhouse::Value, N> SszSize for milhouse::List<T, N> {
        const SIZE: Size = Size::Variable { minimum_size: 0 };
    }

    impl<C, T: SszRead<C> + milhouse::Value, N: Unsigned> SszRead<C> for milhouse::List<T, N> {
        fn from_ssz_unchecked(context: &C, bytes: &[u8]) -> Result<Self, ReadError> {
            let results = shared::read_list(context, bytes)?;
            itertools::process_results(results, |elements| Self::try_from_iter(elements))?
        }
    }

    impl<T: SszWrite + milhouse::Value, N> SszWrite for milhouse::List<T, N> {
        fn write_variable(&self, bytes: &mut Vec<u8>) -> Result<(), WriteError> {
            shared::write_list(bytes, self)
        }
    }

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
