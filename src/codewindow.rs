use crate::zvm::{self, ZEvent, ZVMState, STATE_PTR, ZVM};
use crate::FanzApp;
use egui::{
    pos2, Align2, Color32, Id, Key, LayerId, Painter, Rect, RichText, Sense, Stroke, Vec2, Widget,
};
use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize)]
pub struct CodeWindow {
    pub enabled: bool,
}
impl Default for CodeWindow {
    fn default() -> Self {
        CodeWindow { enabled: false }
    }
}
impl CodeWindow {
    pub fn ui<'a>(&mut self, app: &mut FanzApp<'a>, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.add(
                egui::TextEdit::multiline(&mut app.code)
                    .font(egui::TextStyle::Monospace) // for cursor height
                    .code_editor()
                    .desired_rows(40)
                    .lock_focus(true)
                    .desired_width(f32::INFINITY),
            );
        });
    }
}
