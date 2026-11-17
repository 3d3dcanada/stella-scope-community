//! The demo printer's camera (app-final board WO-01, 2026-11-01): a picture of the demo machine's bed from a fixed camera at
//! the front, drawn here from what the machine is doing, so the camera panel, the snapshot and the timelapse show the print
//! growing. The part is the file's own toolpath up to where the print is, lit from the front left; the head is where the
//! machine reports it. Nothing is fetched from anywhere.
//!
//! The camera rides with the bed (as a bed-mounted camera does on a bed slinger), so the bed fills the same place in every
//! frame. ponytail: one fixed view at 640 x 360 and points, not a mesh; a second view or a real render if the demo needs it.

use super::*;

const W: usize = 640;
const H: usize = 360;

/// A pinhole camera looking at the bed.
struct View {
    eye: [f64; 3],
    right: [f64; 3],
    up: [f64; 3],
    fwd: [f64; 3],
    focal: f64,
}

fn sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
fn norm(a: [f64; 3]) -> [f64; 3] {
    let l = dot(a, a).sqrt().max(1e-9);
    [a[0] / l, a[1] / l, a[2] / l]
}

impl View {
    /// Aimed at the print (the file's own bounds, or the middle of the bed), from the front left and a little above, close
    /// enough that the part fills about half the picture.
    fn aimed(bed: [f64; 2], bounds: Option<[f64; 4]>, height: f64) -> View {
        let b = bounds.filter(|b| b[2] > b[0]).unwrap_or([
            bed[0] * 0.35,
            bed[1] * 0.35,
            bed[0] * 0.65,
            bed[1] * 0.65,
        ]);
        let span = (b[2] - b[0]).max(b[3] - b[1]).max(height * 0.9).max(90.0);
        let target = [(b[0] + b[2]) / 2.0, (b[1] + b[3]) / 2.0, height * 0.45];
        let dist = span * 3.4;
        let dir = norm([-0.38, -1.0, 0.5]);
        let eye = [
            target[0] + dir[0] * dist,
            target[1] + dir[1] * dist,
            target[2] + dir[2] * dist,
        ];
        let fwd = norm(sub(target, eye));
        let right = norm(cross(fwd, [0.0, 0.0, 1.0]));
        let up = cross(right, fwd);
        View {
            eye,
            right,
            up,
            fwd,
            focal: (W as f64 * 0.5).min(H as f64 * 0.62) * dist / span,
        }
    }
    /// Pixel x, y and the distance from the camera.
    fn project(&self, p: [f64; 3]) -> Option<(f64, f64, f32)> {
        let d = sub(p, self.eye);
        let z = dot(d, self.fwd);
        if z < 1.0 {
            return None;
        }
        Some((
            W as f64 / 2.0 + self.focal * dot(d, self.right) / z,
            H as f64 / 2.0 - self.focal * dot(d, self.up) / z,
            z as f32,
        ))
    }
}

/// What one machine's camera has drawn so far: the empty bed, and the part with its depth, up to move `drawn` of `file`.
struct Film {
    base: Vec<u8>,
    part: Vec<u8>,
    depth: Vec<f32>,
    file: String,
    drawn: usize,
}

fn put(buf: &mut [u8], x: usize, y: usize, c: [u8; 3]) {
    let i = (y * W + x) * 3;
    buf[i..i + 3].copy_from_slice(&c);
}

/// The empty machine: a dark enclosure, the plate with its grid.
fn base(view: &View, bed: [f64; 2]) -> Vec<u8> {
    let mut img = vec![0u8; W * H * 3];
    for y in 0..H {
        for x in 0..W {
            let g = 18.0 + 16.0 * (1.0 - y as f64 / H as f64);
            put(&mut img, x, y, [g as u8, (g + 2.0) as u8, (g + 4.0) as u8]);
        }
    }
    // the plate: every pixel whose ray meets z = 0 inside the bed
    for py in 0..H {
        for px in 0..W {
            let dir = norm([
                view.fwd[0] * view.focal + view.right[0] * (px as f64 - W as f64 / 2.0)
                    - view.up[0] * (py as f64 - H as f64 / 2.0),
                view.fwd[1] * view.focal + view.right[1] * (px as f64 - W as f64 / 2.0)
                    - view.up[1] * (py as f64 - H as f64 / 2.0),
                view.fwd[2] * view.focal + view.right[2] * (px as f64 - W as f64 / 2.0)
                    - view.up[2] * (py as f64 - H as f64 / 2.0),
            ]);
            if dir[2] >= -1e-6 {
                continue;
            }
            let t = -view.eye[2] / dir[2];
            let (bx, by) = (view.eye[0] + dir[0] * t, view.eye[1] + dir[1] * t);
            if !(0.0..=bed[0]).contains(&bx) || !(0.0..=bed[1]).contains(&by) {
                continue;
            }
            let grid = (bx % 50.0 < 0.9) || (by % 50.0 < 0.9);
            let shade = 0.75 + 0.25 * (by / bed[1]);
            let c = if grid {
                [150.0, 120.0, 70.0]
            } else {
                [176.0, 140.0, 82.0]
            };
            put(
                &mut img,
                px,
                py,
                [
                    (c[0] * shade) as u8,
                    (c[1] * shade) as u8,
                    (c[2] * shade) as u8,
                ],
            );
        }
    }
    img
}

fn films() -> &'static Mutex<HashMap<String, Film>> {
    static F: OnceLock<Mutex<HashMap<String, Film>>> = OnceLock::new();
    F.get_or_init(Default::default)
}

/// The filament colour of a machine's demo spool: one of the brand's neons, fixed per machine.
fn colour(seed: u64) -> [f64; 3] {
    [
        [4.0, 217.0, 196.0],
        [255.0, 106.0, 0.0],
        [255.0, 45.0, 146.0],
        [24.0, 227.0, 107.0],
    ][(seed % 4) as usize]
}

/// One frame from the demo machine's camera, as JPEG.
pub fn jpeg(cfg: &Config) -> Result<Vec<u8>> {
    let m = Demo::open(cfg).m;
    let (plan, file, upto, head, tools) = {
        let mut s = m.lock();
        m.advance(&mut s, crate::model::now());
        let f = s
            .current
            .map(|i| (s.files[i].plan.clone(), s.files[i].name.clone()));
        let upto = match (&f, s.job) {
            (Some((p, _)), Job::Complete) => p.moves.len(),
            (Some((p, _)), Job::Printing | Job::Paused | Job::Cancelled | Job::Error) => {
                p.index_at(s.clock)
            }
            _ => 0,
        };
        (
            f.as_ref().map(|x| x.0.clone()),
            f.map(|x| x.1).unwrap_or_default(),
            upto,
            s.pos,
            m.tool_positions(&s),
        )
    };
    let height = plan
        .as_ref()
        .and_then(|p| p.moves.last())
        .map_or(40.0, |mv| mv.z as f64);
    let view = View::aimed(m.bed, plan.as_ref().map(|p| p.bounds), height);
    let key = format!("{}|{}", cfg.database, cfg.printer_id);
    let mut all = films().lock().unwrap_or_else(|e| e.into_inner());
    let film = all.entry(key).or_insert_with(|| Film {
        base: base(&view, m.bed),
        part: vec![0; W * H * 3],
        depth: vec![f32::MAX; W * H],
        file: String::new(),
        drawn: 0,
    });
    if film.file != file || upto < film.drawn {
        film.base = base(&view, m.bed);
        film.part.iter_mut().for_each(|b| *b = 0);
        film.depth.iter_mut().for_each(|d| *d = f32::MAX);
        film.file = file;
        film.drawn = 0;
    }
    if let Some(plan) = plan {
        let b = plan.bounds;
        let centre = [(b[0] + b[2]) / 2.0, (b[1] + b[3]) / 2.0];
        let light = norm([-0.6, -0.8, 0.4]);
        let fil = colour(m.kit.seed);
        let mut prev = film
            .drawn
            .checked_sub(1)
            .and_then(|i| plan.moves.get(i))
            .copied();
        for mv in &plan.moves[film.drawn..upto.min(plan.moves.len())] {
            if let Some(p) = prev {
                if mv.e > p.e + 1e-4 {
                    let (a, z) = (
                        [p.x as f64, p.y as f64, p.z as f64],
                        [mv.x as f64, mv.y as f64, mv.z as f64],
                    );
                    let len = ((z[0] - a[0]).powi(2) + (z[1] - a[1]).powi(2)).sqrt();
                    let steps = (len / 0.8).ceil().max(1.0) as usize;
                    for k in 0..=steps {
                        let f = k as f64 / steps as f64;
                        let q = [a[0] + (z[0] - a[0]) * f, a[1] + (z[1] - a[1]) * f, z[2]];
                        let n = norm([q[0] - centre[0], q[1] - centre[1], 0.15]);
                        let lit =
                            0.5 + 0.55 * dot(n, light).max(0.0) + 0.1 * (q[2] / m.max_z.max(1.0));
                        let c = [
                            (fil[0] * lit).min(255.0) as u8,
                            (fil[1] * lit).min(255.0) as u8,
                            (fil[2] * lit).min(255.0) as u8,
                        ];
                        if let Some((px, py, d)) = view.project(q) {
                            for (dx, dy) in [
                                (0, 0),
                                (1, 0),
                                (2, 0),
                                (0, 1),
                                (1, 1),
                                (2, 1),
                                (0, 2),
                                (1, 2),
                                (2, 2),
                            ] {
                                let (x, y) = (px as i64 + dx - 1, py as i64 + dy - 1);
                                if (0..W as i64).contains(&x) && (0..H as i64).contains(&y) {
                                    let i = y as usize * W + x as usize;
                                    if d < film.depth[i] {
                                        film.depth[i] = d;
                                        film.part[i * 3..i * 3 + 3].copy_from_slice(&c);
                                    }
                                }
                            }
                        }
                    }
                }
            }
            prev = Some(*mv);
        }
        film.drawn = upto.min(plan.moves.len());
    }
    let mut img = film.base.clone();
    for i in 0..W * H {
        if film.depth[i] < f32::MAX {
            img[i * 3..i * 3 + 3].copy_from_slice(&film.part[i * 3..i * 3 + 3]);
        }
    }
    drop(all);
    // the head (and any parked tools): a dark block with the nozzle under it, drawn over the part
    let heads = if tools.is_empty() { vec![head] } else { tools };
    for p in heads {
        let corners: Vec<(f64, f64)> = [
            [-14.0, -14.0, 6.0],
            [14.0, -14.0, 6.0],
            [14.0, -14.0, 30.0],
            [-14.0, -14.0, 30.0],
        ]
        .iter()
        .filter_map(|o| {
            view.project([p[0] + o[0], p[1] + o[1], p[2] + o[2]])
                .map(|(x, y, _)| (x, y))
        })
        .collect();
        if corners.len() == 4 {
            let (x0, x1) = (
                corners.iter().map(|c| c.0).fold(f64::MAX, f64::min),
                corners.iter().map(|c| c.0).fold(f64::MIN, f64::max),
            );
            let (y0, y1) = (
                corners.iter().map(|c| c.1).fold(f64::MAX, f64::min),
                corners.iter().map(|c| c.1).fold(f64::MIN, f64::max),
            );
            for y in y0.max(0.0) as usize..(y1.min(H as f64 - 1.0)).max(0.0) as usize {
                for x in x0.max(0.0) as usize..(x1.min(W as f64 - 1.0)).max(0.0) as usize {
                    let edge = (y as f64 - y0) < 3.0;
                    put(
                        &mut img,
                        x,
                        y,
                        if edge { [255, 106, 0] } else { [34, 36, 40] },
                    );
                }
            }
            if let (Some((nx, ny, _)), Some((tx, ty, _))) =
                (view.project(p), view.project([p[0], p[1], p[2] + 8.0]))
            {
                for y in ty.max(0.0) as usize..(ny.min(H as f64 - 1.0)).max(0.0) as usize {
                    for x in (nx - 2.0).max(0.0) as usize..(nx + 2.0).min(W as f64 - 1.0) as usize {
                        put(&mut img, x, y, [200, 170, 90]);
                    }
                }
                let _ = tx;
            }
        }
    }
    // the recording dot, top left
    for y in 10..20usize {
        for x in 10..20usize {
            if (x as f64 - 15.0).powi(2) + (y as f64 - 15.0).powi(2) <= 20.0 {
                put(&mut img, x, y, [255, 45, 60]);
            }
        }
    }
    let mut out = Vec::new();
    image::codecs::jpeg::JpegEncoder::new_with_quality(&mut out, 80)
        .encode(&img, W as u32, H as u32, image::ExtendedColorType::Rgb8)
        .map_err(|e| format!("the demo camera could not encode a picture: {e}"))?;
    Ok(out)
}
