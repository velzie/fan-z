// use c
use crate::zvm::{self, ZEvent, ZVMState, STATE_PTR, ZVM};
use crate::FanzApp;
use egui::{
    pos2, Align2, Color32, Id, Key, LayerId, Painter, Rect, RichText, Sense, Stroke, Vec2, Widget,
};
pub struct GameWindow<'a> {
    pub enabled: bool,
    pub vm: Option<ZVM<'a>>,
}
impl<'a> Default for GameWindow<'a> {
    fn default() -> Self {
        GameWindow {
            enabled: false,
            vm: None,
        }
    }
}
impl<'a> GameWindow<'a> {
    pub fn ui(&mut self, app: &mut FanzApp<'a>, ui: &mut egui::Ui) {
        let (_resp, painter) =
            ui.allocate_painter(Vec2::new(320.0, 240.0), Sense::click_and_drag());

        match &mut self.vm {
            Some(vm) => {
                let mut state = unsafe { &mut *STATE_PTR };
                match vm.draw() {
                    Err(e) => {
                        let o = zvm::errfmt(e, &app.code);
                        app.output
                            .push(RichText::new(o.to_string()).color(Color32::RED));
                    }
                    _ => (),
                }

                let startx = painter.clip_rect().min.x;
                let starty = painter.clip_rect().min.y;
                painter.rect_filled(painter.clip_rect(), 0.0, Color32::BLACK);
                for i in &state.buffer {
                    match i {
                        ZEvent::Put(s) => app.output.push(RichText::new(s)),
                        ZEvent::GSet { x, y, color } => painter.rect_filled(
                            Rect::from_min_size(
                                egui::pos2(startx + x, starty + y),
                                Vec2::new(2.0, 2.0),
                            ),
                            0.0,
                            *color,
                        ),
                        ZEvent::Rect { x, y, h, w, color } => painter.rect_filled(
                            Rect::from_min_size(
                                pos2(startx + x, starty + y),
                                Vec2 { x: *w, y: *h },
                            ),
                            0.0,
                            *color,
                        ),
                        _ => panic!(),
                    }
                }
                state.buffer = vec![];

                state.keys.clear();
                for i in ui.input().keys_down.iter() {
                    state.keys.push(format!("{:?}", i));
                }

                ui.ctx().request_repaint();
            }
            None => (),
        }
    }
}
