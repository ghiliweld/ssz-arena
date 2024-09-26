# ssz-arena
The arena where ssz crates duke it out!

## Installation
```sh
cargo build
```

## Usage

### Wall Time Benchmarking

```sh
cargo bench --bench wall_time
```

### Allocation Benchmarking

```sh
cargo bench --bench alloc
```

### Benchmarking Features

Beacon Block benchmarking with lighthouse and grandine crates:
```sh
cargo bench --bench <bench> --features block,sigp,grandine
```

Beacon State benchmarking with grandine:
```sh
cargo bench --bench <bench> --features state,grandine
```

By default, the suite will use the latest checkpoint state served on [sync-mainnet.beaconcha.in](https://sync-mainnet.beaconcha.in/).
However, the requests do add a bit of latency to the benchmarks. Users can override this by adding their own `beacon-block.ssz` and `beacon-state.ssz` files to the root directory.
This will perform decoding on the provided files.
