use super::interface::{GcData, ResourceHandle};

/// False for handles of collected resources.
pub fn is_alive(data: &GcData, handle: ResourceHandle) -> bool {
    data.entries
        .get(handle.slot as usize)
        .map_or(false, |entry| entry.alive && entry.generation == handle.generation)
}

pub fn ref_count(data: &GcData, handle: ResourceHandle) -> Option<u32> {
    data.entries
        .get(handle.slot as usize)
        .filter(|entry| entry.alive && entry.generation == handle.generation)
        .map(|entry| entry.ref_count)
}

pub fn alive_count(data: &GcData) -> usize {
    data.entries.iter().filter(|entry| entry.alive).count()
}
