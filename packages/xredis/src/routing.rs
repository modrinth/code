use std::collections::HashMap;

use redis::cluster_routing::{
    MultiSlotArgPattern, MultipleNodeRoutingInfo, ResponsePolicy, Route,
    RoutingInfo, SlotAddr,
};

/// Returns a routing specification to split an MGET by hash slots and
/// route to **primaries only**.
///
/// Use this if you need a command to be routed to primaries only.
///
/// It's not needed just to split an MGET; redis-rs already does that by default.
pub(crate) fn primary_mget_routing<K: AsRef<[u8]>>(keys: &[K]) -> RoutingInfo {
    let mut keys_by_slot: HashMap<Route, Vec<usize>> = HashMap::new();

    for (index, key) in keys.iter().enumerate() {
        let route = Route::with_key(key.as_ref(), SlotAddr::Master);
        keys_by_slot.entry(route).or_default().push(index);
    }

    RoutingInfo::MultiNode((
        MultipleNodeRoutingInfo::MultiSlot((
            keys_by_slot.into_iter().collect(),
            MultiSlotArgPattern::KeysOnly,
        )),
        Some(ResponsePolicy::CombineArrays),
    ))
}
