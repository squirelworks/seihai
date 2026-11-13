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
//!
//! Both renderers need the desktop's windowing system (winit's event
//! loop), and winit makes that only once per run. If it cannot start (no
//! display, or on Linux a missing `libxkbcommon-x11`), the OpenGL attempt
//! fails while making it, and wgpu then only hears "EventLoop can't be
//! recreated" (#508). That case is reported as the windowing system's
//! failure with the first attempt's reason, not as two renderer failures,
//! and later windows in the same run fail at once with the same reason.

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

/// Picks the renderer for the whole run.
pub const RENDERER_ENV: &str = "SEIHAI_RENDERER";

/// OpenGL failed once this run: go straight to wgpu.
static GLOW_FAILED: AtomicBool = AtomicBool::new(false);

/// The windowing system could not start this run, and why. winit cannot
/// try again, so every later window fails with this.
static NO_WINDOWING: Mutex<Option<String>> = Mutex::new(None);

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
    /// The windowing system itself could not start, and why. No renderer
    /// can open a window then, so `tried` is not the cause.
    pub windowing: Option<String>,
}

impl std::fmt::Display for WindowError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(why) = &self.windowing {
            return write!(f, "the windowing system could not start: {why}");
        }
        let parts: Vec<String> = self
            .tried
            .iter()
            .map(|(r, why)| format!("{r}: {why}"))
            .collect();
        write!(f, "{}", parts.join("; "))
    }
}

impl std::error::Error for WindowError {}

impl WindowError {
    /// What the player can do about it, as one line.
    pub fn hint(&self) -> String {
        if self.windowing.is_some() {
            return "no window can open in this session; run seihai from a desktop session (on Linux one with DISPLAY or WAYLAND_DISPLAY set, and libxkbcommon-x11 installed for X11)".into();
        }
        let tried: Vec<&str> = self.tried.iter().map(|(r, _)| *r).collect();
        format!(
            "no renderer could draw a window here (tried {}); install the graphics driver, or put Mesa's software opengl32.dll next to seihai.exe",
            tried.join(", then ")
        )
    }
}

/// How one renderer's attempt failed.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Failure {
    /// The renderer said this.
    Said(String),
    /// winit refused to make the event loop because an earlier attempt
    /// already tried (and failed) to make it.
    NoEventLoop,
}

impl Failure {
    fn from_eframe(e: eframe::Error) -> Failure {
        use winit::error::EventLoopError;
        match e {
            eframe::Error::WinitEventLoop(EventLoopError::RecreationAttempt) => {
                Failure::NoEventLoop
            }
            e => Failure::Said(e.to_string()),
        }
    }
}

/// One renderer's attempt. wgpu 22 reports a surface it cannot configure
/// (no display, a session with no desktop) by panicking inside eframe, not
/// as an error, so the attempt is run under `catch_unwind` and a panic
/// becomes this renderer's error. The panic message is kept for the error
/// and not printed as a crash.
fn attempt(
    title: &str,
    options: eframe::NativeOptions,
    create: eframe::AppCreator<'_>,
    started: &Arc<AtomicBool>,
) -> Result<(), Failure> {
    let mut no_loop = false;
    let r = caught(started, || {
        eframe::run_native(title, options, create).map_err(|e| match Failure::from_eframe(e) {
            Failure::Said(why) => why,
            Failure::NoEventLoop => {
                no_loop = true;
                String::new()
            }
        })
    });
    match r {
        Ok(()) => Ok(()),
        Err(_) if no_loop => Err(Failure::NoEventLoop),
        Err(why) => Err(Failure::Said(why)),
    }
}

/// Run `f`; a panic in it before `started` is set is returned as its
/// message instead. Only the opening of the window is covered: a panic on
/// another thread, or on this one once the app is made (`started`), goes
/// to the panic hook that was there before and keeps unwinding, as it
/// would without this.
fn caught(started: &Arc<AtomicBool>, f: impl FnOnce() -> Result<(), String>) -> Result<(), String> {
    let caught: Arc<Mutex<Option<String>>> = Default::default();
    let prev: Arc<Hook> = Arc::new(std::panic::take_hook());
    let here = std::thread::current().id();
    {
        let (caught, prev, started) = (caught.clone(), prev.clone(), started.clone());
        std::panic::set_hook(Box::new(move |info| {
            if std::thread::current().id() != here || started.load(Ordering::SeqCst) {
                return prev(info);
            }
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
    // Put the old hook back: ours holds the only other reference to it.
    drop(std::panic::take_hook());
    match Arc::try_unwrap(prev) {
        Ok(hook) => std::panic::set_hook(hook),
        Err(prev) => std::panic::set_hook(Box::new(move |info| prev(info))),
    }
    match r {
        Ok(r) => r,
        // A panic in the running app is the app's, not the renderer's.
        Err(payload) if started.load(Ordering::SeqCst) => std::panic::resume_unwind(payload),
        Err(_) => Err(caught
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .take()
            .unwrap_or_else(|| "panicked".into())),
    }
}

/// The panic hook as `take_hook` returns it.
type Hook = Box<dyn Fn(&std::panic::PanicHookInfo<'_>) + Sync + Send + 'static>;

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
    if let Some(why) = no_windowing() {
        return Err(WindowError {
            tried: Vec::new(),
            windowing: Some(why),
        });
    }
    let slot: Rc<RefCell<Option<eframe::AppCreator<'a>>>> = Rc::new(RefCell::new(Some(create)));
    let mut tried: Vec<(&'static str, String)> = Vec::new();
    for &renderer in order(Wanted::from_env(), GLOW_FAILED.load(Ordering::Relaxed)) {
        let Some(_) = slot.borrow().as_ref() else {
            break;
        };
        let options = eframe::NativeOptions {
            renderer,
            ..clone_options(&options)
        };
        let s = slot.clone();
        let started = Arc::new(AtomicBool::new(false));
        let up = started.clone();
        let r = attempt(
            title,
            options,
            Box::new(move |cc| {
                // The window and renderer are up: from here a panic is the
                // app's own and is not caught.
                up.store(true, Ordering::SeqCst);
                let create = s.borrow_mut().take().expect("the app is made once");
                create(cc)
            }),
            &started,
        );
        match r {
            Ok(()) => return Ok(()),
            Err(Failure::NoEventLoop) => {
                let why = windowing_reason(&tried);
                eprintln!("seihai: the windowing system could not start: {why}");
                *NO_WINDOWING.lock().unwrap_or_else(|e| e.into_inner()) = Some(why.clone());
                return Err(WindowError {
                    tried,
                    windowing: Some(why),
                });
            }
            Err(Failure::Said(e)) => {
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
    Err(WindowError {
        tried,
        windowing: None,
    })
}

/// The windowing system's earlier failure this run, if any.
fn no_windowing() -> Option<String> {
    NO_WINDOWING
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone()
}

/// winit refused a second event loop: the attempt before this one failed
/// while making it, so its reason is the windowing system's. With no
/// earlier attempt in this call (an earlier window of the run used up the
/// event loop without keeping it), there is no better reason to give.
fn windowing_reason(tried: &[(&'static str, String)]) -> String {
    tried
        .last()
        .map(|(_, why)| why.clone())
        .unwrap_or_else(|| "winit could not make its event loop again".into())
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
    fn a_panic_while_opening_is_an_error_with_its_message() {
        let opening = Arc::new(AtomicBool::new(false));
        assert_eq!(caught(&opening, || Ok(())), Ok(()));
        assert_eq!(caught(&opening, || Err("no".into())), Err("no".into()));
        let e = caught(&opening, || {
            panic!("Error in Surface::configure: Invalid surface")
        })
        .unwrap_err();
        assert!(
            e.starts_with("Error in Surface::configure: Invalid surface ("),
            "{e}"
        );
    }

    /// Once the app is made, a panic is the app's: it keeps unwinding and
    /// is not turned into a renderer error.
    #[test]
    fn a_panic_in_the_running_app_is_not_caught() {
        let up = Arc::new(AtomicBool::new(false));
        let r = std::panic::catch_unwind(|| {
            caught(&up, || {
                up.store(true, Ordering::SeqCst);
                panic!("the game broke")
            })
        });
        let payload = r.expect_err("the panic goes on");
        assert_eq!(payload.downcast_ref::<&str>(), Some(&"the game broke"));
    }

    /// A panic on another thread while the window opens is not taken for
    /// the window's.
    #[test]
    fn a_panic_on_another_thread_is_left_alone() {
        let opening = Arc::new(AtomicBool::new(false));
        let r = caught(&opening, || {
            let t = std::thread::spawn(|| panic!("network thread"));
            assert!(t.join().is_err());
            Ok(())
        });
        assert_eq!(r, Ok(()));
    }

    #[test]
    fn the_error_names_each_renderer_tried() {
        let e = WindowError {
            tried: vec![
                ("OpenGL", "needs 2.0".into()),
                ("wgpu", "no surface".into()),
            ],
            windowing: None,
        };
        assert_eq!(e.to_string(), "OpenGL: needs 2.0; wgpu: no surface");
    }

    /// #508: winit's "can't be recreated" means the event loop never came
    /// up, so the attempt before it is the windowing system's reason.
    #[test]
    fn a_second_event_loop_is_the_windowing_systems_failure() {
        use winit::error::EventLoopError;
        let f = Failure::from_eframe(eframe::Error::WinitEventLoop(
            EventLoopError::RecreationAttempt,
        ));
        assert_eq!(f, Failure::NoEventLoop);
        let f = Failure::from_eframe(eframe::Error::WinitEventLoop(EventLoopError::ExitFailure(
            1,
        )));
        assert!(matches!(f, Failure::Said(_)), "{f:?}");

        let tried = vec![(
            "OpenGL",
            "Library libxkbcommon-x11.so could not be loaded.".to_string(),
        )];
        let why = windowing_reason(&tried);
        assert_eq!(why, "Library libxkbcommon-x11.so could not be loaded.");
        let e = WindowError {
            tried,
            windowing: Some(why),
        };
        assert_eq!(
            e.to_string(),
            "the windowing system could not start: Library libxkbcommon-x11.so could not be loaded."
        );
        assert!(windowing_reason(&[]).contains("event loop"));
        assert!(e.hint().contains("libxkbcommon-x11"), "{}", e.hint());
        let renderers = WindowError {
            tried: vec![("OpenGL", "needs 2.0".into())],
            windowing: None,
        };
        assert!(
            renderers.hint().contains("tried OpenGL)"),
            "{}",
            renderers.hint()
        );
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
