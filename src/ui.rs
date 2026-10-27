//! Panel UI: 2600-style slider sections, patch bay, 2-octave keyboard.
//! All continuous params are vertical sliders (the 2600's own idiom).
//! Drawn with the 5x7 bitmap font; no external assets.

use crate::font::{draw_text, text_width};
use crate::osc::Wave;
use crate::patch::{inp, out, IN_LABELS, OUT_LABELS};
use crate::voice::{NoiseType, Params, Synth};
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use std::collections::HashSet;

pub const W: u32 = 1280;
pub const H: u32 = 760;

pub const PANEL: Color = Color::RGB(0x2E, 0x3D, 0x4F);
pub const SECTION: Color = Color::RGB(0x24, 0x30, 0x3D);
pub const INK: Color = Color::RGB(0xE8, 0xE4, 0xD8);
pub const DIM: Color = Color::RGB(0x9A, 0xA8, 0xB5);
pub const ACCENT: Color = Color::RGB(0xFF, 0x8C, 0x42);
pub const TRACK: Color = Color::RGB(0x14, 0x1C, 0x26);
pub const WHITE_KEY: Color = Color::RGB(0xE8, 0xE4, 0xD8);
pub const BLACK_KEY: Color = Color::RGB(0x1A, 0x1E, 0x24);
pub const HELD_KEY: Color = Color::RGB(0xFF, 0x8C, 0x42);

const CABLE_COLORS: [Color; 6] = [
    Color::RGB(0xE5, 0x48, 0x4D),
    Color::RGB(0xFF, 0xB2, 0x24),
    Color::RGB(0x46, 0xA7, 0x58),
    Color::RGB(0x00, 0xA2, 0xC7),
    Color::RGB(0x8E, 0x4E, 0xC6),
    Color::RGB(0xFF, 0x8C, 0x42),
];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Param {
    Vco1Coarse,
    Vco1Fine,
    Vco1Fm,
    Vco1Level,
    Vco2Coarse,
    Vco2Fine,
    Vco2Pw,
    Vco2Fm,
    Vco2Level,
    Vco3Coarse,
    Vco3Fine,
    Vco3Pw,
    Vco3Fm,
    Vco3Level,
    NoiseLevel,
    RingLevel,
    ShRate,
    LagTime,
    VcfCutoff,
    VcfReso,
    VcfKeytrack,
    VcfCv1,
    VcfCv2,
    VcaGain,
    VcaCv1,
    VcaCv2,
    AdsrA,
    AdsrD,
    AdsrS,
    AdsrR,
    ArA,
    ArR,
    Mix0,
    Mix1,
    Mix2,
    Mix3,
    RevMix,
    Glide,
}

fn log_map(s: f32, lo: f32, hi: f32) -> f32 {
    lo * (hi / lo).powf(s.clamp(0.0, 1.0))
}

fn log_inv(v: f32, lo: f32, hi: f32) -> f32 {
    if v <= lo {
        0.0
    } else {
        (v / lo).ln() / (hi / lo).ln()
    }
}

fn fmt_time(v: f32) -> String {
    if v < 1.0 {
        format!("{:.0} ms", v * 1000.0)
    } else {
        format!("{:.1} s", v)
    }
}

fn fmt_freq(v: f32) -> String {
    if v < 1000.0 {
        format!("{:.0} Hz", v)
    } else {
        format!("{:.1} kHz", v / 1000.0)
    }
}

impl Param {
    pub fn label(self) -> &'static str {
        match self {
            Param::Vco1Coarse | Param::Vco2Coarse | Param::Vco3Coarse => "CRS",
            Param::Vco1Fine | Param::Vco2Fine | Param::Vco3Fine => "FIN",
            Param::Vco1Fm | Param::Vco2Fm | Param::Vco3Fm => "FM",
            Param::Vco1Level | Param::Vco2Level | Param::Vco3Level => "LVL",
            Param::Vco2Pw | Param::Vco3Pw => "PW",
            Param::NoiseLevel => "NOI",
            Param::RingLevel => "RNG",
            Param::ShRate => "RAT",
            Param::LagTime => "LAG",
            Param::VcfCutoff => "CUT",
            Param::VcfReso => "RES",
            Param::VcfKeytrack => "KYB",
            Param::VcfCv1 | Param::VcaCv1 => "CV1",
            Param::VcfCv2 | Param::VcaCv2 => "CV2",
            Param::VcaGain => "GAN",
            Param::AdsrA | Param::ArA => "ATK",
            Param::AdsrD => "DCY",
            Param::AdsrS => "SUS",
            Param::AdsrR | Param::ArR => "REL",
            Param::Mix0 => "M1",
            Param::Mix1 => "M2",
            Param::Mix2 => "M3",
            Param::Mix3 => "M4",
            Param::RevMix => "REV",
            Param::Glide => "GLD",
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Param::Vco1Coarse => "VCO1 coarse",
            Param::Vco1Fine => "VCO1 fine",
            Param::Vco1Fm => "VCO1 FM",
            Param::Vco1Level => "VCO1 level",
            Param::Vco2Coarse => "VCO2 coarse",
            Param::Vco2Fine => "VCO2 fine",
            Param::Vco2Pw => "VCO2 pulse width",
            Param::Vco2Fm => "VCO2 FM",
            Param::Vco2Level => "VCO2 level",
            Param::Vco3Coarse => "VCO3 coarse",
            Param::Vco3Fine => "VCO3 fine",
            Param::Vco3Pw => "VCO3 pulse width",
            Param::Vco3Fm => "VCO3 FM",
            Param::Vco3Level => "VCO3 level",
            Param::NoiseLevel => "Noise level",
            Param::RingLevel => "Ring level",
            Param::ShRate => "S&H rate",
            Param::LagTime => "Lag time",
            Param::VcfCutoff => "VCF cutoff",
            Param::VcfReso => "VCF resonance",
            Param::VcfKeytrack => "VCF key track",
            Param::VcfCv1 => "VCF CV1",
            Param::VcfCv2 => "VCF CV2",
            Param::VcaGain => "VCA gain",
            Param::VcaCv1 => "VCA CV1",
            Param::VcaCv2 => "VCA CV2",
            Param::AdsrA => "ADSR attack",
            Param::AdsrD => "ADSR decay",
            Param::AdsrS => "ADSR sustain",
            Param::AdsrR => "ADSR release",
            Param::ArA => "AR attack",
            Param::ArR => "AR release",
            Param::Mix0 => "Mixer 1",
            Param::Mix1 => "Mixer 2",
            Param::Mix2 => "Mixer 3",
            Param::Mix3 => "Mixer 4",
            Param::RevMix => "Reverb mix",
            Param::Glide => "Glide",
        }
    }

    pub fn to01(self, p: &Params) -> f32 {
        match self {
            Param::Vco1Coarse => (p.vco1_coarse / 48.0 + 0.5).clamp(0.0, 1.0),
            Param::Vco2Coarse => (p.vco2_coarse / 48.0 + 0.5).clamp(0.0, 1.0),
            Param::Vco3Coarse => (p.vco3_coarse / 48.0 + 0.5).clamp(0.0, 1.0),
            Param::Vco1Fine => (p.vco1_fine / 200.0 + 0.5).clamp(0.0, 1.0),
            Param::Vco2Fine => (p.vco2_fine / 200.0 + 0.5).clamp(0.0, 1.0),
            Param::Vco3Fine => (p.vco3_fine / 200.0 + 0.5).clamp(0.0, 1.0),
            Param::Vco1Fm => (p.vco1_fm_amt / 2.0).clamp(0.0, 1.0),
            Param::Vco2Fm => (p.vco2_fm_amt / 2.0).clamp(0.0, 1.0),
            Param::Vco3Fm => (p.vco3_fm_amt / 2.0).clamp(0.0, 1.0),
            Param::Vco1Level => p.vco1_level.clamp(0.0, 1.0),
            Param::Vco2Level => p.vco2_level.clamp(0.0, 1.0),
            Param::Vco3Level => p.vco3_level.clamp(0.0, 1.0),
            Param::Vco2Pw => ((p.vco2_pw - 0.05) / 0.9).clamp(0.0, 1.0),
            Param::Vco3Pw => ((p.vco3_pw - 0.05) / 0.9).clamp(0.0, 1.0),
            Param::NoiseLevel => p.noise_level.clamp(0.0, 1.0),
            Param::RingLevel => p.ring_level.clamp(0.0, 1.0),
            Param::ShRate => log_inv(p.sh_rate, 0.1, 20.0),
            Param::LagTime => log_inv(p.lag_time, 0.001, 1.0),
            Param::VcfCutoff => log_inv(p.vcf_cutoff, 20.0, 20000.0),
            Param::VcfReso => p.vcf_reso.clamp(0.0, 1.0),
            Param::VcfKeytrack => p.vcf_keytrack.clamp(0.0, 1.0),
            Param::VcfCv1 => p.vcf_cv1_amt.clamp(0.0, 1.0),
            Param::VcfCv2 => p.vcf_cv2_amt.clamp(0.0, 1.0),
            Param::VcaGain => p.vca_gain.clamp(0.0, 1.0),
            Param::VcaCv1 => p.vca_cv1_amt.clamp(0.0, 1.0),
            Param::VcaCv2 => p.vca_cv2_amt.clamp(0.0, 1.0),
            Param::AdsrA => log_inv(p.adsr_a, 0.001, 10.0),
            Param::AdsrD => log_inv(p.adsr_d, 0.001, 10.0),
            Param::AdsrS => p.adsr_s.clamp(0.0, 1.0),
            Param::AdsrR => log_inv(p.adsr_r, 0.001, 10.0),
            Param::ArA => log_inv(p.ar_a, 0.001, 10.0),
            Param::ArR => log_inv(p.ar_r, 0.001, 10.0),
            Param::Mix0 => p.mix_levels[0].clamp(0.0, 1.0),
            Param::Mix1 => p.mix_levels[1].clamp(0.0, 1.0),
            Param::Mix2 => p.mix_levels[2].clamp(0.0, 1.0),
            Param::Mix3 => p.mix_levels[3].clamp(0.0, 1.0),
            Param::RevMix => p.rev_mix.clamp(0.0, 1.0),
            Param::Glide => {
                if p.glide <= 0.0 {
                    0.0
                } else {
                    log_inv(p.glide, 0.005, 0.5)
                }
            }
        }
    }

    pub fn from01(self, p: &mut Params, s: f32) {
        let s = s.clamp(0.0, 1.0);
        match self {
            Param::Vco1Coarse => p.vco1_coarse = (s - 0.5) * 48.0,
            Param::Vco2Coarse => p.vco2_coarse = (s - 0.5) * 48.0,
            Param::Vco3Coarse => p.vco3_coarse = (s - 0.5) * 48.0,
            Param::Vco1Fine => p.vco1_fine = (s - 0.5) * 200.0,
            Param::Vco2Fine => p.vco2_fine = (s - 0.5) * 200.0,
            Param::Vco3Fine => p.vco3_fine = (s - 0.5) * 200.0,
            Param::Vco1Fm => p.vco1_fm_amt = s * 2.0,
            Param::Vco2Fm => p.vco2_fm_amt = s * 2.0,
            Param::Vco3Fm => p.vco3_fm_amt = s * 2.0,
            Param::Vco1Level => p.vco1_level = s,
            Param::Vco2Level => p.vco2_level = s,
            Param::Vco3Level => p.vco3_level = s,
            Param::Vco2Pw => p.vco2_pw = 0.05 + s * 0.9,
            Param::Vco3Pw => p.vco3_pw = 0.05 + s * 0.9,
            Param::NoiseLevel => p.noise_level = s,
            Param::RingLevel => p.ring_level = s,
            Param::ShRate => p.sh_rate = log_map(s, 0.1, 20.0),
            Param::LagTime => p.lag_time = log_map(s, 0.001, 1.0),
            Param::VcfCutoff => p.vcf_cutoff = log_map(s, 20.0, 20000.0),
            Param::VcfReso => p.vcf_reso = s,
            Param::VcfKeytrack => p.vcf_keytrack = s,
            Param::VcfCv1 => p.vcf_cv1_amt = s,
            Param::VcfCv2 => p.vcf_cv2_amt = s,
            Param::VcaGain => p.vca_gain = s,
            Param::VcaCv1 => p.vca_cv1_amt = s,
            Param::VcaCv2 => p.vca_cv2_amt = s,
            Param::AdsrA => p.adsr_a = log_map(s, 0.001, 10.0),
            Param::AdsrD => p.adsr_d = log_map(s, 0.001, 10.0),
            Param::AdsrS => p.adsr_s = s,
            Param::AdsrR => p.adsr_r = log_map(s, 0.001, 10.0),
            Param::ArA => p.ar_a = log_map(s, 0.001, 10.0),
            Param::ArR => p.ar_r = log_map(s, 0.001, 10.0),
            Param::Mix0 => p.mix_levels[0] = s,
            Param::Mix1 => p.mix_levels[1] = s,
            Param::Mix2 => p.mix_levels[2] = s,
            Param::Mix3 => p.mix_levels[3] = s,
            Param::RevMix => p.rev_mix = s,
            Param::Glide => {
                p.glide = if s <= 0.0 {
                    0.0
                } else {
                    log_map(s, 0.005, 0.5)
                }
            }
        }
    }

    pub fn display(self, p: &Params) -> String {
        match self {
            Param::Vco1Coarse => format!("{:+.0} st", p.vco1_coarse),
            Param::Vco2Coarse => format!("{:+.0} st", p.vco2_coarse),
            Param::Vco3Coarse => format!("{:+.0} st", p.vco3_coarse),
            Param::Vco1Fine => format!("{:+.0} ct", p.vco1_fine),
            Param::Vco2Fine => format!("{:+.0} ct", p.vco2_fine),
            Param::Vco3Fine => format!("{:+.0} ct", p.vco3_fine),
            Param::Vco1Fm => format!("{:.2} oct", p.vco1_fm_amt),
            Param::Vco2Fm => format!("{:.2} oct", p.vco2_fm_amt),
            Param::Vco3Fm => format!("{:.2} oct", p.vco3_fm_amt),
            Param::Vco1Level => format!("{:.0}%", p.vco1_level * 100.0),
            Param::Vco2Level => format!("{:.0}%", p.vco2_level * 100.0),
            Param::Vco3Level => format!("{:.0}%", p.vco3_level * 100.0),
            Param::Vco2Pw => format!("{:.0}%", p.vco2_pw * 100.0),
            Param::Vco3Pw => format!("{:.0}%", p.vco3_pw * 100.0),
            Param::NoiseLevel => format!("{:.0}%", p.noise_level * 100.0),
            Param::RingLevel => format!("{:.0}%", p.ring_level * 100.0),
            Param::ShRate => format!("{:.1} Hz", p.sh_rate),
            Param::LagTime => fmt_time(p.lag_time),
            Param::VcfCutoff => fmt_freq(p.vcf_cutoff),
            Param::VcfReso => format!("{:.0}%", p.vcf_reso * 100.0),
            Param::VcfKeytrack => format!("{:.0}%", p.vcf_keytrack * 100.0),
            Param::VcfCv1 => format!("{:.0}%", p.vcf_cv1_amt * 100.0),
            Param::VcfCv2 => format!("{:.0}%", p.vcf_cv2_amt * 100.0),
            Param::VcaGain => format!("{:.0}%", p.vca_gain * 100.0),
            Param::VcaCv1 => format!("{:.0}%", p.vca_cv1_amt * 100.0),
            Param::VcaCv2 => format!("{:.0}%", p.vca_cv2_amt * 100.0),
            Param::AdsrA => fmt_time(p.adsr_a),
            Param::AdsrD => fmt_time(p.adsr_d),
            Param::AdsrS => format!("{:.0}%", p.adsr_s * 100.0),
            Param::AdsrR => fmt_time(p.adsr_r),
            Param::ArA => fmt_time(p.ar_a),
            Param::ArR => fmt_time(p.ar_r),
            Param::Mix0 => format!("{:.0}%", p.mix_levels[0] * 100.0),
            Param::Mix1 => format!("{:.0}%", p.mix_levels[1] * 100.0),
            Param::Mix2 => format!("{:.0}%", p.mix_levels[2] * 100.0),
            Param::Mix3 => format!("{:.0}%", p.mix_levels[3] * 100.0),
            Param::RevMix => format!("{:.0}%", p.rev_mix * 100.0),
            Param::Glide => {
                if p.glide <= 0.0 {
                    "off".to_string()
                } else {
                    fmt_time(p.glide)
                }
            }
        }
    }
}

#[derive(Clone, Copy)]
pub struct Slider {
    pub x: i32,
    pub y: i32,
    pub h: i32,
    pub param: Param,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Button {
    VcoWave(usize, Wave),
    NoiseType(NoiseType),
}

pub struct UiState {
    pub pending_out: Option<usize>,
    pub held_keys: HashSet<u8>,
    pub mouse_note: Option<u8>,
    pub drag_slider: Option<usize>,
    pub hover_param: Option<Param>,
    pub status: String,
    pub preset_slot: usize,
    pub octave_shift: i32,
}

impl UiState {
    pub fn new() -> Self {
        Self {
            pending_out: None,
            held_keys: HashSet::new(),
            mouse_note: None,
            drag_slider: None,
            hover_param: None,
            status: String::new(),
            preset_slot: 1,
            octave_shift: 0,
        }
    }
}

// Section geometry: (x, width, title, switch row, sliders).
struct Section {
    x: i32,
    w: i32,
    title: &'static str,
    sliders: &'static [Param],
}

const SECTIONS: &[Section] = &[
    Section {
        x: 8,
        w: 120,
        title: "VCO 1",
        sliders: &[
            Param::Vco1Coarse,
            Param::Vco1Fine,
            Param::Vco1Fm,
            Param::Vco1Level,
        ],
    },
    Section {
        x: 136,
        w: 120,
        title: "VCO 2",
        sliders: &[
            Param::Vco2Coarse,
            Param::Vco2Fine,
            Param::Vco2Pw,
            Param::Vco2Fm,
            Param::Vco2Level,
        ],
    },
    Section {
        x: 264,
        w: 120,
        title: "VCO 3",
        sliders: &[
            Param::Vco3Coarse,
            Param::Vco3Fine,
            Param::Vco3Pw,
            Param::Vco3Fm,
            Param::Vco3Level,
        ],
    },
    Section {
        x: 392,
        w: 120,
        title: "NOISE / MOD",
        sliders: &[
            Param::NoiseLevel,
            Param::RingLevel,
            Param::ShRate,
            Param::LagTime,
        ],
    },
    Section {
        x: 520,
        w: 120,
        title: "VCF",
        sliders: &[
            Param::VcfCutoff,
            Param::VcfReso,
            Param::VcfKeytrack,
            Param::VcfCv1,
            Param::VcfCv2,
        ],
    },
    Section {
        x: 648,
        w: 120,
        title: "VCA",
        sliders: &[Param::VcaGain, Param::VcaCv1, Param::VcaCv2],
    },
    Section {
        x: 776,
        w: 120,
        title: "ADSR",
        sliders: &[Param::AdsrA, Param::AdsrD, Param::AdsrS, Param::AdsrR],
    },
    Section {
        x: 904,
        w: 90,
        title: "AR",
        sliders: &[Param::ArA, Param::ArR],
    },
    Section {
        x: 1002,
        w: 120,
        title: "MIXER",
        sliders: &[Param::Mix0, Param::Mix1, Param::Mix2, Param::Mix3],
    },
    Section {
        x: 1130,
        w: 142,
        title: "OUT",
        sliders: &[Param::RevMix, Param::Glide],
    },
];

pub fn sliders() -> Vec<Slider> {
    let mut v = Vec::new();
    for s in SECTIONS {
        for (i, p) in s.sliders.iter().enumerate() {
            v.push(Slider {
                x: s.x + 10 + i as i32 * 22,
                y: 76,
                h: 130,
                param: *p,
            });
        }
    }
    v
}

fn section_sliders(section_idx: usize) -> Vec<Slider> {
    sliders()
        .into_iter()
        .filter(|sl| {
            let s = &SECTIONS[section_idx];
            sl.x >= s.x && sl.x < s.x + s.w
        })
        .collect()
}

pub fn slider_at(x: i32, y: i32) -> Option<usize> {
    sliders()
        .iter()
        .position(|sl| x >= sl.x - 4 && x < sl.x + 18 && y >= sl.y - 4 && y < sl.y + sl.h + 20)
}

pub fn out_jack_pos(i: usize) -> (i32, i32) {
    let col = (i / 8) as i32;
    let row = (i % 8) as i32;
    (16 + col * 200, 292 + row * 24)
}

pub fn in_jack_pos(i: usize) -> (i32, i32) {
    let col = (i / 10) as i32;
    let row = (i % 10) as i32;
    (664 + col * 200, 292 + row * 24)
}

pub fn out_jack_at(x: i32, y: i32) -> Option<usize> {
    for i in 0..out::COUNT {
        let (jx, jy) = out_jack_pos(i);
        if (x - jx).abs() <= 10 && (y - jy).abs() <= 10 {
            return Some(i);
        }
    }
    None
}

pub fn in_jack_at(x: i32, y: i32) -> Option<usize> {
    for i in 0..inp::COUNT {
        let (jx, jy) = in_jack_pos(i);
        if (x - jx).abs() <= 10 && (y - jy).abs() <= 10 {
            return Some(i);
        }
    }
    None
}

// Keyboard: 2 octaves C3..B4. White-key midis in order.
const WHITE_MIDIS: [u8; 14] = [48, 50, 52, 53, 55, 57, 59, 60, 62, 64, 65, 67, 69, 71];
// (white index after which the black key sits, midi)
const BLACK_KEYS: [(usize, u8); 10] = [
    (0, 49),
    (1, 51),
    (3, 54),
    (4, 56),
    (5, 58),
    (7, 61),
    (8, 63),
    (10, 66),
    (11, 68),
    (12, 70),
];

pub const KEY_TOP: i32 = 540;
pub const KEY_H: i32 = 130;
pub const WHITE_W: i32 = 90;

pub fn key_at(x: i32, y: i32) -> Option<u8> {
    if y < KEY_TOP || y >= KEY_TOP + KEY_H || x < 16 || x >= 16 + 14 * WHITE_W {
        return None;
    }
    // Black keys first (they sit on top).
    if y < KEY_TOP + 80 {
        for (wi, midi) in BLACK_KEYS {
            let cx = 16 + (wi as i32 + 1) * WHITE_W;
            if (x - cx).abs() <= 28 {
                return Some(midi);
            }
        }
    }
    let wi = ((x - 16) / WHITE_W) as usize;
    WHITE_MIDIS.get(wi).copied()
}

pub fn midi_name(m: u8) -> String {
    const NAMES: [&str; 12] = [
        "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
    ];
    format!("{}{}", NAMES[(m % 12) as usize], m as i32 / 12 - 1)
}

fn fill_rect<T: sdl2::render::RenderTarget>(
    canvas: &mut Canvas<T>,
    x: i32,
    y: i32,
    w: u32,
    h: u32,
    c: Color,
) {
    canvas.set_draw_color(c);
    let _ = canvas.fill_rect(Rect::new(x, y, w, h));
}

fn draw_switch_row<T: sdl2::render::RenderTarget>(
    canvas: &mut Canvas<T>,
    x: i32,
    y: i32,
    options: &[(&str, bool, Button)],
    ui_buttons: &mut Vec<(Rect, Button)>,
) {
    let mut bx = x;
    for (label, active, btn) in options {
        let w = 26;
        let bg = if *active { ACCENT } else { TRACK };
        let fg = if *active {
            Color::RGB(0x14, 0x1C, 0x26)
        } else {
            INK
        };
        fill_rect(canvas, bx, y, w as u32, 16, bg);
        draw_text(canvas, bx + 4, y + 4, label, 1, fg);
        ui_buttons.push((Rect::new(bx, y, w as u32, 16), *btn));
        bx += w + 2;
    }
}

pub fn draw<T: sdl2::render::RenderTarget>(
    canvas: &mut Canvas<T>,
    synth: &Synth,
    ui: &UiState,
    midi_port: &Option<String>,
    buttons_out: &mut Vec<(Rect, Button)>,
) {
    canvas.set_draw_color(PANEL);
    let _ = canvas.clear();
    let p = &synth.params;

    // Title.
    draw_text(canvas, 16, 8, "ARP 2600", 2, INK);
    draw_text(canvas, 16 + text_width("ARP 2600", 2) + 12, 14, "INSPIRED VARIANT", 1, DIM);
    let slot_text = format!("PRESET {}", ui.preset_slot);
    draw_text(canvas, 1280 - 16 - text_width(&slot_text, 1), 14, &slot_text, 1, DIM);

    // Sections.
    for (si, s) in SECTIONS.iter().enumerate() {
        fill_rect(canvas, s.x, 32, s.w as u32, 208, SECTION);
        draw_text(canvas, s.x + 8, 38, s.title, 1, INK);
        // Switch rows for VCO wave + noise type.
        match si {
            0 => {
                let opts: [(&str, bool, Button); 2] = [
                    ("SAW", p.vco1_wave == Wave::Saw, Button::VcoWave(0, Wave::Saw)),
                    ("SQR", p.vco1_wave == Wave::Square, Button::VcoWave(0, Wave::Square)),
                ];
                draw_switch_row(canvas, s.x + 8, 54, &opts, buttons_out);
            }
            1 => {
                let opts: [(&str, bool, Button); 4] = [
                    ("SAW", p.vco2_wave == Wave::Saw, Button::VcoWave(1, Wave::Saw)),
                    ("PLS", p.vco2_wave == Wave::Pulse, Button::VcoWave(1, Wave::Pulse)),
                    ("SIN", p.vco2_wave == Wave::Sine, Button::VcoWave(1, Wave::Sine)),
                    ("TRI", p.vco2_wave == Wave::Triangle, Button::VcoWave(1, Wave::Triangle)),
                ];
                draw_switch_row(canvas, s.x + 8, 54, &opts, buttons_out);
            }
            2 => {
                let opts: [(&str, bool, Button); 4] = [
                    ("SAW", p.vco3_wave == Wave::Saw, Button::VcoWave(2, Wave::Saw)),
                    ("PLS", p.vco3_wave == Wave::Pulse, Button::VcoWave(2, Wave::Pulse)),
                    ("SIN", p.vco3_wave == Wave::Sine, Button::VcoWave(2, Wave::Sine)),
                    ("TRI", p.vco3_wave == Wave::Triangle, Button::VcoWave(2, Wave::Triangle)),
                ];
                draw_switch_row(canvas, s.x + 8, 54, &opts, buttons_out);
            }
            3 => {
                let opts: [(&str, bool, Button); 2] = [
                    (
                        "WHT",
                        p.noise_type == NoiseType::White,
                        Button::NoiseType(NoiseType::White),
                    ),
                    (
                        "PNK",
                        p.noise_type == NoiseType::Pink,
                        Button::NoiseType(NoiseType::Pink),
                    ),
                ];
                draw_switch_row(canvas, s.x + 8, 54, &opts, buttons_out);
            }
            _ => {}
        }
        for sl in section_sliders(si) {
            let v = sl.param.to01(p);
            let fill_h = (v * sl.h as f32) as i32;
            fill_rect(canvas, sl.x, sl.y, 14, sl.h as u32, TRACK);
            if fill_h > 0 {
                fill_rect(
                    canvas,
                    sl.x,
                    sl.y + sl.h - fill_h,
                    14,
                    fill_h as u32,
                    ACCENT,
                );
            }
            // Handle line.
            let hy = sl.y + sl.h - fill_h;
            fill_rect(canvas, sl.x - 2, hy - 1, 18, 3, INK);
            draw_text(canvas, sl.x - 2, sl.y + sl.h + 6, sl.param.label(), 1, DIM);
        }
    }

    // Patch bay title + hint.
    draw_text(canvas, 16, 252, "PATCH BAY", 1, INK);
    draw_text(
        canvas,
        16 + text_width("PATCH BAY", 1) + 12,
        252,
        "CLICK AN OUT JACK, THEN AN IN JACK. CLICK A PATCHED IN JACK TO UNPLUG.",
        1,
        DIM,
    );
    // Cables.
    for i in 0..inp::COUNT {
        if let Some(o) = synth.patch.cable_for(i) {
            let (ox, oy) = out_jack_pos(o);
            let (ix, iy) = in_jack_pos(i);
            canvas.set_draw_color(CABLE_COLORS[o % 6]);
            let _ = canvas.draw_line(
                sdl2::rect::Point::new(ox, oy),
                sdl2::rect::Point::new(ix, iy),
            );
        }
    }
    // Jacks.
    for i in 0..out::COUNT {
        let (jx, jy) = out_jack_pos(i);
        let pending = ui.pending_out == Some(i);
        let c = if pending { ACCENT } else { INK };
        fill_rect(canvas, jx - 6, jy - 6, 12, 12, c);
        fill_rect(canvas, jx - 3, jy - 3, 6, 6, SECTION);
        draw_text(canvas, jx + 10, jy - 3, OUT_LABELS[i], 1, DIM);
    }
    for i in 0..inp::COUNT {
        let (jx, jy) = in_jack_pos(i);
        let patched = synth.patch.cable_for(i).is_some();
        let c = if patched { ACCENT } else { INK };
        fill_rect(canvas, jx - 6, jy - 6, 12, 12, c);
        fill_rect(canvas, jx - 3, jy - 3, 6, 6, SECTION);
        draw_text(canvas, jx + 10, jy - 3, IN_LABELS[i], 1, DIM);
    }

    // Keyboard.
    for (i, m) in WHITE_MIDIS.iter().enumerate() {
        let x = 16 + i as i32 * WHITE_W;
        let held = ui.held_keys.contains(m) || synth.gate() && (synth.current_midi() as u8) == *m;
        let c = if held { HELD_KEY } else { WHITE_KEY };
        fill_rect(canvas, x + 1, KEY_TOP, (WHITE_W - 2) as u32, KEY_H as u32, c);
        draw_text(canvas, x + 6, KEY_TOP + KEY_H - 16, &midi_name(*m), 1, DIM);
    }
    for (wi, m) in BLACK_KEYS {
        let cx = 16 + (wi as i32 + 1) * WHITE_W;
        let held = ui.held_keys.contains(&m);
        let c = if held { HELD_KEY } else { BLACK_KEY };
        fill_rect(canvas, cx - 28, KEY_TOP, 56, 80, c);
    }

    // Footer.
    let mut status = String::new();
    if let Some(param) = ui.hover_param {
        status.push_str(&format!("{}  {}   ", param.name(), param.display(p)));
    }
    if !ui.status.is_empty() {
        status.push_str(&ui.status);
        status.push_str("   ");
    }
    match midi_port {
        Some(name) => status.push_str(&format!("MIDI {name}   ")),
        None => status.push_str("MIDI NONE   "),
    }
    if synth.gate() {
        status.push_str(&format!("NOTE {}", midi_name(synth.current_midi() as u8)));
    }
    draw_text(canvas, 16, 684, &status, 1, INK);
    draw_text(
        canvas,
        16,
        706,
        "1-8 LOAD PRESET / SHIFT+1-8 SAVE  |  A-ROW + Z-ROW PLAY, ARROWS SHIFT OCTAVE  |  SPACE PANIC  |  ESC QUIT",
        1,
        DIM,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_hit_testing() {
        assert_eq!(key_at(16 + 45, 600), Some(48)); // C3 white
        assert_eq!(key_at(16 + 90 + 45, 600), Some(50)); // D3 white
        assert_eq!(key_at(16 + 90, 580), Some(49)); // C#3 black
        assert_eq!(key_at(16, 400), None); // above keyboard
        assert_eq!(key_at(2000, 600), None); // off the right edge
    }

    #[test]
    fn jack_hit_testing() {
        let (x, y) = out_jack_pos(0);
        assert_eq!(out_jack_at(x, y), Some(0));
        let (x, y) = in_jack_pos(28);
        assert_eq!(in_jack_at(x, y), Some(28));
        assert_eq!(out_jack_at(640, 400), None);
    }

    #[test]
    fn slider_hit_testing() {
        let sl = sliders()[0];
        assert_eq!(slider_at(sl.x + 7, sl.y + 65), Some(0));
        assert_eq!(slider_at(640, 400), None);
    }
}
