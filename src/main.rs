#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui;
use std::rc::{Rc, Weak};
use tree_sitter::{Node, Parser, Tree, TreeCursor};
use tree_sitter_rust;

fn main() -> Result<(), eframe::Error> {
    env_logger::init();

    let options = eframe::NativeOptions::default();

    let code = r#"
        fn test() -> String {
            return "Hello, world!".to_string();
        }
    "#
    .to_string();
    let passed_code = code.as_bytes().to_owned();
    let mut parser = Parser::new();
    let language = tree_sitter_rust::language();
    parser
        .set_language(&language)
        .expect("Error setting language");
    let tree = parser.parse(code, None).expect("Error parsing code");
    let node_id = tree.root_node().id();

    let app = MyApp {
        tree,
        node_id,
        code: passed_code,
    };
    eframe::run_native("My egui App", options, Box::new(|_cc| Box::new(app)))
}

struct MyApp {
    tree: Tree,
    node_id: usize,
    code: Vec<u8>,
}

impl MyApp {
    fn display_tree_with_cursor<'tree>(
        &'tree self,
        ui: &mut egui::Ui,
        cursor: &mut TreeCursor<'tree>,
    ) -> Option<Node<'tree>> {
        let mut selected_node: Option<Node> = None;
        loop {
            {
                let node = cursor.node();
                let field_name = cursor.field_name();
                ui.horizontal(|ui| {
                    ui.add_space(cursor.depth() as f32 * 20.0);
                    let string = self.code[node.byte_range()];
                    let label_response =
                        ui.label(format!("({}) {:?}: {:?}", node.kind(), field_name, string));
                    if (node.id() == self.node_id) | label_response.hovered() {
                        selected_node = Some(node);
                        label_response.highlight();
                    };
                });
                if cursor.goto_first_child() {
                    if let Some(returned_node) = self.display_tree_with_cursor(ui, cursor) {
                        selected_node = Some(returned_node)
                    }
                }
            }
            if !cursor.goto_next_sibling() {
                break;
            }
        }
        cursor.goto_parent(); // Go to the parent to maintain cursor state for next iteration

        selected_node
    }

    // fn display_tree_with_node<'tree>(
    //     &'tree self,
    //     ui: &mut egui::Ui,
    //     node: Node<'tree>,
    // ) -> Option<Node<'tree>> {
    //     let mut selected_node: Option<Node> = None;

    //     _ = match node.kind() {
    //         "source_file" => ui.label(format!("{} ({})", node.kind())),
    //         _ => ui.label("not source file"),
    //     };

    //     let label = ui.label(format!("{}", node.kind()));

    //     if node.id() == self.node_id {
    //         selected_node = Some(node);
    //     };
    //     selected_node
    // }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let selected_node: Node = egui::CentralPanel::default()
            .show(ctx, |ui| {
                // two attempts:
                // 1) walk entire tree using TreeCursor
                let mut walking_cursor = self.tree.walk();
                self.display_tree_with_cursor(ui, &mut walking_cursor)
                // 2) manually descend from each Node
                // let node = self.tree.root_node();
                // self.display_tree_with_node(ui, node)
            })
            .inner
            .unwrap_or_else(|| self.tree.root_node());

        let mut next_cursor_id: usize = selected_node.id();

        if ctx.input(|i| i.key_pressed(egui::Key::ArrowUp) | i.key_pressed(egui::Key::K)) {
            if let Some(sibling) = selected_node.prev_named_sibling() {
                next_cursor_id = sibling.id();
            } else if let Some(parent) = selected_node.parent() {
                next_cursor_id = parent.id();
            }
        }
        if ctx.input(|i| i.key_pressed(egui::Key::ArrowDown) | i.key_pressed(egui::Key::J)) {
            if let Some(sibling) = selected_node.next_named_sibling() {
                next_cursor_id = sibling.id();
            } else if let Some(child) = selected_node.named_child(0) {
                next_cursor_id = child.id();
            } else if let Some(parent) = selected_node.parent() {
                if let Some(parent_sibling) = parent.next_named_sibling() {
                    next_cursor_id = parent_sibling.id();
                }
            }
        }
        if ctx.input(|i| i.key_pressed(egui::Key::ArrowRight) | i.key_pressed(egui::Key::L)) {
            if let Some(child) = selected_node.named_child(0) {
                next_cursor_id = child.id();
            } else if let Some(sibling) = selected_node.next_named_sibling() {
                next_cursor_id = sibling.id();
            } else if let Some(parent) = selected_node.parent() {
                if let Some(parent_sibling) = parent.next_named_sibling() {
                    next_cursor_id = parent_sibling.id();
                }
            }
        }
        if ctx.input(|i| i.key_pressed(egui::Key::ArrowLeft) | i.key_pressed(egui::Key::H)) {
            if let Some(parent) = selected_node.parent() {
                next_cursor_id = parent.id();
            } else if let Some(sibling) = selected_node.prev_named_sibling() {
                next_cursor_id = sibling.id();
            }
        }

        self.node_id = next_cursor_id;
    }
}
