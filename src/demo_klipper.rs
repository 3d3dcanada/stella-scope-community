//! The demo printer's Klipper (the owner 2026-11-01: "Why can't a demo printer level?").
//!
//! A demo printer used to answer only the driver's neutral actions, so levelling, the bed mesh, the Z offset, the console,
//! macros, restarts, the camera and filament loading were hidden on every demo machine. Here the demo machine is given the
//! Klipper a real one of its kind runs: the service's Moonraker client asks this module instead of the network when the
//! printer is a demo (`moonraker::Moonraker::get/post/delete`), so every page that reads or commands a Klipper printer
//! works on a demo printer through the same code a real one uses. Nothing leaves the machine.
//!
//! What it acts out, each with real motion where a real machine moves: homing, moves, quad gantry level and Z tilt (rounds
//! that converge under the tolerance), the bed mesh and its profiles, SCREWS_TILT_CALCULATE, probe accuracy, the Z offset
//! and Z_OFFSET_APPLY_PROBE, the probe and endstop calibrations, PID tuning, velocity and pressure advance, resonance tests
//! and the shaper calibration, filament load and unload, run-out, a heater fault, object exclusion, the job queue, firmware
//! and Klipper restarts and SAVE_CONFIG. The console answers in Klipper's words (the formats the probe and QGL parsers were
//! written against, src/probe.rs and src/qgl.rs). The numbers are made up per machine and say so nowhere else: every view
//! badges a demo printer DEMO.

use super::*;
use std::collections::BTreeMap;

/// Probe, travel and lift speeds the routines move at, mm/s (Klipper's defaults: probe speed 5, lift speed = probe speed x 2).
const PROBE_SPEED: f64 = 5.0;
const LIFT_SPEED: f64 = 10.0;
const TRAVEL_SPEED: f64 = 150.0;
/// How high the head travels between probe points, mm (`horizontal_move_z`).
const HOP: f64 = 5.0;
/// How long a restart keeps Klipper away, seconds.
const FIRMWARE_RESTART_S: f64 = 4.0;
const RESTART_S: f64 = 3.0;
/// A routine never holds a command longer than this (the longest, a mesh of a 350 mm bed, takes about a minute).
const ROUTINE_CAP_S: f64 = 600.0;
/// Routines and restarts run this many times faster than a real machine: 1 in the app, faster under `cargo test`.
const SCALE: f64 = if cfg!(test) { 40.0 } else { 1.0 };

/// How this machine levels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Level {
    /// Four Z motors and a gantry that tilts (Voron 2.4, Micron, SV08): QUAD_GANTRY_LEVEL.
    Gantry,
    /// Three Z motors under the bed (Trident, V-Core): Z_TILT_ADJUST.
    Tilt,
    /// One Z: the bed is set by its screws (SCREWS_TILT_CALCULATE) and the mesh does the rest.
    Screws,
}

/// What Klipper hardware a demo machine has, from what the machine is.
#[derive(Clone, Debug)]
pub struct Kit {
    pub level: Level,
    /// The probe's config section: `probe` (Tap, Klicky, a load cell, a PINDA) or `bltouch`.
    pub probe: &'static str,
    pub z_motors: usize,
    /// A Z endstop of its own as well as the probe (Z_ENDSTOP_CALIBRATE).
    pub endstop: bool,
    /// Mesh points along each side, and how far the mesh stays from the bed's edge, mm.
    pub grid: usize,
    pub inset: f64,
    pub tolerance: f64,
    /// Voron macros (CLEAN_NOZZLE, the Voron PRINT_START).
    pub voron: bool,
    /// Per-machine numbers (the bed's shape, how far the gantry is out, the resonances).
    pub seed: u64,
}

pub fn kit(spec: &Spec) -> Kit {
    let id = spec.id.as_str();
    let seed = id.bytes().fold(0xcbf29ce484222325u64, |h, b| {
        (h ^ b as u64).wrapping_mul(0x100000001b3)
    });
    let small = spec.bed[0].min(spec.bed[1]) < 200.0;
    let (level, z_motors, endstop, voron) =
        if id.starts_with("voron_v24") || id.starts_with("micron") {
            (Level::Gantry, 4, true, true)
        } else if id.starts_with("sovol_sv08") {
            (Level::Gantry, 4, false, false)
        } else if id.starts_with("voron_trident") {
            (Level::Tilt, 3, true, true)
        } else if id.starts_with("voron_v0") {
            (Level::Screws, 1, true, true)
        } else if id.starts_with("ender3") || id.starts_with("prusa") {
            (Level::Screws, 1, false, false)
        } else {
            match spec.frame {
                Frame::GantryZ => (Level::Gantry, 4, false, false),
                Frame::BedZ => (Level::Tilt, 3, false, false),
                _ => (Level::Screws, 1, false, false),
            }
        };
    Kit {
        level,
        probe: if id.starts_with("ender3") {
            "bltouch"
        } else {
            "probe"
        },
        z_motors,
        endstop,
        grid: if small { 5 } else { 7 },
        inset: if small { 10.0 } else { 25.0 },
        tolerance: if level == Level::Gantry { 0.02 } else { 0.01 },
        voron,
        seed,
    }
}

/// A number from the machine's seed, 0 to 1, different for each `n`.
fn unit(seed: u64, n: u64) -> f64 {
    let mut x = seed ^ n.wrapping_mul(0x9E3779B97F4A7C15);
    x ^= x >> 33;
    x = x.wrapping_mul(0xff51afd7ed558ccd);
    x ^= x >> 33;
    (x % 10_000) as f64 / 10_000.0
}

/// What a routine leaves behind when it ends.
#[derive(Clone, Debug)]
pub enum Done {
    Nothing,
    Homed,
    Levelled,
    Mesh(Vec<Vec<f64>>),
    /// PID_CALIBRATE: the heater's section and the three numbers.
    Pid(&'static str, [f64; 3]),
    /// PROBE_CALIBRATE or Z_ENDSTOP_CALIBRATE: the manual probe starts at this Z.
    Manual(&'static str, f64),
    /// SHAPER_CALIBRATE: axis, shaper, frequency.
    Shaper(char, &'static str, f64),
    Loaded(bool),
}

/// Something the machine is doing that takes time: the head follows `path`, the console gets `lines` as they come due.
#[derive(Clone, Debug)]
pub struct Routine {
    /// What the page calls it while it runs.
    pub what: &'static str,
    t0: f64,
    path: Vec<(f64, [f64; 3])>,
    lines: Vec<(f64, String)>,
    said: usize,
    end: f64,
    done: Done,
    /// Filament through the nozzle while it runs, mm/s (negative pulls it back).
    extrude: f64,
    /// Seconds into the routine the filament starts to move.
    extrude_from: f64,
    /// PID tuning: the heater swings round this target while it runs.
    swing: Option<(bool, f64)>,
}

/// A routine being written: moves at a speed from where the last one ended, with console lines on the way.
struct Path {
    t: f64,
    at: [f64; 3],
    pts: Vec<(f64, [f64; 3])>,
    lines: Vec<(f64, String)>,
    vmax: f64,
}

impl Path {
    fn from(at: [f64; 3], vmax: f64) -> Path {
        Path {
            t: 0.0,
            at,
            pts: vec![(0.0, at)],
            lines: Vec::new(),
            vmax,
        }
    }
    fn go(&mut self, to: [f64; 3], speed: f64) {
        let d = ((to[0] - self.at[0]).powi(2)
            + (to[1] - self.at[1]).powi(2)
            + (to[2] - self.at[2]).powi(2))
        .sqrt();
        if d < 1e-6 {
            return;
        }
        self.t += (d / speed.min(self.vmax).max(0.5)).max(0.05);
        self.at = to;
        self.pts.push((self.t, to));
    }
    fn wait(&mut self, s: f64) {
        self.t += s;
        self.pts.push((self.t, self.at));
    }
    fn say(&mut self, line: impl Into<String>) {
        self.lines.push((self.t, line.into()));
    }
    /// One probe: down at probe speed to where the probe triggers, the reading, back up to the travel height.
    fn probe(&mut self, x: f64, y: f64, contact: f64) {
        self.go([x, y, HOP], TRAVEL_SPEED);
        self.go([x, y, (1.0 + contact - 1.5).max(0.4)], PROBE_SPEED);
        self.wait(0.15);
        self.say(format!(
            "probe: at {x:.3},{y:.3} bed will contact at z={contact:.6}"
        ));
        self.go([x, y, HOP], LIFT_SPEED);
    }
    fn into_routine(self, what: &'static str, t0: f64, done: Done) -> Routine {
        let path = self.pts.into_iter().map(|(t, p)| (t / SCALE, p)).collect();
        let lines = self
            .lines
            .into_iter()
            .map(|(t, l)| (t / SCALE, l))
            .collect();
        Routine {
            what,
            t0,
            end: self.t / SCALE,
            path,
            lines,
            said: 0,
            done,
            extrude: 0.0,
            extrude_from: 0.0,
            swing: None,
        }
    }
}

/// One saved or uploaded config file (printer.cfg for the hotend-fan edit).
type Files = BTreeMap<String, String>;

/// Klipper's side of a demo printer.
pub struct Klip {
    /// `ready`, `startup` or `shutdown`, and Klipper's message for it.
    pub klippy: &'static str,
    pub message: String,
    back_at: f64,
    /// The console as Moonraker's gcode_store keeps it: (sequence, time, text, `command` or `response`).
    console: Vec<(u64, f64, String, &'static str)>,
    seq: u64,
    read: u64,
    last_said: f64,
    relative: bool,
    /// The G-code Z offset (SET_GCODE_OFFSET), mm.
    pub z_adjust: f64,
    /// configfile settings, as Klipper reports them (lower-case section names).
    settings: Value,
    /// What a SAVE_CONFIG would write: section, key, value.
    pending: BTreeMap<String, BTreeMap<String, String>>,
    /// The gantry or the bed has been levelled since the last restart.
    pub applied: bool,
    /// How far each Z corner is out before levelling, mm (front left, back left, back right, front right).
    tilt: [f64; 4],
    pub mesh: Option<(String, Vec<Vec<f64>>)>,
    profiles: BTreeMap<String, Vec<Vec<f64>>>,
    velocity: f64,
    accel: f64,
    scv: f64,
    pa: f64,
    pub routine: Option<Routine>,
    /// A manual probe (PROBE_CALIBRATE, Z_ENDSTOP_CALIBRATE): what it calibrates and the nozzle's Z now.
    manual: Option<(&'static str, f64)>,
    excluded: Vec<String>,
    /// Moonraker's job queue: (job id, file, time added).
    queue: Vec<(String, String, f64)>,
    queue_paused: bool,
    next_job: u64,
    /// The run-out sensor sees filament.
    filament: bool,
    last_query: bool,
    last_z: f64,
    files: Files,
    restarts: u64,
    /// SIMULATE_RUNOUT / SIMULATE_HEATER_FAULT pressed with no print running: it happens this long into the next print
    /// (a page cannot run a macro during a print), counted from when that print is first seen running.
    armed: Option<(Trouble, Option<f64>)>,
}

/// What a demo can be asked to go wrong.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Trouble {
    Runout,
    HeaterFault,
}
const ARMED_AFTER_S: f64 = 20.0;

fn section(settings: &mut Value, name: &str, v: Value) {
    settings[name] = v;
}

impl Klip {
    pub fn new(spec: &Spec, kit: &Kit) -> Klip {
        let seed = kit.seed;
        // A gantry that has been switched off sags: 0.6 to 1.9 mm across. A bed on three screws is out by much less.
        let span = match kit.level {
            Level::Gantry => 0.6 + 1.3 * unit(seed, 1),
            Level::Tilt => 0.3 + 0.5 * unit(seed, 1),
            Level::Screws => 0.12 + 0.2 * unit(seed, 1),
        };
        let tilt = [
            0.0,
            span * unit(seed, 2),
            span,
            span * (0.3 + 0.5 * unit(seed, 3)),
        ];
        let mut k = Klip {
            klippy: "ready",
            message: "Printer is ready".into(),
            back_at: 0.0,
            console: Vec::new(),
            seq: 0,
            read: 0,
            last_said: 0.0,
            relative: false,
            z_adjust: 0.0,
            settings: json!({}),
            pending: BTreeMap::new(),
            applied: false,
            tilt,
            mesh: None,
            profiles: BTreeMap::new(),
            velocity: spec.max_velocity,
            accel: if spec.max_velocity >= 400.0 {
                10000.0
            } else {
                3000.0
            },
            scv: 5.0,
            pa: 0.035 + 0.02 * unit(seed, 4),
            routine: None,
            manual: None,
            excluded: Vec::new(),
            queue: Vec::new(),
            queue_paused: false,
            next_job: 1,
            filament: true,
            last_query: false,
            last_z: 0.0,
            files: Files::new(),
            restarts: 0,
            armed: None,
        };
        k.settings = settings_for(spec, kit, &k);
        // A machine that has been used has a mesh saved as `default`, loaded at start like Klipper does.
        let grid = mesh_grid(spec, kit, &k, true);
        k.profiles.insert("default".into(), grid.clone());
        k.mesh = Some(("default".into(), grid));
        k
    }

    fn say(&mut self, ts: f64, text: &str, kind: &'static str) {
        // Two lines in the same instant keep their order and are both kept (the store drops a repeat of time and text).
        let ts = if ts <= self.last_said {
            self.last_said + 0.001
        } else {
            ts
        };
        self.last_said = ts;
        self.seq += 1;
        self.console.push((self.seq, ts, text.to_owned(), kind));
        if self.console.len() > 400 {
            self.console.drain(..100);
        }
    }
    /// An answer, as Klipper's `respond_info` puts it in the console: every line behind `// `.
    pub(super) fn info(&mut self, ts: f64, text: &str) {
        let text: Vec<String> = text.lines().map(|l| format!("// {l}")).collect();
        self.say(ts, &text.join("\n"), "response");
    }
    fn error(&mut self, ts: f64, text: &str) {
        self.say(ts, &format!("!! {text}"), "response");
    }

    pub fn ready(&self) -> bool {
        self.klippy == "ready"
    }
    pub fn probe_name(&self) -> &str {
        if self.settings.get("bltouch").is_some() {
            "bltouch"
        } else {
            "probe"
        }
    }
    fn probe_offset(&self) -> f64 {
        self.settings[self.probe_name()]["z_offset"]
            .as_f64()
            .unwrap_or(0.0)
    }
    fn pend(&mut self, section: &str, key: &str, value: String) {
        self.pending
            .entry(section.into())
            .or_default()
            .insert(key.into(), value);
    }
}

/// The bed's own shape at (x, y), mm: a shallow bowl or dome, a twist and a little waviness, different on every machine.
fn surface(spec: &Spec, kit: &Kit, x: f64, y: f64) -> f64 {
    let s = kit.seed;
    let (fx, fy) = (x / spec.bed[0] - 0.5, y / spec.bed[1] - 0.5);
    let amp = if kit.grid > 5 { 0.18 } else { 0.1 };
    let bowl = (unit(s, 10) - 0.4) * amp * (fx * fx + fy * fy) * 4.0;
    let twist = (unit(s, 11) - 0.5) * amp * 0.8 * fx * fy * 4.0;
    let wave =
        0.012 * ((fx * 9.0 + unit(s, 12) * 6.0).sin() + (fy * 7.0 + unit(s, 13) * 6.0).cos());
    bowl + twist + wave
}

/// How far the gantry or bed is out at (x, y) before it is levelled: the corners' errors, spread across the bed.
fn tilt_at(spec: &Spec, k: &Klip, x: f64, y: f64) -> f64 {
    if k.applied {
        return 0.0;
    }
    let (u, v) = (
        (x / spec.bed[0]).clamp(0.0, 1.0),
        (y / spec.bed[1]).clamp(0.0, 1.0),
    );
    let [fl, bl, br, fr] = k.tilt;
    let mean = (fl + bl + br + fr) / 4.0;
    (fl * (1.0 - u) * (1.0 - v) + fr * u * (1.0 - v) + bl * (1.0 - u) * v + br * u * v) - mean
}

fn mesh_points(spec: &Spec, kit: &Kit) -> (Vec<f64>, Vec<f64>) {
    let n = kit.grid;
    let axis = |len: f64| {
        (0..n)
            .map(|i| kit.inset + (len - 2.0 * kit.inset) * i as f64 / (n - 1) as f64)
            .collect::<Vec<_>>()
    };
    (axis(spec.bed[0]), axis(spec.bed[1]))
}

/// The mesh this machine probes now: rows front to back, each left to right, rounded as Klipper reports them.
fn mesh_grid(spec: &Spec, kit: &Kit, k: &Klip, levelled: bool) -> Vec<Vec<f64>> {
    let (xs, ys) = mesh_points(spec, kit);
    ys.iter()
        .map(|y| {
            xs.iter()
                .map(|x| {
                    let t = if levelled {
                        0.0
                    } else {
                        tilt_at(spec, k, *x, *y)
                    };
                    ((surface(spec, kit, *x, *y) + t) * 1e6).round() / 1e6
                })
                .collect()
        })
        .collect()
}

fn mesh_params(spec: &Spec, kit: &Kit) -> Value {
    json!({"min_x":kit.inset,"min_y":kit.inset,"max_x":spec.bed[0]-kit.inset,"max_y":spec.bed[1]-kit.inset,
        "x_count":kit.grid,"y_count":kit.grid,"mesh_x_pps":2,"mesh_y_pps":2,"algo":"bicubic","tension":0.2})
}

/// The probe points of a level, in Klipper's order: front left, back left, back right, front right (a Z tilt's three: front
/// left, back middle, front right).
fn level_points(spec: &Spec, kit: &Kit) -> Vec<(f64, f64)> {
    let (i, w, d) = (kit.inset, spec.bed[0], spec.bed[1]);
    match kit.level {
        Level::Tilt => vec![(i + 5.0, i), (w / 2.0, d - i), (w - i - 5.0, i)],
        _ => vec![(i, i), (i, d - i), (w - i, d - i), (w - i, i)],
    }
}

/// The macros a demo machine has: name, what Klipper's help says, the macro's body (its parameters are read from it).
fn macros(kit: &Kit) -> Vec<(&'static str, &'static str, &'static str)> {
    let mut m = vec![
        ("PRINT_START", "Heat, home and get ready to print", "{% set bed = params.BED|default(60)|float %}\n{% set extruder = params.EXTRUDER|default(220)|float %}\nM190 S{bed}\nG28\nM109 S{extruder}"),
        ("PRINT_END", "Heaters off, park the head", "TURN_OFF_HEATERS\nG91\nG1 Z10\nG90"),
        ("PARK", "Park the head at the back, up high", "G1 X{x} Y{y} Z{z}"),
        ("LOAD_FILAMENT", "Heat, push 50 mm of filament through", "{% set temp = params.TEMP|default(220)|float %}\nM109 S{temp}\nM83\nG1 E50 F300"),
        ("UNLOAD_FILAMENT", "Heat, pull 100 mm of filament out", "{% set temp = params.TEMP|default(220)|float %}\nM109 S{temp}\nM83\nG1 E-100 F1200"),
        ("M600", "Filament change: pause the print", "PAUSE"),
        ("TURN_OFF_HEATERS", "Turn off all heaters", ""),
        // Two the demo has so the alerts can be seen without breaking anything: they act out what a real printer does.
        ("SIMULATE_RUNOUT", "Demo printer: the run-out sensor loses the filament", ""),
        ("SIMULATE_HEATER_FAULT", "Demo printer: the hotend stops heating and Klipper shuts down", ""),
    ];
    if kit.voron {
        m.push((
            "CLEAN_NOZZLE",
            "Wipe the nozzle on the brush",
            "{% set wipes = params.WIPES|default(6)|int %}",
        ));
    }
    m
}

/// Klipper's own extended commands this machine registers, with the help text Klipper gives them.
fn commands(kit: &Kit) -> Vec<(&'static str, &'static str)> {
    let mut c = vec![
        ("SET_HEATER_TEMPERATURE", "Sets a heater temperature"),
        ("TEMPERATURE_WAIT", "Wait for a temperature on a sensor"),
        (
            "SET_GCODE_OFFSET",
            "Set a virtual offset to g-code positions",
        ),
        ("SAVE_GCODE_STATE", "Save G-Code coordinate state"),
        (
            "RESTORE_GCODE_STATE",
            "Restore a previously saved G-Code state",
        ),
        (
            "GET_POSITION",
            "Return information on the current location of the toolhead",
        ),
        ("SET_VELOCITY_LIMIT", "Set printer velocity limits"),
        ("SET_PRESSURE_ADVANCE", "Set pressure advance parameters"),
        ("PID_CALIBRATE", "Run PID calibration test"),
        (
            "TUNING_TOWER",
            "Tool to adjust a parameter at each Z height",
        ),
        ("SAVE_CONFIG", "Overwrite config file and restart"),
        ("PAUSE", "Pauses the current print"),
        ("RESUME", "Resumes the print from a pause"),
        ("CANCEL_PRINT", "Cancel the current print"),
        (
            "FIRMWARE_RESTART",
            "Restart firmware, host, and reload config",
        ),
        ("RESTART", "Reload config file and restart host software"),
        ("STATUS", "Report the printer status"),
        (
            "HELP",
            "Report the list of available extended G-Code commands",
        ),
        ("ECHO", "Echo the parameters"),
        ("RESPOND", "Echo the message prepended with a prefix"),
        ("PROBE", "Probe Z-height at current XY position"),
        ("QUERY_PROBE", "Return the status of the z-probe"),
        (
            "PROBE_ACCURACY",
            "Probe Z-height accuracy at current XY position",
        ),
        ("PROBE_CALIBRATE", "Calibrate the probe's z_offset"),
        ("Z_OFFSET_APPLY_PROBE", "Adjust the probe's z_offset"),
        ("TESTZ", "Move to new Z height"),
        ("ACCEPT", "Accept the current Z position"),
        ("ABORT", "Abort manual Z probing tool"),
        ("BED_MESH_CALIBRATE", "Perform Mesh Bed Leveling"),
        ("BED_MESH_PROFILE", "Bed Mesh Persistent Storage management"),
        (
            "BED_MESH_CLEAR",
            "Clear the Mesh so no z-adjustment is made",
        ),
        (
            "BED_MESH_OUTPUT",
            "Retrieve interpolated grid of probed z-points",
        ),
        (
            "TEST_RESONANCES",
            "Runs the resonance test for a specifed axis",
        ),
        (
            "SHAPER_CALIBRATE",
            "Simular to TEST_RESONANCES but suggest input shaper config",
        ),
        ("EXCLUDE_OBJECT", "Cancel moves for a specific object"),
        ("EXCLUDE_OBJECT_DEFINE", "Provides a summary of an object"),
        ("SET_FILAMENT_SENSOR", "Sets the filament sensor on/off"),
        (
            "QUERY_FILAMENT_SENSOR",
            "Query the status of the Filament Sensor",
        ),
        ("M117", "Set the display message"),
    ];
    match kit.level {
        Level::Gantry => c.push((
            "QUAD_GANTRY_LEVEL",
            "Conform a moving, twistable gantry to the shape of a stationary bed",
        )),
        Level::Tilt => c.push(("Z_TILT_ADJUST", "Adjust the Z tilt")),
        Level::Screws => c.push((
            "SCREWS_TILT_CALCULATE",
            "Tool to help adjust bed leveling screws",
        )),
    }
    if kit.endstop {
        c.push(("Z_ENDSTOP_CALIBRATE", "Calibrate a Z endstop"));
    }
    c
}

fn help(kit: &Kit) -> Value {
    let mut h = serde_json::Map::new();
    for (n, t) in commands(kit) {
        h.insert(n.into(), json!(t));
    }
    for (n, t, _) in macros(kit) {
        h.insert(n.into(), json!(t));
    }
    Value::Object(h)
}

pub fn help_names(kit: &Kit) -> std::collections::BTreeSet<String> {
    help(kit)
        .as_object()
        .map(|m| m.keys().cloned().collect())
        .unwrap_or_default()
}

/// printer.cfg as Klipper reports it: the sections and values the pages read (Calibrate, Levelling, the macro parameters,
/// the hotend fan), for the machine this is.
fn settings_for(spec: &Spec, kit: &Kit, k: &Klip) -> Value {
    let mut s = json!({});
    let kin = if spec.frame == Frame::BedY {
        "cartesian"
    } else {
        "corexy"
    };
    section(
        &mut s,
        "printer",
        json!({"kinematics":kin,"max_velocity":spec.max_velocity,"max_accel":k.accel,"square_corner_velocity":5.0,"max_z_velocity":15.0}),
    );
    section(
        &mut s,
        "stepper_x",
        json!({"position_endstop":spec.bed[0],"position_max":spec.bed[0],"homing_speed":80.0,"rotation_distance":40.0,"microsteps":32}),
    );
    section(
        &mut s,
        "stepper_y",
        json!({"position_endstop":spec.bed[1],"position_max":spec.bed[1],"homing_speed":80.0,"rotation_distance":40.0,"microsteps":32}),
    );
    let mut z = json!({"position_max":spec.max_z,"rotation_distance":if kit.level == Level::Gantry {40.0} else {8.0},"microsteps":32});
    if kit.endstop {
        z["position_endstop"] = json!(-0.5 + 0.2 * unit(kit.seed, 20));
    }
    section(&mut s, "stepper_z", z);
    for i in 1..kit.z_motors {
        section(
            &mut s,
            &format!("stepper_z{i}"),
            json!({"rotation_distance":if kit.level == Level::Gantry {40.0} else {8.0},"microsteps":32}),
        );
    }
    let r =
        |n: u64, lo: f64, span: f64| ((lo + span * unit(kit.seed, n)) * 1000.0).round() / 1000.0;
    section(
        &mut s,
        "extruder",
        json!({"nozzle_diameter":0.4,"filament_diameter":1.75,"rotation_distance":r(21, 22.4, 0.6),
        "gear_ratio":if kit.voron {"50:10"} else {""},"pressure_advance":(k.pa*10000.0).round()/10000.0,"pressure_advance_smooth_time":0.04,
        "control":"pid","pid_kp":r(22, 20.0, 9.0),"pid_ki":r(23, 1.1, 1.2),"pid_kd":r(24, 70.0, 40.0),"min_extrude_temp":170.0,"max_temp":300.0,"min_temp":0.0}),
    );
    section(
        &mut s,
        "heater_bed",
        json!({"control":"pid","pid_kp":r(25, 38.0, 30.0),"pid_ki":r(26, 0.9, 1.6),"pid_kd":r(27, 400.0, 900.0),"max_temp":120.0,"min_temp":0.0}),
    );
    section(
        &mut s,
        kit.probe,
        json!({"z_offset":r(28, 0.6, 1.8),"x_offset":0.0,"y_offset":if kit.probe == "bltouch" {-42.0} else {0.0},"speed":PROBE_SPEED,"lift_speed":LIFT_SPEED,"samples":1}),
    );
    let (lo, hi) = (
        kit.inset,
        (spec.bed[0] - kit.inset, spec.bed[1] - kit.inset),
    );
    section(
        &mut s,
        "bed_mesh",
        json!({"speed":TRAVEL_SPEED,"horizontal_move_z":HOP,"mesh_min":format!("{lo:.0},{lo:.0}"),"mesh_max":format!("{:.0},{:.0}",hi.0,hi.1),
        "probe_count":format!("{0},{0}",kit.grid),"algorithm":"bicubic"}),
    );
    let pts: Vec<String> = level_points(spec, kit)
        .iter()
        .map(|(x, y)| format!("{x:.0},{y:.0}"))
        .collect();
    match kit.level {
        Level::Gantry => section(
            &mut s,
            "quad_gantry_level",
            json!({"points":pts.join("\n"),"speed":TRAVEL_SPEED,"horizontal_move_z":HOP,"retries":5,"retry_tolerance":kit.tolerance,"max_adjust":10.0,
            "gantry_corners":format!("-60,-10\n{:.0},{:.0}",spec.bed[0]+60.0,spec.bed[1]+70.0)}),
        ),
        Level::Tilt => section(
            &mut s,
            "z_tilt",
            json!({"points":pts.join("\n"),"z_positions":format!("-17,-18\n{:.0},{:.0}\n{:.0},-18",spec.bed[0]/2.0,spec.bed[1]+29.0,spec.bed[0]+17.0),
            "speed":TRAVEL_SPEED,"horizontal_move_z":HOP,"retries":5,"retry_tolerance":kit.tolerance}),
        ),
        Level::Screws => {
            let (i, w, d) = (kit.inset + 20.0, spec.bed[0], spec.bed[1]);
            section(
                &mut s,
                "screws_tilt_adjust",
                json!({"screw1":format!("{i:.0},{i:.0}"),"screw1_name":"front left screw","screw2":format!("{:.0},{i:.0}",w-i),"screw2_name":"front right screw",
                "screw3":format!("{:.0},{:.0}",w-i,d-i),"screw3_name":"rear right screw","screw4":format!("{i:.0},{:.0}",d-i),"screw4_name":"rear left screw","speed":TRAVEL_SPEED,"screw_thread":"CW-M3"}),
            );
        }
    }
    section(
        &mut s,
        "input_shaper",
        json!({"shaper_type_x":"mzv","shaper_freq_x":r(29, 44.0, 20.0),"shaper_type_y":"mzv","shaper_freq_y":r(30, 34.0, 18.0)}),
    );
    section(&mut s, "fan", json!({}));
    section(
        &mut s,
        "heater_fan hotend_fan",
        json!({"heater":["extruder"],"heater_temp":50.0}),
    );
    section(&mut s, "controller_fan controller_fan", json!({}));
    section(
        &mut s,
        "filament_switch_sensor runout",
        json!({"pause_on_runout":true,"runout_gcode":"M117 Filament run-out","switch_pin":"^PG12"}),
    );
    for name in [
        "virtual_sdcard",
        "pause_resume",
        "display_status",
        "exclude_object",
        "respond",
        "idle_timeout",
    ] {
        section(&mut s, name, json!({}));
    }
    for (name, _, body) in macros(kit) {
        section(
            &mut s,
            &format!("gcode_macro {}", name.to_ascii_lowercase()),
            json!({"gcode":body}),
        );
    }
    s
}

/// printer.cfg as text, from the settings (the hotend fan edit downloads and changes it).
fn printer_cfg(settings: &Value) -> String {
    let mut out = String::from("# Stella Scope demo printer: this file is made by the demo; nothing here drives a real machine.\n");
    for (name, body) in settings.as_object().into_iter().flatten() {
        out.push_str(&format!("\n[{name}]\n"));
        for (k, v) in body.as_object().into_iter().flatten() {
            let text = match v {
                Value::String(t) => t.replace('\n', "\n  "),
                Value::Array(a) => a
                    .iter()
                    .filter_map(Value::as_str)
                    .collect::<Vec<_>>()
                    .join(", "),
                other => other.to_string(),
            };
            if text.contains('\n') {
                out.push_str(&format!("{k}:\n  {text}\n"));
            } else {
                out.push_str(&format!("{k}: {text}\n"));
            }
        }
    }
    out
}

// ------------------------------------------------------------------------------------------------
// The clock: restarts, routines, the job queue. Called at the end of every `Machine::advance`.
// ------------------------------------------------------------------------------------------------

pub fn tick(m: &Machine, s: &mut State, ts: f64) {
    if let Some((what, seen)) = s.k.armed {
        if matches!(s.job, Job::Heating | Job::Printing) {
            let since = seen.unwrap_or(ts);
            s.k.armed = Some((what, Some(since)));
            if ts - since >= ARMED_AFTER_S {
                s.k.armed = None;
                match what {
                    Trouble::Runout => runout(s, ts),
                    Trouble::HeaterFault => heater_fault(s, ts),
                }
            }
        }
    }
    if s.k.klippy == "startup" && ts >= s.k.back_at {
        s.k.klippy = "ready";
        s.k.message = "Printer is ready".into();
        s.k.info(ts, "Klipper state: Ready");
    }
    let Some(r) = s.k.routine.as_mut() else {
        queue_tick(m, s, ts);
        return;
    };
    let t = ts - r.t0;
    // where the head is on the path now
    let i = r.path.partition_point(|(pt, _)| *pt < t);
    let pos = match (i.checked_sub(1).and_then(|p| r.path.get(p)), r.path.get(i)) {
        (Some((t0, a)), Some((t1, b))) => {
            let f = if t1 > t0 {
                ((t - t0) / (t1 - t0)).clamp(0.0, 1.0)
            } else {
                1.0
            };
            [
                a[0] + (b[0] - a[0]) * f,
                a[1] + (b[1] - a[1]) * f,
                a[2] + (b[2] - a[2]) * f,
            ]
        }
        (_, Some((_, b))) => *b,
        (Some((_, a)), None) => *a,
        (None, None) => s.pos,
    };
    let mut due = Vec::new();
    while let Some((lt, text)) = r.lines.get(r.said) {
        if *lt > t {
            break;
        }
        due.push((r.t0 + lt, text.clone()));
        r.said += 1;
    }
    let swing = r.swing;
    let finished = t >= r.end;
    s.pos = pos;
    for (at, text) in due {
        s.k.info(at, &text);
    }
    if let Some((nozzle, target)) = swing {
        let h = if nozzle { &mut s.nozzle } else { &mut s.bed };
        if (h.0 - target).abs() < 6.0 {
            h.0 = target
                + 2.2 * (ts * std::f64::consts::TAU / if nozzle { 14.0 } else { 40.0 }).sin();
        }
    }
    if finished {
        let r = s.k.routine.take().expect("checked above");
        finish(m, s, r, ts);
    }
}

fn finish(m: &Machine, s: &mut State, r: Routine, ts: f64) {
    s.pos = r.path.last().map_or(s.pos, |p| p.1);
    let kit = &m.kit;
    let what = r.what;
    match r.done {
        Done::Nothing => {}
        Done::Homed => s.homed = true,
        Done::Levelled => s.k.applied = true,
        Done::Mesh(grid) => {
            s.k.profiles.insert("default".into(), grid.clone());
            s.k.mesh = Some(("default".into(), grid));
            s.k.info(ts, "Mesh Bed Leveling Complete\nBed Mesh state has been saved to profile [default]\nfor the current session.  The SAVE_CONFIG command will\nupdate the printer config file and restart the printer.");
            s.k.pend("bed_mesh default", "version", "1".into());
        }
        Done::Pid(heater, [kp, ki, kd]) => {
            s.k.info(ts, &format!("PID parameters: pid_Kp={kp:.3} pid_Ki={ki:.3} pid_Kd={kd:.3}\nThe SAVE_CONFIG command will update the printer config file\nwith these parameters and restart the printer."));
            for (key, v) in [
                ("control", "pid".to_owned()),
                ("pid_kp", format!("{kp:.3}")),
                ("pid_ki", format!("{ki:.3}")),
                ("pid_kd", format!("{kd:.3}")),
            ] {
                s.k.pend(heater, key, v);
            }
            if heater == "extruder" {
                s.nozzle.1 = 0.0;
            } else {
                s.bed.1 = 0.0;
            }
        }
        Done::Manual(what, z) => {
            s.k.manual = Some((what, z));
            s.k.info(ts, "Starting manual Z probe. Use TESTZ to adjust position.\nFinish with ACCEPT or ABORT command.");
        }
        Done::Shaper(axis, shaper, f) => {
            let a = axis.to_ascii_lowercase();
            s.k.info(
                ts,
                &format!("Recommended shaper_type_{a} = {shaper}, shaper_freq_{a} = {f:.1} Hz"),
            );
            s.k.pend("input_shaper", &format!("shaper_type_{a}"), shaper.into());
            s.k.pend(
                "input_shaper",
                &format!("shaper_freq_{a}"),
                format!("{f:.1}"),
            );
            s.k.info(ts, "The SAVE_CONFIG command will update the printer config file\nwith these parameters and restart the printer.");
        }
        Done::Loaded(load) => {
            s.k.filament = load;
            s.k.info(
                ts,
                match (what, load) {
                    ("extruding", _) => "Filament Sensor runout: filament detected",
                    (_, true) => "LOAD_FILAMENT finished",
                    _ => "UNLOAD_FILAMENT finished",
                },
            );
        }
    }
    let _ = kit;
}

/// Moonraker's job queue: when the printer is free, the next file starts (Moonraker waits a moment between jobs).
fn queue_tick(m: &Machine, s: &mut State, ts: f64) {
    if s.k.queue_paused || s.k.queue.is_empty() || !s.k.ready() || s.job.active() {
        return;
    }
    if matches!(s.job, Job::Complete | Job::Cancelled) && ts - s.finished_at < 5.0 {
        return;
    }
    let (_, file, _) = s.k.queue.remove(0);
    if let Some(i) = Demo::find(s, &file) {
        s.repeat = false;
        m.start(s, i, ts, 0.0);
        s.k.info(ts, &format!("Job queue: starting {file}"));
    }
}

// ------------------------------------------------------------------------------------------------
// The report: the Klipper objects a real one of these machines has, added to the demo's own.
// ------------------------------------------------------------------------------------------------

pub fn extend(m: &Machine, s: &State, v: &mut Value, ts: f64) {
    let k = &s.k;
    let kit = &m.kit;
    let shutdown = s.job == Job::Error || k.klippy == "shutdown";
    v["webhooks"] =
        json!({"state":if shutdown {"shutdown"} else {k.klippy},"state_message":k.message});
    v["gcode_move"]["homing_origin"] = json!([0.0, 0.0, k.z_adjust, 0.0]);
    v["gcode_move"]["gcode_position"] = json!([s.pos[0], s.pos[1], s.pos[2] - k.z_adjust, 0.0]);
    v["gcode_move"]["absolute_coordinates"] = json!(!k.relative);
    v["toolhead"]["max_velocity"] = json!(k.velocity);
    v["toolhead"]["max_accel"] = json!(k.accel);
    v["toolhead"]["square_corner_velocity"] = json!(k.scv);
    v["toolhead"]["axis_minimum"] = json!([0.0, 0.0, -2.0, 0.0]);
    v["extruder"]["pressure_advance"] = json!(k.pa);
    v["extruder"]["smooth_time"] = json!(0.04);
    v["extruder"]["can_extrude"] = json!(s.nozzle.0 >= 170.0);
    if let Some(r) = &k.routine {
        if r.extrude != 0.0 && ts - r.t0 >= r.extrude_from {
            v["motion_report"]["live_extruder_velocity"] = json!(r.extrude);
        }
        if s.job != Job::Printing {
            v["idle_timeout"]["state"] = json!("Printing");
        }
    }
    v["probe"] = json!({"name":k.probe_name(),"last_query":k.last_query,"last_z_result":k.last_z});
    let (min, max) = (kit.inset, [m.bed[0] - kit.inset, m.bed[1] - kit.inset]);
    let profiles: serde_json::Map<String, Value> = k
        .profiles
        .iter()
        .map(|(n, g)| {
            (
                n.clone(),
                json!({"points":g,"mesh_params":mesh_params(&m.spec, kit)}),
            )
        })
        .collect();
    v["bed_mesh"] = match &k.mesh {
        Some((name, grid)) => {
            json!({"profile_name":name,"mesh_min":[min,min],"mesh_max":max,"probed_matrix":grid,"mesh_matrix":grid,"profiles":profiles})
        }
        None => {
            json!({"profile_name":"","mesh_min":[min,min],"mesh_max":max,"probed_matrix":[[]],"mesh_matrix":[[]],"profiles":profiles})
        }
    };
    match kit.level {
        Level::Gantry => v["quad_gantry_level"] = json!({"applied":k.applied}),
        Level::Tilt => v["z_tilt"] = json!({"applied":k.applied}),
        Level::Screws => {}
    }
    v["manual_probe"] = match k.manual {
        Some((_, z)) => {
            json!({"is_active":true,"z_position":z,"z_position_lower":Value::Null,"z_position_upper":Value::Null})
        }
        None => {
            json!({"is_active":false,"z_position":Value::Null,"z_position_lower":Value::Null,"z_position_upper":Value::Null})
        }
    };
    v["filament_switch_sensor runout"] = json!({"filament_detected":k.filament,"enabled":true});
    let mut steppers = serde_json::Map::new();
    for name in ["stepper_x", "stepper_y", "stepper_z"] {
        steppers.insert(name.into(), json!(s.homed));
    }
    for i in 1..kit.z_motors {
        steppers.insert(format!("stepper_z{i}"), json!(s.homed));
    }
    steppers.insert("extruder".into(), json!(s.homed || s.nozzle.1 > 0.0));
    // the motor drivers (TMC2209 on every demo machine), as Klipper reports them: the Health page reads their current and standstill
    // ("No motor driver readings recorded yet" on every demo printer)
    let moving = k.routine.is_some() || s.job == Job::Printing;
    for (name, on) in &steppers {
        let run = if name.starts_with("stepper_z") {
            0.8
        } else if name == "extruder" {
            0.5
        } else {
            1.1
        };
        let cs = if !on.as_bool().unwrap_or(false) {
            0
        } else if moving {
            20 + (unit(kit.seed, name.len() as u64) * 8.0) as u64
        } else {
            9
        };
        v[format!("tmc2209 {name}")] = json!({"run_current":run,"hold_current":run * 0.6,
            "drv_status":{"cs_actual":cs,"stst":if moving { 0 } else { 1 },"otpw":0,"ot":0,"s2ga":0,"s2gb":0,"ola":0,"olb":0}});
    }
    v["stepper_enable"] = json!({"steppers":steppers});
    v["pause_resume"] = json!({"is_paused":s.job == Job::Paused});
    // the file being printed is one object, as a slicer with object labels declares it
    if let Some(f) = s.current.map(|i| &s.files[i]) {
        let b = f.plan.bounds;
        let name = object_name(&f.name);
        v["exclude_object"] = json!({"objects":[{"name":name,"center":[(b[0]+b[2])/2.0,(b[1]+b[3])/2.0],
            "polygon":[[b[0],b[1]],[b[2],b[1]],[b[2],b[3]],[b[0],b[3]],[b[0],b[1]]]}],
            "excluded_objects":k.excluded,"current_object":if s.job.active() && !k.excluded.contains(&name) {json!(name)} else {Value::Null}});
    } else {
        v["exclude_object"] =
            json!({"objects":[],"excluded_objects":[],"current_object":Value::Null});
    }
}

fn object_name(file: &str) -> String {
    let stem = file
        .rsplit('/')
        .next()
        .unwrap_or(file)
        .trim_end_matches(".gcode");
    stem.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_uppercase()
            } else {
                '_'
            }
        })
        .collect::<String>()
        + "_ID_0"
}

/// configfile as Klipper reports it, with what a SAVE_CONFIG would write.
pub fn configfile(s: &State) -> Value {
    let items: serde_json::Map<String, Value> =
        s.k.pending
            .iter()
            .map(|(sec, kv)| (sec.clone(), json!(kv)))
            .collect();
    json!({"settings":s.k.settings,"save_config_pending":!s.k.pending.is_empty(),"save_config_pending_items":items})
}

pub fn settings(s: &State) -> Value {
    s.k.settings.clone()
}

/// New console lines since the last time the collector asked (it records each once).
pub fn console_since(s: &mut State) -> Vec<crate::store::Event> {
    let from = s.k.read;
    s.k.read = s.k.seq;
    s.k.console
        .iter()
        .filter(|(n, ..)| *n > from)
        .map(|(_, ts, text, kind)| crate::store::Event::new(*ts, kind, text.clone()))
        .collect()
}

// ------------------------------------------------------------------------------------------------
// G-code: one line at a time, as Klipper runs it.
// ------------------------------------------------------------------------------------------------

/// What the caller has to wait for before the next line (Klipper runs a script's lines in order and answers when all ran).
enum Wait {
    Routine,
    Heat(bool, f64),
}

fn upper_words(line: &str) -> Vec<String> {
    line.split_whitespace().map(str::to_owned).collect()
}

/// `KEY=value` of an extended command (case-blind key).
fn param<'a>(words: &'a [String], key: &str) -> Option<&'a str> {
    words.iter().skip(1).find_map(|w| {
        let (k, v) = w.split_once('=')?;
        k.eq_ignore_ascii_case(key).then_some(v)
    })
}
fn num(words: &[String], key: &str) -> Option<f64> {
    param(words, key)
        .and_then(|v| v.parse().ok())
        .filter(|v: &f64| v.is_finite())
}
/// `S215` of a classic G-code.
fn letter(words: &[String], l: char) -> Option<f64> {
    words.iter().skip(1).find_map(|w| {
        let mut c = w.chars();
        (c.next()?.to_ascii_uppercase() == l)
            .then(|| c.as_str().parse().ok())
            .flatten()
    })
}

fn routine(s: &mut State, r: Routine) -> Option<Wait> {
    s.k.routine = Some(r);
    Some(Wait::Routine)
}

/// The probe reading a level round gets at each point: the gantry's error there, shrinking each round.
fn level_round(m: &Machine, s: &State, p: &mut Path, round: usize, range: f64, last: bool) {
    let kit = &m.kit;
    let pts = level_points(&m.spec, kit);
    let base = 1.5 + 0.3 * unit(kit.seed, 40);
    let raw: Vec<f64> = pts
        .iter()
        .enumerate()
        .map(|(i, _)| s.k.tilt.get(i).copied().unwrap_or(0.0))
        .collect();
    let (lo, hi) = raw
        .iter()
        .fold((f64::MAX, f64::MIN), |(a, b), v| (a.min(*v), b.max(*v)));
    let scale = if hi - lo > 1e-9 {
        range / (hi - lo)
    } else {
        0.0
    };
    let z: Vec<f64> = raw.iter().map(|v| base + (v - lo) * scale).collect();
    for ((x, y), zz) in pts.iter().zip(&z) {
        p.probe(*x, *y, *zz);
    }
    let mean = z.iter().sum::<f64>() / z.len() as f64;
    let names: Vec<String> = (0..z.len())
        .map(|i| {
            if i == 0 {
                "stepper_z".into()
            } else {
                format!("stepper_z{i}")
            }
        })
        .collect();
    let adjust: Vec<String> = names
        .iter()
        .zip(&z)
        .map(|(n, v)| format!("{n} = {:.6}", mean - v))
        .collect();
    let rel: Vec<String> = z
        .iter()
        .enumerate()
        .map(|(i, v)| format!("{i}: {:.6}", 15.4 + v - base))
        .collect();
    let head = if kit.level == Level::Gantry {
        "Gantry-relative probe points:\n".to_owned() + &rel.join(" ") + "\n"
    } else {
        String::new()
    };
    let tail = if last { String::new() } else { String::new() };
    p.say(format!("{head}Making the following Z adjustments:\n{}\nRetries: {round}/5 Probed points range: {range:.6} tolerance: {:.6}{tail}", adjust.join("\n"), kit.tolerance));
}

fn home_path(m: &Machine, s: &State, p: &mut Path) {
    let home = m.spec.frame.home(m.bed);
    let at = p.at;
    // X, then Y, then Z at the middle of the bed (a probe homes Z there), then up to the travel height
    p.go([home[0], at[1], at[2].max(HOP)], 80.0);
    p.go([home[0], home[1], at[2].max(HOP)], 80.0);
    if m.kit.endstop {
        p.go([home[0], home[1], 0.0], 12.0);
    } else {
        p.go(
            [m.bed[0] / 2.0, m.bed[1] / 2.0, at[2].max(HOP)],
            TRAVEL_SPEED,
        );
        p.go([m.bed[0] / 2.0, m.bed[1] / 2.0, 0.6], PROBE_SPEED);
    }
    p.go([home[0], home[1], home[2]], 15.0);
    let _ = s;
}

/// Run one line. Ok(None): done now. Ok(Some(wait)): started; the caller waits. Err: Klipper's error, already in the console.
fn line(
    m: &Machine,
    s: &mut State,
    ts: f64,
    raw: &str,
) -> std::result::Result<Option<Wait>, String> {
    let words = upper_words(raw);
    let Some(head) = words.first().map(|w| w.to_ascii_uppercase()) else {
        return Ok(None);
    };
    let kit = m.kit.clone();
    let spec = m.spec.clone();
    let vmax = s.k.velocity;
    if !s.k.ready()
        && !matches!(
            head.as_str(),
            "FIRMWARE_RESTART" | "RESTART" | "STATUS" | "HELP"
        )
    {
        return Err(if s.k.klippy == "startup" {
            "Klipper is starting up; try again in a moment".into()
        } else {
            format!("{}\nOnce the underlying issue is corrected, use the \"FIRMWARE_RESTART\" command to reset the firmware, reload the config, and restart the host software.", s.k.message)
        });
    }
    let moving_print = s.job == Job::Printing || s.job == Job::Heating;
    let needs_home = |s: &State| -> std::result::Result<(), String> {
        if s.homed {
            Ok(())
        } else {
            Err(format!(
                "Must home axis first: {:.3} {:.3} {:.3} [0.000]",
                s.pos[0], s.pos[1], s.pos[2]
            ))
        }
    };
    let new_path = |s: &State| Path::from(s.pos, vmax);
    match head.as_str() {
        "G28" => {
            if moving_print {
                return Err("the demo printer is printing; pause it to move the head".into());
            }
            let mut p = new_path(s);
            home_path(m, s, &mut p);
            Ok(routine(s, p.into_routine("homing", ts, Done::Homed)))
        }
        "G90" => {
            s.k.relative = false;
            Ok(None)
        }
        "G91" => {
            s.k.relative = true;
            Ok(None)
        }
        "M82"
        | "M83"
        | "G92"
        | "M400"
        | "SAVE_GCODE_STATE"
        | "RESTORE_GCODE_STATE"
        | "M84"
        | "M18"
            if head != "M84" && head != "M18" =>
        {
            Ok(None)
        }
        "M84" | "M18" => {
            if moving_print {
                return Err("the demo printer is printing".into());
            }
            s.homed = false;
            s.k.applied = false;
            Ok(None)
        }
        "G0" | "G1" => {
            let e = letter(&words, 'E');
            let mut to = s.pos;
            let mut any = false;
            for (i, l) in ['X', 'Y', 'Z'].into_iter().enumerate() {
                if let Some(v) = letter(&words, l) {
                    to[i] = if s.k.relative { s.pos[i] + v } else { v };
                    any = true;
                }
            }
            if any && moving_print {
                return Err("the demo printer is printing; pause it to move the head".into());
            }
            if any {
                needs_home(s)?;
            }
            let max = [m.bed[0], m.bed[1], m.max_z];
            if (0..3).any(|i| to[i] < if i == 2 { -2.0 } else { 0.0 } || to[i] > max[i]) {
                return Err(format!(
                    "Move out of range: {:.3} {:.3} {:.3} [0.000]",
                    to[0], to[1], to[2]
                ));
            }
            if e.is_some() && s.nozzle.0 < 170.0 {
                return Err("Extrude below minimum temp\nSee the 'min_extrude_temp' config option for details".into());
            }
            let feed = letter(&words, 'F').map_or(50.0, |f| (f / 60.0).max(0.5));
            let mut p = new_path(s);
            let mut what = "moving";
            if any {
                p.go(to, feed);
            }
            let mut rate = 0.0;
            if let Some(e) = e.filter(|e| *e != 0.0) {
                let secs = (e.abs() / feed).max(0.2);
                if !any {
                    p.wait(secs);
                }
                rate = e / p.t.max(secs);
                what = if e > 0.0 { "extruding" } else { "retracting" };
            }
            // filament pushed in after a run-out is what the sensor sees again (on a real printer, the new spool's end)
            let refill = rate > 0.0 && !s.k.filament;
            let mut r = p.into_routine(
                what,
                ts,
                if refill {
                    Done::Loaded(true)
                } else {
                    Done::Nothing
                },
            );
            r.extrude = rate;
            if r.end <= 0.0 {
                return Ok(None);
            }
            Ok(routine(s, r))
        }
        "M104" | "M109" | "M140" | "M190" => {
            let nozzle = head == "M104" || head == "M109";
            let t = letter(&words, 'S').unwrap_or(0.0);
            let max = if nozzle { 300.0 } else { 120.0 };
            if !(0.0..=max).contains(&t) {
                return Err(format!(
                    "Requested temperature ({t:.1}) out of range (0.0:{max:.1})"
                ));
            }
            if nozzle {
                s.nozzle.1 = t
            } else {
                s.bed.1 = t
            }
            Ok((head == "M109" || head == "M190")
                .then_some(Wait::Heat(nozzle, t))
                .filter(|_| t > 0.0))
        }
        "SET_HEATER_TEMPERATURE" => {
            let heater = param(&words, "HEATER").unwrap_or("").to_ascii_lowercase();
            let t = num(&words, "TARGET").unwrap_or(0.0);
            match heater.as_str() {
                "extruder" if (0.0..=300.0).contains(&t) => s.nozzle.1 = t,
                "heater_bed" if (0.0..=120.0).contains(&t) => s.bed.1 = t,
                "extruder" | "heater_bed" => {
                    return Err(format!("Requested temperature ({t:.1}) out of range"))
                }
                _ => return Err(format!("Unknown heater \"{heater}\"")),
            }
            Ok(None)
        }
        "TURN_OFF_HEATERS" => {
            s.nozzle.1 = 0.0;
            s.bed.1 = 0.0;
            Ok(None)
        }
        "TEMPERATURE_WAIT" => Ok(None),
        "M106" => {
            s.fan = (letter(&words, 'S').unwrap_or(255.0) / 255.0).clamp(0.0, 1.0);
            Ok(None)
        }
        "M107" => {
            s.fan = 0.0;
            Ok(None)
        }
        "M220" => {
            s.speed = letter(&words, 'S').unwrap_or(100.0).clamp(10.0, 500.0);
            Ok(None)
        }
        "M221" => {
            s.flow = letter(&words, 'S').unwrap_or(100.0).clamp(10.0, 500.0);
            Ok(None)
        }
        "M114" => {
            s.k.info(
                ts,
                &format!(
                    "X:{:.3} Y:{:.3} Z:{:.3} E:0.000",
                    s.pos[0],
                    s.pos[1],
                    s.pos[2] - s.k.z_adjust
                ),
            );
            Ok(None)
        }
        "M115" => {
            s.k.info(ts, "FIRMWARE_NAME:Klipper FIRMWARE_VERSION:v0.12.0-demo");
            Ok(None)
        }
        "M117" | "ECHO" | "RESPOND" => {
            let msg = raw
                .split_once(char::is_whitespace)
                .map_or("", |(_, r)| r)
                .trim();
            let msg = msg
                .strip_prefix("MSG=")
                .or_else(|| msg.strip_prefix("msg="))
                .unwrap_or(msg)
                .trim_matches('"');
            if head != "M117" {
                s.k.info(ts, &format!("echo: {msg}"));
            }
            Ok(None)
        }
        "GET_POSITION" => {
            let [x, y, z] = s.pos;
            s.k.info(ts, &format!("mcu: stepper_x:{:.0} stepper_y:{:.0} stepper_z:{:.0}\nstepper: stepper_x:{x:.6} stepper_y:{y:.6} stepper_z:{z:.6}\nkinematic: X:{x:.6} Y:{y:.6} Z:{z:.6}\ntoolhead: X:{x:.6} Y:{y:.6} Z:{z:.6} E:0.000000\ngcode: X:{x:.6} Y:{y:.6} Z:{:.6} E:0.000000\ngcode base: X:0.000000 Y:0.000000 Z:{:.6} E:0.000000\ngcode homing: X:0.000000 Y:0.000000 Z:{:.6}",
                x * 80.0, y * 80.0, z * 400.0, z - s.k.z_adjust, s.k.z_adjust, s.k.z_adjust));
            Ok(None)
        }
        "STATUS" => {
            let st = s.k.klippy;
            s.k.info(
                ts,
                &format!(
                    "Klipper state: {}",
                    if st == "ready" { "Ready" } else { st }
                ),
            );
            Ok(None)
        }
        "HELP" => {
            let mut text = String::from("Available extended commands:");
            for (n, h) in help(&kit).as_object().into_iter().flatten() {
                text.push_str(&format!("\n{n}: {}", h.as_str().unwrap_or("")));
            }
            s.k.info(ts, &text);
            Ok(None)
        }
        "PAUSE" | "M600" => {
            if s.job.active() && s.job != Job::Paused {
                s.job = Job::Paused;
            } else if !s.job.active() {
                return Err("Print is not active".into());
            }
            Ok(None)
        }
        "RESUME" => {
            if s.job != Job::Paused {
                return Err("Print is not paused, resume aborted".into());
            }
            if !s.k.filament {
                return Err("Filament not detected: load filament before resuming".into());
            }
            s.job = Job::Printing;
            Ok(None)
        }
        "CANCEL_PRINT" => {
            if s.job.active() {
                s.job = Job::Cancelled;
                s.repeat = false;
                s.nozzle.1 = 0.0;
                s.bed.1 = 0.0;
                s.fan = 0.0;
                s.finished_at = ts;
            }
            Ok(None)
        }
        "SET_GCODE_OFFSET" => {
            let before = s.k.z_adjust;
            if let Some(z) = num(&words, "Z") {
                s.k.z_adjust = z;
            }
            if let Some(dz) = num(&words, "Z_ADJUST") {
                s.k.z_adjust += dz;
            }
            s.k.z_adjust = (s.k.z_adjust * 1000.0).round() / 1000.0;
            if param(&words, "MOVE") == Some("1") && s.homed && !moving_print {
                s.pos[2] = (s.pos[2] + s.k.z_adjust - before).clamp(-2.0, m.max_z);
            }
            Ok(None)
        }
        "Z_OFFSET_APPLY_PROBE" => {
            if s.k.z_adjust == 0.0 {
                return Err("Nothing to do: Z Offset is 0".into());
            }
            let name = s.k.probe_name().to_owned();
            let new = ((s.k.probe_offset() - s.k.z_adjust) * 1000.0).round() / 1000.0;
            s.k.pend(&name, "z_offset", format!("{new:.3}"));
            s.k.info(ts, &format!("{name}: z_offset: {new:.3}\nThe SAVE_CONFIG command will update the printer config file\nwith the above and restart the printer."));
            Ok(None)
        }
        "SAVE_CONFIG" => {
            for (sec, kv) in std::mem::take(&mut s.k.pending) {
                if sec.starts_with("bed_mesh ") {
                    continue;
                }
                for (k, v) in kv {
                    let value = v.parse::<f64>().map(|n| json!(n)).unwrap_or(json!(v));
                    if !s.k.settings[&sec].is_object() {
                        s.k.settings[&sec] = json!({});
                    }
                    s.k.settings[&sec][&k] = value;
                }
            }
            if let Some(pa) = s.k.settings["extruder"]["pressure_advance"].as_f64() {
                s.k.pa = pa;
            }
            restart(m, s, ts, false);
            Ok(None)
        }
        "FIRMWARE_RESTART" | "RESTART" => {
            restart(m, s, ts, head == "FIRMWARE_RESTART");
            Ok(None)
        }
        "QUAD_GANTRY_LEVEL" | "Z_TILT_ADJUST" => {
            let want = if head == "QUAD_GANTRY_LEVEL" {
                Level::Gantry
            } else {
                Level::Tilt
            };
            if kit.level != want {
                return Err(format!("Unknown command:\"{head}\""));
            }
            if moving_print {
                return Err(
                    "the demo printer is printing; levelling waits for the print to end".into(),
                );
            }
            let mut p = new_path(s);
            let was_homed = s.homed;
            if !was_homed {
                home_path(m, s, &mut p);
            }
            // A levelled gantry is checked once and passes; one that sagged converges round by round.
            let mut range = if s.k.applied {
                0.002 + 0.004 * unit(kit.seed, 50 + s.k.restarts)
            } else {
                let [a, b, c, d] = s.k.tilt;
                let (lo, hi) = [a, b, c, d]
                    .iter()
                    .fold((f64::MAX, f64::MIN), |(lo, hi), v| (lo.min(*v), hi.max(*v)));
                hi - lo
            };
            let mut round = 0;
            loop {
                let last = range < kit.tolerance || round == 5;
                level_round(m, s, &mut p, round, range, last);
                if last {
                    break;
                }
                range *= 0.06 + 0.06 * unit(kit.seed, 60 + round as u64);
                round += 1;
            }
            let home = spec.frame.home(m.bed);
            p.go([home[0], home[1], 10.0], TRAVEL_SPEED);
            let mut r = p.into_routine("levelling", ts, Done::Levelled);
            if !was_homed {
                // homed part way: the levelling needs it
                s.homed = true;
            }
            r.what = "levelling";
            Ok(routine(s, r))
        }
        "SCREWS_TILT_CALCULATE" => {
            if kit.level != Level::Screws {
                return Err(format!("Unknown command:\"{head}\""));
            }
            needs_home(s)?;
            let mut p = new_path(s);
            let set = &s.k.settings["screws_tilt_adjust"];
            let pts: Vec<(f64, f64, String)> = (1..=4)
                .filter_map(|i| {
                    let (x, y) = set[format!("screw{i}")].as_str()?.split_once(',')?;
                    Some((
                        x.trim().parse().ok()?,
                        y.trim().parse().ok()?,
                        set[format!("screw{i}_name")]
                            .as_str()
                            .unwrap_or("screw")
                            .to_owned(),
                    ))
                })
                .collect();
            let base = 2.4 + 0.2 * unit(kit.seed, 70);
            let zs: Vec<f64> = pts
                .iter()
                .map(|(x, y, _)| base + surface(&spec, &kit, *x, *y) + tilt_at(&spec, &s.k, *x, *y))
                .collect();
            for ((x, y, _), z) in pts.iter().zip(&zs) {
                p.probe(*x, *y, *z);
            }
            let mut out = String::from(
                "01:20 means 1 full turn and 20 minutes, CW=clockwise, CCW=counter-clockwise",
            );
            for (i, ((x, y, name), z)) in pts.iter().zip(&zs).enumerate() {
                if i == 0 {
                    out.push_str(&format!("\n{name} (base) : x={x:.1}, y={y:.1}, z={z:.5}"));
                } else {
                    let d = z - zs[0];
                    let minutes = (d.abs() / 0.5 * 60.0).round() as i64;
                    out.push_str(&format!(
                        "\n{name} : x={x:.1}, y={y:.1}, z={z:.5} : adjust {} {:02}:{:02}",
                        if d > 0.0 { "CW" } else { "CCW" },
                        minutes / 60,
                        minutes % 60
                    ));
                }
            }
            p.say(out);
            Ok(routine(s, p.into_routine("levelling", ts, Done::Nothing)))
        }
        "BED_MESH_CALIBRATE" => {
            if moving_print {
                return Err(
                    "the demo printer is printing; a mesh waits for the print to end".into(),
                );
            }
            let mut p = new_path(s);
            if !s.homed {
                home_path(m, s, &mut p);
                s.homed = true;
            }
            let grid = mesh_grid(&spec, &kit, &s.k, false);
            let (xs, ys) = mesh_points(&spec, &kit);
            let base = 1.4 + 0.4 * unit(kit.seed, 80);
            for (j, y) in ys.iter().enumerate() {
                // a snake, as Klipper probes it
                let row: Vec<(usize, &f64)> = if j % 2 == 0 {
                    xs.iter().enumerate().collect()
                } else {
                    xs.iter().enumerate().rev().collect()
                };
                for (i, x) in row {
                    p.probe(*x, *y, base + grid[j][i]);
                }
            }
            p.say(format!("Mesh Bed Leveling Complete"));
            let mut r = p.into_routine("meshing", ts, Done::Mesh(grid));
            r.lines.pop();
            Ok(routine(s, r))
        }
        "BED_MESH_PROFILE" => {
            if let Some(name) = param(&words, "LOAD") {
                let g =
                    s.k.profiles
                        .get(name)
                        .cloned()
                        .ok_or_else(|| format!("bed_mesh: Unknown profile [{name}]"))?;
                s.k.mesh = Some((name.into(), g));
            } else if let Some(name) = param(&words, "SAVE") {
                let (_, g) = s.k.mesh.clone().ok_or("Bed has not been probed")?;
                s.k.profiles.insert(name.into(), g);
                s.k.pend(&format!("bed_mesh {name}"), "version", "1".into());
                s.k.info(ts, &format!("Bed Mesh state has been saved to profile [{name}]\nfor the current session.  The SAVE_CONFIG command will\nupdate the printer config file and restart the printer."));
            } else if let Some(name) = param(&words, "REMOVE") {
                s.k.profiles
                    .remove(name)
                    .ok_or_else(|| format!("bed_mesh: Unknown profile [{name}]"))?;
                s.k.info(ts, &format!("Profile [{name}] removed from storage for this session.\nThe SAVE_CONFIG command will update the printer\nconfiguration and restart the printer"));
            } else {
                return Err("Invalid syntax 'BED_MESH_PROFILE'".into());
            }
            Ok(None)
        }
        "BED_MESH_CLEAR" => {
            s.k.mesh = None;
            Ok(None)
        }
        "BED_MESH_OUTPUT" => {
            let text = match &s.k.mesh {
                Some((n, g)) => format!(
                    "Mesh Leveling Probed Z positions ({n}):\n{}",
                    g.iter()
                        .rev()
                        .map(|r| r
                            .iter()
                            .map(|v| format!("{v:.6}"))
                            .collect::<Vec<_>>()
                            .join(" "))
                        .collect::<Vec<_>>()
                        .join("\n")
                ),
                None => "Bed has not been probed".into(),
            };
            s.k.info(ts, &text);
            Ok(None)
        }
        "PROBE" | "PROBE_ACCURACY" => {
            needs_home(s)?;
            let mut p = new_path(s);
            let (x, y) = (s.pos[0], s.pos[1]);
            let z0 = s.k.probe_offset() + surface(&spec, &kit, x, y) + tilt_at(&spec, &s.k, x, y);
            if head == "PROBE" {
                p.probe(x, y, z0);
                s.k.last_z = z0;
                p.say(format!("Result is z={z0:.6}"));
            } else {
                let n = num(&words, "SAMPLES").map_or(10, |n| n.clamp(1.0, 50.0) as usize);
                p.say(format!("PROBE_ACCURACY at X:{x:.3} Y:{y:.3} Z:{:.3} (samples={n} retract={HOP:.3} speed={PROBE_SPEED:.1} lift_speed={LIFT_SPEED:.1})", s.pos[2]));
                let zs: Vec<f64> = (0..n)
                    .map(|i| z0 + 0.006 * (unit(kit.seed, 90 + i as u64) - 0.5))
                    .collect();
                for z in &zs {
                    p.probe(x, y, *z);
                }
                let mut sorted = zs.clone();
                sorted.sort_by(f64::total_cmp);
                let mean = zs.iter().sum::<f64>() / n as f64;
                let sd = (zs.iter().map(|z| (z - mean).powi(2)).sum::<f64>() / n as f64).sqrt();
                p.say(format!("probe accuracy results: maximum {:.6}, minimum {:.6}, range {:.6}, average {mean:.6}, median {:.6}, standard deviation {sd:.6}",
                    sorted[n - 1], sorted[0], sorted[n - 1] - sorted[0], sorted[n / 2]));
                s.k.last_z = mean;
            }
            Ok(routine(s, p.into_routine("probing", ts, Done::Nothing)))
        }
        "QUERY_PROBE" => {
            s.k.last_query = false;
            s.k.info(ts, "probe: open");
            Ok(None)
        }
        "PROBE_CALIBRATE" | "Z_ENDSTOP_CALIBRATE" => {
            if head == "Z_ENDSTOP_CALIBRATE" && !kit.endstop {
                return Err(format!("Unknown command:\"{head}\""));
            }
            if s.k.manual.is_some() {
                return Err("Already in a manual Z probe. Use ABORT to abort it.".into());
            }
            needs_home(s)?;
            let mut p = new_path(s);
            let (cx, cy) = (m.bed[0] / 2.0, m.bed[1] / 2.0);
            if head == "PROBE_CALIBRATE" {
                p.probe(cx, cy, s.k.probe_offset());
            }
            p.go([cx, cy, 5.0], TRAVEL_SPEED);
            let what = if head == "PROBE_CALIBRATE" {
                "probe"
            } else {
                "endstop"
            };
            Ok(routine(
                s,
                p.into_routine("calibrating", ts, Done::Manual(what, 5.0)),
            ))
        }
        "TESTZ" => {
            let Some((what, z)) = s.k.manual else {
                return Err("Unknown command:\"TESTZ\"".into());
            };
            let dz = num(&words, "Z").ok_or("Error on 'TESTZ': missing Z")?;
            let nz = (z + dz).max(-1.0);
            s.k.manual = Some((what, nz));
            s.pos[2] = nz;
            s.k.info(ts, &format!("Z position: ?????? --> {nz:.3} <-- ??????"));
            Ok(None)
        }
        "ACCEPT" | "ABORT" => {
            let Some((what, z)) = s.k.manual.take() else {
                return Err(format!("Unknown command:\"{head}\""));
            };
            if head == "ABORT" {
                s.k.info(ts, "Manual probe failed");
                return Ok(None);
            }
            // the paper test says the nozzle touches at `z`: that is the new zero
            if what == "probe" {
                let name = s.k.probe_name().to_owned();
                let new = ((s.k.probe_offset() - z + 0.1) * 1000.0).round() / 1000.0;
                s.k.pend(&name, "z_offset", format!("{new:.3}"));
                s.k.info(ts, &format!("{name}: z_offset: {new:.3}\nThe SAVE_CONFIG command will update the printer config file\nwith the above and restart the printer."));
            } else {
                let old = s.k.settings["stepper_z"]["position_endstop"]
                    .as_f64()
                    .unwrap_or(0.0);
                let new = ((old - z + 0.1) * 1000.0).round() / 1000.0;
                s.k.pend("stepper_z", "position_endstop", format!("{new:.3}"));
                s.k.info(ts, &format!("stepper_z: position_endstop: {new:.3}\nThe SAVE_CONFIG command will update the printer config file\nwith the above and restart the printer."));
            }
            Ok(None)
        }
        "PID_CALIBRATE" => {
            let heater = param(&words, "HEATER").unwrap_or("").to_ascii_lowercase();
            let target = num(&words, "TARGET").ok_or("Error on 'PID_CALIBRATE': missing TARGET")?;
            let (nozzle, name): (bool, &'static str) = match heater.as_str() {
                "extruder" => (true, "extruder"),
                "heater_bed" => (false, "heater_bed"),
                _ => return Err(format!("Unknown heater \"{heater}\"")),
            };
            if nozzle {
                s.nozzle.1 = target
            } else {
                s.bed.1 = target
            }
            let now = if nozzle { s.nozzle.0 } else { s.bed.0 };
            let tau = if nozzle { m.nozzle_tau } else { m.bed_tau };
            let warm = tau
                * ((target - AMBIENT) / (now - AMBIENT - (target - AMBIENT)).abs().max(1.0))
                    .abs()
                    .ln()
                    .clamp(1.0, 4.0);
            let mut p = new_path(s);
            p.wait(warm + if nozzle { 5.0 * 14.0 } else { 3.0 * 40.0 } * 0.6);
            let r3 =
                |n: u64, lo: f64, span: f64| lo + span * unit(kit.seed, n + 100 + s.k.restarts);
            let vals = if nozzle {
                [r3(1, 20.0, 9.0), r3(2, 1.1, 1.2), r3(3, 70.0, 40.0)]
            } else {
                [r3(4, 38.0, 30.0), r3(5, 0.9, 1.6), r3(6, 400.0, 900.0)]
            };
            let mut r = p.into_routine("tuning", ts, Done::Pid(name, vals));
            r.swing = Some((nozzle, target));
            Ok(routine(s, r))
        }
        "SET_VELOCITY_LIMIT" => {
            if let Some(v) = num(&words, "VELOCITY") {
                s.k.velocity = v;
            }
            if let Some(v) = num(&words, "ACCEL") {
                s.k.accel = v;
            }
            if let Some(v) = num(&words, "SQUARE_CORNER_VELOCITY") {
                s.k.scv = v;
            }
            let (v, a, c) = (s.k.velocity, s.k.accel, s.k.scv);
            s.k.info(ts, &format!("max_velocity: {v:.6}\nmax_accel: {a:.6}\nminimum_cruise_ratio: 0.500000\nsquare_corner_velocity: {c:.6}"));
            Ok(None)
        }
        "SET_PRESSURE_ADVANCE" => {
            if let Some(v) = num(&words, "ADVANCE") {
                s.k.pa = v;
            }
            let pa = s.k.pa;
            s.k.info(
                ts,
                &format!("pressure_advance: {pa:.6}\npressure_advance_smooth_time: 0.040000"),
            );
            Ok(None)
        }
        "TUNING_TOWER" => {
            let start = num(&words, "START").unwrap_or(0.0);
            let factor = num(&words, "FACTOR").unwrap_or(0.0);
            s.k.info(
                ts,
                &format!("Starting tuning test (start={start:.6} factor={factor:.6})"),
            );
            Ok(None)
        }
        "TEST_RESONANCES" | "SHAPER_CALIBRATE" => {
            if moving_print {
                return Err(
                    "the demo printer is printing; a resonance test waits for the print to end"
                        .into(),
                );
            }
            needs_home(s)?;
            let axis = param(&words, "AXIS")
                .unwrap_or("X")
                .to_ascii_uppercase()
                .chars()
                .next()
                .unwrap_or('X');
            let (fx, fy) = (
                44.0 + 20.0 * unit(kit.seed, 29),
                34.0 + 18.0 * unit(kit.seed, 30),
            );
            let f0 = if axis == 'Y' { fy } else { fx };
            let mut p = new_path(s);
            let c = [m.bed[0] / 2.0, m.bed[1] / 2.0, 20.0];
            p.go(c, TRAVEL_SPEED);
            // the sweep: a short shake that gets quicker, 5 Hz to 133 Hz, as Klipper runs it (shortened for the demo)
            let i = if axis == 'Y' { 1 } else { 0 };
            let mut f = 5.0f64;
            while f <= 133.0 {
                if (f as i64) % 10 == 5 {
                    p.say(format!("Testing frequency {f:.0} Hz"));
                }
                let amp = (15.0f64 / f).max(0.3);
                for side in [1.0, -1.0] {
                    let mut to = c;
                    to[i] += amp * side;
                    p.pts.push((p.t + 0.04, to));
                    p.t += 0.04;
                }
                f += 2.0;
            }
            p.pts.push((p.t + 0.1, c));
            p.t += 0.1;
            p.at = c;
            p.say("Wait for calculations..".to_owned());
            let done = if head == "SHAPER_CALIBRATE" {
                let shapers = [
                    ("zv", 1.08),
                    ("mzv", 1.0),
                    ("ei", 1.18),
                    ("2hump_ei", 1.4),
                    ("3hump_ei", 1.63),
                ];
                for (n, (name, k)) in shapers.iter().enumerate() {
                    let fr = f0 * k;
                    p.say(format!("Fitted shaper '{name}' frequency = {fr:.1} Hz (vibrations = {:.1}%, smoothing ~= {:.3})\nTo avoid too much smoothing with '{name}', suggested max_accel <= {:.0} mm/sec^2",
                        [4.8, 1.9, 2.4, 0.6, 0.2][n], [0.05, 0.07, 0.09, 0.13, 0.18][n], 12000.0 / k));
                }
                Done::Shaper(axis, "mzv", f0)
            } else {
                p.say(format!(
                    "Resonances data written to /tmp/resonances_{}_demo.csv file",
                    axis.to_ascii_lowercase()
                ));
                Done::Nothing
            };
            Ok(routine(s, p.into_routine("shaking", ts, done)))
        }
        "EXCLUDE_OBJECT" => {
            if let Some(name) = param(&words, "NAME") {
                let name = name.to_ascii_uppercase();
                if !s.k.excluded.contains(&name) {
                    s.k.excluded.push(name.clone());
                }
                s.k.info(ts, &format!("Excluding object {name}"));
            } else if words.iter().any(|w| w.eq_ignore_ascii_case("RESET=1")) {
                s.k.excluded.clear();
            }
            Ok(None)
        }
        "EXCLUDE_OBJECT_DEFINE" => Ok(None),
        "SET_FILAMENT_SENSOR" => Ok(None),
        "QUERY_FILAMENT_SENSOR" => {
            let f = s.k.filament;
            s.k.info(
                ts,
                &format!(
                    "Filament Sensor runout: {}",
                    if f {
                        "filament detected"
                    } else {
                        "filament not detected"
                    }
                ),
            );
            Ok(None)
        }
        "LOAD_FILAMENT" | "UNLOAD_FILAMENT" => {
            if moving_print {
                return Err("the demo printer is printing; pause it to change filament".into());
            }
            let load = head == "LOAD_FILAMENT";
            let t = num(&words, "TEMP").unwrap_or(220.0).clamp(170.0, 300.0);
            s.nozzle.1 = t;
            let now = s.nozzle.0;
            let heat = if now >= t - 3.0 {
                0.0
            } else {
                m.nozzle_tau * ((t - AMBIENT) / (t - now).max(1.0)).ln().max(0.5) + 1.0
            };
            let mut p = new_path(s);
            p.wait(heat);
            let push_s = if load { 50.0 / 5.0 } else { 100.0 / 20.0 };
            let start = p.t;
            p.wait(push_s);
            let mut r = p.into_routine(
                if load { "loading" } else { "unloading" },
                ts,
                Done::Loaded(load),
            );
            // filament moves once the nozzle is hot
            r.extrude = if load { 5.0 } else { -20.0 };
            r.extrude_from = start / SCALE;
            r.lines.push((
                start / SCALE,
                format!(
                    "{} {:.0} mm",
                    if load { "Loading" } else { "Unloading" },
                    if load { 50.0 } else { 100.0 }
                ),
            ));
            Ok(routine(s, r))
        }
        "SIMULATE_RUNOUT" | "SIMULATE_HEATER_FAULT" => {
            let what = if head == "SIMULATE_RUNOUT" {
                Trouble::Runout
            } else {
                Trouble::HeaterFault
            };
            if s.job.active() {
                match what {
                    Trouble::Runout => runout(s, ts),
                    Trouble::HeaterFault => heater_fault(s, ts),
                }
            } else {
                s.k.armed = Some((what, None));
                s.k.info(
                    ts,
                    &format!(
                        "Demo printer: {} {ARMED_AFTER_S:.0} s into the next print.",
                        if what == Trouble::Runout {
                            "the spool runs out"
                        } else {
                            "the hotend stops heating"
                        }
                    ),
                );
            }
            Ok(None)
        }
        "PRINT_START" => {
            let bed = num(&words, "BED").unwrap_or(60.0).clamp(0.0, 120.0);
            let ext = num(&words, "EXTRUDER").unwrap_or(220.0).clamp(0.0, 300.0);
            s.bed.1 = bed;
            s.nozzle.1 = ext;
            let mut p = new_path(s);
            home_path(m, s, &mut p);
            let heat = m.bed_tau.max(m.nozzle_tau) * 2.0;
            p.wait(heat);
            Ok(routine(s, p.into_routine("heating", ts, Done::Homed)))
        }
        "PRINT_END" | "PARK" | "CLEAN_NOZZLE" => {
            needs_home(s)?;
            if head == "PRINT_END" {
                s.nozzle.1 = 0.0;
                s.bed.1 = 0.0;
                s.fan = 0.0;
            }
            let mut p = new_path(s);
            if head == "CLEAN_NOZZLE" {
                let (bx, by) = (m.bed[0] * 0.3, m.bed[1] - 5.0);
                p.go([bx, by, 5.0], TRAVEL_SPEED);
                for _ in 0..6 {
                    p.go([bx + 35.0, by, 5.0], 120.0);
                    p.go([bx, by, 5.0], 120.0);
                }
            } else {
                p.go(
                    [
                        m.bed[0] / 2.0,
                        m.bed[1] - 10.0,
                        (s.pos[2] + 10.0).min(m.max_z),
                    ],
                    TRAVEL_SPEED,
                );
            }
            Ok(routine(s, p.into_routine("moving", ts, Done::Nothing)))
        }
        _ => Err(format!("Unknown command:\"{head}\"")),
    }
}

/// The run-out sensor loses the filament: Klipper pauses the print (pause_on_runout) and runs the runout G-code.
fn runout(s: &mut State, ts: f64) {
    s.k.filament = false;
    s.k.info(ts, "Filament Sensor runout: filament not detected");
    if s.job.active() && s.job != Job::Paused {
        s.job = Job::Paused;
        s.k.info(ts, "Filament run-out");
    }
}

/// Klipper's verify_heater: the hotend did not heat as it should, so Klipper shuts the printer down. Heaters go off,
/// the print stops with an error, and only FIRMWARE_RESTART brings it back.
fn heater_fault(s: &mut State, ts: f64) {
    let msg = "Heater extruder not heating at expected rate\nSee the 'verify_heater' section in docs/Config_Reference.md\nfor the parameters that control this check.";
    s.k.info(ts, msg);
    s.k.klippy = "shutdown";
    s.k.message = format!("{msg}\nPrinter is shutdown");
    s.k.routine = None;
    if s.job.active() {
        s.job = Job::Error;
        s.finished_at = ts;
    }
    s.repeat = false;
    s.nozzle.1 = 0.0;
    s.bed.1 = 0.0;
    s.fan = 0.0;
}

/// M112 and the page's STOP.
pub fn emergency(s: &mut State, ts: f64) {
    s.k.klippy = "shutdown";
    s.k.message = "Stopped (emergency stop)\nOnce the underlying issue is corrected, use the \"FIRMWARE_RESTART\" command to reset the firmware, reload the config, and restart the host software.\nPrinter is shutdown".into();
    s.k.routine = None;
    s.k.manual = None;
    s.k.info(ts, "Shutdown due to M112 command");
}

/// FIRMWARE_RESTART, RESTART and SAVE_CONFIG: Klipper goes away for a few seconds and comes back with its config read
/// again. Whatever was printing is gone; the heaters are off, the axes unhomed, the levelling and the offset forgotten,
/// the `default` mesh loaded again.
fn restart(m: &Machine, s: &mut State, ts: f64, firmware: bool) {
    s.k.klippy = "startup";
    s.k.message = "Printer is not ready\nThe klippy host software is attempting to connect.  Please\nretry later.".into();
    s.k.back_at = ts
        + (if firmware {
            FIRMWARE_RESTART_S
        } else {
            RESTART_S
        }) / SCALE.min(3.0);
    s.k.routine = None;
    s.k.manual = None;
    s.k.pending.clear();
    s.k.applied = false;
    s.k.z_adjust = 0.0;
    s.k.excluded.clear();
    s.k.velocity = s.k.settings["printer"]["max_velocity"]
        .as_f64()
        .unwrap_or(m.spec.max_velocity);
    s.k.accel = s.k.settings["printer"]["max_accel"]
        .as_f64()
        .unwrap_or(3000.0);
    s.k.scv = 5.0;
    s.k.mesh =
        s.k.profiles
            .get("default")
            .map(|g| ("default".into(), g.clone()));
    s.k.restarts += 1;
    // a printer.cfg written from the page (the hotend fan edit) is read again
    if let Some(text) = s.k.files.get("printer.cfg").cloned() {
        let mut inside = false;
        for l in text.lines() {
            let t = l.trim();
            if t.starts_with('[') {
                inside = t == "[heater_fan hotend_fan]";
            } else if inside {
                if let Some(v) = t
                    .strip_prefix("heater_temp:")
                    .and_then(|v| v.trim().parse::<f64>().ok())
                {
                    s.k.settings["heater_fan hotend_fan"]["heater_temp"] = json!(v);
                }
            }
        }
    }
    s.current = None;
    s.job = Job::Idle;
    s.repeat = false;
    s.clock = 0.0;
    s.homed = false;
    s.nozzle.1 = 0.0;
    s.bed.1 = 0.0;
    s.fan = 0.0;
    s.k.info(
        ts,
        if firmware {
            "FIRMWARE_RESTART"
        } else {
            "Restarting Klipper"
        },
    );
}

/// Run a script the way Moonraker's /printer/gcode/script does: line by line, each when the one before has finished
/// (a routine waits, a heat-and-wait waits), and answer when all ran or at the first error.
pub fn run_script(m: &Machine, script: &str) -> Result<()> {
    let started = crate::model::now();
    for raw in script.lines().map(str::trim).filter(|l| !l.is_empty()) {
        // a line queues behind whatever is still moving, as Klipper's G-code queue does
        wait_for(m, started, |s| s.k.routine.is_none())?;
        let wait = {
            let mut s = m.lock();
            let ts = crate::model::now();
            m.advance(&mut s, ts);
            s.k.say(ts, raw, "command");
            match line(m, &mut s, ts, raw) {
                Ok(w) => w,
                Err(e) => {
                    s.k.error(ts, &e);
                    return Err(e);
                }
            }
        };
        match wait {
            None => {}
            Some(Wait::Routine) => wait_for(m, started, |s| s.k.routine.is_none())?,
            Some(Wait::Heat(nozzle, t)) => wait_for(m, started, move |s| {
                let h = if nozzle { s.nozzle } else { s.bed };
                (h.0 - t).abs() < 2.0 || h.1 != t
            })?,
        }
    }
    Ok(())
}

fn wait_for(m: &Machine, started: f64, done: impl Fn(&State) -> bool) -> Result<()> {
    loop {
        {
            let mut s = m.lock();
            let ts = crate::model::now();
            m.advance(&mut s, ts);
            if done(&s) {
                return Ok(());
            }
            if !s.k.ready() {
                return Err(s
                    .k
                    .message
                    .lines()
                    .next()
                    .unwrap_or("Klipper stopped")
                    .to_owned());
            }
            if ts - started > ROUTINE_CAP_S {
                return Err("the demo printer did not finish in time".into());
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}

// ------------------------------------------------------------------------------------------------
// Moonraker's API, answered here for a demo printer (src/moonraker.rs sends every call of a demo printer to this).
// ------------------------------------------------------------------------------------------------

/// A query value as the client encoded it (`moonraker::encode`): %XX bytes, `+` a space.
fn decode(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        match b[i] {
            b'%' if i + 2 < b.len() => {
                match u8::from_str_radix(std::str::from_utf8(&b[i + 1..i + 3]).unwrap_or("zz"), 16)
                {
                    Ok(v) => {
                        out.push(v);
                        i += 3;
                        continue;
                    }
                    Err(_) => out.push(b'%'),
                }
            }
            b'+' => out.push(b' '),
            c => out.push(c),
        }
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

pub fn moonraker(cfg: &Config, method: &str, target: &str, body: Option<&Value>) -> Result<Value> {
    let demo = Demo::open(cfg);
    let m = demo.m.clone();
    let (path, query) = target.split_once('?').unwrap_or((target, ""));
    let arg = |k: &str| {
        query.split('&').find_map(|kv| {
            let (a, b) = kv.split_once('=').unwrap_or((kv, ""));
            (a == k).then(|| decode(b))
        })
    };
    let now = crate::model::now();
    match (method, path) {
        ("GET", "/printer/info") => {
            let mut s = m.lock();
            m.advance(&mut s, now);
            let state = if s.job == Job::Error {
                "shutdown"
            } else {
                s.k.klippy
            };
            Ok(
                json!({"state":state,"state_message":s.k.message,"hostname":format!("demo-{}", m.spec.id.replace('_', "-")),
                "software_version":"v0.12.0-demo","cpu_info":"Stella Scope demo printer","klipper_path":"/home/pi/klipper",
                "config_file":"/home/pi/printer_data/config/printer.cfg"}),
            )
        }
        ("GET", "/server/info") => {
            let s = m.lock();
            Ok(
                json!({"klippy_connected":true,"klippy_state":s.k.klippy,"components":["job_queue","webcam"],"failed_components":[],
                "registered_directories":["config","gcodes"],"warnings":[],"moonraker_version":"v0.9.3-demo","api_version":[1,5,0]}),
            )
        }
        ("GET", "/printer/objects/list") => {
            let mut s = m.lock();
            m.advance(&mut s, now);
            let mut all = m.report(&s, now);
            all["configfile"] = configfile(&s);
            Ok(
                json!({"objects":all.as_object().map(|o| o.keys().cloned().collect::<Vec<_>>()).unwrap_or_default()}),
            )
        }
        ("GET", "/printer/objects/query") => {
            let mut s = m.lock();
            m.advance(&mut s, now);
            if !s.k.ready() && s.job != Job::Error && s.k.klippy == "startup" {
                return Err("Moonraker rejected the query: Klippy host not connected; check object names and printer state".into());
            }
            let mut all = m.report(&s, now);
            all["configfile"] = configfile(&s);
            let mut out = serde_json::Map::new();
            for kv in query.split('&').filter(|k| !k.is_empty()) {
                let name = decode(kv.split('=').next().unwrap_or(""));
                if let Some(v) = all.get(&name) {
                    out.insert(name, v.clone());
                }
            }
            Ok(json!({"eventtime":now,"status":out}))
        }
        ("GET", "/printer/gcode/help") => Ok(help(&m.kit)),
        ("GET", "/server/gcode_store") => {
            let s = m.lock();
            let count: usize = arg("count").and_then(|c| c.parse().ok()).unwrap_or(100);
            let rows: Vec<Value> =
                s.k.console
                    .iter()
                    .rev()
                    .take(count)
                    .rev()
                    .map(|(_, t, text, kind)| json!({"message":text,"time":t,"type":kind}))
                    .collect();
            Ok(json!({"gcode_store":rows}))
        }
        ("GET", "/server/files/list") => Ok(Value::Array(
            demo.files()?
                .into_iter()
                .map(|mut f| {
                    f["permissions"] = json!("rw");
                    f
                })
                .collect(),
        )),
        ("GET", "/server/files/metadata") => {
            demo.file_metadata(&arg("filename").unwrap_or_default())
        }
        ("GET", "/server/job_queue/status") => {
            let s = m.lock();
            let jobs: Vec<Value> = s.k.queue.iter().map(|(id, f, at)| json!({"job_id":id,"filename":f,"time_added":at,"time_in_queue":now-at})).collect();
            Ok(
                json!({"queued_jobs":jobs,"queue_state":if s.k.queue_paused {"paused"} else {"ready"}}),
            )
        }
        ("POST", "/server/job_queue/job") => {
            let files: Vec<String> = body
                .and_then(|b| b["filenames"].as_array())
                .into_iter()
                .flatten()
                .filter_map(|f| f.as_str().map(str::to_owned))
                .collect();
            let mut s = m.lock();
            for f in files {
                if Demo::find(&s, &f).is_none() {
                    return Err(format!("Moonraker command returned an error after submission: File not found: {f}; outcome is unknown"));
                }
                let id = format!("{:016X}", 0xD3_0000 + s.k.next_job);
                s.k.next_job += 1;
                s.k.queue.push((id, f, now));
            }
            Ok(json!({"queued_jobs":s.k.queue.len(),"queue_state":"ready"}))
        }
        ("DELETE", "/server/job_queue/job") => {
            let ids = arg("job_ids").unwrap_or_default();
            let mut s = m.lock();
            s.k.queue
                .retain(|(id, ..)| !ids.split(',').any(|i| i == id));
            Ok(json!({"queued_jobs":s.k.queue.len(),"queue_state":"ready"}))
        }
        ("POST", "/server/job_queue/pause") | ("POST", "/server/job_queue/start") => {
            let mut s = m.lock();
            s.k.queue_paused = path.ends_with("pause");
            Ok(json!({"queue_state":if s.k.queue_paused {"paused"} else {"ready"}}))
        }
        ("GET", "/server/webcams/list") => Ok(
            json!({"webcams":[{"name":"Demo camera","enabled":true,"service":"mjpegstreamer-adaptive",
            "target_fps":2,"target_fps_idle":1,"stream_url":"","snapshot_url":"","flip_horizontal":false,"flip_vertical":false,"rotation":0,"aspect_ratio":"16:9"}]}),
        ),
        ("POST", "/printer/gcode/script") => {
            let script = body
                .and_then(|b| b["script"].as_str())
                .unwrap_or("")
                .to_owned();
            if let Some(why) = crate::driver::denied_command(&script) {
                return Err(why);
            }
            run_script(&m, &script)
                .map(|_| json!("ok"))
                .map_err(|e| format!("Klipper {}: {e}", crate::driver::PRINTER_REFUSED))
        }
        ("POST", "/printer/print/start") => {
            let file = arg("filename").unwrap_or_default();
            demo.act(&Action::StartPrint { filename: file })
        }
        ("POST", "/printer/print/pause") => demo.act(&Action::Pause),
        ("POST", "/printer/print/resume") => demo.act(&Action::Resume),
        ("POST", "/printer/print/cancel") => demo.act(&Action::Cancel),
        ("POST", "/printer/emergency_stop") => demo.emergency_stop(),
        ("POST", "/printer/firmware_restart") | ("POST", "/printer/restart") => {
            let mut s = m.lock();
            restart(&m, &mut s, now, path.contains("firmware"));
            Ok(json!("ok"))
        }
        _ => Err(format!("the demo printer does not answer {method} {path}")),
    }
}

/// A config file, as Moonraker serves /server/files/config/<path>.
pub fn download_config(cfg: &Config, path: &str) -> Result<String> {
    let m = Demo::open(cfg).m;
    let s = m.lock();
    match s.k.files.get(path) {
        Some(t) => Ok(t.clone()),
        None if path == "printer.cfg" => Ok(printer_cfg(&s.k.settings)),
        None => Err(format!("Moonraker returned HTTP 404 for {path}")),
    }
}

pub fn upload_config(cfg: &Config, path: &str, content: &str) -> Result<Value> {
    let m = Demo::open(cfg).m;
    let mut s = m.lock();
    s.k.files.insert(path.to_owned(), content.to_owned());
    Ok(json!({"item":{"path":path,"root":"config"},"action":"create_file"}))
}

/// The resonance a demo machine's accelerometer would see on one leg of a capture (src/resonance.rs `legs`): X and Y are
/// the machine's own, the two belt legs nearly match (a well-tensioned pair), all fixed per machine.
pub fn resonance_hz(cfg: &Config, leg: &str) -> f64 {
    let kit = &Demo::open(cfg).m.kit;
    let (fx, fy) = (
        44.0 + 20.0 * unit(kit.seed, 29),
        34.0 + 18.0 * unit(kit.seed, 30),
    );
    match leg {
        "x" => fx,
        "y" => fy,
        "a" => (fx + fy) / 2.0,
        _ => (fx + fy) / 2.0 * (1.0 + 0.04 * (unit(kit.seed, 31) - 0.5)),
    }
}

#[cfg(test)]
mod tests {
    //! one test per capability a demo printer now acts out. Routines run 40 times faster under test (SCALE).
    use super::*;

    fn printer(id: &str, cad: &str) -> (Config, Demo) {
        let mut c = Config::default();
        c.driver = "demo".into();
        c.printer_id = id.into();
        c.database = format!("demo-klipper-test-{id}.sqlite3");
        c.cad_model = Some(cad.into());
        let d = Demo::open(&c);
        (c, d)
    }
    fn api(c: &Config, method: &str, target: &str, body: Option<Value>) -> Result<Value> {
        moonraker(c, method, target, body.as_ref())
    }
    fn gcode(c: &Config, script: &str) -> Result<Value> {
        api(
            c,
            "POST",
            "/printer/gcode/script",
            Some(json!({"script": script})),
        )
    }
    fn status(c: &Config, objects: &str) -> Value {
        api(c, "GET", &format!("/printer/objects/query?{objects}"), None).unwrap()["status"].clone()
    }
    fn console(c: &Config) -> String {
        let v = api(c, "GET", "/server/gcode_store?count=400", None).unwrap();
        v["gcode_store"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| r["message"].as_str().unwrap_or("").to_owned())
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[test]
    fn level_a_voron_gantry_converges_under_its_tolerance_and_an_ender_has_no_gantry() {
        let (c, _) = printer("lvl", "voron_v24_350_printed");
        assert!(
            gcode(&c, "QUAD_GANTRY_LEVEL").is_ok(),
            "homes first when it has to"
        );
        assert_eq!(
            status(&c, "quad_gantry_level")["quad_gantry_level"]["applied"],
            json!(true)
        );
        let rounds = crate::qgl::Parser::new(0).feed(0.0, &console(&c));
        assert!(rounds.len() >= 2, "{rounds:?}");
        assert!(
            rounds[0].range_mm > 0.5 && rounds.last().unwrap().range_mm < 0.02,
            "{rounds:?}"
        );
        assert!(
            rounds.windows(2).all(|w| w[1].range_mm < w[0].range_mm),
            "each round is closer"
        );
        assert!(
            console(&c)
                .lines()
                .any(|l| crate::probe::point_from(l).is_some()),
            "probe lines in Klipper's words"
        );
        let (e, _) = printer("lvl-ender", "ender3_stock_220");
        assert!(gcode(&e, "QUAD_GANTRY_LEVEL")
            .unwrap_err()
            .contains("Unknown command"));
        gcode(&e, "G28\nSCREWS_TILT_CALCULATE").unwrap();
        assert!(
            console(&e).contains("front right screw : x="),
            "{}",
            console(&e)
        );
        let (t, _) = printer("lvl-trident", "voron_trident_300");
        gcode(&t, "G28\nZ_TILT_ADJUST").unwrap();
        assert_eq!(status(&t, "z_tilt")["z_tilt"]["applied"], json!(true));
        assert!(status(&t, "configfile")["configfile"]["settings"]
            .get("quad_gantry_level")
            .is_none());
    }

    #[test]
    fn mesh_probes_a_grid_and_profiles_save_load_and_clear() {
        let (c, d) = printer("mesh", "prusa_core_one_l");
        gcode(&c, "BED_MESH_CLEAR").unwrap();
        assert_eq!(
            status(&c, "bed_mesh")["bed_mesh"]["profile_name"],
            json!("")
        );
        gcode(&c, "G28\nBED_MESH_CALIBRATE").unwrap();
        let mesh = &status(&c, "bed_mesh")["bed_mesh"];
        let grid = mesh["probed_matrix"].as_array().unwrap();
        assert_eq!(grid.len(), d.m.kit.grid);
        assert!(grid
            .iter()
            .all(|r| r.as_array().unwrap().len() == d.m.kit.grid));
        assert_eq!(mesh["profile_name"], json!("default"));
        let probes = console(&c).matches("probe: at ").count();
        assert_eq!(probes, d.m.kit.grid * d.m.kit.grid, "one probe a point");
        gcode(
            &c,
            "BED_MESH_PROFILE SAVE=pei\nBED_MESH_CLEAR\nBED_MESH_PROFILE LOAD=pei",
        )
        .unwrap();
        assert_eq!(
            status(&c, "bed_mesh")["bed_mesh"]["profile_name"],
            json!("pei")
        );
        gcode(&c, "BED_MESH_PROFILE REMOVE=pei").unwrap();
        assert!(gcode(&c, "BED_MESH_PROFILE LOAD=pei").is_err());
    }

    #[test]
    fn z_offset_baby_steps_apply_to_the_probe_and_save_config_writes_it() {
        let (c, mut d) = printer("zoff", "ender3_stock_220");
        gcode(
            &c,
            "SET_GCODE_OFFSET Z_ADJUST=-0.025\nSET_GCODE_OFFSET Z_ADJUST=-0.025",
        )
        .unwrap();
        assert_eq!(
            status(&c, "gcode_move")["gcode_move"]["homing_origin"][2],
            json!(-0.05)
        );
        let before = d.fetch_config().unwrap()["bltouch"]["z_offset"]
            .as_f64()
            .unwrap();
        gcode(&c, "Z_OFFSET_APPLY_PROBE").unwrap();
        assert_eq!(
            status(&c, "configfile")["configfile"]["save_config_pending"],
            json!(true)
        );
        gcode(&c, "SAVE_CONFIG").unwrap();
        assert_eq!(
            d.snapshot().unwrap().state,
            "startup",
            "SAVE_CONFIG restarts Klipper"
        );
        let after = d.fetch_config().unwrap()["bltouch"]["z_offset"]
            .as_f64()
            .unwrap();
        assert!(
            (after - (before + 0.05)).abs() < 1e-6,
            "{before} -> {after}"
        );
        std::thread::sleep(std::time::Duration::from_millis(1600));
        assert_eq!(
            status(&c, "gcode_move")["gcode_move"]["homing_origin"][2],
            json!(0.0),
            "the offset is in the probe now"
        );
    }

    #[test]
    fn console_answers_in_klippers_words_and_refuses_what_it_does_not_know() {
        let (c, d) = printer("console", "voron_v02_120");
        gcode(&c, "STATUS\nM114\nHELP").unwrap();
        let text = console(&c);
        assert!(
            text.contains("// Klipper state: Ready")
                && text.contains("// X:")
                && text.contains("// Available extended commands:"),
            "{text}"
        );
        assert!(gcode(&c, "NOT_A_COMMAND")
            .unwrap_err()
            .contains("Unknown command:\"NOT_A_COMMAND\""));
        assert!(console(&c).contains("!! Unknown command"));
        let lines = d.console().unwrap().0;
        assert!(lines
            .iter()
            .any(|e| e.kind == "command" && e.message == "STATUS"));
        assert!(
            d.console().unwrap().0.is_empty(),
            "the collector is given each line once"
        );
        assert!(gcode(&c, "G1 X5")
            .unwrap_err()
            .contains("Must home axis first"));
    }

    #[test]
    fn macros_run_and_the_do_not_send_list_is_refused_before_it_reaches_the_printer() {
        let (c, _) = printer("macros", "voron_v24_350_printed");
        let help = api(&c, "GET", "/printer/gcode/help", None).unwrap();
        for m in [
            "PRINT_START",
            "PRINT_END",
            "LOAD_FILAMENT",
            "CLEAN_NOZZLE",
            "QUAD_GANTRY_LEVEL",
        ] {
            assert!(help.get(m).is_some(), "{m}");
        }
        let settings = &status(&c, "configfile")["configfile"]["settings"];
        assert!(settings["gcode_macro print_start"]["gcode"]
            .as_str()
            .unwrap()
            .contains("params.BED"));
        gcode(&c, "G28\nCLEAN_NOZZLE").unwrap();
        for denied in [
            "REMOVE_ZMOD",
            "CLEAR_EMMC",
            "RESET_PASSWD",
            "RESTORE_TAR_CONFIG",
            "NEW_SAVE_CONFIG",
            "SDCARD_RESET_FILE",
        ] {
            assert!(
                gcode(&c, denied).unwrap_err().contains("do-not-send"),
                "{denied}"
            );
        }
    }

    #[test]
    fn camera_draws_a_jpeg_of_the_print() {
        let (c, d) = printer("cam", "voron_trident_300");
        assert_eq!(d.cameras().unwrap()[0].kind, "demo");
        d.act(&Action::StartPrint {
            filename: "3D3D twist vase.gcode".into(),
        })
        .unwrap();
        let jpg = crate::demo::camera_jpeg(&c).unwrap();
        assert_eq!(&jpg[..2], &[0xFF, 0xD8]);
        assert!(image::load_from_memory(&jpg).is_ok_and(|i| i.width() == 640 && i.height() == 360));
    }

    #[test]
    fn push_a_run_out_pauses_the_print_and_the_alert_says_why() {
        let (c, d) = printer("runout", "prusa_core_one_l");
        gcode(&c, "SIMULATE_RUNOUT").unwrap();
        assert!(console(&c).contains("next print"), "armed while idle");
        {
            let mut s = d.m.lock();
            let i = Demo::find(&s, "3D3D twist bowl.gcode").unwrap();
            let ts = crate::model::now();
            d.m.start(&mut s, i, ts, 0.5);
            tick(&d.m, &mut s, ts);
            tick(&d.m, &mut s, ts + ARMED_AFTER_S + 1.0);
            assert_eq!(s.job, Job::Paused);
        }
        let st = status(&c, "print_stats&filament_switch_sensor%20runout");
        assert_eq!(
            st["filament_switch_sensor runout"]["filament_detected"],
            json!(false)
        );
        assert!(gcode(&c, "RESUME")
            .unwrap_err()
            .contains("Filament not detected"));
        let text = crate::collector::pause_text("CORE One L", Some(&st));
        assert!(text.contains("ran out of filament"), "{text}");
    }

    #[test]
    fn filament_loads_hot_with_the_extruder_moving_and_the_spool_is_charged() {
        let (c, d) = printer("fil", "voron_v02_120");
        {
            let mut s = d.m.lock();
            let ts = crate::model::now();
            assert!(matches!(
                line(&d.m, &mut s, ts, "LOAD_FILAMENT TEMP=200"),
                Ok(Some(Wait::Routine))
            ));
            assert_eq!(s.nozzle.1, 200.0, "it heats first");
            let (from, end) =
                s.k.routine
                    .as_ref()
                    .map(|r| (r.extrude_from, r.end))
                    .unwrap();
            assert!(from > 0.0, "no filament moves before the nozzle is hot");
            assert!(
                d.m.report(&s, ts + from / 2.0)["motion_report"]["live_extruder_velocity"]
                    .is_null()
            );
            assert_eq!(
                d.m.report(&s, ts + from + 0.01)["motion_report"]["live_extruder_velocity"],
                json!(5.0)
            );
            tick(&d.m, &mut s, ts + end + 0.01);
            assert!(s.k.routine.is_none() && s.k.filament);
        }
        assert!(
            console(&c).contains("LOAD_FILAMENT finished"),
            "{}",
            console(&c)
        );
        d.act(&Action::StartPrint {
            filename: "3D3D small twist vase.gcode".into(),
        })
        .unwrap();
        {
            let mut s = d.m.lock();
            s.clock = 600.0;
            s.job = Job::Printing;
        }
        let used = status(&c, "print_stats")["print_stats"]["filament_used"]
            .as_f64()
            .unwrap();
        assert!(used > 100.0, "{used} mm");
    }

    #[test]
    fn klipper_and_restart_drop_the_data_lose_the_print_and_come_back() {
        let (c, mut d) = printer("restart", "sovol_sv08_350");
        d.act(&Action::StartPrint {
            filename: "3D3D twist vase.gcode".into(),
        })
        .unwrap();
        gcode(&c, "FIRMWARE_RESTART").unwrap();
        assert_eq!(d.snapshot().unwrap().state, "startup");
        let (sample, _) = d.sample(crate::model::now()).unwrap();
        assert!(
            sample.get("extruder").is_none(),
            "readings drop while it restarts: {sample}"
        );
        assert!(api(&c, "GET", "/printer/objects/query?extruder", None).is_err());
        std::thread::sleep(std::time::Duration::from_millis(1600));
        let (sample, _) = d.sample(crate::model::now()).unwrap();
        assert_eq!(sample["webhooks"]["state"], json!("ready"));
        assert_eq!(
            sample["print_stats"]["state"],
            json!("standby"),
            "the print is gone"
        );
        // a heater fault stops Klipper; only FIRMWARE_RESTART brings it back
        gcode(&c, "SIMULATE_HEATER_FAULT").unwrap();
        d.act(&Action::StartPrint {
            filename: "3D3D twist vase.gcode".into(),
        })
        .unwrap();
        {
            let mut s = d.m.lock();
            let ts = crate::model::now();
            tick(&d.m, &mut s, ts);
            tick(&d.m, &mut s, ts + ARMED_AFTER_S + 1.0);
        }
        assert_ne!(
            api(&c, "GET", "/printer/info", None).unwrap()["state"],
            json!("ready")
        );
        assert!(gcode(&c, "G28").is_err());
        gcode(&c, "FIRMWARE_RESTART").unwrap();
        std::thread::sleep(std::time::Duration::from_millis(1600));
        assert_eq!(
            api(&c, "GET", "/printer/info", None).unwrap()["state"],
            json!("ready")
        );
        // the Klipper-only tools answer too: limits, pressure advance, position, the queue
        gcode(
            &c,
            "SET_VELOCITY_LIMIT ACCEL=4000\nSET_PRESSURE_ADVANCE ADVANCE=0.05\nGET_POSITION",
        )
        .unwrap();
        assert_eq!(
            status(&c, "toolhead")["toolhead"]["max_accel"],
            json!(4000.0)
        );
        api(
            &c,
            "POST",
            "/server/job_queue/job",
            Some(json!({"filenames":["3D3D twist vase.gcode"]})),
        )
        .unwrap();
        assert_eq!(
            api(&c, "GET", "/server/job_queue/status", None).unwrap()["queued_jobs"]
                .as_array()
                .unwrap()
                .len(),
            1
        );
    }
}
