//! The 3.x start window: the first screen of 3.04 and 3.153, before the
//! server list.
//!
//! Layout, texts and colours are the shipped Torque GUI files of the two
//! clients (`mainMenuGui.gui` in 3.04, `MainMenuGui2D.gui` in 3.153; the
//! two differ only in the Tutorial button's text and the background
//! images), their English language tables and `customProfiles.cs`. Seihai
//! ships none of the clients' images, so the two background bitmaps are
//! plain fills. Start opens the server list ([`crate::server_list`]), or
//! logs in at once when `--online` names a game server. Tutorial (offline
//! mode) and Options only say they are not in Ato yet.
//!
//! Run on its own (`seihai ato`), the window starts from the account,
//! nickname and typed listserver the `seihai` start window saved
//! (`seihai_play::start::Saved`, `launcher.conf`) and saves them back on
//! Start, never the password.

use crate::online::Login;
use seihai_core::protocol::ClientVersion;
use seihai_play::start::Saved;
use std::sync::{Arc, Mutex};

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
const LANGUAGES_3153: &[&str] = &[
    "Deutsch",
    "English",
    "Espanol",
    "Francais",
    "Italiano",
    "Nederlands",
    "Norsk",
    "Portguese",
    "Svenska",
];

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
        ClientVersion::V304 => Layout {
            texts: TEXTS_304,
            languages: LANGUAGES_304,
            start_clears_password: false,
        },
        // 3.153 changes only the Tutorial button's text.
        _ => Layout {
            texts: Texts {
                tutorial: "Offline Test",
                ..TEXTS_304
            },
            languages: LANGUAGES_3153,
            start_clears_password: true,
        },
    }
}

/// Where Start goes next.
#[derive(Debug, Clone, PartialEq)]
pub enum Next {
    /// Nothing: Start was not pressed.
    Stay,
    /// Log in at this game server (`host:port`).
    Login(String),
    /// Open the server list.
    ServerList,
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
    /// The game server Start logs in to (`--online`). Without it, Start
    /// opens the server list.
    pub online: Option<String>,
    /// The listserver the server list comes from (`host:port`).
    pub listserver: String,
    /// Set by Start when there is a server: the form, taken before 3.153
    /// clears the password.
    pub started: Arc<Mutex<Option<Login>>>,
}

impl StartWindow {
    pub fn new(version: ClientVersion, nick: String, account: String, password: String) -> Self {
        StartWindow {
            version,
            layout: layout(version),
            nick,
            language: "English".into(),
            account,
            password,
            status: String::new(),
            online: None,
            listserver: crate::server_list::default_listserver(version),
            started: Arc::default(),
        }
    }

    /// The Start button. The real client stores the fields and opens the
    /// server list. Ato does the same, or with `--online` goes straight to
    /// that game server. The form is handed over in `started` before 3.153
    /// clears the password.
    pub fn press_start(&mut self) -> Next {
        let login = Login {
            account: self.account.clone(),
            password: self.password.clone(),
            nick: self.nick.clone(),
            language: self.language.clone(),
        };
        if self.layout.start_clears_password {
            self.password.clear();
        }
        *self.started.lock().unwrap() = Some(login);
        match &self.online {
            Some(addr) => {
                self.status = format!("Start: logging in at {addr}.");
                Next::Login(addr.clone())
            }
            None => {
                self.status = format!("Start: asking {} for the server list.", self.listserver);
                if let Some(note) = crate::server_list::borrowed_list_note(self.version) {
                    self.status = format!("{} {note}", self.status);
                }
                Next::ServerList
            }
        }
    }

    /// Fill the fields the command line left empty from the shared start
    /// window's saved form (`launcher.conf`), so `seihai ato` and the
    /// `seihai` start window remember the same account. A listserver is
    /// taken only when the user typed one there.
    pub fn fill_from(&mut self, saved: &Saved) {
        if self.account.is_empty() {
            if let Some(a) = &saved.account {
                self.account = a.clone();
            }
        }
        if self.nick.is_empty() {
            // The saved nickname goes with the saved account only.
            self.nick = saved
                .nick
                .clone()
                .filter(|_| saved.account.as_deref() == Some(self.account.as_str()))
                .unwrap_or_else(|| self.account.clone());
        }
        if let Some(ls) = &saved.listserver {
            self.listserver = ls.clone();
        }
    }

    /// The shared form with this window's account and nickname in it. The
    /// password is never saved.
    pub fn remembered(&self, mut saved: Saved) -> Saved {
        let some = |s: &str| Some(s.to_string()).filter(|s| !s.is_empty());
        saved.account = some(&self.account).or(saved.account);
        saved.nick = some(&self.nick).or(saved.nick);
        saved
    }

    pub fn press_tutorial(&mut self) {
        self.status = format!(
            "{}: 3.x offline mode is not in Ato yet.",
            self.layout.texts.tutorial
        );
    }

    pub fn press_options(&mut self) {
        self.status = "Options: not in Ato yet.".into();
    }

    pub fn press_get_account(&mut self) {
        self.status = "Get an account: accounts are made by the listserver you play on.".into();
    }
}

fn rgb(c: [u8; 3]) -> egui::Color32 {
    egui::Color32::from_rgb(c[0], c[1], c[2])
}

fn place(origin: egui::Pos2, r: Rect4) -> egui::Rect {
    egui::Rect::from_min_size(origin + egui::vec2(r.0, r.1), egui::vec2(r.2, r.3))
}

impl StartWindow {
    /// Draw the window and take its input: where Start goes, if it was
    /// pressed (see [`Self::press_start`]).
    pub fn show(&mut self, ctx: &egui::Context) -> Next {
        let mut started = Next::Stay;
        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(egui::Color32::BLACK))
            .show(ctx, |ui| {
                let canvas = ui.max_rect().min;
                let origin = canvas + egui::vec2(PANEL.0, PANEL.1);
                // Stand-in for the panel bitmap (`graalscreen` / `graalscreen3d`).
                ui.painter()
                    .rect_filled(place(canvas, PANEL), 0.0, rgb(BUTTON_FILL));
                let t = self.layout.texts;

                ui.put(
                    place(origin, WELCOME),
                    egui::Label::new(
                        egui::RichText::new(t.welcome)
                            .color(rgb(LABEL_FONT))
                            .size(16.0),
                    ),
                );
                // Torque draws a label from the left of its box and lets it run
                // past the box's width.
                for (r, text) in [
                    (NICK_LABEL, t.nickname),
                    (LANGUAGE_LABEL, t.language),
                    (ACCOUNT_LABEL, t.account),
                    (PASS_LABEL, t.password),
                ] {
                    ui.allocate_new_ui(egui::UiBuilder::new().max_rect(place(origin, r)), |ui| {
                        ui.add(
                            egui::Label::new(egui::RichText::new(text).color(rgb(LABEL_FONT)))
                                .extend(),
                        );
                    });
                }
                ui.put(
                    place(origin, NICK_EDIT),
                    egui::TextEdit::singleline(&mut self.nick),
                );
                ui.put(
                    place(origin, ACCOUNT_EDIT),
                    egui::TextEdit::singleline(&mut self.account),
                );
                ui.put(
                    place(origin, PASS_EDIT),
                    egui::TextEdit::singleline(&mut self.password).password(true),
                );
                let languages = self.layout.languages;
                let language = &mut self.language;
                ui.allocate_new_ui(
                    egui::UiBuilder::new().max_rect(place(origin, LANGUAGE_BOX)),
                    |ui| {
                        egui::ComboBox::from_id_salt("language")
                            .width(LANGUAGE_BOX.2)
                            .selected_text(language.as_str())
                            .show_ui(ui, |ui| {
                                for l in languages {
                                    ui.selectable_value(language, (*l).to_string(), *l);
                                }
                            });
                    },
                );

                let button = |ui: &mut egui::Ui, r: Rect4, text: &str| {
                    let b = egui::Button::new(egui::RichText::new(text).color(rgb(BUTTON_FONT)))
                        .fill(rgb(BUTTON_FILL))
                        .stroke(egui::Stroke::new(1.0_f32, rgb(BUTTON_FONT_HL)));
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
                    started = self.press_start();
                }

                if !self.status.is_empty() {
                    let r = (90.0, 390.0, 460.0, 18.0);
                    ui.put(
                        place(origin, r),
                        egui::Label::new(egui::RichText::new(&self.status).color(rgb(LABEL_FONT))),
                    );
                }
            });
        started
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn saved(account: &str, nick: &str, listserver: Option<&str>) -> Saved {
        Saved {
            account: Some(account.into()),
            nick: Some(nick.into()),
            listserver: listserver.map(String::from),
            ..Saved::default()
        }
    }

    #[test]
    fn the_saved_form_fills_only_what_the_command_line_left_empty() {
        let mut w = StartWindow::new(ClientVersion::V304, "".into(), "".into(), "".into());
        let default_ls = w.listserver.clone();
        w.fill_from(&saved("lab", "Lab Nick", None));
        assert_eq!((w.account.as_str(), w.nick.as_str()), ("lab", "Lab Nick"));
        assert_eq!(
            w.listserver, default_ls,
            "no typed listserver: follow the version"
        );

        let mut w = StartWindow::new(ClientVersion::V304, "".into(), "me".into(), "".into());
        w.fill_from(&saved("lab", "Lab Nick", Some("127.0.0.1:14922")));
        assert_eq!(w.account, "me");
        assert_eq!(w.nick, "me", "another account's nickname is not borrowed");
        assert_eq!(w.listserver, "127.0.0.1:14922");
    }

    #[test]
    fn start_remembers_the_account_and_nickname_but_not_the_password() {
        let w = StartWindow::new(
            ClientVersion::V3153,
            "N".into(),
            "acc".into(),
            "secret".into(),
        );
        let s = w.remembered(saved("old", "Old", Some("h:1")));
        assert_eq!(s.account.as_deref(), Some("acc"));
        assert_eq!(s.nick.as_deref(), Some("N"));
        assert_eq!(s.listserver.as_deref(), Some("h:1"));
        assert!(!s.text().contains("secret"));

        let empty = StartWindow::new(ClientVersion::V3153, "".into(), "".into(), "".into());
        let s = empty.remembered(saved("old", "Old", None));
        assert_eq!(s.account.as_deref(), Some("old"));
    }

    #[test]
    fn controls_fit_in_the_panel() {
        for r in [
            WELCOME,
            NICK_LABEL,
            NICK_EDIT,
            LANGUAGE_LABEL,
            LANGUAGE_BOX,
            ACCOUNT_LABEL,
            ACCOUNT_EDIT,
            PASS_LABEL,
            PASS_EDIT,
            TUTORIAL_BUTTON,
            OPTIONS_BUTTON,
            GET_ACCOUNT_BUTTON,
            START_BUTTON,
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
        assert_eq!(
            Texts {
                tutorial: a.texts.tutorial,
                ..b.texts
            },
            a.texts
        );
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

    #[test]
    fn start_opens_the_list_or_logs_in_at_the_given_server() {
        let mut w = StartWindow::new(ClientVersion::V3153, "n".into(), "a".into(), "p".into());
        assert_eq!(w.press_start(), Next::ServerList);
        assert!(w.started.lock().unwrap().is_some());
        assert!(w.status.contains("listserver.graal.in:"), "{}", w.status);
        let mut w = StartWindow::new(ClientVersion::V3153, "n".into(), "a".into(), "p".into());
        w.online = Some("127.0.0.1:14900".into());
        assert_eq!(w.press_start(), Next::Login("127.0.0.1:14900".into()));
        let login = w.started.lock().unwrap().take().unwrap();
        // Taken before 3.153 clears the field.
        assert_eq!(
            (
                login.account.as_str(),
                login.password.as_str(),
                login.nick.as_str()
            ),
            ("a", "p", "n")
        );
        assert_eq!(w.password, "");
    }
}
