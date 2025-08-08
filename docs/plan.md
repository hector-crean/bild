# 2D Schematic View Implementation Plan

## Overview

This plan outlines the implementation of a 2D schematic view inspired by Rayon's successful approach to 2D canvas drawing for architectural diagrams. The system properly distinguishes between **Views** (what we're looking at) and **Tools** (how we interact with the view).

## ✅ Implementation Status

### ✅ Completed - Phase 1: View System Foundation
- [x] **ViewState enum**: Created top-level view system with Model3D and Schematic2D states
- [x] **View switching system**: Implemented view state transitions with proper event handling
- [x] **Schematic2D view**: Basic 2D schematic view with tool selection
- [x] **View-specific toolset**: Schematic2DToolState with Node, Edge, Select, Pan tools
- [x] **Integration**: ViewPlugin integrated into main app architecture

### ✅ Completed - Basic Node System
- [x] **Node entities**: SchematicNode component with ID, type, size, connection points
- [x] **Node tool**: NodeToolPlugin with placement functionality
- [x] **Node placement**: Click-to-place nodes in 2D space
- [x] **Basic rendering**: 2D rectangles with proper materials and transforms
- [x] **Node types**: Basic, Process, Decision, Terminal node types with default connection points

### 🔄 In Progress
- [ ] **Connection points visualization**: Show connection points on nodes
- [ ] **Node selection**: Click to select nodes, visual feedback
- [ ] **Edge tool**: Basic edge drawing between nodes

### 📋 Pending
- [ ] **Edge system**: Complete edge drawing and rendering
- [ ] **View UI integration**: Proper 2D viewport integration
- [ ] **Advanced features**: Copy/paste, undo/redo, export

## Architectural Distinction: Views vs Tools

### Views
- **Purpose**: Define what type of content we're viewing/editing
- **Examples**: 2D Schematic View, 3D Model View, Text Editor View, etc.
- **Characteristics**: Each view has its own coordinate system, rendering pipeline, and applicable toolset
- **State**: Views can be switched between and have their own persistent state

### Tools  
- **Purpose**: Define how we interact with the current view
- **Examples**: Node Tool, Edge Tool, Select Tool, Pan Tool
- **Characteristics**: Tools are context-specific to the current view
- **State**: Tools have their own state machines for interaction workflows

## Architecture Analysis

### Current System (3D-focused)
- **Single view context**: Currently assumes 3D world space
- **Global tools**: Tools like Transform, Comment, Markup, Block work in 3D space
- **Mixed concerns**: Tools contain both interaction logic and view-specific rendering

### ✅ Implemented View-Tool Architecture

```rust
// Top-level view system
#[derive(States)]
pub enum ViewState {
    Model3D,        // Existing 3D model view
    Schematic2D,    // New 2D schematic view
    // Future views can be added here
}

// Each view has its own tool context
#[derive(SubStates)]
#[source(ViewState = ViewState::Schematic2D)]
pub enum Schematic2DToolState {
    Node,     // Node placement/editing ✅ IMPLEMENTED
    Edge,     // Edge drawing (to be implemented)
    Select,   // Selection and manipulation (to be implemented)
    Pan,      // Pan/zoom navigation (to be implemented)
}
```

### ✅ Implemented Data Model

```rust
/// Core schematic node component - view-independent data
#[derive(Component)]
pub struct SchematicNode {
    pub id: NodeId,
    pub node_type: NodeType,
    pub label: String,
    pub size: Vec2,
    pub connection_points: Vec<ConnectionPoint>,
}

/// Connection point on a node
#[derive(Debug, Clone)]
pub struct ConnectionPoint {
    pub id: ConnectionPointId,
    pub local_position: Vec2, // Relative to node center
    pub point_type: ConnectionType,
    pub label: Option<String>,
}
```

## Usage Instructions

### Switching to 2D Schematic View

The view system is now integrated into the main application. To switch to the 2D Schematic view:

1. **Programmatic switching**: Change the ViewState resource
```rust
fn switch_to_schematic(mut next_view: ResMut<NextState<ViewState>>) {
    next_view.set(ViewState::Schematic2D);
}
```

2. **Tool selection**: When entering Schematic2D view, a radial menu appears with available tools
3. **Node placement**: Select the Node tool and click to place nodes in 2D space

### Current Functionality

- **View switching**: Seamless transition between 3D Model and 2D Schematic views
- **Node placement**: Click anywhere in 2D space to place basic nodes
- **Tool states**: Proper state management for different tools within each view
- **Visual feedback**: Nodes render as blue rectangles with proper 2D transforms

## Implementation Strategy

#### Phase 1: View System Foundation ✅ COMPLETE
- [x] Create ViewState enum and view switching system
- [x] Implement SchematicView with 2D camera and viewport
- [x] Set up 2D coordinate system and infinite grid
- [x] Create view-specific rendering layers

#### Phase 2: Basic Schematic Tools ✅ PARTIALLY COMPLETE
- [x] Implement Node Tool for 2D schematic view
- [x] Basic node placement and rendering
- [ ] Node selection and highlighting
- [x] Tool state management within schematic view

#### Phase 3: Connection System
- [ ] Implement Edge Tool for connecting nodes
- [ ] Connection point detection and snapping
- [ ] Edge rendering and path calculation
- [ ] Edge selection and manipulation

#### Phase 4: Advanced Features
- [ ] Copy/paste within schematic view
- [ ] Undo/redo system
- [ ] Export schematic as image/PDF
- [ ] Import/export schematic data formats

#### Phase 5: Multi-View Integration
- [ ] Smooth transitions between views
- [ ] Shared data between 3D and 2D representations
- [ ] Cross-view operations (e.g., generate 3D from 2D schematic)

### Benefits of This Architecture

#### Separation of Concerns ✅ ACHIEVED
- Views handle rendering and coordinate systems
- Tools handle user interaction patterns
- Data models remain view-independent

#### Extensibility ✅ DEMONSTRATED
- Easy to add new views (Timeline, Code Editor, etc.)
- Each view can have specialized tools
- Tools can be shared between compatible views

#### User Experience ✅ IMPROVED
- Clear mental model: "I'm in the schematic view using the node tool"
- Context-appropriate toolbars and shortcuts
- Consistent interaction patterns within each view

#### Development ✅ VALIDATED
- Parallel development of different views
- Easier testing of view-specific functionality
- Clear ownership of code by view/tool responsibility

## Migration Strategy

### Existing Code ✅ PRESERVED
- Keep existing 3D tools as Model3DToolState
- Gradually migrate to view-aware architecture
- Maintain backward compatibility during transition

### New 2D Code ✅ IMPLEMENTED
- Start with clean view-tool separation
- Use as proof of concept for architecture
- Apply lessons learned to refactor 3D tools

## Next Steps

1. **Implement connection points visualization** - Show connection points on nodes
2. **Add node selection** - Click to select nodes with visual feedback  
3. **Create Edge Tool** - Basic edge drawing between connection points
4. **Integrate with 2D viewport** - Proper camera and grid system
5. **Test and refine** - Based on usage and feedback

This architecture provides a solid foundation for multiple views while maintaining clear separation between what we're viewing and how we interact with it.
