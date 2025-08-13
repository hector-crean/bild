
use bevy::prelude::*;





// A trace: continous conductive path belonging to one net
// That path can:
// Connect exactly two pins (simple point-to-point trace)
// Connect three or more pins (with a branch/junction along the trace)
// Include vias to jump layers
// Spread into planes or wide copper pours
// So a single trace segment can touch multiple pins if they share the same net.


//Net → made of one or more conductive regions (traces, vias, pours)
//Each conductive region → can touch multiple pins



// Can break routes into segments between juncture points






