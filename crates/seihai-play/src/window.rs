//! Opening an eframe window on any machine.
//!
//! eframe draws with OpenGL (glow) by default, and egui_glow needs OpenGL
//! 2.0. Windows with no 3D driver (a VM, a driverless or very old PC) only
//! has the built-in GDI OpenGL 1.1, so the window never opens (#475). Every
//! Seihai window opens through [`run_native`] instead: it tries OpenGL
//! first and, when that cannot start, opens the same window with wgpu. On
//! Windows wgpu reaches Direct3D 12, which Windows 10 and later provide in
//! software (WARP) when there is no driver; elsewhere it uses Vulkan or
//! Metal.
//!
//! Once OpenGL has failed, the rest of the run goes straight to wgpu, so
//! the game window after the start window does not try OpenGL again.
//! `SEIHAI_RENDERER=glow` or `wgpu` picks one and does not fall back.

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};

/// Picks the renderer for the whole run.
pub const RENDERER_ENV: &str = "SEIHAI_RENDERER";

/// OpenGL failed once this run: go straight to wgpu.
static GLOW_FAILED: AtomicBool = AtomicBool::new(false);

/// What `SEIHAI_RENDERER` asks for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Wanted {
    /// OpenGL, then wgpu if OpenGL cannot start.
    Auto,
    Glow,
    Wgpu,
}

impl Wanted {
    /// The variable's value. Anything else (or unset) is `Auto`.
    pub fn parse(v: Option<&str>) -> Wanted {
        match v.map(|v| v.trim().to_ascii_lowercase()).as_deref() {
            Some("glow" | "gl" | "opengl") => Wanted::Glow,
            Some("wgpu" | "dx12" | "d3d12" | "warp" | "vulkan" | "metal") => Wanted::Wgpu,
            _ => Wanted::Auto,
        }
    }

    fn from_env() -> Wanted {
        Wanted::parse(std::env::var(RENDERER_ENV).ok().as_deref())
    }
}

/// The renderers to try, in order.
fn order(wanted: Wanted, glow_failed: bool) -> &'static [eframe::Renderer] {
    use eframe::Renderer::{Glow, Wgpu};
    match wanted {
        Wanted::Glow => &[Glow],
        Wanted::Wgpu => &[Wgpu],
        Wanted::Auto if glow_failed => &[Wgpu],
        Wanted::Auto => &[Glow, Wgpu],
    }
}

/// The renderer's name, for the log line.
fn name(r: eframe::Renderer) -> &'static str {
    match r {
        eframe::Renderer::Glow => "OpenGL",
        eframe::Renderer::Wgpu => "wgpu (Direct3D 12 / WARP, Vulkan or Metal)",
    }
}

/// Why no window opened: each renderer tried and what it said.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WindowError {
    /// `(renderer, what it said)`, in the order tried.
    pub tried: Vec<(&'static str, String)>,
}

impl std::fmt::Display for WindowError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let parts: Vec<String> = self
            .tried
            .iter()
            .map(|(r, why)| format!("{r}: {why}"))
            .collect();
        write!(f, "{}", parts.join("; "))
    }
}

impl std::error::Error for WindowError {}

/// One renderer's attempt. wgpu 22 reports a surface it cannot configure
/// (no display, a session with no desktop) by panicking inside eframe, not
/// as an error, so the attempt is run under `catch_unwind` and a panic
/// becomes this renderer's error. The panic message is kept for the error
/// and not printed as a crash.
fn attempt(
    title: &str,
    options: eframe::NativeOptions,
    create: eframe::AppCreator<'_>,
) -> Result<(), String> {
    caught(|| eframe::run_native(title, options, create).map_err(|e| e.to_string()))
}

/// Run `f`; a panic in it is returned as its message instead.
fn caught(f: impl FnOnce() -> Result<(), String>) -> Result<(), String> {
    let caught: std::sync::Arc<std::sync::Mutex<Option<String>>> = Default::default();
    let hook = std::panic::take_hook();
    {
        let caught = caught.clone();
        std::panic::set_hook(Box::new(move |info| {
            let msg = info
                .payload()
                .downcast_ref::<&str>()
                .map(|s| s.to_string())
                .or_else(|| info.payload().downcast_ref::<String>().cloned())
                .unwrap_or_else(|| "panicked".into());
            let at = info
                .location()
                .map(|l| format!(" ({}:{})", l.file(), l.line()))
                .unwrap_or_default();
            *caught.lock().unwrap_or_else(|e| e.into_inner()) = Some(format!("{msg}{at}"));
        }));
    }
    let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(f));
    std::panic::set_hook(hook);
    match r {
        Ok(r) => r,
        Err(_) => Err(caught
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .take()
            .unwrap_or_else(|| "panicked".into())),
    }
}

/// `eframe::run_native`, with the wgpu fallback. The creator is kept
/// until a renderer has made the window, so a second one can use it.
/// `options.renderer` is overwritten. Blocks until the window closes. If a
/// renderer fails after the app was made, the app cannot be made again and
/// the error is returned.
pub fn run_native<'a>(
    title: &str,
    options: eframe::NativeOptions,
    create: eframe::AppCreator<'a>,
) -> Result<(), WindowError> {
    let slot: Rc<RefCell<Option<eframe::AppCreator<'a>>>> = Rc::new(RefCell::new(Some(create)));
    let mut tried = Vec::new();
    for &renderer in order(Wanted::from_env(), GLOW_FAILED.load(Ordering::Relaxed)) {
        let Some(_) = slot.borrow().as_ref() else {
            break;
        };
        let options = eframe::NativeOptions {
            renderer,
            ..clone_options(&options)
        };
        let s = slot.clone();
        let r = attempt(
            title,
            options,
            Box::new(move |cc| {
                let create = s.borrow_mut().take().expect("the app is made once");
                create(cc)
            }),
        );
        match r {
            Ok(()) => return Ok(()),
            Err(e) => {
                eprintln!("seihai: {} could not open the window: {e}", name(renderer));
                // Only a window that never got as far as the app counts
                // against OpenGL; an error of the app's own is not retried.
                if renderer == eframe::Renderer::Glow && slot.borrow().is_some() {
                    GLOW_FAILED.store(true, Ordering::Relaxed);
                }
                tried.push((name(renderer), e));
            }
        }
    }
    Err(WindowError { tried })
}

/// `NativeOptions` is not `Clone`; the fields Seihai sets are copied and
/// the rest are eframe's defaults.
fn clone_options(o: &eframe::NativeOptions) -> eframe::NativeOptions {
    eframe::NativeOptions {
        viewport: o.viewport.clone(),
        vsync: o.vsync,
        multisampling: o.multisampling,
        depth_buffer: o.depth_buffer,
        stencil_buffer: o.stencil_buffer,
        centered: o.centered,
        persist_window: o.persist_window,
        run_and_return: o.run_and_return,
        hardware_acceleration: o.hardware_acceleration,
        dithering: o.dithering,
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use eframe::Renderer::{Glow, Wgpu};

    #[test]
    fn the_variable_names_a_renderer() {
        assert_eq!(Wanted::parse(None), Wanted::Auto);
        assert_eq!(Wanted::parse(Some("")), Wanted::Auto);
        assert_eq!(Wanted::parse(Some("GLOW")), Wanted::Glow);
        assert_eq!(Wanted::parse(Some(" opengl ")), Wanted::Glow);
        assert_eq!(Wanted::parse(Some("wgpu")), Wanted::Wgpu);
        assert_eq!(Wanted::parse(Some("warp")), Wanted::Wgpu);
        assert_eq!(Wanted::parse(Some("dx9")), Wanted::Auto);
    }

    #[test]
    fn opengl_first_then_wgpu_and_wgpu_only_once_opengl_failed() {
        assert_eq!(order(Wanted::Auto, false), &[Glow, Wgpu]);
        assert_eq!(order(Wanted::Auto, true), &[Wgpu]);
        // A named renderer has no fallback.
        assert_eq!(order(Wanted::Glow, true), &[Glow]);
        assert_eq!(order(Wanted::Wgpu, false), &[Wgpu]);
    }

    #[test]
    fn a_panic_is_an_error_with_its_message() {
        assert_eq!(caught(|| Ok(())), Ok(()));
        assert_eq!(caught(|| Err("no".into())), Err("no".into()));
        let e = caught(|| panic!("Error in Surface::configure: Invalid surface")).unwrap_err();
        assert!(
            e.starts_with("Error in Surface::configure: Invalid surface ("),
            "{e}"
        );
        // The usual hook is back afterwards.
        assert_eq!(caught(|| Ok(())), Ok(()));
    }

    #[test]
    fn the_error_names_each_renderer_tried() {
        let e = WindowError {
            tried: vec![
                ("OpenGL", "needs 2.0".into()),
                ("wgpu", "no surface".into()),
            ],
        };
        assert_eq!(e.to_string(), "OpenGL: needs 2.0; wgpu: no surface");
    }

    #[test]
    fn copied_options_keep_the_window_size() {
        let o = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default().with_inner_size([640.0, 480.0]),
            ..Default::default()
        };
        assert_eq!(
            clone_options(&o).viewport.inner_size,
            Some(egui::vec2(640.0, 480.0))
        );
    }
}
