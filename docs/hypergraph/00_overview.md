Short take
Schematics are best seen as a hypergraph: a net is a hyperedge that connects many pins (ports), which belong to parts (hypernodes with ports).
For computation and ECS, use the incidence bipartite graph (standard netlist form): Pins/Parts on one side, Nets on the other; edges are Pin→Net relations.
For drawing, build a geometric overlay: wires, junctions, labels are layout artifacts of a single Net hyperedge.
Terms mapped
Hypernode: a Part entity with multiple ports (Pins).
Port: a Pin entity (belongs to a Part).
Hyperedge: a Net entity connecting many Pins.
Incidence bipartite graph: Nodes = {Nets} ∪ {Pins or Parts}, Edges = Pin→Net (what you already have with OfPart and ConnectedToNet).
Practical recommendation
Keep the logical model as bipartite (Pins ↔ Nets). It’s simple, queryable, and equivalent to a hypergraph.
Add a rendering layer:
Net geometry: WireSegment entities (polylines) + Junction nodes; all reference the same Net.
Labels/ports/global symbols: small node-like anchors referencing the Net.
Use dual views freely:
Logic/analysis: bipartite (Pins↔Nets).
Layout: geometric graph for each Net (segments + junctions), but still one Net hyperedge.
This way, “what is a Node vs Edge” is unambiguous:
Logic: Nodes = Pins/Parts and Nets; Edges = Pin→Net relations.
Drawing: Nodes = Junctions/Anchors; Edges = WireSegments; they are just a visual decomposition of one Net.