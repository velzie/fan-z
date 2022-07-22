use egui::{pos2, vec2, Painter, Pos2, Response, Sense, Ui, Vec2};

pub struct Viewport {
    pub offset: Pos2,
    pub zoom: f32,
}
impl Default for Viewport {
    fn default() -> Self {
        // why do i need this. the containing struct has a default too.
        Viewport {
            offset: pos2(0.0, 0.0),
            zoom: 1.0,
        }
    }
}
impl Viewport {
    pub fn draw(
        &mut self,
        ui: &mut Ui,
        viewportsize: Vec2,
        fullsize: Vec2,
    ) -> (Response, Painter, Pos2) {
        // annoying clamping section
        self.zoom = self.zoom.max(0.5);

        let offsetgridsize = fullsize * self.zoom;
        let scaledviewportsize = viewportsize * self.zoom;

        let maxoffset = (offsetgridsize - scaledviewportsize)
            .max(vec2(0.1, 0.1))
            .to_pos2();
        // clamp panicks if the min is greater than the max. no real reason why but ok

        // also why does doing anything to a pos2 make it a vec2
        // they both do the same thing, even if they have different semantic meanings. this is just annoying
        self.offset = self.offset.clamp(pos2(0.0, 0.0), maxoffset);
        let (resp, painter) = ui.allocate_painter(viewportsize, Sense::click_and_drag());
        let start = painter.clip_rect().min - (self.offset.to_vec2() / self.zoom);

        (resp, painter, start)
    }
}
