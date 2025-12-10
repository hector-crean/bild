use bevy::prelude::*;


// [Part]                              [Net]
//   ^                                    ^
//   |                                    |
//   |                                    +---------------------------+
//   |                                                                    |
//   | reverse index                                                      | reverse index
//   |                                                                    |
// [Pins(Vec<Entity>)]  --contains-->  [Pin]  -- OnNet -->  [Net]      [NetPins(Vec<Entity>)]
//        ^                                 ^                                  ^
//        |                                 |                                  |
//        +---------- OfPart ---------------+                                  +-- contains Pins


/// Relationship: `Pin` belongs to a `Part`.
#[derive(Component, Debug, Clone, Copy)]
#[relationship(relationship_target = Pins)]
pub struct OfPart(pub Entity);

/// Reverse index of a part's pins.
#[derive(Component, Debug, Default)]
#[relationship_target(relationship = OfPart)]
pub struct Pins(Vec<Entity>);

impl Pins {
    /// Iterate contained pins in stored order.
    pub fn iter(&self) -> impl ExactSizeIterator<Item = Entity> + '_ { self.0.iter().copied() }
}

/// Relationship: `Pin` is on a `Net`.
#[derive(Component, Debug, Clone, Copy)]
#[relationship(relationship_target = NetPins)]
pub struct OnNet(pub Entity);

/// Reverse index: all pins that are on a net.
#[derive(Component, Debug, Default)]
#[relationship_target(relationship = OnNet)]
pub struct NetPins(Vec<Entity>);

impl NetPins {
    /// Iterate pins contained in this net.
    pub fn iter(&self) -> impl ExactSizeIterator<Item = Entity> + '_ { self.0.iter().copied() }
}



















