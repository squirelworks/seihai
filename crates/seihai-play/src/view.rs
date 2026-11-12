//! The game in an egui window: the flat frame, the status line, the
//! position and the other players' names. The arrows walk.

use crate::game::{name_tags, Game};
use crate::render::{self, TILE_PX};
use std::time::{Duration, Instant};

/// A [`Game`] on a canvas of `canvas` pixels.
pub struct View {
    pub game: Game,
    canvas: (f32, f32),
    texture: Option<egui::TextureHandle>,
}

impl View {
    pub fn new(game: Game, canvas: (f32, f32)) -> View {
        View {
            game,
            canvas,
            texture: None,
        }
    }

    /// The view in tiles.
    pub fn tiles(&self) -> (u32, u32) {
        view_tiles(self.canvas)
    }

    /// One frame: run a turn of the loop for the arrows held and draw it.
    pub fn show(&mut self, ctx: &egui::Context) {
        let held = |k| ctx.input(|i| i.key_down(k));
        let dx = held(egui::Key::ArrowRight) as i8 - held(egui::Key::ArrowLeft) as i8;
        let dy = held(egui::Key::ArrowDown) as i8 - held(egui::Key::ArrowUp) as i8;
        let changed = self.game.tick(dx, dy, Instant::now());
        if changed || self.texture.is_none() {
            let f = self.game.draw(self.tiles());
            let image = egui::ColorImage::from_rgb(
                [f.width as usize, f.height as usize],
                &render::rgb_of(&f),
            );
            match &mut self.texture {
                Some(t) => t.set(image, egui::TextureOptions::NEAREST),
                None => {
                    self.texture =
                        Some(ctx.load_texture("level", image, egui::TextureOptions::NEAREST))
                }
            }
        }
        let (w, h) = self.canvas;
        let tiles = self.tiles();
        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(egui::Color32::BLACK))
            .show(ctx, |ui| {
                let origin = ui.max_rect().min;
                if let Some(t) = &self.texture {
                    let rect = egui::Rect::from_min_size(origin, egui::vec2(w, h));
                    ui.painter().image(
                        t.id(),
                        rect,
                        egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                        egui::Color32::WHITE,
                    );
                }
                let bar = egui::Rect::from_min_size(origin, egui::vec2(w, 18.0));
                ui.painter()
                    .rect_filled(bar, 0.0, egui::Color32::from_black_alpha(160));
                ui.put(
                    bar,
                    egui::Label::new(
                        egui::RichText::new(&self.game.status)
                            .color(egui::Color32::WHITE)
                            .size(12.0),
                    ),
                );
                // Other players' names under their markers, as the clients
                // write a nickname under the player.
                let (cam_x, cam_y) = render::camera_origin(&self.game.session, tiles.0, tiles.1);
                for (nick, x, y) in name_tags(self.game.others(), cam_x, cam_y) {
                    ui.painter().text(
                        origin + egui::vec2(x, y),
                        egui::Align2::CENTER_TOP,
                        nick,
                        egui::FontId::proportional(11.0),
                        egui::Color32::WHITE,
                    );
                }
                if let Some(pos) = self.game.position() {
                    let at = egui::Rect::from_min_size(
                        origin + egui::vec2(0.0, 18.0),
                        egui::vec2(w - 6.0, 16.0),
                    );
                    ui.painter().text(
                        at.right_top(),
                        egui::Align2::RIGHT_TOP,
                        pos,
                        egui::FontId::monospace(12.0),
                        egui::Color32::WHITE,
                    );
                }
            });
        // Walking needs frames while a key is held; otherwise poll the
        // network now and then.
        ctx.request_repaint_after(Duration::from_millis(if dx != 0 || dy != 0 {
            16
        } else {
            100
        }));
    }
}

/// A canvas of `canvas` pixels in whole tiles.
pub fn view_tiles(canvas: (f32, f32)) -> (u32, u32) {
    (canvas.0 as u32 / TILE_PX, canvas.1 as u32 / TILE_PX)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_640_by_480_canvas_is_40_by_30_tiles() {
        assert_eq!(view_tiles((640.0, 480.0)), (40, 30));
    }
}
