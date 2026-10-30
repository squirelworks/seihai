//! The level editor window: a tile palette, click-to-paint on the board, and
//! save. It sits on the command-line slice (`Document`): painting is
//! `Document::paint` with a 1 x 1 rectangle and saving is `Document::save`, so
//! the file rules (byte-for-byte round trip, canonical rows after a change,
//! write through a temporary file) are the same as the CLI's.
//!
//! The editor state (`Editor`) knows nothing about egui and is tested on its
//! own; `EditorApp` draws it. `EditorApp::ui` takes a bare `egui::Context`, so
//! the tests also drive the real window code with synthetic mouse input.

use crate::{Document, EditError, Version, SIZE, TILE_LIMIT};
use eframe::egui;
use egui::{Color32, ColorImage, Pos2, Rect, Sense, Stroke, TextureHandle, TextureOptions, Vec2};
use seihai::assets::read_png_rgb;
use seihai::render::{Framebuffer, TILE_PX};
use std::path::{Path, PathBuf};

/// Columns of 16 px tiles on a `pics1.png` sheet (2048 px).
pub const SHEET_COLS: u32 = 128;
/// Rows of 16 px tiles on a `pics1.png` sheet (512 px).
pub const SHEET_ROWS: u32 = 32;

/// Where tile `id` sits on the sheet, as (column, row) in tiles. The same
/// formula as the renderer (`render/tiles.rs`): eight 16-column blocks of 512.
pub fn sheet_cell(id: u16) -> (u32, u32) {
    let t = id as u32;
    (t % 16 + (t / 512) * 16, (t / 16) % SHEET_ROWS)
}

/// The tile id at sheet cell (column, row); the inverse of `sheet_cell`.
pub fn sheet_tile(col: u32, row: u32) -> Option<u16> {
    (col < SHEET_COLS && row < SHEET_ROWS).then(|| ((col % 16) + row * 16 + (col / 16) * 512) as u16)
}

/// The board cells on the straight line from `a` to `b` (both included), so
/// a fast drag that skips cells between two frames still paints a stroke.
pub fn line_cells(a: (usize, usize), b: (usize, usize)) -> Vec<(usize, usize)> {
    let (mut x, mut y) = (a.0 as i64, a.1 as i64);
    let (x1, y1) = (b.0 as i64, b.1 as i64);
    let (dx, dy) = ((x1 - x).abs(), -(y1 - y).abs());
    let (sx, sy) = (if x < x1 { 1 } else { -1 }, if y < y1 { 1 } else { -1 });
    let mut err = dx + dy;
    let mut out = vec![(x as usize, y as usize)];
    while (x, y) != (x1, y1) {
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x += sx;
        }
        if e2 <= dx {
            err += dx;
            y += sy;
        }
        out.push((x as usize, y as usize));
    }
    out
}

/// The editor without its window: one open level, the selected tile, the
/// version, and whether there are unsaved changes.
pub struct Editor {
    pub version: Version,
    /// The tile the left button paints.
    pub tile: u16,
    doc: Option<Document>,
    path: Option<PathBuf>,
    unsaved: bool,
    /// Bumped on every change of the board, so the window knows when to redraw it.
    pub board_rev: u64,
}

impl Editor {
    pub fn new(version: Version) -> Editor {
        Editor { version, tile: 0, doc: None, path: None, unsaved: false, board_rev: 0 }
    }

    pub fn doc(&self) -> Option<&Document> {
        self.doc.as_ref()
    }

    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    /// Painted since the last open or save.
    pub fn has_unsaved(&self) -> bool {
        self.unsaved
    }

    /// Open a level; the current one is dropped (the window asks first when
    /// it has unsaved changes). On an error nothing changes.
    pub fn open(&mut self, path: &Path) -> Result<(), EditError> {
        let doc = Document::open(path)?;
        self.doc = Some(doc);
        self.path = Some(path.to_path_buf());
        self.unsaved = false;
        self.board_rev += 1;
        Ok(())
    }

    /// Paint the selected tile at board cell (x, y). Returns the number of
    /// tiles that changed (0 or 1).
    pub fn paint_at(&mut self, x: usize, y: usize) -> Result<usize, EditError> {
        let doc = self.doc.as_mut().ok_or_else(|| EditError::Io("no level is open".into()))?;
        let changed = doc.paint(x, y, 1, 1, self.tile as u32)?;
        if changed > 0 {
            self.unsaved = true;
            self.board_rev += 1;
        }
        Ok(changed)
    }

    /// Select the tile under board cell (x, y) (the right button).
    pub fn pick_at(&mut self, x: usize, y: usize) -> Option<u16> {
        let t = self.doc.as_ref()?.tile(x, y)?;
        self.tile = t;
        Some(t)
    }

    /// Select a tile id; ids the text format cannot hold are refused.
    pub fn select(&mut self, id: u32) -> Result<(), EditError> {
        if id >= TILE_LIMIT as u32 {
            return Err(EditError::TileRange(id));
        }
        self.tile = id as u16;
        Ok(())
    }

    /// Save to the file that was opened.
    pub fn save(&mut self) -> Result<PathBuf, EditError> {
        let path = self.path.clone().ok_or_else(|| EditError::Io("no level is open".into()))?;
        self.save_as(&path)
    }

    /// Save to `path`, which becomes the file that Save writes from now on.
    pub fn save_as(&mut self, path: &Path) -> Result<PathBuf, EditError> {
        let doc = self.doc.as_ref().ok_or_else(|| EditError::Io("no level is open".into()))?;
        doc.save(path)?;
        self.path = Some(path.to_path_buf());
        self.unsaved = false;
        Ok(path.to_path_buf())
    }
}

/// An RGB tile sheet.
pub struct Sheet {
    pub width: u32,
    pub height: u32,
    pub rgb: Vec<u8>,
}

impl Sheet {
    /// Read a PNG sheet (`pics1.png` layout, 2048 x 512).
    pub fn from_png(data: &[u8]) -> Result<Sheet, String> {
        let (width, height, rgb) = read_png_rgb(data).map_err(|e| format!("cannot read the tile sheet: {e}"))?;
        Ok(Sheet { width, height, rgb })
    }

    /// The `pics1.png` of the built-in default asset set.
    pub fn builtin() -> Sheet {
        let data = seihai::assets::builtin::bytes("pics1.png").expect("pics1.png is in the built-in set");
        Sheet::from_png(data).expect("the built-in pics1.png decodes")
    }
}

/// The 64 x 64 board drawn with `sheet`, 16 px per tile.
pub fn board_image(doc: &Document, sheet: &Sheet) -> Framebuffer {
    let px = SIZE as u32 * TILE_PX;
    let mut fb = Framebuffer::new(px, px);
    for y in 0..SIZE {
        let row = doc.row(y).expect("row in range");
        for (x, &t) in row.iter().enumerate() {
            fb.blit_tile(&sheet.rgb, sheet.width, sheet.height, t, (x as u32 * TILE_PX) as i32, (y as u32 * TILE_PX) as i32);
        }
    }
    fb
}

/// A question the window is waiting on before it drops unsaved changes.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Pending {
    Open(PathBuf),
    Close,
}

/// The window.
pub struct EditorApp {
    pub editor: Editor,
    sheet: Sheet,
    sheet_tex: Option<TextureHandle>,
    board_tex: Option<TextureHandle>,
    board_tex_rev: u64,
    /// The text boxes for Open and Save as.
    open_path: String,
    save_as_path: String,
    zoom: f32,
    grid: bool,
    status: String,
    hover: Option<(usize, usize)>,
    /// The last cell painted in the current drag.
    stroke_last: Option<(usize, usize)>,
    pending: Option<Pending>,
    title: String,
    /// Where the board and the palette were drawn in the last frame (for tests).
    pub board_rect: Option<Rect>,
    pub palette_rect: Option<Rect>,
    /// Set once the window may close (no unsaved changes, or the user chose).
    pub closing: bool,
}

impl EditorApp {
    pub fn new(version: Version, sheet: Sheet, file: Option<&Path>) -> EditorApp {
        let mut app = EditorApp {
            editor: Editor::new(version),
            sheet,
            sheet_tex: None,
            board_tex: None,
            board_tex_rev: 0,
            open_path: String::new(),
            save_as_path: String::new(),
            zoom: 1.0,
            grid: false,
            status: String::new(),
            hover: None,
            stroke_last: None,
            pending: None,
            title: String::new(),
            board_rect: None,
            palette_rect: None,
            closing: false,
        };
        if let Some(p) = file {
            app.open_path = p.display().to_string();
            app.do_open(p.to_path_buf());
        } else {
            app.status = "Type a .nw path and press Open.".into();
        }
        app
    }

    fn do_open(&mut self, path: PathBuf) {
        match self.editor.open(&path) {
            Ok(()) => {
                self.save_as_path = path.display().to_string();
                self.open_path = path.display().to_string();
                self.status = format!("Opened {}", path.display());
            }
            Err(e) => self.status = e.to_string(),
        }
    }

    fn do_save(&mut self, to: Option<PathBuf>) -> bool {
        let r = match to {
            Some(p) => self.editor.save_as(&p),
            None => self.editor.save(),
        };
        match r {
            Ok(p) => {
                self.status = format!("Saved {}", p.display());
                true
            }
            Err(e) => {
                self.status = e.to_string();
                false
            }
        }
    }

    /// One frame of the window. `update` calls this; tests call it with a bare context.
    pub fn ui(&mut self, ctx: &egui::Context) {
        if ctx.input(|i| i.viewport().close_requested()) && !self.closing {
            if self.editor.has_unsaved() {
                ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                self.pending = Some(Pending::Close);
            } else {
                self.closing = true;
            }
        }
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::S)) && self.editor.doc().is_some() {
            self.do_save(None);
        }

        let title = format!(
            "{}{} - Seihai level editor ({})",
            if self.editor.has_unsaved() { "*" } else { "" },
            self.editor.path().map(|p| p.display().to_string()).unwrap_or_else(|| "no level".into()),
            self.editor.version.name()
        );
        if title != self.title {
            ctx.send_viewport_cmd(egui::ViewportCommand::Title(title.clone()));
            self.title = title;
        }

        self.toolbar(ctx);
        self.status_bar(ctx);
        self.palette(ctx);
        self.board(ctx);
        self.prompt(ctx);
    }

    fn toolbar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Level");
                let field = ui.add(egui::TextEdit::singleline(&mut self.open_path).desired_width(260.0).hint_text("path/to/level.nw"));
                let enter = field.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                if (ui.button("Open").clicked() || enter) && !self.open_path.trim().is_empty() {
                    let p = PathBuf::from(self.open_path.trim());
                    if self.editor.has_unsaved() {
                        self.pending = Some(Pending::Open(p));
                    } else {
                        self.do_open(p);
                    }
                }
                ui.separator();
                let has_doc = self.editor.doc().is_some();
                if ui.add_enabled(has_doc, egui::Button::new("Save")).on_hover_text("Ctrl+S").clicked() {
                    self.do_save(None);
                }
                ui.add(egui::TextEdit::singleline(&mut self.save_as_path).desired_width(200.0));
                if ui.add_enabled(has_doc && !self.save_as_path.trim().is_empty(), egui::Button::new("Save as")).clicked() {
                    self.do_save(Some(PathBuf::from(self.save_as_path.trim())));
                }
                ui.separator();
                ui.label("Version");
                egui::ComboBox::from_id_salt("version")
                    .selected_text(self.editor.version.name())
                    .show_ui(ui, |ui| {
                        for v in Version::SUPPORTED {
                            ui.selectable_value(&mut self.editor.version, v, v.name());
                        }
                    })
                    .response
                    .on_hover_text("Other versions are not yet supported (see documentation/level-editor.md)");
                ui.separator();
                ui.label("Zoom");
                for z in [1.0, 2.0] {
                    ui.selectable_value(&mut self.zoom, z, format!("{z}x"));
                }
                ui.checkbox(&mut self.grid, "Grid");
            });
        });
    }

    fn status_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("status").show(ctx, |ui| {
            ui.horizontal(|ui| {
                let t = self.editor.tile;
                ui.label(format!("Tile {t} (0x{t:03X})"));
                let mut id = t as u32;
                if ui.add(egui::DragValue::new(&mut id).range(0..=TILE_LIMIT as u32 - 1)).changed() {
                    let _ = self.editor.select(id);
                }
                ui.separator();
                if let (Some((x, y)), Some(doc)) = (self.hover, self.editor.doc()) {
                    let h = doc.tile(x, y).unwrap_or(0);
                    ui.label(format!("x {x}, y {y}: tile {h} (0x{h:03X})"));
                    ui.separator();
                }
                if self.editor.has_unsaved() {
                    ui.colored_label(Color32::from_rgb(220, 160, 40), "unsaved");
                    ui.separator();
                }
                ui.label(&self.status);
            });
        });
    }

    fn palette(&mut self, ctx: &egui::Context) {
        let sheet = &self.sheet;
        let tex = self.sheet_tex.get_or_insert_with(|| {
            let img = ColorImage::from_rgb([sheet.width as usize, sheet.height as usize], &sheet.rgb);
            ctx.load_texture("tile-sheet", img, TextureOptions::NEAREST)
        });
        let tex_id = tex.id();
        let size = Vec2::new(self.sheet.width as f32, self.sheet.height as f32);
        let mut picked = None;
        egui::TopBottomPanel::bottom("palette").resizable(true).default_height(300.0).show(ctx, |ui| {
            ui.label("Tile palette (left click selects; right click on the board picks the tile under the mouse)");
            egui::ScrollArea::both().id_salt("palette-scroll").show(ui, |ui| {
                let (rect, resp) = ui.allocate_exact_size(size, Sense::click());
                let painter = ui.painter_at(rect);
                painter.image(tex_id, rect, Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)), Color32::WHITE);
                let (c, r) = sheet_cell(self.editor.tile);
                let tp = TILE_PX as f32;
                let sel = Rect::from_min_size(rect.min + Vec2::new(c as f32 * tp, r as f32 * tp), Vec2::splat(tp));
                painter.rect_stroke(sel, 0.0, Stroke::new(2.0_f32, Color32::YELLOW));
                if resp.clicked() {
                    if let Some(pos) = resp.interact_pointer_pos() {
                        let d = pos - rect.min;
                        picked = sheet_tile((d.x / tp) as u32, (d.y / tp) as u32);
                    }
                }
                self.palette_rect = Some(rect);
            });
        });
        if let Some(id) = picked {
            self.editor.tile = id;
        }
    }

    fn board(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let Some(doc) = self.editor.doc() else {
                ui.centered_and_justified(|ui| ui.label("No level open."));
                self.board_rect = None;
                return;
            };
            if self.board_tex.is_none() || self.board_tex_rev != self.editor.board_rev {
                let fb = board_image(doc, &self.sheet);
                let img = ColorImage::from_rgb([fb.width as usize, fb.height as usize], &fb.pixels);
                match &mut self.board_tex {
                    Some(t) => t.set(img, TextureOptions::NEAREST),
                    None => self.board_tex = Some(ctx.load_texture("board", img, TextureOptions::NEAREST)),
                }
                self.board_tex_rev = self.editor.board_rev;
            }
            let tex_id = self.board_tex.as_ref().expect("board texture").id();
            let cell = TILE_PX as f32 * self.zoom;
            let size = Vec2::splat(cell * SIZE as f32);
            egui::ScrollArea::both().id_salt("board-scroll").show(ui, |ui| {
                let (rect, resp) = ui.allocate_exact_size(size, Sense::click_and_drag());
                let painter = ui.painter_at(rect);
                painter.image(tex_id, rect, Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)), Color32::WHITE);
                if self.grid {
                    let s = Stroke::new(1.0_f32, Color32::from_black_alpha(70));
                    for i in 1..SIZE {
                        let o = i as f32 * cell;
                        painter.vline(rect.min.x + o, rect.y_range(), s);
                        painter.hline(rect.x_range(), rect.min.y + o, s);
                    }
                }
                let to_cell = |p: Pos2| -> Option<(usize, usize)> {
                    let d = p - rect.min;
                    (d.x >= 0.0 && d.y >= 0.0 && d.x < size.x && d.y < size.y).then(|| ((d.x / cell) as usize, (d.y / cell) as usize))
                };
                self.hover = resp.hover_pos().and_then(to_cell);
                if let Some((x, y)) = self.hover {
                    let r = Rect::from_min_size(rect.min + Vec2::new(x as f32 * cell, y as f32 * cell), Vec2::splat(cell));
                    painter.rect_stroke(r, 0.0, Stroke::new(1.0_f32, Color32::WHITE));
                }

                let primary = resp.is_pointer_button_down_on() && ui.input(|i| i.pointer.primary_down());
                if primary {
                    if let Some(c) = resp.interact_pointer_pos().and_then(to_cell) {
                        let cells = match self.stroke_last {
                            Some(last) => line_cells(last, c),
                            None => vec![c],
                        };
                        for (x, y) in cells {
                            if let Err(e) = self.editor.paint_at(x, y) {
                                self.status = e.to_string();
                            }
                        }
                        self.stroke_last = Some(c);
                    }
                } else {
                    self.stroke_last = None;
                }
                if resp.secondary_clicked() {
                    if let Some((x, y)) = resp.interact_pointer_pos().and_then(to_cell) {
                        self.editor.pick_at(x, y);
                    }
                }
                self.board_rect = Some(rect);
            });
        });
    }

    fn prompt(&mut self, ctx: &egui::Context) {
        let Some(pending) = self.pending.clone() else { return };
        let mut choice = None;
        egui::Window::new("Unsaved changes")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
            .show(ctx, |ui| {
                let name = self.editor.path().map(|p| p.display().to_string()).unwrap_or_default();
                ui.label(format!("{name} has changes that are not saved."));
                ui.horizontal(|ui| {
                    if ui.button("Save").clicked() {
                        choice = Some(0);
                    }
                    if ui.button("Discard").clicked() {
                        choice = Some(1);
                    }
                    if ui.button("Cancel").clicked() {
                        choice = Some(2);
                    }
                });
            });
        let go = match choice {
            Some(0) => self.do_save(None),
            Some(1) => true,
            Some(_) => {
                self.pending = None;
                return;
            }
            None => return,
        };
        if !go {
            self.pending = None;
            return;
        }
        self.pending = None;
        match pending {
            Pending::Open(p) => self.do_open(p),
            Pending::Close => {
                self.closing = true;
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }
    }
}

impl eframe::App for EditorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.ui(ctx);
    }
}

/// Open the window. Returns when it is closed.
pub fn run(version: Version, sheet: Sheet, file: Option<&Path>) -> Result<(), String> {
    let app = EditorApp::new(version, sheet, file);
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1100.0, 860.0]),
        ..Default::default()
    };
    eframe::run_native("Seihai level editor", options, Box::new(move |_cc| Ok(Box::new(app)))).map_err(|e| e.to_string())
}
