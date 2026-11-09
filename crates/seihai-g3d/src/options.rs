//! The two option windows of the client's main window (Graal_analysis
//! `graal3d/README.md`): F1 "Hardware options" (`TMoreOptions`: the fixed
//! UDP port, default 14898, and "Don't use UDP") and F2 "Player look"
//! (`TEditLook`: `bodyx`, `headx`, `bodytex`, and the arm, shoe and hand
//! colours). Both edit a [`Config`], which the caller writes back to
//! `graal3dconfig.txt`.
//!
//! The windows' names and fields are the client's; the layout, the buttons
//! and when the UDP options take effect (the next connect) are Seihai's,
//! since the forms' positions and handlers were not read. While a window
//! is open the caller's `keepalive` runs every tick, so the session keeps
//! reading and sending.

use crate::config::Config;
use crate::session::TICK;
use std::time::Instant;

/// Which window.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Page {
    /// F1, "Hardware options".
    Hardware,
    /// F2, "Player look".
    Look,
}

impl Page {
    pub fn title(self) -> &'static str {
        match self {
            Page::Hardware => "Hardware options",
            Page::Look => "Player look",
        }
    }
}

/// What the window edits: the config's text fields as typed.
#[derive(Debug, Clone, PartialEq)]
pub struct Form {
    pub fixed_port: bool,
    pub fixed_udp_port: String,
    pub no_udp: bool,
    pub body_model: String,
    pub head_model: String,
    pub body_texture: String,
    pub arm: [u8; 3],
    pub shoe: [u8; 3],
    pub hand: [u8; 3],
}

fn rgb(c: u32) -> [u8; 3] {
    [(c >> 16) as u8, (c >> 8) as u8, c as u8]
}

fn packed(c: [u8; 3]) -> u32 {
    (c[0] as u32) << 16 | (c[1] as u32) << 8 | c[2] as u32
}

/// A file name the asset folders can hold: not empty, no folder part.
fn file_name(what: &str, v: &str) -> Result<String, String> {
    let v = v.trim();
    if v.is_empty() {
        return Err(format!("Enter the {what}."));
    }
    if v.contains(['/', '\\', ':']) || v == "." || v == ".." {
        return Err(format!(
            "The {what} is a file name in the Graal3D folder, without a folder."
        ));
    }
    Ok(v.to_string())
}

impl Form {
    pub fn of(c: &Config) -> Form {
        Form {
            fixed_port: c.fixed_port,
            fixed_udp_port: c.fixed_udp_port.to_string(),
            no_udp: c.no_udp,
            body_model: c.body_model.clone(),
            head_model: c.head_model.clone(),
            body_texture: c.body_texture.clone(),
            arm: rgb(c.arm_colour),
            shoe: rgb(c.shoe_colour),
            hand: rgb(c.hand_colour),
        }
    }

    /// The config with this page's fields applied, or what is wrong.
    /// Fields of the other page are left as they were.
    pub fn apply(&self, page: Page, c: &Config) -> Result<Config, String> {
        let mut out = c.clone();
        match page {
            Page::Hardware => {
                out.fixed_udp_port = self
                    .fixed_udp_port
                    .trim()
                    .parse()
                    .ok()
                    .filter(|&p: &u16| p > 0)
                    .ok_or("The UDP port is a number from 1 to 65535.")?;
                out.fixed_port = self.fixed_port;
                out.no_udp = self.no_udp;
            }
            Page::Look => {
                out.body_model = file_name("body model", &self.body_model)?;
                out.head_model = file_name("head model", &self.head_model)?;
                out.body_texture = file_name("body texture", &self.body_texture)?;
                out.arm_colour = packed(self.arm);
                out.shoe_colour = packed(self.shoe);
                out.hand_colour = packed(self.hand);
            }
        }
        Ok(out)
    }
}

struct App<'a> {
    page: Page,
    form: Form,
    base: Config,
    status: String,
    out: &'a mut Option<Config>,
    keepalive: &'a mut dyn FnMut() -> bool,
    next: Instant,
}

impl App<'_> {
    fn ok(&mut self, ctx: &egui::Context) {
        match self.form.apply(self.page, &self.base) {
            Ok(c) => {
                *self.out = Some(c);
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
            Err(e) => self.status = e,
        }
    }
}

impl eframe::App for App<'_> {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Keep the session ticking while the window is open: at most one
        // tick a frame, since a tick itself can wait up to a tick on the
        // socket and catching up would never end.
        if Instant::now() >= self.next {
            self.next = Instant::now() + TICK;
            if !(self.keepalive)() {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }
        ctx.request_repaint_after(TICK);
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading(self.page.title());
            ui.add_space(8.0);
            let f = &mut self.form;
            match self.page {
                Page::Hardware => {
                    ui.checkbox(&mut f.fixed_port, "Use a fixed UDP port");
                    ui.horizontal(|ui| {
                        ui.label("UDP port");
                        ui.add_enabled(
                            f.fixed_port,
                            egui::TextEdit::singleline(&mut f.fixed_udp_port).desired_width(80.0),
                        );
                    });
                    ui.checkbox(&mut f.no_udp, "Don't use UDP");
                    ui.add_space(4.0);
                    ui.small("Takes effect at the next connect.");
                }
                Page::Look => {
                    egui::Grid::new("look")
                        .num_columns(2)
                        .spacing([12.0, 6.0])
                        .show(ui, |ui| {
                            for (label, value) in [
                                ("Body model", &mut f.body_model),
                                ("Head model", &mut f.head_model),
                                ("Body texture", &mut f.body_texture),
                            ] {
                                ui.label(label);
                                ui.add(egui::TextEdit::singleline(value).desired_width(160.0));
                                ui.end_row();
                            }
                            for (label, value) in [
                                ("Arm colour", &mut f.arm),
                                ("Shoe colour", &mut f.shoe),
                                ("Hand colour", &mut f.hand),
                            ] {
                                ui.label(label);
                                ui.color_edit_button_srgb(value);
                                ui.end_row();
                            }
                        });
                }
            }
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                if ui.button("OK").clicked() || ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    self.ok(ctx);
                }
                if ui.button("Cancel").clicked() || ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            });
            if !self.status.is_empty() {
                ui.add_space(6.0);
                ui.colored_label(egui::Color32::LIGHT_RED, &self.status);
            }
        });
    }
}

/// Show one window until OK (the edited config) or Cancel / close
/// (`None`). `keepalive` runs once a tick meanwhile; when it returns false
/// (the session closed) the window closes as if cancelled.
pub fn run(
    page: Page,
    cfg: &Config,
    keepalive: &mut dyn FnMut() -> bool,
) -> Result<Option<Config>, String> {
    let mut out = None;
    let app = App {
        page,
        form: Form::of(cfg),
        base: cfg.clone(),
        status: String::new(),
        out: &mut out,
        keepalive,
        next: Instant::now() + TICK,
    };
    let size = match page {
        Page::Hardware => [300.0, 170.0],
        Page::Look => [320.0, 260.0],
    };
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size(size)
            .with_resizable(false),
        ..Default::default()
    };
    eframe::run_native(
        page.title(),
        options,
        Box::new(move |_cc| Ok(Box::new(app))),
    )
    .map_err(|e| e.to_string())?;
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_form_round_trips_the_shipped_values() {
        let c = Config::default();
        let f = Form::of(&c);
        assert_eq!(f.fixed_udp_port, "14898");
        assert_eq!((f.arm, f.hand), ([0xFF, 0, 0], [0xFD, 0xCC, 0x47]));
        assert_eq!(f.apply(Page::Hardware, &c).unwrap(), c);
        assert_eq!(f.apply(Page::Look, &c).unwrap(), c);
    }

    #[test]
    fn hardware_options() {
        let c = Config::default();
        let mut f = Form::of(&c);
        f.fixed_port = true;
        f.fixed_udp_port = " 15000 ".into();
        f.no_udp = true;
        // The look fields are not this page's: a bad one does not matter.
        f.body_model = String::new();
        let out = f.apply(Page::Hardware, &c).unwrap();
        assert_eq!(
            (out.fixed_port, out.fixed_udp_port, out.no_udp),
            (true, 15000, true)
        );
        assert_eq!(out.body_model, "body.x");
        for bad in ["0", "70000", "x", ""] {
            f.fixed_udp_port = bad.into();
            assert!(f.apply(Page::Hardware, &c).is_err(), "{bad}");
        }
    }

    #[test]
    fn player_look() {
        let c = Config::default();
        let mut f = Form::of(&c);
        f.body_texture = " body1.bmp ".into();
        f.arm = [0x00, 0x80, 0x00];
        f.fixed_udp_port = "x".into();
        let out = f.apply(Page::Look, &c).unwrap();
        assert_eq!(
            (out.body_texture.as_str(), out.arm_colour),
            ("body1.bmp", 0x008000)
        );
        assert_eq!(out.fixed_udp_port, 14898);
        for bad in ["", "../body.x", "dir/body.x", "c:body.x", "..", "a\\b.x"] {
            f.body_model = bad.into();
            assert!(f.apply(Page::Look, &c).is_err(), "{bad:?}");
        }
    }
}
