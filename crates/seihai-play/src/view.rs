//! The game in an egui window: the flat frame, the status line, the
//! position, the other players' names, the bubbles and the chat. The
//! arrows walk.
//!
//! Tab (or Enter) opens a chat line at the foot of the window; Enter says
//! it in the level and Escape closes it. The arrows do not walk while it
//! is open. The 6.x builds take chat through the server's GUI scripts and
//! have no chat bar of their own (lab, #444), so this line is Seihai's
//! own until those scripts run (#447). Toalls, PMs and admin lines show
//! for a while above it.
//!
//! S swings the sword ([`Game::swing`]).

use crate::game::{bubble_at, bubbles, name_tags, npc_bubbles, Game};
use crate::render::{self, TILE_PX};
use seihai_core::protocol::ChatKind;
use std::time::{Duration, Instant};

/// A [`Game`] on a canvas of `canvas` pixels.
pub struct View {
    pub game: Game,
    canvas: (f32, f32),
    texture: Option<egui::TextureHandle>,
    /// The chat line being typed, while it is open.
    typing: Option<String>,
}

/// How long a toall, PM or admin line stays on screen.
pub const CHAT_SHOWN: Duration = Duration::from_secs(10);
/// How many of those lines show at once.
pub const CHAT_ROWS: usize = 6;

impl View {
    pub fn new(game: Game, canvas: (f32, f32)) -> View {
        View {
            game,
            canvas,
            texture: None,
            typing: None,
        }
    }

    /// The view in tiles.
    pub fn tiles(&self) -> (u32, u32) {
        view_tiles(self.canvas)
    }

    /// One frame: run a turn of the loop for the arrows held and draw it.
    pub fn show(&mut self, ctx: &egui::Context) {
        let now = Instant::now();
        let pressed = |k| ctx.input(|i| i.key_pressed(k));
        match self.typing.take() {
            None if pressed(egui::Key::Tab) || pressed(egui::Key::Enter) => {
                self.typing = Some(String::new())
            }
            Some(_) if pressed(egui::Key::Escape) => {}
            Some(text) if pressed(egui::Key::Enter) => self.game.say(&text, now),
            other => self.typing = other,
        }
        // S swings the sword, as in the classic key set.
        if self.typing.is_none() && pressed(egui::Key::S) {
            self.game.swing();
        }
        let held = |k| self.typing.is_none() && ctx.input(|i| i.key_down(k));
        let dx = held(egui::Key::ArrowRight) as i8 - held(egui::Key::ArrowLeft) as i8;
        let dy = held(egui::Key::ArrowDown) as i8 - held(egui::Key::ArrowUp) as i8;
        let changed = self.game.tick(dx, dy, now);
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
                // The frame fills the window, scaled to keep its shape and
                // centred, so a maximised window shows the whole game.
                let (origin, scale) = fit((w, h), ui.max_rect());
                if let Some(t) = &self.texture {
                    let rect = egui::Rect::from_min_size(origin, egui::vec2(w, h) * scale);
                    ui.painter().image(
                        t.id(),
                        rect,
                        egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                        egui::Color32::WHITE,
                    );
                }
                // With a status display the top belongs to it: the status
                // line and the position go to the foot of the window.
                let bottom = self.game.has_hud();
                let bar_y = if bottom { h * scale - 18.0 } else { 0.0 };
                let bar = egui::Rect::from_min_size(
                    origin + egui::vec2(0.0, bar_y),
                    egui::vec2(w * scale, 18.0),
                );
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
                        origin + egui::vec2(x, y) * scale,
                        egui::Align2::CENTER_TOP,
                        nick,
                        egui::FontId::proportional(11.0),
                        egui::Color32::WHITE,
                    );
                }
                if let Some(pos) = self.game.position() {
                    let y = if bottom { bar_y - 16.0 } else { 18.0 };
                    let at = egui::Rect::from_min_size(
                        origin + egui::vec2(0.0, y),
                        egui::vec2(w * scale - 6.0, 16.0),
                    );
                    ui.painter().text(
                        at.right_top(),
                        egui::Align2::RIGHT_TOP,
                        pos,
                        egui::FontId::monospace(12.0),
                        egui::Color32::WHITE,
                    );
                }
                // Bubbles over the players who said something, the
                // player's own included.
                let mut said = bubbles(self.game.others(), cam_x, cam_y);
                if let Some(level) = &self.game.session.current_level {
                    said.extend(npc_bubbles(level, cam_x, cam_y));
                }
                if let Some(text) = self.game.own_bubble(now) {
                    let s = &self.game.session;
                    let (x, y) = bubble_at(s.player_x, s.player_y, cam_x, cam_y);
                    said.push((text.to_string(), x, y));
                }
                for (text, x, y) in said {
                    outlined(
                        ui.painter(),
                        origin + egui::vec2(x, y) * scale,
                        egui::Align2::CENTER_BOTTOM,
                        &text,
                    );
                }
                // Toalls, PMs and admin lines, newest at the foot, above
                // the chat line and the status line.
                let foot = if bottom {
                    bar_y - 34.0
                } else {
                    h * scale - 18.0
                };
                let shown: Vec<String> = chat_rows(self.game.chat(), now);
                for (i, line) in shown.iter().rev().enumerate() {
                    outlined(
                        ui.painter(),
                        origin + egui::vec2(6.0, foot - 18.0 * (i as f32 + 1.0)),
                        egui::Align2::LEFT_TOP,
                        line,
                    );
                }
                if let Some(text) = self.typing.as_mut() {
                    let at = egui::Rect::from_min_size(
                        origin + egui::vec2(4.0, foot),
                        egui::vec2(w * scale - 8.0, 18.0),
                    );
                    let edit = ui.put(
                        at,
                        egui::TextEdit::singleline(text)
                            .hint_text("Say something (Enter to say, Esc to close)")
                            .desired_width(f32::INFINITY),
                    );
                    edit.request_focus();
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

/// White text with a dark outline, as the clients draw names and chat.
fn outlined(p: &egui::Painter, at: egui::Pos2, align: egui::Align2, text: &str) {
    let font = egui::FontId::proportional(13.0);
    for (dx, dy) in [(-1.0, 0.0), (1.0, 0.0), (0.0, -1.0), (0.0, 1.0)] {
        p.text(
            at + egui::vec2(dx, dy),
            align,
            text,
            font.clone(),
            egui::Color32::BLACK,
        );
    }
    p.text(at, align, text, font, egui::Color32::WHITE);
}

/// The toall, PM and admin lines still on screen at `now`, oldest first,
/// at most [`CHAT_ROWS`]. What was said in the level shows as a bubble
/// instead.
pub fn chat_rows(chat: &[(seihai_core::protocol::ChatLine, Instant)], now: Instant) -> Vec<String> {
    let mut rows: Vec<String> = chat
        .iter()
        .filter(|(l, at)| {
            l.kind != ChatKind::Say && now.saturating_duration_since(*at) < CHAT_SHOWN
        })
        .map(|(l, _)| match l.kind {
            ChatKind::Private => format!("(PM) {}: {}", l.from, l.text),
            ChatKind::Toall => format!("{}: {}", l.from, l.text),
            _ => l.text.clone(),
        })
        .collect();
    if rows.len() > CHAT_ROWS {
        rows.drain(..rows.len() - CHAT_ROWS);
    }
    rows
}

/// Where a frame of `size` pixels goes in `area`: its top left and its
/// scale, as large as fits with its shape kept, centred.
pub fn fit(size: (f32, f32), area: egui::Rect) -> (egui::Pos2, f32) {
    let scale = (area.width() / size.0)
        .min(area.height() / size.1)
        .max(0.01);
    let shown = egui::vec2(size.0, size.1) * scale;
    (area.center() - shown / 2.0, scale)
}

/// A canvas of `canvas` pixels in whole tiles.
pub fn view_tiles(canvas: (f32, f32)) -> (u32, u32) {
    (canvas.0 as u32 / TILE_PX, canvas.1 as u32 / TILE_PX)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chat_rows_show_the_last_lines_that_are_not_bubbles() {
        use seihai_core::protocol::ChatLine;
        let t0 = Instant::now();
        let line = |kind, text: &str| ChatLine {
            kind,
            from: "Ann".into(),
            text: text.into(),
            born_ms: 0,
        };
        let mut chat = vec![
            (line(ChatKind::Say, "bubble"), t0),
            (line(ChatKind::Private, "psst"), t0),
            (line(ChatKind::Admin, "Server going down"), t0),
        ];
        assert_eq!(
            chat_rows(&chat, t0),
            vec!["(PM) Ann: psst".to_string(), "Server going down".into()]
        );
        assert!(chat_rows(&chat, t0 + CHAT_SHOWN).is_empty(), "faded");
        for i in 0..10 {
            chat.push((line(ChatKind::Toall, &i.to_string()), t0));
        }
        let rows = chat_rows(&chat, t0);
        assert_eq!(rows.len(), CHAT_ROWS);
        assert_eq!(rows.last().map(String::as_str), Some("Ann: 9"));
    }

    #[test]
    fn a_640_by_480_canvas_is_40_by_30_tiles() {
        assert_eq!(view_tiles((640.0, 480.0)), (40, 30));
    }

    #[test]
    fn a_maximised_window_scales_the_frame_and_centres_it() {
        let r = |w, h| egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(w, h));
        // The window's own size: drawn as it is.
        assert_eq!(
            fit((640.0, 480.0), r(640.0, 480.0)),
            (egui::pos2(0.0, 0.0), 1.0)
        );
        // 1920 x 1080: scaled by 2.25 to 1440 x 1080, bars left and right.
        assert_eq!(
            fit((640.0, 480.0), r(1920.0, 1080.0)),
            (egui::pos2(240.0, 0.0), 2.25)
        );
    }
}
