//! arp2600: ARP 2600-inspired semi-modular synth (Rust + SDL2).
//!
//! - `--smoke [secs]`: render headless for N seconds (default 2), exit 0.
//! - `--render [path] [secs]`: render a demo phrase to a WAV file.
//! - `--shot [path]`: render the faceplate offscreen to a BMP file.
//! - `--list-devices`: list MIDI input ports.

mod audio;
mod dsp;
mod env;
mod filter;
mod font;
mod midi;
mod osc;
mod patch;
mod presets;
mod reverb;
mod ui;
mod voice;
mod wav;

use audio::SharedSynth;
use midi::MidiMsg;
use patch::{IN_LABELS, OUT_LABELS};
use sdl2::event::Event;
use sdl2::keyboard::{Keycode, Mod};
use sdl2::pixels::PixelFormatEnum;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use voice::Synth;

const SR: f32 = 44100.0;

fn has_flag(args: &[String], flag: &str) -> bool {
    args.iter().any(|a| a == flag)
}

fn flag_value(args: &[String], flag: &str) -> Option<String> {
    args.iter()
        .position(|a| a == flag)
        .and_then(|i| args.get(i + 1).cloned())
        .filter(|v| !v.starts_with("--"))
}

fn render_phrase(synth: &mut Synth, secs: f32) -> Vec<f32> {
    // C major arpeggio, 0.5s per note, last note held longer.
    let notes: [(f32, f32, f32); 4] = [
        (0.0, 48.0, 0.5),
        (0.5, 52.0, 0.5),
        (1.0, 55.0, 0.5),
        (1.5, 60.0, 1.0),
    ];
    let total = (secs * SR) as usize;
    let mut buf = Vec::with_capacity(total * 2);
    let mut on = vec![false; notes.len()];
    let mut off = vec![false; notes.len()];
    let mut chunk = vec![0.0; 2048];
    let mut t = 0.0f32;
    while (t * SR) as usize <= total {
        for (i, (start, midi, dur)) in notes.iter().enumerate() {
            if !on[i] && t >= *start {
                synth.note_on(*midi);
                on[i] = true;
            }
            if on[i] && !off[i] && t >= start + dur {
                synth.note_off(*midi);
                off[i] = true;
            }
        }
        let n = 1024.min(total + 1 - (t * SR) as usize).max(1);
        let frames = n.min(1024);
        chunk.resize(frames * 2, 0.0);
        synth.render(&mut chunk);
        buf.extend_from_slice(&chunk);
        t += frames as f32 / SR;
    }
    buf
}

fn check_render(buf: &[f32]) -> (bool, f32, f32) {
    let finite = buf.iter().all(|x| x.is_finite());
    let peak = buf.iter().fold(0.0f32, |m, x| m.max(x.abs()));
    let rms = (buf.iter().map(|x| x * x).sum::<f32>() / buf.len().max(1) as f32).sqrt();
    (finite, peak, rms)
}

fn run_smoke(secs: f32) -> i32 {
    let mut synth = Synth::new(SR);
    let buf = render_phrase(&mut synth, secs);
    let (finite, peak, rms) = check_render(&buf);
    println!("smoke: {secs}s, {} samples, finite={finite}, peak={peak:.3}, rms={rms:.3}", buf.len());
    if !finite || peak < 0.05 {
        eprintln!("smoke FAILED: bad render");
        1
    } else {
        0
    }
}

fn run_render(path: &str, secs: f32) -> i32 {
    let mut synth = Synth::new(SR);
    let buf = render_phrase(&mut synth, secs);
    match wav::write_wav_stereo(path, &buf, SR as u32) {
        Ok(()) => {
            println!("wrote {path} ({}s)", secs);
            0
        }
        Err(e) => {
            eprintln!("render failed: {e}");
            1
        }
    }
}

fn run_shot(path: &str) -> i32 {
    if std::env::var("SDL_VIDEODRIVER").is_err() {
        std::env::set_var("SDL_VIDEODRIVER", "dummy");
    }
    let sdl = match sdl2::init() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("sdl init failed: {e}");
            return 1;
        }
    };
    let _video = match sdl.video() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("video init failed: {e}");
            return 1;
        }
    };
    let surface = match sdl2::surface::Surface::new(ui::W, ui::H, PixelFormatEnum::RGB24) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("surface failed: {e}");
            return 1;
        }
    };
    let mut canvas = match surface.into_canvas() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("canvas failed: {e}");
            return 1;
        }
    };
    let synth = Synth::new(SR);
    let ui_state = ui::UiState::new();
    let mut buttons = Vec::new();
    ui::draw(&mut canvas, &synth, &ui_state, &None, &mut buttons);
    canvas.present();
    match canvas.surface().save_bmp(path) {
        Ok(()) => {
            println!("wrote {path}");
            0
        }
        Err(e) => {
            eprintln!("screenshot failed: {e}");
            1
        }
    }
}

/// Computer-key note map: Z-row = lower octave, Q-row = upper octave.
/// Returns the semitone offset from the row base.
fn key_note(key: Keycode) -> Option<(i32, i32)> {
    let lower = [
        (Keycode::Z, 0),
        (Keycode::S, 1),
        (Keycode::X, 2),
        (Keycode::D, 3),
        (Keycode::C, 4),
        (Keycode::V, 5),
        (Keycode::G, 6),
        (Keycode::B, 7),
        (Keycode::H, 8),
        (Keycode::N, 9),
        (Keycode::J, 10),
        (Keycode::M, 11),
    ];
    for (k, off) in lower {
        if key == k {
            return Some((48, off));
        }
    }
    let upper = [
        (Keycode::Q, 0),
        (Keycode::Num2, 1),
        (Keycode::W, 2),
        (Keycode::Num3, 3),
        (Keycode::E, 4),
        (Keycode::R, 5),
        (Keycode::Num5, 6),
        (Keycode::T, 7),
        (Keycode::Num6, 8),
        (Keycode::Y, 9),
        (Keycode::Num7, 10),
        (Keycode::U, 11),
        (Keycode::I, 12),
    ];
    for (k, off) in upper {
        if key == k {
            return Some((60, off));
        }
    }
    None
}

fn run_ui() -> i32 {
    let sdl = match sdl2::init() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("sdl init failed: {e}");
            return 1;
        }
    };
    let video = match sdl.video() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("video init failed: {e}");
            return 1;
        }
    };
    let window = match video
        .window("arp2600", ui::W, ui::H)
        .position_centered()
        .build()
    {
        Ok(w) => w,
        Err(e) => {
            eprintln!("window failed: {e}");
            return 1;
        }
    };
    let mut canvas = match window.into_canvas().build() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("canvas failed: {e}");
            return 1;
        }
    };

    let synth: SharedSynth = Arc::new(Mutex::new(Synth::new(SR)));
    let _audio = match audio::open_audio(&sdl, synth.clone(), SR as u32) {
        Ok((dev, actual)) => {
            if let Ok(mut s) = synth.lock() {
                s.sr = actual as f32;
            }
            Some(dev)
        }
        Err(e) => {
            eprintln!("audio unavailable ({e}); UI still runs, no sound");
            None
        }
    };
    let mut midi_in = midi::Midi::new();
    let mut ui_state = ui::UiState::new();
    let mut buttons: Vec<(sdl2::rect::Rect, ui::Button)> = Vec::new();
    let mut held_computer: HashMap<Keycode, u8> = HashMap::new();
    let mut mouse_xy: (i32, i32) = (0, 0);
    let mut event_pump = match sdl.event_pump() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("event pump failed: {e}");
            return 1;
        }
    };

    'running: loop {
        // MIDI drain.
        for msg in midi_in.poll() {
            if let Ok(mut s) = synth.lock() {
                match msg {
                    MidiMsg::NoteOn(m) => {
                        s.note_on(m as f32);
                        ui_state.held_keys.insert(m);
                    }
                    MidiMsg::NoteOff(m) => {
                        s.note_off(m as f32);
                        ui_state.held_keys.remove(&m);
                    }
                    MidiMsg::Bend(b) => s.bend(b),
                    MidiMsg::Pedal(d) => s.pedal(d),
                }
            }
        }
        let midi_port = midi_in.port_name.clone();

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::KeyDown {
                    keycode: Some(k),
                    repeat: false,
                    keymod,
                    ..
                } => {
                    if let Some((base, off)) = key_note(k) {
                        let m = (base + off + ui_state.octave_shift * 12)
                            .clamp(0, 127) as u8;
                        if let Ok(mut s) = synth.lock() {
                            s.note_on(m as f32);
                        }
                        ui_state.held_keys.insert(m);
                        held_computer.insert(k, m);
                    } else if let Some(n) = (1..=presets::SLOTS).find(|n| {
                        k == match n {
                            1 => Keycode::Num1,
                            2 => Keycode::Num2,
                            3 => Keycode::Num3,
                            4 => Keycode::Num4,
                            5 => Keycode::Num5,
                            6 => Keycode::Num6,
                            7 => Keycode::Num7,
                            _ => Keycode::Num8,
                        }
                    }) {
                        let shift = keymod.contains(Mod::LSHIFTMOD)
                            || keymod.contains(Mod::RSHIFTMOD);
                        if shift {
                            if let Ok(s) = synth.lock() {
                                match presets::save_slot(n, &s.params) {
                                    Ok(()) => {
                                        ui_state.status = format!("saved preset {n}");
                                    }
                                    Err(e) => {
                                        ui_state.status = format!("save failed: {e}");
                                    }
                                }
                            }
                        } else if let Ok(mut s) = synth.lock() {
                            match presets::load_slot(n) {
                                Some(p) => {
                                    s.params = p;
                                    ui_state.status = format!("loaded preset {n}");
                                }
                                None => ui_state.status = format!("preset {n} empty"),
                            }
                        }
                        ui_state.preset_slot = n;
                    } else if k == Keycode::Left {
                        ui_state.octave_shift = (ui_state.octave_shift - 1).max(-2);
                        ui_state.status =
                            format!("octave shift {}", ui_state.octave_shift);
                    } else if k == Keycode::Right {
                        ui_state.octave_shift = (ui_state.octave_shift + 1).min(2);
                        ui_state.status =
                            format!("octave shift {}", ui_state.octave_shift);
                    } else if k == Keycode::Space {
                        if let Ok(mut s) = synth.lock() {
                            s.all_notes_off();
                        }
                        ui_state.held_keys.clear();
                        held_computer.clear();
                        ui_state.mouse_note = None;
                        ui_state.status = "all notes off".to_string();
                    }
                }
                Event::KeyUp {
                    keycode: Some(k), ..
                } => {
                    if let Some(m) = held_computer.remove(&k) {
                        if let Ok(mut s) = synth.lock() {
                            s.note_off(m as f32);
                        }
                        ui_state.held_keys.remove(&m);
                    }
                }
                Event::MouseButtonDown { x, y, .. } => {
                    mouse_xy = (x, y);
                    let mut handled = false;
                    // Switch buttons (from last frame).
                    for (rect, btn) in &buttons {
                        if x >= rect.x() && x < rect.x() + rect.width() as i32
                            && y >= rect.y() && y < rect.y() + rect.height() as i32
                        {
                            if let Ok(mut s) = synth.lock() {
                                match btn {
                                    ui::Button::VcoWave(v, w) => match v {
                                        0 => s.params.vco1_wave = *w,
                                        1 => s.params.vco2_wave = *w,
                                        _ => s.params.vco3_wave = *w,
                                    },
                                    ui::Button::NoiseType(n) => s.params.noise_type = *n,
                                }
                            }
                            handled = true;
                            break;
                        }
                    }
                    if handled {
                        continue;
                    }
                    if let Some(idx) = ui::slider_at(x, y) {
                        ui_state.drag_slider = Some(idx);
                        if let Ok(mut s) = synth.lock() {
                            let sl = ui::sliders()[idx];
                            let v = 1.0 - (y - sl.y) as f32 / sl.h as f32;
                            sl.param.from01(&mut s.params, v);
                            ui_state.hover_param = Some(sl.param);
                        }
                    } else if let Some(o) = ui::out_jack_at(x, y) {
                        ui_state.pending_out = Some(o);
                    } else if let Some(i) = ui::in_jack_at(x, y) {
                        if let Ok(mut s) = synth.lock() {
                            if s.patch.cable_for(i).is_some() {
                                s.patch.disconnect(i);
                                ui_state.status = format!("unplugged {}", IN_LABELS[i]);
                            } else if let Some(o) = ui_state.pending_out {
                                s.patch.connect(o, i);
                                ui_state.status = format!(
                                    "patched {} -> {}",
                                    OUT_LABELS[o],
                                    IN_LABELS[i]
                                );
                            }
                        }
                        ui_state.pending_out = None;
                    } else if let Some(m) = ui::key_at(x, y) {
                        if let Ok(mut s) = synth.lock() {
                            s.note_on(m as f32);
                        }
                        ui_state.held_keys.insert(m);
                        ui_state.mouse_note = Some(m);
                    }
                }
                Event::MouseMotion { x, y, .. } => {
                    mouse_xy = (x, y);
                    if let Some(idx) = ui_state.drag_slider {
                        if let Ok(mut s) = synth.lock() {
                            let sl = ui::sliders()[idx];
                            let v = 1.0 - (y - sl.y) as f32 / sl.h as f32;
                            sl.param.from01(&mut s.params, v);
                            ui_state.hover_param = Some(sl.param);
                        }
                    } else if let Some(idx) = ui::slider_at(x, y) {
                        ui_state.hover_param = Some(ui::sliders()[idx].param);
                    } else {
                        ui_state.hover_param = None;
                    }
                }
                Event::MouseButtonUp { .. } => {
                    ui_state.drag_slider = None;
                    if let Some(m) = ui_state.mouse_note.take() {
                        if let Ok(mut s) = synth.lock() {
                            s.note_off(m as f32);
                        }
                        ui_state.held_keys.remove(&m);
                    }
                }
                Event::MouseWheel { y, .. } => {
                    // Wheel over a slider nudges it; y is the scroll delta.
                    let (mx, my) = mouse_xy;
                    if let Some(idx) = ui::slider_at(mx, my) {
                        if let Ok(mut s) = synth.lock() {
                            let sl = ui::sliders()[idx];
                            let cur = sl.param.to01(&s.params);
                            let step = 0.05 * y as f32;
                            sl.param.from01(&mut s.params, cur + step);
                            ui_state.hover_param = Some(sl.param);
                        }
                    }
                }
                _ => {}
            }
        }

        buttons.clear();
        if let Ok(s) = synth.lock() {
            ui::draw(&mut canvas, &s, &ui_state, &midi_port, &mut buttons);
        }
        canvas.present();
        std::thread::sleep(std::time::Duration::from_millis(16));
    }
    0
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let code = if has_flag(&args, "--smoke") {
        let secs = flag_value(&args, "--smoke")
            .and_then(|v| v.parse().ok())
            .unwrap_or(2.0);
        run_smoke(secs)
    } else if has_flag(&args, "--render") {
        let path = flag_value(&args, "--render").unwrap_or_else(|| "out.wav".to_string());
        let secs = args
            .iter()
            .position(|a| a == "--render")
            .and_then(|i| args.get(i + 2))
            .and_then(|v| v.parse().ok())
            .unwrap_or(3.0);
        run_render(&path, secs)
    } else if has_flag(&args, "--shot") {
        let path = flag_value(&args, "--shot").unwrap_or_else(|| "shot.bmp".to_string());
        run_shot(&path)
    } else if has_flag(&args, "--list-devices") {
        for d in midi::Midi::list_devices() {
            println!("{d}");
        }
        0
    } else {
        run_ui()
    };
    std::process::exit(code);
}
