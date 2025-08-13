use bevy::prelude::*;

use crate::circuit::net::Net;

use crate::circuit::net::NetKind;
use crate::circuit::relations::OnNet;
use crate::circuit::relations::OfPart;
use crate::circuit::part::Part;
use crate::circuit::pin::Pin;

/// Ergonomic helpers for building small circuit graphs.
pub trait CommandsCircuitExt {
    /// Spawn a net with a name.
    fn spawn_net(&mut self, name: impl Into<String>) -> Entity;
    /// Spawn a net with a name and explicit kind.
    fn spawn_net_with_kind(&mut self, name: impl Into<String>, kind: NetKind) -> Entity;
    /// Spawn a part with refdes, kind, value and return the part entity and created pins.
    fn spawn_part_with_pins(
        &mut self,
        refdes: &str,
        part: Part,
        pin_names: &[&str],
    ) -> (Entity, Vec<Entity>);
    /// Connect a `Pin` to a `Net` (inserts/updates `ConnectedToNet`).
    fn connect_pin_to_net(&mut self, pin: Entity, net: Entity);
    /// Connect multiple `Pin`s to a `Net`.
    fn connect_pins_to_net(&mut self, pins: &[Entity], net: Entity);
    /// Create a net and connect provided pins to it. Returns the new net entity.
    fn create_net_and_connect(&mut self, name: impl Into<String>, pins: &[Entity]) -> Entity;
    /// Create a net by name and connect two pins to it. Returns the new net entity.
    fn connect_between_new_net(
        &mut self,
        pin_a: Entity,
        pin_b: Entity,
        net_name: impl Into<String>,
    ) -> Entity;
    /// Disconnect a `Pin` from any `Net` (removes `OnNet` if present).
    fn disconnect_pin(&mut self, pin: Entity);
}

impl CommandsCircuitExt for Commands<'_, '_> {
    fn spawn_net(&mut self, name: impl Into<String>) -> Entity {
        let name_string: String = name.into();
        self.spawn((Net::new(NetKind::Signal), Name::new(name_string))).id()
    }

    fn spawn_net_with_kind(&mut self, name: impl Into<String>, kind: NetKind) -> Entity {
        let name_string: String = name.into();
        self.spawn((Net::new(kind), Name::new(name_string))).id()
    }

    fn spawn_part_with_pins(
        &mut self,
        refdes: &str,
        part: Part,
        pin_names: &[&str],
    ) -> (Entity, Vec<Entity>) {
        let part_entity = self
            .spawn((
                part,
                Name::new(refdes.to_string()),
            ))
            .id();

        let mut pin_entities = Vec::with_capacity(pin_names.len());
        for (i, pin_name) in pin_names.iter().enumerate() {
            let pin_entity = self
                .spawn((
                    Pin { name: (*pin_name).to_string(), index: (i as u8) + 1 },
                    Name::new(format!("{refdes}.{}", pin_name)),
                    OfPart(part_entity),
                ))
                .id();
            pin_entities.push(pin_entity);
        }

        (part_entity, pin_entities)
    }

    fn connect_pin_to_net(&mut self, pin: Entity, net: Entity) {
        self.entity(pin).insert(OnNet(net));
    }

    fn connect_pins_to_net(&mut self, pins: &[Entity], net: Entity) {
        for &pin in pins {
            self.entity(pin).insert(OnNet(net));
        }
    }

    fn create_net_and_connect(&mut self, name: impl Into<String>, pins: &[Entity]) -> Entity {
        let net = self.spawn_net(name);
        self.connect_pins_to_net(pins, net);
        net
    }

    fn connect_between_new_net(
        &mut self,
        pin_a: Entity,
        pin_b: Entity,
        net_name: impl Into<String>,
    ) -> Entity {
        let net = self.spawn_net(net_name);
        self.connect_pin_to_net(pin_a, net);
        self.connect_pin_to_net(pin_b, net);
        net
    }

    fn disconnect_pin(&mut self, pin: Entity) {
        self.entity(pin).remove::<OnNet>();
    }
}


