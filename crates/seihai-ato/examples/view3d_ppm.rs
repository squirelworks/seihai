//! Draws a small Graal3D-like scene with the 3D view and writes it as a binary PPM:
//! a ground slab and a cube under a Sun, seen from behind the cube.
//!
//!   cargo run -p seihai-ato --example view3d_png -- out.ppm

use seihai_ato::three_ds::write;
use seihai_ato::view3d::{draw, Transform, World3d};

fn main() {
    let out = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "view3d.ppm".into());
    let cube_v = [
        [-1.0, -1.0, 0.0],
        [1.0, -1.0, 0.0],
        [1.0, 1.0, 0.0],
        [-1.0, 1.0, 0.0],
        [-1.0, -1.0, 2.0],
        [1.0, -1.0, 2.0],
        [1.0, 1.0, 2.0],
        [-1.0, 1.0, 2.0],
    ];
    let cube_f = [
        [0, 1, 5],
        [0, 5, 4],
        [1, 2, 6],
        [1, 6, 5],
        [2, 3, 7],
        [2, 7, 6],
        [3, 0, 4],
        [3, 4, 7],
        [4, 5, 6],
        [4, 6, 7],
        [0, 2, 1],
        [0, 3, 2],
    ];
    let ground_v = [
        [-40.0, -40.0, 0.0],
        [40.0, -40.0, 0.0],
        [40.0, 40.0, 0.0],
        [-40.0, 40.0, 0.0],
    ];
    let mut w = World3d::default();
    w.shape_file(
        "labcube.3ds",
        Some(&write::mesh("cube", &cube_v, &cube_f, [220, 60, 50])),
    );
    w.shape_file(
        "ground.3ds",
        Some(&write::mesh(
            "ground",
            &ground_v,
            &[[0, 1, 2], [0, 2, 3]],
            [70, 150, 70],
        )),
    );
    let at = |x: f32, y: f32, angle: f32| {
        Some(Transform {
            position: [x, y, 0.0],
            axis: [0.0, 0.0, 1.0],
            angle,
        })
    };
    w.object(0, Some(b"ground.3ds"), at(0.0, 0.0, 0.0));
    w.object(1, Some(b"labcube.3ds"), at(0.0, 0.0, 0.5));
    w.object(2, Some(b"labcube.3ds"), at(4.0, 8.0, 0.2));
    w.control = Some(1);
    w.sun = Some((
        [0.4, 0.6, -0.7],
        [0.9, 0.9, 0.8, 1.0],
        [0.35, 0.35, 0.4, 1.0],
    ));
    w.sky_colour = Some([0.45, 0.6, 0.85]);
    let f = draw(&w, 640, 480);
    let mut ppm = format!("P6\n{} {}\n255\n", f.width, f.height).into_bytes();
    for p in f.rgba.chunks(4) {
        ppm.extend_from_slice(&p[..3]);
    }
    std::fs::write(&out, ppm).expect("write the picture");
    println!("wrote {out}");
}
