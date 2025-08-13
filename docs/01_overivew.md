Here's the plan: I’ll (1) outline how design flows in Cadence-style EDA toolchains work, then (2) map that to a Bevy app with a split-pane 2D schematic + 3D layout, with bidirectional sync and thermal simulation.

### How chip design is done today (Cadence, etc.)

- **Digital flow (ASIC/SoC)**
  - **Spec → RTL (SystemVerilog/VHDL) → Synthesis**: HDL → gate-level netlist using standard cells from a PDK.
  - **Floorplan → Place & Route (P&R)**: Automatic placement of cell instances and routing of interconnect across metal layers under design rules.
  - **DRC/LVS**:
    - **DRC**: Geometry checks against PDK rules (spacing, width, density).
    - **LVS**: Extracted netlist from layout vs synthesized netlist; must be equivalent.
  - **PEX/STA/Signoff**: Parasitic extraction (R/C), static timing, IR drop, EM, thermal; sign-off with foundry rule decks.
  - Output layout in **GDSII/OASIS** for fabrication.

- **Analog/Mixed-signal flow**
  - **Schematic capture**: Graph of devices (transistors, resistors, caps, etc.) and nets; parameters on devices (W/L, fingers, multiplier).
  - **PCells**: Parameterized layout “cells” instantiated from schematic parameters.
  - **Schematic-driven layout + constraints**: Matching, symmetry, common-centroid, guard rings, routing constraints. Often manual or semi-automatic.
  - **DRC/LVS/PEX**: As above; aggressively used to ensure the layout-derived netlist matches the schematic and to simulate with parasitics.

- **Thermal analysis**
  - Power density from activity and leakage mapped to geometry (cells/macros, metal density).
  - Solved via compact models or FEA; modern tools (e.g., Cadence Celsius) do electro-thermal co-sim (couple temperature to timing/leakage).
  - Results feed back into design (e.g., floorplan changes, power grid, throttling).

- **Key concept (your app’s core):**
  - The schematic (graph: devices + nets) and the layout (geometric instances + routed shapes) are maintained in sync via unique IDs and LVS/constraints. They are not literally the same object, but there is a maintained isomorphism: instance-level correspondences and net equivalence.

### Designing your Bevy application

- **Goal**
  - Right pane: interactive 2D schematic editor (graph).
  - Left pane: 3D layout reconstruction (volumetric stack of devices/metals/vias, plus package/heat paths).
  - **Bidirectional edits**: Schematic ↔ Layout changes propagate through a canonical domain model.

- **Core data model (single source of truth)**
  - `DesignModel` resource containing:
    - `SchematicGraph`: nodes = components (with params), edges = nets; stable `NodeId`, `NetId`.
    - `PhysicalLayout`: instances with transforms, layer shapes, vias; stable `InstanceId`, `ShapeId`, `NetId`.
    - `IsomorphismMap`: `NodeId ↔ InstanceId`, `NetId ↔ RoutedNetId`.
    - `TechStack`: layer stack, thickness, conductivity, design rules (PDK abstraction).
    - `ThermalModel`: material props; power per instance/net segment; solver state.
  - All edits mutate `DesignModel`; views are projections.

- **Eventing + consistency**
  - Change events: `SchematicEdited`, `LayoutEdited`, `MappingUpdated`, `DRCReport`, `LVSReport`, `ThermalUpdated`.
  - Systems consume these to:
    - Validate constraints (matching, symmetry, DRC).
    - Maintain isomorphism (update/add/delete mappings; run LVS-like checks).
    - Rebuild view meshes/graphs incrementally.
  - Undo/redo via an operation log on `DesignModel`.

- **UI/Viewport structure (use your existing crates)**
  - Split pane via `pane_layout` with two `viewport`s:
    - Right: 2D (use `viewport_2d`, `camera_2d`, `widget_2d` + your `geometry` polylines).
    - Left: 3D (use `viewport_3d`, `camera_3d`, `widget_3d`, `meshable`).
  - Input/picking via `picking`/`interaction`; shared selection state to keep panes in sync.
  - Tooling menus via `context_menu`, `styles`, `widget_2d::toolbar`.

- **Rendering representations**
  - 2D schematic:
    - Devices as nodes (param-driven glyphs), nets as polylines with routing hints; labels for pins/nets.
    - Constraint overlays (matching groups, symmetry axes).
  - 3D layout:
    - Per-layer instanced meshes (metals/vias/diffusion) extruded by `TechStack` thickness.
    - Group by `InstanceId` and `NetId` for highlighting, selection, and power attribution.
    - Optional package/heat-sink block for boundary conditions.

- **Bidirectional edit flow**
  - Schematic→Layout:
    - Add device/net → create/parameterize PCell instance and initial placement → auto-route or leave unrouted → maintain `IsomorphismMap`.
    - Param changes (e.g., W/L) → regenerate geometry for that PCell instance.
  - Layout→Schematic:
    - Move/clone/delete an instance → update mapping → schematic node position/mirroring metadata → trigger constraint checks.
    - Manual routing edits → update routed net segments → net equivalence preserved.
  - Equivalence maintenance:
    - Continuous LVS-like check on diffs: layout-extracted net connectivity vs schematic; warn on mismatches.

- **Thermal simulation integration**
  - Build a 3D voxel/fused-mesh lattice from `PhysicalLayout` and package layers with material properties.
  - Power map: sum power per instance and spread to geometry; route dynamic power from activity (optional).
  - Solver options:
    - CPU: sparse diffusion (steady-state and transient).
    - GPU (fast, interactive): WGSL compute shader solving heat equation on a grid; your `compute` crate can host this.
  - Visualization: temperature as color/volume slice/isosurfaces; link hover/selection to temperature at instance/net.

- **Performance**
  - Instanced meshes per layer/material, frustum-culling per pane, chunked rebuilds on edits.
  - Run thermal solve in background tasks; throttle to ~10–30 Hz; incremental updates on small edits.

- **Persistence/interop**
  - Import/export:
    - Schematic: JSON/Spice-like.
    - Layout: internal JSON or GDSII/OASIS importer for viewing; preserve `InstanceId`/`NetId` mapping where possible.
  - Tech/PDK abstraction: start simplified (few layers, generic rules); later plug real PDKs.

- **Minimal ECS types (sketch)**
  - Resources: `DesignModel`, `TechStack`, `ThermalState`, `Selection`.
  - Components: `SchematicNode`, `SchematicNetEdge`, `LayoutInstance`, `LayoutShape`, `ThermalRenderable`.
  - Events: `SchematicEdited`, `LayoutEdited`, `SelectionChanged`, `ThermalUpdated`.

- **Workflow to prototype**
  - Start with a small analog block (e.g., differential pair or inverter chain).
  - Implement schematic creation → PCell layout instantiation → 3D reconstruction.
  - Add thermal lattice and GPU solver; colorize layout by temperature.
  - Add selection sync, undo/redo, and basic LVS/DRC checks.

Status update: I outlined the current EDA flow (digital vs analog, DRC/LVS/PEX) and proposed a Bevy-based architecture using your existing crates (`pane_layout`, `viewport_2d/3d`, `camera_2d/3d`, `picking`, `widget_2d/3d`, `compute`) with a canonical model and bidirectional sync. If you want, I can scaffold the split-pane and core resources next.

- Built a crisp picture of modern Cadence-like flows: schematic/RTL → netlist → layout → DRC/LVS/PEX → thermal.
- Proposed a concrete Bevy architecture: single `DesignModel`, event-driven systems, two synchronized viewports, and thermal GPU solver.
- Mapped bidirectional edits with isomorphism maintenance and lightweight LVS.
- Suggested a practical prototype path (small analog block → thermal coloring → sync + undo/redo).