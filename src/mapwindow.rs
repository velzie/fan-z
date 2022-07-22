use std::fmt::Debug;

use crate::app::draw_sprite;
use crate::viewport::Viewport;
use crate::zvm::{self, ZEvent, ZVMState, ZVM};
use crate::FanzApp;
use array2d::Array2D;
use egui::{
    pos2, vec2, Align2, Color32, Id, Key, LayerId, Painter, Pos2, Rect, RichText, Sense, Stroke,
    Vec2, Widget,
};
use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize)]
pub struct MapWindow {
    pub enabled: bool,

    #[serde(skip)]
    pub viewport: Viewport,

    #[serde(skip)]
    pub tool: Tool,
}

pub enum Tool {
    Pencil,
    Eraser,
    Picker,
    Rect,
    Move,
}
impl Default for Tool {
    fn default() -> Self {
        Self::Pencil
    }
}
impl Default for MapWindow {
    fn default() -> Self {
        MapWindow {
            viewport: Viewport::default(),
            enabled: false,
            tool: Tool::Pencil,
        }
    }
}
impl MapWindow {
    pub fn ui<'a>(&mut self, app: &mut FanzApp<'a>, ui: &mut egui::Ui) {
        let tilesize = 32.0 / self.viewport.zoom;
        let map = &mut app.cart.map;
        ui.horizontal(|ui| {
            let mut columns = map.num_columns();
            let mut rows = map.num_rows();

            ui.label("rows: ");
            ui.add(egui::DragValue::new(&mut rows));
            ui.label("columns: ");
            ui.add(egui::DragValue::new(&mut columns));

            while columns < map.row_len() {
                map.popcolumn();
            }
            while columns > map.row_len() {
                map.addcolumn(None);
            }

            while rows < map.column_len() {
                map.poprow();
            }
            while rows > map.column_len() {
                map.addrow(None);
            }
        });
        let (resp, painter, start) = self.viewport.draw(
            ui,
            vec2(ui.available_width(), ui.available_height()),
            vec2(
                map.num_rows() as f32 * tilesize,
                map.num_columns() as f32 * tilesize,
            ),
        );

        for x in 0..map.num_rows() {
            for y in 0..map.num_columns() {
                let tilerect = Rect::from_min_size(
                    start + vec2(x as f32 * tilesize, y as f32 * tilesize),
                    vec2(tilesize, tilesize),
                );

                match map.get(x, y).unwrap() {
                    Some(s) => draw_sprite(&painter, tilerect, &app.cart.sprites[*s]),
                    None => (),
                }
                painter.rect_stroke(tilerect, 0f32, Stroke::new(2f32, Color32::WHITE));

                match &resp.hover_pos() {
                    Some(s) => {
                        for e in &ui.input().events {
                            match e {
                                egui::Event::Scroll(s) => {
                                    self.viewport.zoom -= s.y / 128.0;
                                }
                                _ => (),
                            }
                        }
                        if tilerect.contains(*s) {
                            match self.tool {
                                Tool::Pencil | Tool::Rect => {
                                    painter.rect_filled(tilerect.clone(), 0f32, Color32::BROWN);
                                }
                                Tool::Eraser => {
                                    painter.rect_filled(tilerect.clone(), 0f32, Color32::BLACK);
                                }
                                Tool::Picker => {
                                    resp.clone().on_hover_cursor(egui::CursorIcon::Crosshair);
                                    painter.rect_filled(
                                        tilerect.clone(),
                                        0f32,
                                        Color32::DEBUG_COLOR,
                                    );
                                }
                                Tool::Move => {}
                            }

                            match self.tool {
                                Tool::Pencil => {
                                    if ui.input().key_down(Key::Space) && resp.dragged() {
                                        self.viewport.offset -=
                                            resp.drag_delta() * self.viewport.zoom;
                                    } else if resp.clicked_by(egui::PointerButton::Secondary)
                                        || resp.dragged_by(egui::PointerButton::Secondary)
                                    {
                                        *map.get_mut(x, y).unwrap() = None;
                                    } else if resp.clicked_by(egui::PointerButton::Primary)
                                        || resp.dragged_by(egui::PointerButton::Primary)
                                    {
                                        *map.get_mut(x, y).unwrap() = Some(app.selectedsprite);
                                    }
                                }
                                Tool::Eraser => {
                                    if resp.clicked_by(egui::PointerButton::Primary)
                                        || resp.dragged_by(egui::PointerButton::Primary)
                                    {
                                        *map.get_mut(x, y).unwrap() = None;
                                    }
                                }
                                Tool::Picker => {
                                    if resp.drag_started() {
                                        match map.get(x, y).unwrap() {
                                            Some(s) => {
                                                app.selectedsprite = *s;
                                                self.tool = Tool::Pencil;
                                            }
                                            None => {
                                                self.tool = Tool::Eraser;
                                            }
                                        }
                                    }
                                }
                                Tool::Move => {
                                    if resp.dragged() {
                                        self.viewport.offset -=
                                            resp.drag_delta() * self.viewport.zoom;
                                    }
                                }
                                Tool::Rect => todo!(),
                            }
                        }
                    }
                    None => {}
                }
            }
        }
    }
}

pub trait Resize<T> {
    fn addrow(&mut self, default: T);
    fn addcolumn(&mut self, default: T);

    fn poprow(&mut self) -> Vec<T>;
    fn popcolumn(&mut self) -> Vec<T>;
}

impl<T> Resize<T> for Array2D<T>
where
    T: Clone + Debug,
{
    fn addrow(&mut self, default: T) {
        let mut tvec = self.as_rows();
        tvec.push(vec![default; self.row_len()]);
        *self = Array2D::from_rows(tvec.as_slice());
    }
    fn addcolumn(&mut self, default: T) {
        let mut tvec = self.as_columns();
        tvec.push(vec![default; self.column_len()]);
        *self = Array2D::from_columns(tvec.as_slice());
    }
    fn poprow(&mut self) -> Vec<T> {
        let mut rows = self.as_rows();
        let popped = rows.pop();
        *self = Array2D::from_rows(rows.as_slice());
        popped.unwrap()
    }
    fn popcolumn(&mut self) -> Vec<T> {
        let mut columns = self.as_columns();
        let popped = columns.pop();
        *self = Array2D::from_columns(columns.as_slice());
        popped.unwrap()
    }
}
