//! The 3.x start window: the first screen of 3.04 and 3.153, before the
//! server list.
//!
//! Layout, texts and colours are the shipped Torque GUI files of the two
//! clients (`mainMenuGui.gui` in 3.04, `MainMenuGui2D.gui` in 3.153; the
//! two differ only in the Tutorial button's text and the background
//! images), their English language tables and `customProfiles.cs`. Seihai
//! ships none of the clients' images, so the two background bitmaps are
//! plain fills. The buttons do not do anything yet beyond saying what the
//! real client would do next: the server list and offline mode come in
//! later stages.

use seihai_core::protocol::ClientVersion;

/// A control's place inside the login panel, in the client's 640×480
/// pixels: (x, y, width, height).
type Rect4 = (f32, f32, f32, f32);

/// The whole canvas.
pub const CANVAS: (f32, f32) = (640.0, 480.0);
/// The login panel (`GraalLogin`) inside it; every position below is
/// relative to this panel.
pub const PANEL: Rect4 = (31.0, 32.0, 577.0, 416.0);

pub const WELCOME: Rect4 = (155.0, 204.0, 136.0, 23.0);
pub const NICK_LABEL: Rect4 = (90.0, 240.0, 53.0, 18.0);
pub const NICK_EDIT: Rect4 = (209.0, 238.0, 150.0, 22.0);
pub const LANGUAGE_LABEL: Rect4 = (90.0, 273.0, 44.0, 18.0);
pub const LANGUAGE_BOX: Rect4 = (209.0, 271.0, 150.0, 22.0);
pub const ACCOUNT_LABEL: Rect4 = (90.0, 323.0, 73.0, 18.0);
pub const ACCOUNT_EDIT: Rect4 = (210.0, 321.0, 150.0, 22.0);
pub const PASS_LABEL: Rect4 = (90.0, 356.0, 53.0, 18.0);
pub const PASS_EDIT: Rect4 = (209.0, 354.0, 150.0, 22.0);
pub const TUTORIAL_BUTTON: Rect4 = (427.0, 212.0, 110.0, 25.0);
pub const OPTIONS_BUTTON: Rect4 = (427.0, 243.0, 110.0, 25.0);
pub const GET_ACCOUNT_BUTTON: Rect4 = (410.0, 274.0, 138.0, 25.0);
pub const START_BUTTON: Rect4 = (427.0, 335.0, 110.0, 25.0);

/// `GuiStartScreenButtonProfile`: font colour, highlighted font colour,
/// fill colour.
pub const BUTTON_FONT: [u8; 3] = [204, 204, 154];
pub const BUTTON_FONT_HL: [u8; 3] = [255, 255, 204];
pub const BUTTON_FILL: [u8; 3] = [40, 0, 0];
/// `GuiStartScreenTextProfile` font colour, used by the labels.
pub const LABEL_FONT: [u8; 3] = [255, 224, 160];

/// Where "Get an account" sends the player in both builds. Shown, not
/// opened.
pub const SIGNUP_URL: &str = "http://www.graalonline.com/accounts/signup.php";

/// The texts of one build, from its English language table.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Texts {
    pub welcome: &'static str,
    pub nickname: &'static str,
    pub language: &'static str,
    pub account: &'static str,
    pub password: &'static str,
    pub tutorial: &'static str,
    pub options: &'static str,
    pub get_account: &'static str,
    pub start: &'static str,
}

const TEXTS_304: Texts = Texts {
    welcome: "Welcome to Graal!",
    nickname: "Nickname:",
    language: "Language:",
    account: "Account:",
    password: "Password:",
    tutorial: "Tutorial",
    options: "Options",
    get_account: "Get an account",
    start: "Start",
};

/// The language box lists the language tables found on disk, sorted.
/// 3.04 ships two; 3.153 nine (`languages3D/`), one of them spelled
/// `languagePortguese.cs`, so the box reads "Portguese".
const LANGUAGES_304: &[&str] = &["Deutsch", "English"];
const LANGUAGES_3153: &[&str] = &["Deutsch", "English", "Espanol", "Francais", "Italiano", "Nederlands", "Norsk", "Portguese", "Svenska"];

/// One build's start window.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Layout {
    pub texts: Texts,
    pub languages: &'static [&'static str],
    /// 3.153's Start clears the password field after handing it over;
    /// 3.04's keeps it.
    pub start_clears_password: bool,
}

pub fn layout(v: ClientVersion) -> Layout {
    match v {
        ClientVersion::V304 => Layout { texts: TEXTS_304, languages: LANGUAGES_304, start_clears_password: false },
        // 3.153 changes only the Tutorial button's text.
        _ => Layout { texts: Texts { tutorial: "Offline Test", ..TEXTS_304 }, languages: LANGUAGES_3153, start_clears_password: true },
    }
}

/// What the player typed, and the last button pressed.
pub struct StartWindow {
    pub version: ClientVersion,
    pub layout: Layout,
    pub nick: String,
    pub language: String,
    pub account: String,
    pub password: String,
    pub status: String,
}

impl StartWindow {
    pub fn new(version: ClientVersion, nick: String, account: String, password: String) -> Self {
        StartWindow { version, layout: layout(version), nick, language: "English".into(), account, password, status: String::new() }
    }

    /// The Start button. The real client stores the fields and opens the
    /// server list; Ato has no server list yet.
    pub fn press_start(&mut self) {
        if self.layout.start_clears_password {
            self.password.clear();
        }
        self.status = "Start: the 3.x server list is not in Ato yet.".into();
    }

    pub fn press_tutorial(&mut self) {
        self.status = format!("{}: 3.x offline mode is not in Ato yet.", self.layout.texts.tutorial);
    }

    pub fn press_options(&mut self) {
        self.status = "Options: not in Ato yet.".into();
    }

    pub fn press_get_account(&mut self) {
        self.status = format!("Get an account: {SIGNUP_URL}");
    }
}

fn rgb(c: [u8; 3]) -> egui::Color32 {
    egui::Color32::from_rgb(c[0], c[1], c[2])
}

fn place(origin: egui::Pos2, r: Rect4) -> egui::Rect {
    egui::Rect::from_min_size(origin + egui::vec2(r.0, r.1), egui::vec2(r.2, r.3))
}

impl eframe::App for StartWindow {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().frame(egui::Frame::none().fill(egui::Color32::BLACK)).show(ctx, |ui| {
            let canvas = ui.max_rect().min;
            let origin = canvas + egui::vec2(PANEL.0, PANEL.1);
            // Stand-in for the panel bitmap (`graalscreen` / `graalscreen3d`).
            ui.painter().rect_filled(place(canvas, PANEL), 0.0, rgb(BUTTON_FILL));
            let t = self.layout.texts;

            ui.put(place(origin, WELCOME), egui::Label::new(egui::RichText::new(t.welcome).color(rgb(LABEL_FONT)).size(16.0)));
            // Torque draws a label from the left of its box and lets it run
            // past the box's width.
            for (r, text) in [(NICK_LABEL, t.nickname), (LANGUAGE_LABEL, t.language), (ACCOUNT_LABEL, t.account), (PASS_LABEL, t.password)] {
                ui.allocate_new_ui(egui::UiBuilder::new().max_rect(place(origin, r)), |ui| {
                    ui.add(egui::Label::new(egui::RichText::new(text).color(rgb(LABEL_FONT))).extend());
                });
            }
            ui.put(place(origin, NICK_EDIT), egui::TextEdit::singleline(&mut self.nick));
            ui.put(place(origin, ACCOUNT_EDIT), egui::TextEdit::singleline(&mut self.account));
            ui.put(place(origin, PASS_EDIT), egui::TextEdit::singleline(&mut self.password).password(true));
            let languages = self.layout.languages;
            let language = &mut self.language;
            ui.allocate_new_ui(egui::UiBuilder::new().max_rect(place(origin, LANGUAGE_BOX)), |ui| {
                egui::ComboBox::from_id_salt("language").width(LANGUAGE_BOX.2).selected_text(language.as_str()).show_ui(ui, |ui| {
                    for l in languages {
                        ui.selectable_value(language, (*l).to_string(), *l);
                    }
                });
            });

            let button = |ui: &mut egui::Ui, r: Rect4, text: &str| {
                let b = egui::Button::new(egui::RichText::new(text).color(rgb(BUTTON_FONT))).fill(rgb(BUTTON_FILL)).stroke(egui::Stroke::new(1.0_f32, rgb(BUTTON_FONT_HL)));
                ui.put(place(origin, r), b).clicked()
            };
            if button(ui, TUTORIAL_BUTTON, t.tutorial) {
                self.press_tutorial();
            }
            if button(ui, OPTIONS_BUTTON, t.options) {
                self.press_options();
            }
            if button(ui, GET_ACCOUNT_BUTTON, t.get_account) {
                self.press_get_account();
            }
            if button(ui, START_BUTTON, t.start) {
                self.press_start();
            }

            if !self.status.is_empty() {
                let r = (90.0, 390.0, 460.0, 18.0);
                ui.put(place(origin, r), egui::Label::new(egui::RichText::new(&self.status).color(rgb(LABEL_FONT))));
            }
        });
    }
}

/// Open the start window and block until it is closed.
pub fn run(window: StartWindow) -> Result<(), String> {
    let title = format!("Graal {} (Seihai Ato)", window.version.label());
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([CANVAS.0, CANVAS.1]).with_resizable(false),
        ..Default::default()
    };
    eframe::run_native(&title, options, Box::new(move |_cc| Ok(Box::new(window)))).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn controls_fit_in_the_panel() {
        for r in [
            WELCOME, NICK_LABEL, NICK_EDIT, LANGUAGE_LABEL, LANGUAGE_BOX, ACCOUNT_LABEL, ACCOUNT_EDIT, PASS_LABEL, PASS_EDIT, TUTORIAL_BUTTON, OPTIONS_BUTTON,
            GET_ACCOUNT_BUTTON, START_BUTTON,
        ] {
            assert!(r.0 + r.2 <= PANEL.2 && r.1 + r.3 <= PANEL.3, "{r:?}");
        }
        assert!(PANEL.0 + PANEL.2 <= CANVAS.0 && PANEL.1 + PANEL.3 <= CANVAS.1);
    }

    #[test]
    fn builds_differ_where_the_files_differ() {
        let a = layout(ClientVersion::V304);
        let b = layout(ClientVersion::V3153);
        assert_eq!(a.texts.tutorial, "Tutorial");
        assert_eq!(b.texts.tutorial, "Offline Test");
        assert_eq!(Texts { tutorial: a.texts.tutorial, ..b.texts }, a.texts);
        assert_eq!(a.languages.len(), 2);
        assert_eq!(b.languages.len(), 9);
        assert!(b.languages.windows(2).all(|w| w[0] < w[1]), "sorted");
    }

    #[test]
    fn start_clears_the_password_only_in_3153() {
        let mut w = StartWindow::new(ClientVersion::V304, "n".into(), "a".into(), "p".into());
        w.press_start();
        assert_eq!(w.password, "p");
        let mut w = StartWindow::new(ClientVersion::V3153, "n".into(), "a".into(), "p".into());
        w.press_start();
        assert_eq!(w.password, "");
    }
}
