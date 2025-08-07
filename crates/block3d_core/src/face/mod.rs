use crate::connection::OrientedInterface;
use serde::{Serialize, Deserialize};
#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq)]
pub struct Face {
   oriented_interface: OrientedInterface
}

impl Face {
    pub fn new(oriented_interface: OrientedInterface) -> Self {
        Self { oriented_interface }
    }
    
    pub fn oriented_interface(&self) -> OrientedInterface {
        self.oriented_interface
    }

    pub fn can_connect_to(&self, other: &Self) -> bool {
        // Use the existing connection system to check if these interfaces can connect
        (self.oriented_interface + other.oriented_interface).is_some()
    }
}
