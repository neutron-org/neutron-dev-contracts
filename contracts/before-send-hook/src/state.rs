use cw_storage_plus::Item;

/// contains number of transfers to addresses observed by the contract.
pub const SUDO_RES_BLOCK: Item<bool> = Item::new("sudo_res_block");
pub const SUDO_RES_TRACK: Item<bool> = Item::new("sudo_res_track");
