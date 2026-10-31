//! Render the player model: `model ASSETS OUT.bmp`. Panels: front and
//! side standing, then side at walk times 1, 11 and 21.
use seihai_g3d::props::Player;
use seihai_g3d::render::{Camera, Frame};
use seihai_g3d::view::{draw_player, Assets};

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let mut assets = Assets::new(vec![a[1].clone().into()]);
    let (w, h) = (160, 200);
    let shots = [(0.0f32, 0u16), (90.0, 0), (90.0, 1), (90.0, 11), (90.0, 21)];
    let n = shots.len();
    let mut out = vec![0u32; w * n * h];
    for (k, (view, anim)) in shots.iter().enumerate() {
        let mut f = Frame::new(w, h);
        f.clear(0x404060);
        let (s, c) = view.to_radians().sin_cos();
        let cam = Camera::look_at([s * 110.0, 30.0, c * 110.0], [0.0, 25.0, 0.0], 40.0, w);
        let p = Player { anim: *anim, ..Player::default() };
        draw_player(&mut f, &cam, &mut assets, &p, 0.0);
        for y in 0..h {
            out[y * w * n + k * w..y * w * n + k * w + w].copy_from_slice(&f.px[y * w..y * w + w]);
        }
    }
    std::fs::write(&a[2], seihai_g3d::bmp::encode(w * n, h, &out)).unwrap();
}
