use milhouse::List;
#[cfg(all(feature = "sigp", feature = "state"))]
use sigp_types::{BeaconState as SigpBeaconState, ChainSpec, MainnetEthSpec};
#[cfg(all(feature = "sigp", feature = "block"))]
use sigp_types::{ForkName, MainnetEthSpec, SignedBeaconBlock as SigpBeaconBlock};
use ssz::{Decode, Encode};
#[cfg(feature = "block")]
use ssz_arena::get_block_bytes;
#[cfg(feature = "state")]
use ssz_arena::get_state_bytes;

#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

fn milhouse_decode_profiling() {
    type C = typenum::U1099511627776;
    const N: u64 = 1_000_000;

    // basic test case
    let size = N;
    let list = List::<u64, C>::try_from_iter(0..size).unwrap();
    let list_bytes = list.as_ssz_bytes();

    let _profiler = dhat::Profiler::builder()
        .testing()
        .trim_backtraces(None)
        .build();

    let start_stats = dhat::HeapStats::get();

    // measure decoding
    let _list = <List<u64, C> as Decode>::from_ssz_bytes(list_bytes.as_slice()).unwrap();

    // print decoding stats
    let end_stats = dhat::HeapStats::get();

    println!("== milhouse list decoding");
    println!(
        "   ├─ allocated {} bytes",
        end_stats.curr_bytes - start_stats.curr_bytes
    );
    println!(
        "   ╰─ allocated {} times",
        end_stats.curr_blocks - start_stats.curr_blocks
    );
}

fn milhouse_encode_profiling() {
    type C = typenum::U1099511627776;
    const N: u64 = 1_000_000;

    // basic test case
    let size = N;
    let list = List::<u64, C>::try_from_iter(0..size).unwrap();

    let _profiler = dhat::Profiler::builder()
        .testing()
        .trim_backtraces(None)
        .build();

    let start_stats = dhat::HeapStats::get();

    // measure encoding
    let _list_bytes = list.as_ssz_bytes();

    // print encoding stats
    let end_stats = dhat::HeapStats::get();

    println!("== milhouse list encoding");
    println!(
        "   ├─ allocated {} bytes",
        end_stats.curr_bytes - start_stats.curr_bytes
    );
    println!(
        "   ╰─ allocated {} times",
        end_stats.curr_blocks - start_stats.curr_blocks
    );
}

#[cfg(all(feature = "sigp", feature = "block"))]
fn sigp_block_decode_profiling() {
    let beacon_block_bytes: Vec<u8> =
        std::fs::read("beacon-block.ssz").unwrap_or(get_block_bytes().unwrap());

    let _profiler = dhat::Profiler::builder()
        .testing()
        .trim_backtraces(None)
        .build();

    let start_stats = dhat::HeapStats::get();

    // measure decoding
    let _beacon_block = SigpBeaconBlock::<MainnetEthSpec>::from_ssz_bytes_for_fork(
        beacon_block_bytes.as_slice(),
        ForkName::Deneb,
    )
    .unwrap();

    // print decoding stats
    let end_stats = dhat::HeapStats::get();

    println!("== beacon block decoding");
    println!(
        "   ├─ allocated {} bytes",
        end_stats.curr_bytes - start_stats.curr_bytes
    );
    println!(
        "   ╰─ allocated {} times",
        end_stats.curr_blocks - start_stats.curr_blocks
    );
}

#[cfg(all(feature = "sigp", feature = "block"))]
fn sigp_block_encode_profiling() {
    let beacon_block_bytes: Vec<u8> =
        std::fs::read("beacon-block.ssz").unwrap_or(get_block_bytes().unwrap());
    let beacon_block = SigpBeaconBlock::<MainnetEthSpec>::from_ssz_bytes_for_fork(
        beacon_block_bytes.as_slice(),
        ForkName::Deneb,
    )
    .unwrap();

    let _profiler = dhat::Profiler::builder()
        .testing()
        .trim_backtraces(None)
        .build();

    let start_stats = dhat::HeapStats::get();

    // measure encoding
    //let bytes = beacon_block.as_ssz_bytes();

    // print encoding stats
    let end_stats = dhat::HeapStats::get();

    println!("== beacon block encoding");
    println!(
        "   ├─ allocated {} bytes",
        end_stats.curr_bytes - start_stats.curr_bytes
    );
    println!(
        "   ╰─ allocated {} times",
        end_stats.curr_blocks - start_stats.curr_blocks
    );
}

fn main() {
    milhouse_decode_profiling();
    milhouse_encode_profiling();
    #[cfg(all(feature = "sigp", feature = "block"))]
    sigp_block_decode_profiling();
    #[cfg(all(feature = "sigp", feature = "block"))]
    sigp_block_encode_profiling();
}
