use bevy::prelude::*;

use crate::circuit::net::Net;
use crate::circuit::relations::{OnNet, NetPins, OfPart, Pins};

/// Call `f` for each pin owned by `part` (stored order).
pub fn for_each_pin_of_part(
    pins_index: &Query<&Pins>,
    part: Entity,
    mut f: impl FnMut(Entity),
) {
    if let Ok(pins) = pins_index.get(part) {
        for pin in pins.iter() {
            f(pin);
        }
    }
}

/// Call `f` for each pin that is a member of `net` (stored order).
pub fn for_each_pin_on_net(
    net_members: &Query<&NetPins>,
    net: Entity,
    mut f: impl FnMut(Entity),
) {
    if let Ok(members) = net_members.get(net) {
        for pin in members.iter() {
            f(pin);
        }
    }
}

/// Owning part of `pin`, if any (via OfPart).
pub fn part_of_pin(of_part: &Query<&OfPart>, pin: Entity) -> Option<Entity> {
    of_part.get(pin).ok().map(|r| r.0)
}

/// Connected net of `pin`, if any (via OnNet).
pub fn net_of_pin(connected: &Query<&OnNet>, pin: Entity) -> Option<Entity> {
    connected.get(pin).ok().map(|r| r.0)
}

/// All nets touched by `part` (deduplicated).
pub fn nets_of_part_collect(
    pins_index: &Query<&Pins>,
    connected: &Query<&OnNet>,
    part: Entity,
) -> Vec<Entity> {
    let mut seen: std::collections::HashSet<Entity> = std::collections::HashSet::new();
    for_each_pin_of_part(pins_index, part, |pin| {
        if let Some(net) = net_of_pin(connected, pin) {
            seen.insert(net);
        }
    });
    seen.into_iter().collect()
}

/// All parts on `net` (via member pins). Deduplicated.
pub fn parts_on_net_collect(
    net_members: &Query<&NetPins>,
    of_part: &Query<&OfPart>,
    net: Entity,
) -> Vec<Entity> {
    let mut seen: std::collections::HashSet<Entity> = std::collections::HashSet::new();
    for_each_pin_on_net(net_members, net, |pin| {
        if let Some(owner) = part_of_pin(of_part, pin) {
            seen.insert(owner);
        }
    });
    seen.into_iter().collect()
}

/// All parts sharing any net with `part` (excluding `part`). Deduplicated.
pub fn parts_connected_to_part_collect(
    pins_index: &Query<&Pins>,
    connected: &Query<&OnNet>,
    net_members: &Query<&NetPins>,
    of_part: &Query<&OfPart>,
    part: Entity,
) -> Vec<Entity> {
    let mut connected_parts: std::collections::HashSet<Entity> = std::collections::HashSet::new();
    for net in nets_of_part_collect(pins_index, connected, part) {
        for other in parts_on_net_collect(net_members, of_part, net) {
            if other != part {
                connected_parts.insert(other);
            }
        }
    }
    connected_parts.into_iter().collect()
}

/// Netlist view: for each net, list (part, pin) pairs.
pub fn netlist_collect(
    nets: &Query<Entity, With<Net>>,
    net_members: &Query<&NetPins>,
    of_part: &Query<&OfPart>,
) -> Vec<(Entity /*net*/, Vec<(Entity /*part*/, Entity /*pin*/)> )> {
    let mut out: Vec<(Entity, Vec<(Entity, Entity)>)> = Vec::new();
    for net in nets.iter() {
        let mut members_pp: Vec<(Entity, Entity)> = Vec::new();
        for_each_pin_on_net(net_members, net, |pin| {
            if let Some(owner) = part_of_pin(of_part, pin) {
                members_pp.push((owner, pin));
            }
        });
        out.push((net, members_pp));
    }
    out
}





