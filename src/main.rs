use eframe::egui;
use egui::*;
use std::collections::BinaryHeap;

#[derive(Default)]
struct Node {
    pos: Pos2,
    edges: Vec<Edge>,
    label: String, // Optional label for the node
}

#[derive(Clone)]
struct Edge {
    target: usize, // Index of the target node
    weight: f32,   // Weight of the edge
}

#[derive(Default)]
struct GraphEditor {
    nodes: Vec<Node>,
    selected_node: Option<usize>,
    edge_weight_input: String,           // Temporary input for edge weight
    node_label_input: String,            // Temporary input for node label
    start_node_input: String,            // Input for start node index
    end_node_input: String,              // Input for end node index
    solution: Option<(f32, Vec<usize>)>, // Stores the solution (distance, path)
}

// Custom struct to use in the BinaryHeap for Dijkstra's algorithm
#[derive(PartialEq)]
struct MinHeapItem {
    cost: f32,
    node: usize,
}

// Implement Eq for MinHeapItem
impl Eq for MinHeapItem {}

// Implement PartialOrd for MinHeapItem
impl PartialOrd for MinHeapItem {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        // Reverse the comparison to create a min-heap
        other.cost.partial_cmp(&self.cost)
    }
}

// Implement Ord for MinHeapItem
impl Ord for MinHeapItem {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Reverse the comparison to create a min-heap
        self.partial_cmp(other).unwrap()
    }
}

impl GraphEditor {
    fn ui(&mut self, ui: &mut Ui) {
        // Node label input
        ui.horizontal(|ui| {
            ui.label("Node label:");
            ui.text_edit_singleline(&mut self.node_label_input);
        });

        // Edge weight input
        ui.horizontal(|ui| {
            ui.label("Edge weight:");
            ui.text_edit_singleline(&mut self.edge_weight_input);
        });

        // Start node input
        ui.horizontal(|ui| {
            ui.label("Start node:");
            ui.text_edit_singleline(&mut self.start_node_input);
        });

        // End node input
        ui.horizontal(|ui| {
            ui.label("End node:");
            ui.text_edit_singleline(&mut self.end_node_input);
        });

        // Solve button (only enabled if start and end nodes are valid)
        let start_node = self.start_node_input.parse::<usize>().ok();
        let end_node = self.end_node_input.parse::<usize>().ok();
        let valid_inputs = start_node.is_some() && end_node.is_some();
        if ui
            .add_enabled(valid_inputs, egui::Button::new("Solve"))
            .clicked()
        {
            if let (Some(start), Some(end)) = (start_node, end_node) {
                self.solution = self.dijkstra(start, end);
            }
        }

        // Display solution
        if let Some((distance, path)) = &self.solution {
            ui.label(format!("Distance: {:.1}", distance));
            ui.label(format!("Path: {:?}", path));
        }

        // Edges
        let mut new_edge: Option<(usize, usize, f32)> = None;
        let mut edge_updates: Vec<(usize, usize, f32)> = Vec::new(); // Store edge updates here

        // Draw edges
        for (i, node) in self.nodes.iter().enumerate() {
            for edge in &node.edges {
                let start = node.pos;
                let end = self.nodes[edge.target].pos;
                ui.painter()
                    .line_segment([start, end], (2.0, Color32::WHITE));

                // Draw the weight label at the midpoint of the edge
                let midpoint = (start + end.to_vec2()) / 2.0; // Fixed: Use .to_vec2() for Pos2
                let weight_label = format!("{:.1}", edge.weight);
                let response = ui.put(
                    Rect::from_center_size(midpoint, Vec2::new(30.0, 20.0)),
                    egui::Label::new(weight_label),
                );

                // Check if the weight label is clicked
                if response.hovered() && ui.input(|i| i.pointer.primary_clicked()) {
                    if let Ok(weight) = self.edge_weight_input.parse::<f32>() {
                        // Store the update instead of applying it immediately
                        edge_updates.push((i, edge.target, weight));
                        self.edge_weight_input = "1".to_string(); // Reset to default value
                    }
                }
            }
        }

        // Apply edge updates after the iteration
        for (from, to, weight) in edge_updates {
            if let Some(edge) = self.nodes[from].edges.iter_mut().find(|e| e.target == to) {
                edge.weight = weight;
            }
        }

        // Draw nodes and handle interactions
        let next_node_index = self.nodes.len();
        for (i, node) in self.nodes.iter_mut().enumerate() {
            // Define the node's interactive area
            let node_size = Vec2::new(30.0, 30.0);
            let node_rect = Rect::from_center_size(node.pos, node_size);

            // Check for interactions (click and drag)
            let response = ui.interact(node_rect, Id::new(i), Sense::click_and_drag());

            // Update node position if dragging
            if response.dragged() {
                node.pos += response.drag_delta();
            }

            // Select node on click
            if response.clicked() {
                self.selected_node = Some(i);
            }

            // Determine node color based on start/end inputs and solution path
            let node_color = if self.start_node_input.parse::<usize>().ok() == Some(i) {
                Color32::GREEN // Start node is green
            } else if self.end_node_input.parse::<usize>().ok() == Some(i) {
                Color32::RED // End node is red
            } else if let Some((_, path)) = &self.solution {
                if path.contains(&i) {
                    Color32::LIGHT_BLUE // Nodes in the solution path are light blue
                } else {
                    Color32::BLUE // Default node color
                }
            } else {
                Color32::BLUE // Default node color
            };

            // Draw the node
            ui.painter().circle_filled(node.pos, 15.0, node_color);

            // Highlight the selected node
            if self.selected_node == Some(i) {
                ui.painter()
                    .circle_stroke(node.pos, 15.0, (2.0, Color32::YELLOW));
            }

            // Draw the node label with index in parentheses
            let label = if node.label.is_empty() {
                format!("{} ({})", i, i) // Default to node index if label is empty
            } else {
                format!("{} ({})", node.label, i) // Append index in parentheses
            };
            let label_response = ui.put(
                Rect::from_center_size(node.pos + Vec2::new(20.0, 20.0), Vec2::new(50.0, 20.0)),
                egui::Label::new(label),
            );

            // Check if the node label is clicked
            if label_response.hovered() && ui.input(|i| i.pointer.primary_clicked()) {
                node.label = self.node_label_input.clone();
                self.node_label_input = format!("{}", next_node_index); // Reset to next node index
            }
        }

        // Handle edge creation with the "E" key
        if let Some(selected_node) = self.selected_node {
            if ui.input(|i| i.key_pressed(Key::E)) {
                if let Some(hovered_node) = self.nodes.iter().position(|node| {
                    ui.ctx()
                        .pointer_hover_pos()
                        .map_or(false, |pos| (node.pos - pos).length() < 15.0)
                }) {
                    // Parse the edge weight from the input (default to 1 if invalid)
                    let weight = self.edge_weight_input.parse::<f32>().unwrap_or(1.0);
                    new_edge = Some((selected_node, hovered_node, weight));
                    self.edge_weight_input = "1".to_string(); // Reset to default value
                }
            }
        }

        // Add the new edge
        if let Some((from, to, weight)) = new_edge {
            self.nodes[from].edges.push(Edge { target: to, weight });
        }

        // Add new node with the "N" key over the pointer position
        if ui.input(|i| i.key_pressed(Key::N)) {
            if let Some(pos) = ui.ctx().pointer_hover_pos() {
                let label = if self.node_label_input.is_empty() {
                    format!("{}", self.nodes.len()) // Default to next node index
                } else {
                    self.node_label_input.clone()
                };
                self.nodes.push(Node {
                    pos,
                    edges: Vec::new(),
                    label,
                });
                self.node_label_input = format!("{}", self.nodes.len()); // Reset to next node index
            }
        }
    }

    // Dijkstra's algorithm to find the shortest path
    fn dijkstra(&self, start: usize, end: usize) -> Option<(f32, Vec<usize>)> {
        let mut distances = vec![f32::INFINITY; self.nodes.len()];
        let mut previous = vec![None; self.nodes.len()];
        let mut heap = BinaryHeap::new();

        distances[start] = 0.0;
        heap.push(MinHeapItem {
            cost: 0.0,
            node: start,
        });

        while let Some(MinHeapItem { cost, node: u }) = heap.pop() {
            if u == end {
                // Reconstruct the path
                let mut path = Vec::new();
                let mut current = end;
                while let Some(prev) = previous[current] {
                    path.push(current);
                    current = prev;
                }
                path.push(start);
                path.reverse();
                return Some((cost, path));
            }

            if cost > distances[u] {
                continue;
            }

            for edge in &self.nodes[u].edges {
                let next = edge.target;
                let next_cost = cost + edge.weight;
                if next_cost < distances[next] {
                    distances[next] = next_cost;
                    previous[next] = Some(u);
                    heap.push(MinHeapItem {
                        cost: next_cost,
                        node: next,
                    });
                }
            }
        }

        None // No path found
    }
}

impl eframe::App for GraphEditor {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.ui(ui);
        });
    }
}

fn main() {
    let options = eframe::NativeOptions::default();
    let _ = eframe::run_native(
        "Graph Editor",
        options,
        Box::new(|_cc| {
            let graph_editor = GraphEditor {
                node_label_input: "0".to_string(),
                edge_weight_input: "1".to_string(),
                ..Default::default()
            };

            Ok(Box::new(graph_editor))
        }),
    );
}
