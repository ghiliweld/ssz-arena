mod checkpointz;
pub use checkpointz::{get_block_bytes, get_latest_served_checkpoint_slot, get_state_bytes};

mod mock_struct;
pub use mock_struct::Foo;

mod beacon_block;
pub use beacon_block::SignedBeaconBlock;
