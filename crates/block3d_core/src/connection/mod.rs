use std::ops::Add;
use serde::{Deserialize, Serialize};

use crate::Orientation;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Connector {
    /// Thermal conduction path
    ThermalConduction,
    /// Electrical connection (wire bond, solder ball, etc.)
    ElectricalConnection,
    /// Mechanical bonding (adhesive, solder joint, etc.)
    MechanicalBond,
    /// Thermal interface material connection
    ThermalInterface,
    /// Direct die attach
    DieAttach,
    /// Package to board connection
    PackageConnection,
    /// Heat sink attachment
    HeatSinkAttachment,
}

/// Semiconductor component interfaces
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum ConnectorInterface {
    /// Active silicon surface (with transistors)
    ActiveSurface,
    /// Passive silicon surface (backside)
    PassiveSurface,
    /// Metal pad for electrical connection
    MetalPad,
    /// Solder ball interface
    SolderBall,
    /// Wire bond pad
    WireBondPad,
    /// Thermal pad for heat conduction
    ThermalPad,
    /// Mechanical mounting surface
    MountingSurface,
    /// Heat sink interface
    HeatSinkInterface,
    /// Underfill interface
    UnderfillInterface,
    /// PCB trace connection
    PCBTrace,
    /// Via connection (through substrate)
    Via,
    /// Air gap (for thermal modeling)
    AirGap,
}

impl ConnectorInterface {
    /// If this interface has a "natural inverse," return it. Otherwise, None.
    pub fn inverse(&self) -> Option<Self> {
        use ConnectorInterface::*;
        match self {
            ActiveSurface => Some(ThermalPad),
            PassiveSurface => Some(ThermalPad),
            MetalPad => Some(SolderBall),
            SolderBall => Some(MetalPad),
            WireBondPad => Some(MetalPad),
            ThermalPad => Some(HeatSinkInterface),
            HeatSinkInterface => Some(ThermalPad),
            MountingSurface => Some(MountingSurface),
            _ => None,
        }
    }
}

/// An oriented interface: The interface + how it's rotated/flipped.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub struct OrientedInterface {
    pub interface: ConnectorInterface,
    pub orientation: Orientation,
}

/// A connection is (Interface1, Option<Connector>, Interface2) 
/// with orientation info included for extra realism.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct OrientedConnection {
    pub left: OrientedInterface,
    pub connector: Option<Connector>,
    pub right: OrientedInterface,
}

impl Add for OrientedInterface {
    type Output = Option<OrientedConnection>;

    fn add(self, rhs: Self) -> Self::Output {
        use Connector::*;
        
        // Helper macro to build an OrientedConnection
        macro_rules! connection {
            ($c:expr) => {
                Some(OrientedConnection {
                    left: self,
                    connector: Some($c),
                    right: rhs,
                })
            };
        }

        match (self.interface, rhs.interface) {
            // Die to substrate thermal connection
            (ConnectorInterface::PassiveSurface, ConnectorInterface::ThermalPad) | 
            (ConnectorInterface::ThermalPad, ConnectorInterface::PassiveSurface) => connection!(ThermalConduction),
            
            // Electrical connections
            (ConnectorInterface::MetalPad, ConnectorInterface::SolderBall) | 
            (ConnectorInterface::SolderBall, ConnectorInterface::MetalPad) => connection!(ElectricalConnection),
            
            (ConnectorInterface::WireBondPad, ConnectorInterface::MetalPad) => connection!(ElectricalConnection),
            
            // Thermal interface connections
            (ConnectorInterface::ThermalPad, ConnectorInterface::HeatSinkInterface) | 
            (ConnectorInterface::HeatSinkInterface, ConnectorInterface::ThermalPad) => connection!(ThermalInterface),
            
            // Mechanical connections
            (ConnectorInterface::MountingSurface, ConnectorInterface::MountingSurface) => connection!(MechanicalBond),
            
            // Die attach
            (ConnectorInterface::ActiveSurface, ConnectorInterface::ThermalPad) => connection!(DieAttach),
            
            // PCB connections
            (ConnectorInterface::SolderBall, ConnectorInterface::PCBTrace) | 
            (ConnectorInterface::PCBTrace, ConnectorInterface::SolderBall) => connection!(PackageConnection),
            
            // Via connections
            (ConnectorInterface::Via, ConnectorInterface::PCBTrace) | 
            (ConnectorInterface::PCBTrace, ConnectorInterface::Via) => connection!(ElectricalConnection),
            
            _ => None,
        }
    }
}

// Convenience method
impl OrientedInterface {
    pub fn connect(self, other: OrientedInterface) -> Option<OrientedConnection> {
        self + other
    }
}

/// Analog circuit specific connectors
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AnalogConnector {
    /// Direct electrical connection (wire, trace)
    ElectricalWire,
    /// Capacitive coupling
    CapacitiveCoupling,
    /// Inductive coupling (transformer, mutual inductance)
    InductiveCoupling,
    /// Ground connection
    GroundPlane,
    /// Power supply connection
    PowerRail,
    /// Signal trace
    SignalTrace,
    /// Differential pair
    DifferentialPair,
    /// Coaxial connection
    CoaxialLine,
    /// Transmission line
    TransmissionLine,
}

/// Analog circuit interfaces
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum AnalogInterface {
    /// Component pin/lead
    ComponentPin,
    /// Test point
    TestPoint,
    /// Power pin (VCC, VDD, etc.)
    PowerPin,
    /// Ground pin (GND, VSS, etc.)
    GroundPin,
    /// Signal input
    SignalInput,
    /// Signal output
    SignalOutput,
    /// Differential input positive
    DifferentialInputPlus,
    /// Differential input negative
    DifferentialInputMinus,
    /// Analog input
    AnalogInput,
    /// Analog output
    AnalogOutput,
    /// Control input
    ControlInput,
    /// Feedback connection
    FeedbackConnection,
}

impl AnalogInterface {
    /// Check if this interface can connect to another
    pub fn can_connect_to(&self, other: &Self) -> bool {
        use AnalogInterface::*;
        match (self, other) {
            // Power connections
            (PowerPin, PowerPin) => true,
            (GroundPin, GroundPin) => true,
            
            // Signal connections
            (SignalOutput, SignalInput) => true,
            (SignalInput, SignalOutput) => true,
            (AnalogOutput, AnalogInput) => true,
            (AnalogInput, AnalogOutput) => true,
            
            // Differential pairs
            (DifferentialInputPlus, DifferentialInputPlus) => false, // Can't connect same polarity
            (DifferentialInputMinus, DifferentialInputMinus) => false,
            (DifferentialInputPlus, DifferentialInputMinus) => true,
            (DifferentialInputMinus, DifferentialInputPlus) => true,
            
            // Test points can connect to anything
            (TestPoint, _) => true,
            (_, TestPoint) => true,
            
            // Component pins are general purpose
            (ComponentPin, _) => true,
            (_, ComponentPin) => true,
            
            // Control and feedback
            (ControlInput, SignalOutput) => true,
            (SignalOutput, ControlInput) => true,
            (FeedbackConnection, _) => true,
            (_, FeedbackConnection) => true,
            
            _ => false,
        }
    }
}



