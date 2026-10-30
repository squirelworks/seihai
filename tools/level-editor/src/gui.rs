//! The level editor window: a tile palette, painting on layer 0 or 1, stamps
//! from a `GOBJSET01` object set, link / sign / chest / NPC markers, a
//! `.gmap` grid that opens its levels, and save. It sits on the
//! command-line slice (`Document`): painting is `Document::paint_layer`,
//! objects go through `Document::edit_object` and friends, and saving is
//! `Document::save`, so the file rules (byte-for-byte round trip, canonical
//! rows after a change, write through a temporary file) are the same as the
//! CLI's.
//!
//! The editor state (`Editor`) knows nothing about egui and is tested on its
//! own; `EditorApp` draws it. `EditorApp::ui` takes a bare `egui::Context`, so
//! the tests also drive the real window code with synthetic mouse input.

use crate::{Document, EditError, Object, ObjectEdit, ObjectKind, ScriptHead, Version, SCRIPT_PANE_LINES, LAYERS, SIZE, TILE_LIMIT};
use eframe::egui;
use egui::{Align2, Color32, ColorImage, FontId, Pos2, Rect, Sense, Stroke, TextureHandle, TextureOptions, Vec2};
use seihai::assets::{read_png_rgb, tile_id, ObjectSet, Stamp};
use seihai::gmap::Gmap;
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

/// What the left button does on the board.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tool {
    /// Paint the selected tile on the active layer.
    Paint,
    /// Empty cells of layer 1.
    Erase,
    /// Drop the selected stamp with its top-left corner on the cell.
    Stamp,
    /// Select and drag links, signs, chests and NPCs.
    Objects,
}

/// An opened `.gmap`: its grid and the folder its levels are in.
#[derive(Debug, Clone)]
pub struct GmapView {
    pub path: PathBuf,
    pub map: Gmap,
}

impl GmapView {
    /// The file of the level at cell (x, y), next to the `.gmap`.
    pub fn level_path(&self, x: usize, y: usize) -> Option<PathBuf> {
        let name = self.map.level_at(x as i32, y as i32)?;
        Some(self.path.parent().unwrap_or(Path::new(".")).join(name))
    }
}

/// The editor without its window: one open level, the active layer and
/// tool, the selected tile, stamp and object, an optional gmap, the version,
/// and whether there are unsaved changes.
pub struct Editor {
    pub version: Version,
    /// The tile the left button paints.
    pub tile: u16,
    /// The layer painting, erasing and stamps go to (0 or 1).
    pub layer: usize,
    /// Which layers are drawn.
    pub show: [bool; LAYERS],
    pub tool: Tool,
    /// The loaded object set, and the stamp picked from it.
    pub stamps: Option<(PathBuf, ObjectSet)>,
    pub stamp: Option<usize>,
    /// The selected object (an id of `Document::objects`).
    pub selected: Option<usize>,
    pub gmap: Option<GmapView>,
    doc: Option<Document>,
    path: Option<PathBuf>,
    unsaved: bool,
    /// Bumped on every change of the board, so the window knows when to redraw it.
    pub board_rev: u64,
}

fn no_level() -> EditError {
    EditError::Io("no level is open".into())
}

impl Editor {
    pub fn new(version: Version) -> Editor {
        Editor {
            version,
            tile: 0,
            layer: 0,
            show: [true; LAYERS],
            tool: Tool::Paint,
            stamps: None,
            stamp: None,
            selected: None,
            gmap: None,
            doc: None,
            path: None,
            unsaved: false,
            board_rev: 0,
        }
    }

    pub fn doc(&self) -> Option<&Document> {
        self.doc.as_ref()
    }

    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    /// Painted or edited since the last open or save.
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
        self.selected = None;
        self.board_rev += 1;
        Ok(())
    }

    /// Open a `.gmap` (read with the client's parser). The open level stays.
    pub fn open_gmap(&mut self, path: &Path) -> Result<(), EditError> {
        let text = std::fs::read_to_string(path).map_err(|e| EditError::Io(format!("cannot read {}: {e}", path.display())))?;
        let map = Gmap::parse(&text).ok_or_else(|| EditError::NotText(format!("{} is not a GRMAP001 gmap with at least one cell", path.display())))?;
        self.gmap = Some(GmapView { path: path.to_path_buf(), map });
        Ok(())
    }

    /// Load a `GOBJSET01` object set (read with the client's parser).
    pub fn load_stamps(&mut self, path: &Path) -> Result<usize, EditError> {
        let set = ObjectSet::load(path).map_err(|e| EditError::NotText(format!("cannot read the object set {}: {e}", path.display())))?;
        let n = set.stamps.len();
        self.stamp = (n > 0).then_some(0);
        self.stamps = Some((path.to_path_buf(), set));
        Ok(n)
    }

    /// The picked stamp.
    pub fn current_stamp(&self) -> Option<&Stamp> {
        self.stamps.as_ref()?.1.stamps.get(self.stamp?)
    }

    fn changed(&mut self, n: usize) -> usize {
        if n > 0 {
            self.unsaved = true;
            self.board_rev += 1;
        }
        n
    }

    /// Paint the selected tile at board cell (x, y) on the active layer.
    /// Returns the number of tiles that changed (0 or 1).
    pub fn paint_at(&mut self, x: usize, y: usize) -> Result<usize, EditError> {
        let doc = self.doc.as_mut().ok_or_else(no_level)?;
        let n = doc.paint_layer(self.layer, x, y, 1, 1, Some(self.tile as u32))?;
        Ok(self.changed(n))
    }

    /// Empty board cell (x, y) of layer 1.
    pub fn erase_at(&mut self, x: usize, y: usize) -> Result<usize, EditError> {
        if self.layer != 1 {
            return Err(EditError::Bounds("only layer 1 has empty cells; switch to layer 1 to erase".into()));
        }
        let doc = self.doc.as_mut().ok_or_else(no_level)?;
        let n = doc.paint_layer(1, x, y, 1, 1, None)?;
        Ok(self.changed(n))
    }

    /// Drop the picked stamp with its top-left corner at (x, y) on the active
    /// layer. Cells off the board are left out. Returns the tiles changed.
    pub fn place_stamp(&mut self, x: usize, y: usize) -> Result<usize, EditError> {
        let stamp = self.current_stamp().cloned().ok_or_else(|| EditError::Io("no stamp is picked; load an object set first".into()))?;
        let layer = self.layer;
        let doc = self.doc.as_mut().ok_or_else(no_level)?;
        let mut n = 0;
        for sy in 0..stamp.height {
            for sx in 0..stamp.width {
                let (bx, by) = (x + sx, y + sy);
                if bx >= SIZE || by >= SIZE {
                    continue;
                }
                if let Some(t) = stamp.tile_at(sx, sy) {
                    n += doc.paint_layer(layer, bx, by, 1, 1, Some(tile_id(t) as u32))?;
                }
            }
        }
        Ok(self.changed(n))
    }

    /// Select the tile under board cell (x, y) on the active layer (the right
    /// button). An empty layer-1 cell selects nothing.
    pub fn pick_at(&mut self, x: usize, y: usize) -> Option<u16> {
        let t = self.doc.as_ref()?.layer_tile(self.layer, x, y)?;
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

    pub fn objects(&self) -> Vec<Object> {
        self.doc.as_ref().map(|d| d.objects()).unwrap_or_default()
    }

    /// The topmost object whose marker covers the point (in tiles).
    pub fn object_at(&self, x: f32, y: f32) -> Option<Object> {
        self.objects().into_iter().rev().find(|o| x >= o.x && y >= o.y && x < o.x + o.w && y < o.y + o.h)
    }

    pub fn edit_object(&mut self, id: usize, edit: ObjectEdit) -> Result<(), EditError> {
        self.doc.as_mut().ok_or_else(no_level)?.edit_object(id, edit)?;
        self.unsaved = true;
        Ok(())
    }

    /// The first script lines of an NPC, for the script pane.
    pub fn script_head(&self, id: usize) -> Result<ScriptHead, EditError> {
        self.doc.as_ref().ok_or_else(no_level)?.script_head(id)
    }

    /// Add a link, sign, chest or NPC at (x, y) and select it.
    pub fn add_object(&mut self, kind: ObjectKind, x: i32, y: i32) -> Result<usize, EditError> {
        let id = self.doc.as_mut().ok_or_else(no_level)?.add_object(kind, x, y)?;
        self.unsaved = true;
        self.selected = Some(id);
        Ok(id)
    }

    pub fn delete_object(&mut self, id: usize) -> Result<(), EditError> {
        self.doc.as_mut().ok_or_else(no_level)?.delete_object(id)?;
        self.unsaved = true;
        self.selected = None;
        Ok(())
    }

    /// Save to the file that was opened.
    pub fn save(&mut self) -> Result<PathBuf, EditError> {
        let path = self.path.clone().ok_or_else(no_level)?;
        self.save_as(&path)
    }

    /// Save to `path`, which becomes the file that Save writes from now on.
    pub fn save_as(&mut self, path: &Path) -> Result<PathBuf, EditError> {
        let doc = self.doc.as_ref().ok_or_else(no_level)?;
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

/// The 64 x 64 board drawn with `sheet`, 16 px per tile: layer 0, then the
/// filled cells of layer 1 over it, each when `show` says so. A hidden
/// layer 0 is drawn dark grey.
pub fn board_image(doc: &Document, sheet: &Sheet, show: [bool; LAYERS]) -> Framebuffer {
    let px = SIZE as u32 * TILE_PX;
    let mut fb = Framebuffer::new(px, px);
    if !show[0] {
        fb.fill_rect(0, 0, px as i32, px as i32, [40, 40, 40]);
    }
    for layer in 0..LAYERS {
        if !show[layer] {
            continue;
        }
        for y in 0..SIZE {
            for x in 0..SIZE {
                if let Some(t) = doc.layer_tile(layer, x, y) {
                    fb.blit_tile(&sheet.rgb, sheet.width, sheet.height, t, (x as u32 * TILE_PX) as i32, (y as u32 * TILE_PX) as i32);
                }
            }
        }
    }
    fb
}

/// The marker colour of each kind.
fn kind_color(kind: ObjectKind) -> Color32 {
    match kind {
        ObjectKind::Link => Color32::from_rgb(60, 200, 255),
        ObjectKind::Sign => Color32::from_rgb(255, 230, 60),
        ObjectKind::Chest => Color32::from_rgb(255, 150, 40),
        ObjectKind::Npc => Color32::from_rgb(230, 80, 230),
        ObjectKind::Baddy => Color32::from_rgb(230, 60, 60),
    }
}

/// A question the window is waiting on before it drops unsaved changes.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Pending {
    Open(PathBuf),
    Close,
}

/// An object being dragged: its id, where it was, and where the pointer
/// grabbed it (both in tiles).
#[derive(Debug, Clone, Copy)]
struct Drag {
    id: usize,
    from: (f32, f32),
    grab: (f32, f32),
    now: (f32, f32),
}

/// The text boxes of the selected object's editor.
#[derive(Debug, Clone, Default)]
struct ObjectFields {
    id: Option<usize>,
    dest: String,
    arrival_x: String,
    arrival_y: String,
    first_line: String,
    x: f32,
    y: f32,
    w: i32,
    h: i32,
}

/// The NPC script pane: which NPC, the text being edited, and how many
/// script lines there are past the ones shown (they are kept on save).
#[derive(Debug, Clone)]
pub struct ScriptPane {
    pub id: usize,
    pub text: String,
    pub shown: usize,
    pub total: usize,
}

/// The window.
pub struct EditorApp {
    pub editor: Editor,
    sheet: Sheet,
    sheet_tex: Option<TextureHandle>,
    board_tex: Option<TextureHandle>,
    board_tex_key: (u64, [bool; LAYERS]),
    /// The text boxes for Open, Save as and the object set.
    open_path: String,
    save_as_path: String,
    stamps_path: String,
    zoom: f32,
    grid: bool,
    /// Show the gmap grid instead of the board.
    pub map_view: bool,
    status: String,
    hover: Option<(usize, usize)>,
    /// The last cell painted in the current drag.
    stroke_last: Option<(usize, usize)>,
    drag: Option<Drag>,
    /// The cell new objects go to (the last empty cell clicked with the Objects tool).
    add_cell: (i32, i32),
    fields: ObjectFields,
    /// The open NPC script pane (double-click an NPC marker).
    pub script: Option<ScriptPane>,
    pending: Option<Pending>,
    title: String,
    /// Where things were drawn in the last frame (for tests).
    pub board_rect: Option<Rect>,
    pub palette_rect: Option<Rect>,
    pub gmap_cells: Vec<((usize, usize), Rect)>,
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
            board_tex_key: (0, [true; LAYERS]),
            open_path: String::new(),
            save_as_path: String::new(),
            stamps_path: String::new(),
            zoom: 1.0,
            grid: false,
            map_view: false,
            status: String::new(),
            hover: None,
            stroke_last: None,
            drag: None,
            add_cell: (30, 30),
            fields: ObjectFields::default(),
            script: None,
            pending: None,
            title: String::new(),
            board_rect: None,
            palette_rect: None,
            gmap_cells: Vec::new(),
            closing: false,
        };
        if let Some(p) = file {
            app.open_path = p.display().to_string();
            app.request_open(p.to_path_buf());
        } else {
            app.status = "Type a .nw or .gmap path and press Open.".into();
        }
        app
    }

    fn is_gmap(p: &Path) -> bool {
        p.extension().map(|e| e.eq_ignore_ascii_case("gmap")).unwrap_or(false)
    }

    /// Open a level (asking first about unsaved changes) or a gmap.
    fn request_open(&mut self, p: PathBuf) {
        if Self::is_gmap(&p) {
            match self.editor.open_gmap(&p) {
                Ok(()) => {
                    self.map_view = true;
                    self.status = format!("Opened the gmap {}; click a cell to open its level", p.display());
                }
                Err(e) => self.status = e.to_string(),
            }
        } else if self.editor.has_unsaved() {
            self.pending = Some(Pending::Open(p));
        } else {
            self.do_open(p);
        }
    }

    fn do_open(&mut self, path: PathBuf) {
        match self.editor.open(&path) {
            Ok(()) => {
                self.save_as_path = path.display().to_string();
                self.open_path = path.display().to_string();
                self.status = format!("Opened {}", path.display());
                self.map_view = false;
                self.fields = ObjectFields::default();
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

    fn report(&mut self, r: Result<(), EditError>) {
        if let Err(e) = r {
            self.status = e.to_string();
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
        self.side_panel(ctx);
        if self.map_view && self.editor.gmap.is_some() {
            self.gmap_grid(ctx);
        } else {
            self.board(ctx);
        }
        self.script_window(ctx);
        self.prompt(ctx);
    }

    /// Open the script pane for an NPC (what a double-click on its marker does).
    pub fn open_script(&mut self, id: usize) {
        match self.editor.script_head(id) {
            Ok(head) => {
                self.editor.selected = Some(id);
                self.script = Some(ScriptPane { id, text: head.lines.join("\n"), shown: head.lines.len(), total: head.total });
            }
            Err(e) => self.status = e.to_string(),
        }
    }

    /// Put the pane's text into the level in place of the lines it showed.
    /// Returns true when the level took it.
    pub fn apply_script(&mut self) -> bool {
        let Some(pane) = self.script.clone() else { return false };
        let lines: Vec<String> = if pane.text.is_empty() { Vec::new() } else { pane.text.split('\n').map(|l| l.trim_end_matches('\r').to_string()).collect() };
        match self.editor.edit_object(pane.id, ObjectEdit::ScriptHead(lines)) {
            Ok(()) => {
                self.status = format!("Changed the script of NPC {}", pane.id);
                self.script = None;
                true
            }
            Err(e) => {
                self.status = e.to_string();
                false
            }
        }
    }

    fn script_window(&mut self, ctx: &egui::Context) {
        let Some(pane) = self.script.as_mut() else { return };
        let mut open = true;
        let mut cancel = false;
        let mut action = None;
        let count = if pane.text.is_empty() { 0 } else { pane.text.split('\n').count() };
        egui::Window::new(format!("NPC {} script", pane.id))
            .open(&mut open)
            .collapsible(false)
            .default_width(520.0)
            .show(ctx, |ui| {
                let rest = pane.total - pane.shown;
                ui.label(if rest > 0 {
                    format!("Lines 1 to {} of {}. The {rest} lines after them are kept as they are.", pane.shown, pane.total)
                } else {
                    format!("The whole script ({} lines). The pane edits up to {SCRIPT_PANE_LINES}.", pane.total)
                });
                ui.add(egui::TextEdit::multiline(&mut pane.text).code_editor().desired_rows(SCRIPT_PANE_LINES).desired_width(f32::INFINITY));
                if count > SCRIPT_PANE_LINES {
                    ui.colored_label(Color32::LIGHT_RED, format!("{count} lines: the pane saves {SCRIPT_PANE_LINES} at most."));
                }
                ui.horizontal(|ui| {
                    if ui.add_enabled(count <= SCRIPT_PANE_LINES, egui::Button::new("Apply")).on_hover_text("Into the level; save the level to write it").clicked() {
                        action = Some(false);
                    }
                    if ui.add_enabled(count <= SCRIPT_PANE_LINES, egui::Button::new("Apply and save level")).clicked() {
                        action = Some(true);
                    }
                    if ui.button("Cancel").clicked() {
                        cancel = true;
                    }
                });
            });
        match action {
            Some(save) => {
                if self.apply_script() && save {
                    self.do_save(None);
                }
            }
            None if !open || cancel => self.script = None,
            None => {}
        }
    }

    fn toolbar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Open");
                let field = ui.add(egui::TextEdit::singleline(&mut self.open_path).desired_width(260.0).hint_text("level.nw or world.gmap"));
                let enter = field.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                if (ui.button("Open").clicked() || enter) && !self.open_path.trim().is_empty() {
                    self.request_open(PathBuf::from(self.open_path.trim()));
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
            });
            ui.horizontal(|ui| {
                ui.label("Layer");
                for l in 0..LAYERS {
                    ui.radio_value(&mut self.editor.layer, l, l.to_string())
                        .on_hover_text("The layer painting, erasing and stamps go to. The 2.22 client does not read the layer: it draws layer 1 tiles over layer 0 on one board.");
                }
                ui.label("Show");
                for l in 0..LAYERS {
                    ui.checkbox(&mut self.editor.show[l], l.to_string());
                }
                ui.separator();
                ui.label("Tool");
                ui.selectable_value(&mut self.editor.tool, Tool::Paint, "Paint");
                ui.add_enabled_ui(self.editor.layer == 1, |ui| {
                    ui.selectable_value(&mut self.editor.tool, Tool::Erase, "Erase").on_disabled_hover_text("Layer 0 has no empty cells");
                });
                ui.selectable_value(&mut self.editor.tool, Tool::Stamp, "Stamp");
                ui.selectable_value(&mut self.editor.tool, Tool::Objects, "Objects");
                if self.editor.layer == 0 && self.editor.tool == Tool::Erase {
                    self.editor.tool = Tool::Paint;
                }
                ui.separator();
                ui.label("Zoom");
                for z in [1.0, 2.0] {
                    ui.selectable_value(&mut self.zoom, z, format!("{z}x"));
                }
                ui.checkbox(&mut self.grid, "Grid");
                if self.editor.gmap.is_some() {
                    ui.separator();
                    ui.toggle_value(&mut self.map_view, "Gmap");
                }
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
                    let l0 = doc.layer_tile(0, x, y).unwrap_or(0);
                    let l1 = doc.layer_tile(1, x, y).map(|t| t.to_string()).unwrap_or_else(|| "empty".into());
                    ui.label(format!("x {x}, y {y}: layer 0 tile {l0}, layer 1 {l1}"));
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
        egui::TopBottomPanel::bottom("palette").resizable(true).default_height(260.0).show(ctx, |ui| {
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
            if self.editor.tool == Tool::Erase {
                self.editor.tool = Tool::Paint;
            }
        }
    }

    fn side_panel(&mut self, ctx: &egui::Context) {
        if !matches!(self.editor.tool, Tool::Stamp | Tool::Objects) {
            return;
        }
        egui::SidePanel::right("side").default_width(280.0).show(ctx, |ui| {
            egui::ScrollArea::vertical().id_salt("side-scroll").show(ui, |ui| match self.editor.tool {
                Tool::Stamp => self.stamp_panel(ui),
                _ => self.object_panel(ui),
            });
        });
    }

    fn stamp_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading("Stamps");
        ui.label("Object set (GOBJSET01)");
        ui.horizontal(|ui| {
            ui.add(egui::TextEdit::singleline(&mut self.stamps_path).desired_width(180.0).hint_text("objects.txt"));
            if ui.button("Load").clicked() && !self.stamps_path.trim().is_empty() {
                match self.editor.load_stamps(Path::new(self.stamps_path.trim())) {
                    Ok(n) => self.status = format!("Loaded {n} stamps"),
                    Err(e) => self.status = e.to_string(),
                }
            }
        });
        let Some((_, set)) = &self.editor.stamps else {
            ui.label("No object set loaded.");
            return;
        };
        let names: Vec<String> = set.stamps.iter().map(|s| format!("{} ({} x {})", s.name, s.width, s.height)).collect();
        for (i, name) in names.into_iter().enumerate() {
            if ui.selectable_label(self.editor.stamp == Some(i), name).clicked() {
                self.editor.stamp = Some(i);
            }
        }
        ui.label(format!("Click the board to drop the stamp on layer {}.", self.editor.layer));
    }

    fn object_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading("Objects");
        if self.editor.doc().is_none() {
            ui.label("No level open.");
            return;
        }
        let (ax, ay) = self.add_cell;
        ui.label(format!("New at ({ax}, {ay}), the last empty cell clicked:"));
        ui.horizontal_wrapped(|ui| {
            for kind in ObjectKind::EDITABLE {
                if ui.button(format!("+ {}", kind.name())).clicked() {
                    let r = self.editor.add_object(kind, ax, ay).map(|_| ());
                    self.report(r);
                }
            }
        });
        ui.separator();
        let objects = self.editor.objects();
        for o in &objects {
            let label = match o.kind {
                ObjectKind::Link => format!("link {} at {}, {} to {}", o.id, o.x, o.y, o.name),
                ObjectKind::Npc => format!("NPC {} at {}, {} {}", o.id, o.x, o.y, o.name),
                k => format!("{} {} at {}, {} {}", k.name(), o.id, o.x, o.y, o.name),
            };
            let text = egui::RichText::new(label).color(kind_color(o.kind));
            if ui.selectable_label(self.editor.selected == Some(o.id), text).clicked() {
                self.editor.selected = Some(o.id);
            }
        }
        ui.separator();
        let Some(o) = self.editor.selected.and_then(|id| objects.get(id).cloned()) else {
            ui.label("Click a marker or a row to select it; drag a marker to move it.");
            return;
        };
        if self.fields.id != Some(o.id) || self.drag.is_none() && (self.fields.x != o.x || self.fields.y != o.y) {
            self.fields = ObjectFields {
                id: Some(o.id),
                dest: o.name.clone(),
                arrival_x: o.arrival.0.clone(),
                arrival_y: o.arrival.1.clone(),
                first_line: o.first_line.clone(),
                x: o.x,
                y: o.y,
                w: o.w as i32,
                h: o.h as i32,
            };
        }
        ui.strong(format!("{} {}", o.kind.name(), o.id));
        if o.unreadable {
            ui.label("The client's parser could not read this record; it is kept as it is.");
            return;
        }
        if o.kind == ObjectKind::Baddy {
            ui.label("Baddies are shown, not edited, in this slice.");
            return;
        }
        let mut edits: Vec<ObjectEdit> = Vec::new();
        ui.horizontal(|ui| {
            ui.label("x");
            ui.add(egui::DragValue::new(&mut self.fields.x).speed(0.5));
            ui.label("y");
            ui.add(egui::DragValue::new(&mut self.fields.y).speed(0.5));
            if ui.button("Move").clicked() {
                edits.push(ObjectEdit::Move(self.fields.x, self.fields.y));
            }
        });
        match o.kind {
            ObjectKind::Link => {
                ui.horizontal(|ui| {
                    ui.label("w");
                    ui.add(egui::DragValue::new(&mut self.fields.w).range(1..=64));
                    ui.label("h");
                    ui.add(egui::DragValue::new(&mut self.fields.h).range(1..=64));
                    if ui.button("Resize").clicked() {
                        edits.push(ObjectEdit::LinkSize(self.fields.w, self.fields.h));
                    }
                });
                ui.horizontal(|ui| {
                    ui.label("To");
                    ui.text_edit_singleline(&mut self.fields.dest);
                });
                ui.horizontal(|ui| {
                    ui.label("Arrive at");
                    ui.add(egui::TextEdit::singleline(&mut self.fields.arrival_x).desired_width(80.0));
                    ui.add(egui::TextEdit::singleline(&mut self.fields.arrival_y).desired_width(80.0));
                });
                if ui.button("Apply destination and arrival").clicked() {
                    edits.push(ObjectEdit::LinkDest(self.fields.dest.clone()));
                    edits.push(ObjectEdit::LinkArrival(self.fields.arrival_x.clone(), self.fields.arrival_y.clone()));
                }
            }
            ObjectKind::Npc | ObjectKind::Sign => {
                if o.kind == ObjectKind::Npc {
                    ui.label(format!("Image: {}", o.name));
                }
                ui.label(if o.kind == ObjectKind::Npc { "First script line" } else { "First text line" });
                ui.text_edit_singleline(&mut self.fields.first_line);
                if ui.button("Apply line").clicked() {
                    edits.push(ObjectEdit::FirstLine(self.fields.first_line.clone()));
                }
                if o.kind == ObjectKind::Npc && ui.button("Edit script (20 lines)").on_hover_text("Or double-click the marker").clicked() {
                    self.open_script(o.id);
                }
            }
            ObjectKind::Chest => {
                ui.label(format!("Item: {}", o.name));
            }
            ObjectKind::Baddy => {}
        }
        for e in edits {
            let r = self.editor.edit_object(o.id, e);
            if r.is_err() {
                self.report(r);
                break;
            }
            self.status = format!("Changed {} {}", o.kind.name(), o.id);
        }
        ui.separator();
        if ui.button("Delete").clicked() {
            let r = self.editor.delete_object(o.id);
            self.report(r);
        }
    }

    fn gmap_grid(&mut self, ctx: &egui::Context) {
        let Some(view) = self.editor.gmap.clone() else { return };
        let current = self.editor.path().map(Path::to_path_buf);
        let mut open = None;
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label(format!("{} ({} x {} levels). Click a cell to open its level.", view.path.display(), view.map.width, view.map.height));
            self.gmap_cells.clear();
            egui::ScrollArea::both().id_salt("gmap-scroll").show(ui, |ui| {
                egui::Grid::new("gmap-grid").spacing([2.0, 2.0]).show(ui, |ui| {
                    for y in 0..view.map.height {
                        for x in 0..view.map.width {
                            let name = view.map.level_at(x as i32, y as i32).unwrap_or("").to_string();
                            let file = view.level_path(x, y);
                            let exists = file.as_ref().map(|f| f.is_file()).unwrap_or(false);
                            let is_open = file.is_some() && file == current;
                            let mut text = egui::RichText::new(if name.is_empty() { "(hole)".into() } else { name.clone() });
                            if !exists {
                                text = text.color(Color32::GRAY);
                            }
                            if is_open {
                                text = text.strong().color(Color32::YELLOW);
                            }
                            let resp = ui.add_enabled(exists, egui::Button::new(text).min_size(Vec2::new(120.0, 36.0)));
                            let resp = if exists { resp } else { resp.on_disabled_hover_text("no such file next to the gmap") };
                            self.gmap_cells.push(((x, y), resp.rect));
                            if resp.clicked() {
                                open = file;
                            }
                        }
                        ui.end_row();
                    }
                });
            });
        });
        if let Some(p) = open {
            self.open_path = p.display().to_string();
            self.request_open(p);
        }
    }

    fn board(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let Some(doc) = self.editor.doc() else {
                ui.centered_and_justified(|ui| ui.label("No level open."));
                self.board_rect = None;
                return;
            };
            let key = (self.editor.board_rev, self.editor.show);
            if self.board_tex.is_none() || self.board_tex_key != key {
                let fb = board_image(doc, &self.sheet, self.editor.show);
                let img = ColorImage::from_rgb([fb.width as usize, fb.height as usize], &fb.pixels);
                match &mut self.board_tex {
                    Some(t) => t.set(img, TextureOptions::NEAREST),
                    None => self.board_tex = Some(ctx.load_texture("board", img, TextureOptions::NEAREST)),
                }
                self.board_tex_key = key;
            }
            let tex_id = self.board_tex.as_ref().expect("board texture").id();
            let sheet_tex = self.sheet_tex.as_ref().map(|t| t.id());
            let cell = TILE_PX as f32 * self.zoom;
            let size = Vec2::splat(cell * SIZE as f32);
            let objects = self.editor.objects();
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
                let to_tiles = |p: Pos2| -> Option<(f32, f32)> {
                    let d = p - rect.min;
                    (d.x >= 0.0 && d.y >= 0.0 && d.x < size.x && d.y < size.y).then(|| (d.x / cell, d.y / cell))
                };
                let to_cell = |p: Pos2| to_tiles(p).map(|(x, y)| (x as usize, y as usize));
                let tile_rect = |x: f32, y: f32, w: f32, h: f32| Rect::from_min_size(rect.min + Vec2::new(x * cell, y * cell), Vec2::new(w * cell, h * cell));

                // markers
                for o in &objects {
                    let (mut x, mut y) = (o.x, o.y);
                    if let Some(d) = self.drag.filter(|d| d.id == o.id) {
                        (x, y) = d.now;
                    }
                    let r = tile_rect(x, y, o.w, o.h);
                    let c = kind_color(o.kind);
                    let selected = self.editor.selected == Some(o.id);
                    painter.rect_filled(r, 0.0, c.gamma_multiply(0.18));
                    painter.rect_stroke(r, 0.0, Stroke::new(if selected { 3.0_f32 } else { 1.5_f32 }, if selected { Color32::WHITE } else { c }));
                    let label = match o.kind {
                        ObjectKind::Link => o.name.clone(),
                        ObjectKind::Npc => "NPC".to_string(),
                        ObjectKind::Sign => "sign".to_string(),
                        ObjectKind::Chest => "chest".to_string(),
                        ObjectKind::Baddy => "baddy".to_string(),
                    };
                    painter.text(r.left_top() + Vec2::new(2.0, 1.0), Align2::LEFT_TOP, label, FontId::proportional(10.0), Color32::WHITE);
                }

                self.hover = resp.hover_pos().and_then(to_cell);
                if let Some((x, y)) = self.hover {
                    match (self.editor.tool, self.editor.current_stamp(), sheet_tex) {
                        (Tool::Stamp, Some(stamp), Some(st)) => {
                            // the stamp under the mouse, see-through
                            let tint = Color32::from_rgba_unmultiplied(255, 255, 255, 170);
                            for sy in 0..stamp.height {
                                for sx in 0..stamp.width {
                                    let Some(t) = stamp.tile_at(sx, sy) else { continue };
                                    if x + sx >= SIZE || y + sy >= SIZE {
                                        continue;
                                    }
                                    let (c, r) = sheet_cell(tile_id(t));
                                    let uv = Rect::from_min_size(
                                        Pos2::new(c as f32 / SHEET_COLS as f32, r as f32 / SHEET_ROWS as f32),
                                        Vec2::new(1.0 / SHEET_COLS as f32, 1.0 / SHEET_ROWS as f32),
                                    );
                                    painter.image(st, tile_rect((x + sx) as f32, (y + sy) as f32, 1.0, 1.0), uv, tint);
                                }
                            }
                            painter.rect_stroke(tile_rect(x as f32, y as f32, stamp.width as f32, stamp.height as f32), 0.0, Stroke::new(1.0_f32, Color32::WHITE));
                        }
                        _ => {
                            painter.rect_stroke(tile_rect(x as f32, y as f32, 1.0, 1.0), 0.0, Stroke::new(1.0_f32, Color32::WHITE));
                        }
                    }
                }

                let (pressed, down, released) = ui.input(|i| (i.pointer.primary_pressed(), i.pointer.primary_down(), i.pointer.primary_released()));
                let pointer = ui.input(|i| i.pointer.interact_pos().or(i.pointer.hover_pos()));
                match self.editor.tool {
                    Tool::Paint | Tool::Erase => {
                        if resp.is_pointer_button_down_on() && down {
                            if let Some(c) = resp.interact_pointer_pos().and_then(to_cell) {
                                let cells = match self.stroke_last {
                                    Some(last) => line_cells(last, c),
                                    None => vec![c],
                                };
                                for (x, y) in cells {
                                    let r = if self.editor.tool == Tool::Erase { self.editor.erase_at(x, y) } else { self.editor.paint_at(x, y) };
                                    if let Err(e) = r {
                                        self.status = e.to_string();
                                    }
                                }
                                self.stroke_last = Some(c);
                            }
                        } else {
                            self.stroke_last = None;
                        }
                    }
                    Tool::Stamp => {
                        if resp.clicked() {
                            if let Some((x, y)) = resp.interact_pointer_pos().and_then(to_cell) {
                                match self.editor.place_stamp(x, y) {
                                    Ok(n) => self.status = format!("Stamp placed at ({x}, {y}) on layer {}: {n} tiles changed", self.editor.layer),
                                    Err(e) => self.status = e.to_string(),
                                }
                            }
                        }
                    }
                    Tool::Objects => {
                        let at = pointer.and_then(to_tiles);
                        if resp.double_clicked() {
                            if let Some(o) = at.and_then(|(px, py)| self.editor.object_at(px, py)).filter(|o| o.kind == ObjectKind::Npc) {
                                self.open_script(o.id);
                            }
                        }
                        if pressed && resp.hovered() {
                            if let Some((px, py)) = at {
                                match self.editor.object_at(px, py) {
                                    Some(o) => {
                                        self.editor.selected = Some(o.id);
                                        self.drag = Some(Drag { id: o.id, from: (o.x, o.y), grab: (px, py), now: (o.x, o.y) });
                                    }
                                    None => {
                                        self.editor.selected = None;
                                        self.add_cell = (px as i32, py as i32);
                                    }
                                }
                            }
                        }
                        if let (Some(d), Some((px, py))) = (self.drag.as_mut(), at) {
                            // whole-tile steps from where it was, so an NPC keeps its fraction
                            d.now = (d.from.0 + (px - d.grab.0).round(), d.from.1 + (py - d.grab.1).round());
                        }
                        if released || !down {
                            if let Some(d) = self.drag.take() {
                                if d.now != d.from {
                                    let r = self.editor.edit_object(d.id, ObjectEdit::Move(d.now.0, d.now.1));
                                    if r.is_ok() {
                                        self.status = format!("Moved object {} to ({}, {})", d.id, d.now.0, d.now.1);
                                    }
                                    self.report(r);
                                }
                            }
                        }
                    }
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
        viewport: egui::ViewportBuilder::default().with_inner_size([1280.0, 900.0]),
        ..Default::default()
    };
    eframe::run_native("Seihai level editor", options, Box::new(move |_cc| Ok(Box::new(app)))).map_err(|e| e.to_string())
}
