use crate::app::toolbtn_ui;
use crate::zvm::{self, ZEvent, ZVMState, STATE_PTR, ZVM};
use crate::FanzApp;
// use crate::app::
use array2d::Array2D;
use egui::{
    pos2, vec2, Align2, Color32, Id, Key, LayerId, Painter, Rect, RichText, Sense, Stroke, Vec2,
    Widget,
};
use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize)]
pub struct SpritesWindow {
    pub enabled: bool,

    selectedcolor: Color32,
    #[serde(skip)]
    selectedtool: Tool,
}
#[derive(PartialEq, Eq, Debug)]
pub enum Tool {
    Pencil,
    Eraser,
    Picker,
    Rect,
    Line,
}
impl Default for Tool {
    fn default() -> Self {
        Self::Pencil
    }
}
impl Default for SpritesWindow {
    fn default() -> Self {
        SpritesWindow {
            enabled: false,
            selectedcolor: Color32::TRANSPARENT,
            selectedtool: Tool::Pencil,
        }
    }
}
impl SpritesWindow {
    pub fn ui<'a>(&mut self, app: &mut FanzApp<'a>, ui: &mut egui::Ui) {
        match app.cart.sprites.get_mut(app.selectedsprite) {
            Some(sprite) => {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        for variant in [
                            Tool::Pencil,
                            Tool::Eraser,
                            Tool::Picker,
                            Tool::Rect,
                            Tool::Line,
                        ] {
                            if sized_toolbtn_ui(
                                ui,
                                vec2(50.0, 50.0),
                                &format!("{:?}", variant),
                                self.selectedtool == variant,
                            )
                            .clicked()
                            {
                                self.selectedtool = variant;
                            }
                        }
                    });
                    let (resp, painter) =
                        ui.allocate_painter(vec2(8.0 * 32.0, 8.0 * 32.0), Sense::click_and_drag());

                    let start = painter.clip_rect().min;
                    for x in 0..8 {
                        for y in 0..8 {
                            let spritefill = sprite.data.get_mut(x, y).unwrap();
                            let squarerect = Rect::from_min_size(
                                start + vec2(x as f32 * 32.0, y as f32 * 32.0),
                                vec2(32.0, 32.0),
                            );
                            painter.rect(
                                squarerect,
                                0f32,
                                spritefill.clone(),
                                Stroke::new(2.0, Color32::WHITE),
                            );

                            match &resp.hover_pos() {
                                Some(s) => {
                                    if squarerect.contains(*s) {
                                        match self.selectedtool {
                                            Tool::Pencil | Tool::Rect | Tool::Line => {
                                                painter.rect_filled(
                                                    squarerect.clone(),
                                                    0f32,
                                                    self.selectedcolor,
                                                );
                                            }
                                            Tool::Eraser => {
                                                painter.rect_filled(
                                                    squarerect.clone(),
                                                    0f32,
                                                    Color32::BLACK,
                                                );
                                            }
                                            Tool::Picker => {
                                                resp.clone()
                                                    .on_hover_cursor(egui::CursorIcon::Crosshair);
                                                painter.rect_filled(
                                                    squarerect.clone(),
                                                    0f32,
                                                    Color32::DEBUG_COLOR,
                                                );
                                            }
                                        }

                                        match self.selectedtool {
                                            Tool::Pencil => {
                                                if resp.clicked_by(egui::PointerButton::Secondary)
                                                    || resp
                                                        .dragged_by(egui::PointerButton::Secondary)
                                                {
                                                    *spritefill = Color32::TRANSPARENT;
                                                } else if resp
                                                    .clicked_by(egui::PointerButton::Primary)
                                                    || resp.dragged_by(egui::PointerButton::Primary)
                                                {
                                                    *spritefill = self.selectedcolor;
                                                }
                                            }
                                            Tool::Eraser => {
                                                if resp.clicked_by(egui::PointerButton::Primary)
                                                    || resp.dragged_by(egui::PointerButton::Primary)
                                                {
                                                    *spritefill = Color32::TRANSPARENT;
                                                }
                                            }
                                            Tool::Picker => {
                                                if resp.drag_started() {
                                                    self.selectedcolor = *spritefill;
                                                    self.selectedtool = Tool::Pencil;
                                                }
                                            }
                                            Tool::Rect | Tool::Line => todo!(),
                                        }
                                    }
                                }
                                None => {}
                            }
                        }
                    }
                });
            }
            None => (),
        }
        ui.horizontal(|ui| {
            ui.label("color: ");
            ui.color_edit_button_srgba(&mut self.selectedcolor);
            if ui.small_button("+").clicked() {
                app.cart.sprites.push(Sprite::new())
            }
        });
    }
}

#[derive(Serialize, Deserialize)]
pub struct Sprite {
    pub data: Array2D<Color32>,
}
impl Sprite {
    pub fn new() -> Sprite {
        Sprite {
            data: Array2D::filled_with(Color32::TRANSPARENT, 8, 8),
        }
    }
}

pub fn sized_toolbtn_ui(ui: &mut egui::Ui, size: Vec2, text: &str, on: bool) -> egui::Response {
    let (id, rect) = ui.allocate_space(size);
    let response = ui.interact(rect, id, egui::Sense::click());

    if ui.is_rect_visible(rect) {
        let visuals = ui.style().interact(&response);
        let mut color = visuals.bg_fill;
        if on {
            color = Color32::BLACK;
        }
        ui.painter().rect(
            rect,
            5.0,
            color,
            Stroke::new(if response.hovered() { 1.0 } else { 0.0 }, Color32::WHITE),
        );
        ui.painter().text(
            rect.center(),
            Align2::CENTER_CENTER,
            text,
            egui::FontId {
                size: 15.0,
                family: egui::FontFamily::Proportional,
            },
            Color32::WHITE,
        );
    }

    response
}
