use crate::app::Cart;
use crate::spriteswindow::Sprite;
// use c
use crate::zvm::{self, ZEvent, ZVMState, STATE_PTR, ZVM};
use crate::FanzApp;
use array2d::Array2D;
use egui::{
    pos2, Align2, Color32, Id, Key, LayerId, Painter, Pos2, Rect, RichText, Sense, Stroke, Vec2,
    Widget,
};
use zsp_core::exceptions::Exception;
pub struct GameWindow<'a> {
    pub enabled: bool,
    pub game: Option<Game<'a>>, // pub vm: Option<ZVM<'a>>,
}

pub struct Game<'a> {
    pub objects: Vec<GameObject<'a>>,
}
pub struct GameObject<'a> {
    pub vm: ZVM<'a>,
}

impl<'a> Default for GameWindow<'a> {
    fn default() -> Self {
        GameWindow {
            enabled: false,
            game: None,
        }
    }
}
impl<'a> GameWindow<'a> {
    pub fn ui(&mut self, app: &mut FanzApp<'a>, ui: &mut egui::Ui) {
        let scalefactor =
            f32::floor(ui.available_width() / 160.0).min(f32::floor(ui.available_width() / 120.0));

        let (_resp, painter) = ui.allocate_painter(
            Vec2::new(160.0 * scalefactor, 120.0 * scalefactor),
            Sense::click_and_drag(),
        );

        match &mut self.game {
            Some(game) => {
                let mut state = unsafe { &mut *STATE_PTR };

                for obj in &mut game.objects {
                    match obj.vm.draw() {
                        Err(e) => {
                            let o = obj.vm.fmt(e);
                            app.output
                                .push(RichText::new(o.to_string()).color(Color32::RED));
                        }
                        _ => (),
                    }
                }

                let start = painter.clip_rect().min;
                painter.rect_filled(painter.clip_rect(), 0.0, Color32::BLACK);
                for i in &state.buffer {
                    match i {
                        ZEvent::Put(s) => {
                            if app.output.len() > 100 {
                                app.output.clear();
                            }
                            app.output.push(RichText::new(s))
                        }
                        ZEvent::GSet { x, y, color } => {
                            drawpixel(&painter, scalefactor, start, *x, *y, *color)
                        }
                        ZEvent::Rect { x, y, h, w, color } => painter.rect_filled(
                            Rect::from_min_size(
                                pos2(start.x + x * scalefactor, start.y + y * scalefactor),
                                Vec2 {
                                    x: *w * scalefactor,
                                    y: *h * scalefactor,
                                },
                            ),
                            0.0,
                            *color,
                        ),
                        ZEvent::Sprite {
                            x: sx,
                            y: sy,
                            sprite,
                        } => {
                            let spritedata = &app.cart.sprites[*sprite].data;
                            for x in 0..8 {
                                for y in 0..8 {
                                    drawpixel(
                                        &painter,
                                        scalefactor,
                                        start,
                                        *sx + x as f32,
                                        *sy + y as f32,
                                        spritedata.get(x, y).unwrap().clone(),
                                    )
                                }
                            }
                        }
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
    pub fn startgame(cart: &mut Cart) -> Result<Game<'a>, String> {
        let mut objects = vec![];

        for editorobj in &cart.objects {
            let vm = match ZVM::start(editorobj.script.clone()) {
                Ok(vm) => vm,
                Err(e) => {
                    let o = zvm::errfmt(e, &editorobj.script);
                    return Err(o);
                }
            };
            objects.push(GameObject { vm: vm })
        }
        Ok(Game { objects })
    }
}

fn drawpixel(painter: &Painter, scalefactor: f32, start: Pos2, x: f32, y: f32, color: Color32) {
    painter.rect_filled(
        Rect::from_min_size(
            egui::pos2(start.x + x * scalefactor, start.y + y * scalefactor),
            Vec2::new(scalefactor, scalefactor),
        ),
        0.0,
        color,
    )
}
