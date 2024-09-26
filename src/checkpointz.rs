use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct CheckpointzSlotsResponse {
    data: CheckpointzSlotsData,
}

#[derive(Deserialize, Debug)]
struct CheckpointzSlotsData {
    slots: Vec<CheckpointzSlot>,
}

#[derive(Deserialize, Debug)]
struct CheckpointzSlot {
    slot: String,
}

pub fn get_latest_served_checkpoint_slot() -> Result<String, reqwest::Error> {
    // response body has structure:
    // data: { slots: [ { slot, ... }, ...] }
    // we want the number at slot, so we want the laziest way of accessing body.data.slots[0].slot
    let body = reqwest::blocking::get(format!(
        "https://sync-mainnet.beaconcha.in/checkpointz/v1/beacon/slots"
    ))?
    .json::<CheckpointzSlotsResponse>()?;

    let slot = body.data.slots.first().unwrap().slot.clone();
    Ok(slot)
}

pub fn get_block_bytes() -> Result<Vec<u8>, reqwest::Error> {
    let slot = get_latest_served_checkpoint_slot()?;
    let bytes = reqwest::blocking::Client::new()
        .get(format!(
            "https://sync-mainnet.beaconcha.in/eth/v2/beacon/blocks/{}",
            slot
        ))
        .header("Accept", "application/octet-stream")
        .send()?
        .bytes()?;

    Ok(bytes.to_vec())
}

pub fn get_state_bytes() -> Result<Vec<u8>, reqwest::Error> {
    let slot = get_latest_served_checkpoint_slot()?;
    let bytes = reqwest::blocking::Client::new()
        .get(format!(
            "https://sync-mainnet.beaconcha.in/eth/v2/debug/beacon/states/{}",
            slot
        ))
        .header("Accept", "application/octet-stream")
        .send()?
        .bytes()?;

    Ok(bytes.to_vec())
}
