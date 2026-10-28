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

pub const W: u32 = 1390;
pub const H: u32 = 640;

// Hardware-style palette: the 2600's blue-gray faceplate, white
// silkscreen, silver-ring jacks.
pub const PANEL: Color = Color::RGB(0x4E, 0x5F, 0x70);
pub const SECTION: Color = Color::RGB(0x44, 0x54, 0x64);
pub const RULE: Color = Color::RGB(0x6B, 0x7D, 0x8E);
pub const INK: Color = Color::RGB(0xFF, 0xFF, 0xFF);
pub const DIM: Color = Color::RGB(0xC9, 0xD2, 0xDA);
pub const ACCENT: Color = Color::RGB(0xFF, 0x8C, 0x42);
pub const TRACK: Color = Color::RGB(0x2E, 0x3B, 0x48);
pub const CAP: Color = Color::RGB(0xF2, 0xEF, 0xE6);
pub const JACK_RING: Color = Color::RGB(0xC8, 0xCD, 0xD2);
pub const JACK_HOLE: Color = Color::RGB(0x14, 0x18, 0x1D);
pub const WHITE_KEY: Color = Color::RGB(0xE8, 0xE4, 0xD8);
pub const BLACK_KEY: Color = Color::RGB(0x1A, 0x1E, 0x24);
pub const HELD_KEY: Color = Color::RGB(0xFF, 0x8C, 0x42);

// One cable color per module section, so a cord's origin module reads
// at a glance. Indexed by section (see SECTIONS).
const SECTION_CABLE: [Color; 12] = [
    Color::RGB(0xE5, 0x48, 0x4D), // VCO1 red
    Color::RGB(0xFF, 0xB2, 0x24), // VCO2 amber
    Color::RGB(0x46, 0xA7, 0x58), // VCO3 green
    Color::RGB(0x00, 0xC2, 0xE8), // VCF cyan
    Color::RGB(0xB8, 0x8C, 0xE8), // VCA violet
    Color::RGB(0xFF, 0x8C, 0x42), // ADSR orange
    Color::RGB(0xEC, 0x5E, 0xA1), // AR pink
    Color::RGB(0xD8, 0xD8, 0xD8), // MODULATION light gray
    Color::RGB(0x2F, 0xBF, 0x9B), // MIXER teal
    Color::RGB(0xFF, 0xFF, 0xFF), // OUTPUT white
    Color::RGB(0xB8, 0xE6, 0x2E), // KEYBOARD lime
    Color::RGB(0x6E, 0x9F, 0xE8), // PREAMP steel blue
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
    PreGain,
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
            Param::PreGain => "GAN",
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
            Param::PreGain => "Preamp gain",
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
            Param::PreGain => (p.pre_gain / 2.0).clamp(0.0, 1.0),
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
            Param::PreGain => p.pre_gain = s * 2.0,
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
            Param::PreGain => format!("{:.0}%", p.pre_gain * 100.0),
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

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum JackRef {
    Out(usize),
    In(usize),
}

pub struct UiState {
    pub pending_out: Option<usize>,
    pub held_keys: HashSet<u8>,
    pub mouse_note: Option<u8>,
    pub drag_slider: Option<usize>,
    pub hover_param: Option<Param>,
    pub hover_jack: Option<JackRef>,
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
            hover_jack: None,
            status: String::new(),
            preset_slot: 1,
            octave_shift: 0,
        }
    }
}

// Section geometry: (x, width, title, sliders, out jacks, in jacks).
// Jack labels are short on purpose: the section already says which
// module they belong to, exactly like the hardware. Full names appear
// in the footer on hover.
struct Section {
    x: i32,
    w: i32,
    title: &'static str,
    sliders: &'static [Param],
    outs: &'static [(usize, &'static str)],
    ins: &'static [(usize, &'static str)],
}

const SECTIONS: &[Section] = &[
    Section {
        x: 8,
        w: 100,
        title: "VCO 1",
        sliders: &[
            Param::Vco1Coarse,
            Param::Vco1Fine,
            Param::Vco1Fm,
            Param::Vco1Level,
        ],
        outs: &[(out::VCO1_SAW, "SAW"), (out::VCO1_SQR, "SQR")],
        ins: &[
            (inp::VCO1_FM1, "FM1"),
            (inp::VCO1_FM2, "FM2"),
            (inp::VCO1_CV, "CV"),
        ],
    },
    Section {
        x: 116,
        w: 130,
        title: "VCO 2",
        sliders: &[
            Param::Vco2Coarse,
            Param::Vco2Fine,
            Param::Vco2Pw,
            Param::Vco2Fm,
            Param::Vco2Level,
        ],
        outs: &[
            (out::VCO2_SAW, "SAW"),
            (out::VCO2_PLS, "PLS"),
            (out::VCO2_SIN, "SIN"),
            (out::VCO2_TRI, "TRI"),
        ],
        ins: &[
            (inp::VCO2_FM1, "FM1"),
            (inp::VCO2_FM2, "FM2"),
            (inp::VCO2_CV, "CV"),
            (inp::VCO2_PWM, "PWM"),
        ],
    },
    Section {
        x: 254,
        w: 130,
        title: "VCO 3",
        sliders: &[
            Param::Vco3Coarse,
            Param::Vco3Fine,
            Param::Vco3Pw,
            Param::Vco3Fm,
            Param::Vco3Level,
        ],
        outs: &[
            (out::VCO3_SAW, "SAW"),
            (out::VCO3_PLS, "PLS"),
            (out::VCO3_SIN, "SIN"),
            (out::VCO3_TRI, "TRI"),
        ],
        ins: &[
            (inp::VCO3_FM1, "FM1"),
            (inp::VCO3_FM2, "FM2"),
            (inp::VCO3_CV, "CV"),
            (inp::VCO3_PWM, "PWM"),
        ],
    },
    Section {
        x: 392,
        w: 110,
        title: "VCF",
        sliders: &[
            Param::VcfCutoff,
            Param::VcfReso,
            Param::VcfKeytrack,
            Param::VcfCv1,
            Param::VcfCv2,
        ],
        outs: &[(out::VCF, "OUT")],
        ins: &[
            (inp::VCF_CV1, "CV1"),
            (inp::VCF_CV2, "CV2"),
            (inp::VCF_IN1, "IN1"),
            (inp::VCF_IN2, "IN2"),
        ],
    },
    Section {
        x: 510,
        w: 100,
        title: "VCA",
        sliders: &[Param::VcaGain, Param::VcaCv1, Param::VcaCv2],
        outs: &[(out::VCA, "OUT")],
        ins: &[
            (inp::VCA_CV1, "CV1"),
            (inp::VCA_CV2, "CV2"),
            (inp::VCA_IN, "IN"),
        ],
    },
    Section {
        x: 618,
        w: 100,
        title: "ADSR",
        sliders: &[Param::AdsrA, Param::AdsrD, Param::AdsrS, Param::AdsrR],
        outs: &[(out::ADSR, "OUT")],
        ins: &[],
    },
    Section {
        x: 726,
        w: 70,
        title: "AR",
        sliders: &[Param::ArA, Param::ArR],
        outs: &[(out::AR, "OUT")],
        ins: &[],
    },
    Section {
        x: 804,
        w: 150,
        title: "MODULATION",
        sliders: &[
            Param::NoiseLevel,
            Param::RingLevel,
            Param::ShRate,
            Param::LagTime,
        ],
        outs: &[
            (out::NOISE_W, "WHT"),
            (out::NOISE_P, "PNK"),
            (out::RING, "RNG"),
            (out::SH, "S&H"),
            (out::LAG, "LAG"),
        ],
        ins: &[
            (inp::RING_IN1, "R-1"),
            (inp::RING_IN2, "R-2"),
            (inp::SH_IN, "SH-I"),
            (inp::SH_CLK, "SH-C"),
            (inp::LAG_IN, "LG-I"),
        ],
    },
    Section {
        x: 962,
        w: 110,
        title: "MIXER",
        sliders: &[Param::Mix0, Param::Mix1, Param::Mix2, Param::Mix3],
        outs: &[(out::MIXER, "OUT")],
        ins: &[
            (inp::MIX_IN1, "M1"),
            (inp::MIX_IN2, "M2"),
            (inp::MIX_IN3, "M3"),
            (inp::MIX_IN4, "M4"),
        ],
    },
    Section {
        x: 1080,
        w: 100,
        title: "OUTPUT",
        sliders: &[Param::RevMix],
        outs: &[(out::REVERB, "REV"), (out::VOICE, "VOI")],
        ins: &[(inp::REV_IN1, "R-1"), (inp::REV_IN2, "R-2")],
    },
    Section {
        x: 1188,
        w: 84,
        title: "KEYBOARD",
        sliders: &[Param::Glide],
        outs: &[(out::KB_CV, "CV"), (out::GATE, "GATE")],
        ins: &[],
    },
    Section {
        x: 1280,
        w: 102,
        title: "PREAMP",
        sliders: &[Param::PreGain],
        outs: &[(out::PRE, "PRE"), (out::ENV, "ENV")],
        ins: &[(inp::PRE_IN, "IN")],
    },
];

pub fn sliders() -> Vec<Slider> {
    let mut v = Vec::new();
    for s in SECTIONS {
        // Center the slider group in the section.
        let n = s.sliders.len() as i32;
        let span = (n - 1) * 22 + 18;
        let x0 = s.x + (s.w - span) / 2;
        for (i, p) in s.sliders.iter().enumerate() {
            v.push(Slider {
                x: x0 + i as i32 * 22,
                y: 82,
                h: 150,
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
        .position(|sl| x >= sl.x - 2 && x < sl.x + 20 && y >= sl.y - 4 && y < sl.y + sl.h + 20)
}

pub const JACK_Y: i32 = 256;
pub const JACK_ROW_H: i32 = 22;

fn section_of_out(id: usize) -> usize {
    SECTIONS
        .iter()
        .position(|s| s.outs.iter().any(|(o, _)| *o == id))
        .unwrap_or(0)
}

fn section_of_in(id: usize) -> usize {
    SECTIONS
        .iter()
        .position(|s| s.ins.iter().any(|(i, _)| *i == id))
        .unwrap_or(0)
}

fn out_col_x(s: &Section) -> i32 {
    s.x + 8
}

fn in_col_x(s: &Section) -> i32 {
    if s.outs.is_empty() {
        s.x + 8
    } else {
        // Ins sit in a second column; keep at least 50px pitch so
        // short labels never collide with the out column.
        s.x + 8 + ((s.w - 16) / 2).max(50)
    }
}

pub fn out_jack_pos(id: usize) -> (i32, i32) {
    let s = &SECTIONS[section_of_out(id)];
    let row = s.outs.iter().position(|(o, _)| *o == id).unwrap_or(0) as i32;
    (out_col_x(s), JACK_Y + row * JACK_ROW_H)
}

pub fn in_jack_pos(id: usize) -> (i32, i32) {
    let s = &SECTIONS[section_of_in(id)];
    let row = s.ins.iter().position(|(i, _)| *i == id).unwrap_or(0) as i32;
    (in_col_x(s), JACK_Y + row * JACK_ROW_H)
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

pub fn jack_at(x: i32, y: i32) -> Option<JackRef> {
    if let Some(o) = out_jack_at(x, y) {
        return Some(JackRef::Out(o));
    }
    if let Some(i) = in_jack_at(x, y) {
        return Some(JackRef::In(i));
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

pub const KEY_TOP: i32 = 400;
pub const KEY_H: i32 = 160;
pub const WHITE_W: i32 = 90;
/// Keyboard row is centered in the panel width.
pub const KEY_X0: i32 = (W as i32 - 14 * WHITE_W) / 2;

pub fn key_at(x: i32, y: i32) -> Option<u8> {
    if y < KEY_TOP || y >= KEY_TOP + KEY_H || x < KEY_X0 || x >= KEY_X0 + 14 * WHITE_W {
        return None;
    }
    // Black keys first (they sit on top).
    if y < KEY_TOP + 96 {
        for (wi, midi) in BLACK_KEYS {
            let cx = KEY_X0 + (wi as i32 + 1) * WHITE_W;
            if (x - cx).abs() <= 28 {
                return Some(midi);
            }
        }
    }
    let wi = ((x - KEY_X0) / WHITE_W) as usize;
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

fn draw_cable<T: sdl2::render::RenderTarget>(
    canvas: &mut Canvas<T>,
    x0: i32,
    y0: i32,
    x1: i32,
    y1: i32,
    c: Color,
) {
    // draw_line is a no-op on this SDL build's surface canvas
    // (verified: even a full diagonal test line produced zero pixels),
    // so rasterize cords as 2px blocks with Bresenham.
    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    let (mut x, mut y) = (x0, y0);
    canvas.set_draw_color(c);
    loop {
        let _ = canvas.fill_rect(Rect::new(x - 1, y - 1, 2, 2));
        if x == x1 && y == y1 {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x += sx;
        }
        if e2 <= dx {
            err += dx;
            y += sy;
        }
    }
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
    draw_text(canvas, W as i32 - 16 - text_width(&slot_text, 1), 14, &slot_text, 1, DIM);

    // Sections: framed module boxes, like the hardware panel.
    for (si, s) in SECTIONS.iter().enumerate() {
        fill_rect(canvas, s.x, 32, s.w as u32, 348, SECTION);
        // Border.
        canvas.set_draw_color(RULE);
        let _ = canvas.draw_rect(Rect::new(s.x, 32, s.w as u32, 348));
        draw_text(canvas, s.x + 8, 40, s.title, 1, INK);
        fill_rect(
            canvas,
            s.x + 8,
            51,
            text_width(s.title, 1) as u32,
            1,
            INK,
        );
        // Switch rows for VCO wave + noise type.
        match si {
            0 => {
                let opts: [(&str, bool, Button); 2] = [
                    ("SAW", p.vco1_wave == Wave::Saw, Button::VcoWave(0, Wave::Saw)),
                    ("SQR", p.vco1_wave == Wave::Square, Button::VcoWave(0, Wave::Square)),
                ];
                draw_switch_row(canvas, s.x + (s.w - 54) / 2, 58, &opts, buttons_out);
            }
            1 => {
                let opts: [(&str, bool, Button); 4] = [
                    ("SAW", p.vco2_wave == Wave::Saw, Button::VcoWave(1, Wave::Saw)),
                    ("PLS", p.vco2_wave == Wave::Pulse, Button::VcoWave(1, Wave::Pulse)),
                    ("SIN", p.vco2_wave == Wave::Sine, Button::VcoWave(1, Wave::Sine)),
                    ("TRI", p.vco2_wave == Wave::Triangle, Button::VcoWave(1, Wave::Triangle)),
                ];
                draw_switch_row(canvas, s.x + (s.w - 110) / 2, 58, &opts, buttons_out);
            }
            2 => {
                let opts: [(&str, bool, Button); 4] = [
                    ("SAW", p.vco3_wave == Wave::Saw, Button::VcoWave(2, Wave::Saw)),
                    ("PLS", p.vco3_wave == Wave::Pulse, Button::VcoWave(2, Wave::Pulse)),
                    ("SIN", p.vco3_wave == Wave::Sine, Button::VcoWave(2, Wave::Sine)),
                    ("TRI", p.vco3_wave == Wave::Triangle, Button::VcoWave(2, Wave::Triangle)),
                ];
                draw_switch_row(canvas, s.x + (s.w - 110) / 2, 58, &opts, buttons_out);
            }
            7 => {
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
                draw_switch_row(canvas, s.x + (s.w - 54) / 2, 58, &opts, buttons_out);
            }
            _ => {}
        }
        for sl in section_sliders(si) {
            // Hardware slider: thin track, tick marks, rectangular cap.
            let cx = sl.x + 9;
            fill_rect(canvas, cx - 1, sl.y, 2, sl.h as u32, TRACK);
            for t in 0..5 {
                let ty = sl.y + sl.h - t * sl.h / 4;
                fill_rect(canvas, cx - 5, ty - 1, 10, 2, DIM);
            }
            let v = sl.param.to01(p);
            let cy = sl.y + sl.h - (v * sl.h as f32) as i32 - 5;
            fill_rect(canvas, sl.x, cy, 18, 10, CAP);
            fill_rect(canvas, sl.x, cy + 4, 18, 2, TRACK);
            draw_text(
                canvas,
                sl.x + 9 - text_width(sl.param.label(), 1) / 2,
                sl.y + sl.h + 6,
                sl.param.label(),
                1,
                DIM,
            );
        }
    }

    // Patch cords drape over the panel like real cords, colored by
    // their source module so a cord's origin reads at a glance.
    for i in 0..inp::COUNT {
        if let Some(o) = synth.patch.cable_for(i) {
            let (ox, oy) = out_jack_pos(o);
            let (ix, iy) = in_jack_pos(i);
            draw_cable(
                canvas,
                ox,
                oy,
                ix,
                iy,
                SECTION_CABLE[section_of_out(o)],
            );
        }
    }
    // Jacks live inside their module sections, like the hardware.
    for s in SECTIONS {
        for (o, label) in s.outs {
            let (jx, jy) = out_jack_pos(*o);
            let ring = if ui.pending_out == Some(*o) {
                ACCENT
            } else {
                JACK_RING
            };
            fill_rect(canvas, jx - 6, jy - 6, 12, 12, ring);
            fill_rect(canvas, jx - 3, jy - 3, 6, 6, JACK_HOLE);
            draw_text(canvas, jx + 10, jy - 3, label, 1, DIM);
        }
        for (i, label) in s.ins {
            let (jx, jy) = in_jack_pos(*i);
            let ring = if synth.patch.cable_for(*i).is_some() {
                ACCENT
            } else {
                JACK_RING
            };
            fill_rect(canvas, jx - 6, jy - 6, 12, 12, ring);
            fill_rect(canvas, jx - 3, jy - 3, 6, 6, JACK_HOLE);
            draw_text(canvas, jx + 10, jy - 3, label, 1, DIM);
        }
    }
    // Highlight the section being patched from, or hovered over.
    let mut highlight: Vec<usize> = Vec::new();
    if let Some(o) = ui.pending_out {
        highlight.push(section_of_out(o));
    }
    if let Some(j) = ui.hover_jack {
        let si = match j {
            JackRef::Out(o) => section_of_out(o),
            JackRef::In(i) => section_of_in(i),
        };
        if !highlight.contains(&si) {
            highlight.push(si);
        }
    }
    for si in highlight {
        let s = &SECTIONS[si];
        canvas.set_draw_color(ACCENT);
        let _ = canvas.draw_rect(Rect::new(s.x, 32, s.w as u32, 348));
    }

    // Keyboard.
    for (i, m) in WHITE_MIDIS.iter().enumerate() {
        let x = KEY_X0 + i as i32 * WHITE_W;
        let held = ui.held_keys.contains(m) || synth.gate() && (synth.current_midi() as u8) == *m;
        let c = if held { HELD_KEY } else { WHITE_KEY };
        fill_rect(canvas, x + 1, KEY_TOP, (WHITE_W - 2) as u32, KEY_H as u32, c);
        draw_text(canvas, x + 6, KEY_TOP + KEY_H - 16, &midi_name(*m), 1, TRACK);
    }
    for (wi, m) in BLACK_KEYS {
        let cx = KEY_X0 + (wi as i32 + 1) * WHITE_W;
        let held = ui.held_keys.contains(&m);
        let c = if held { HELD_KEY } else { BLACK_KEY };
        fill_rect(canvas, cx - 28, KEY_TOP, 56, 96, c);
    }

    // Footer.
    let mut status = String::new();
    if let Some(o) = ui.pending_out {
        let si = section_of_out(o);
        status.push_str(&format!(
            "PATCHING FROM {} {} - CLICK AN IN JACK   ",
            SECTIONS[si].title, OUT_LABELS[o]
        ));
    }
    if let Some(j) = ui.hover_jack {
        let (si, full, dir) = match j {
            JackRef::Out(o) => (section_of_out(o), OUT_LABELS[o], "OUT"),
            JackRef::In(i) => (section_of_in(i), IN_LABELS[i], "IN"),
        };
        status.push_str(&format!("{} {} {}   ", SECTIONS[si].title, full, dir));
    }
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
    draw_text(canvas, 16, 580, &status, 1, INK);
    draw_text(
        canvas,
        16,
        604,
        "PATCH: CLICK AN OUT JACK, THEN AN IN JACK (JACKS SIT WITH THEIR MODULES)  |  1-8 LOAD / SHIFT+1-8 SAVE  |  SPACE PANIC  |  ESC QUIT",
        1,
        DIM,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_hit_testing() {
        assert_eq!(key_at(KEY_X0 + 45, 460), Some(48)); // C3 white
        assert_eq!(key_at(KEY_X0 + 90 + 45, 460), Some(50)); // D3 white
        assert_eq!(key_at(KEY_X0 + 90, 430), Some(49)); // C#3 black
        assert_eq!(key_at(KEY_X0, 380), None); // above keyboard
        assert_eq!(key_at(2000, 460), None); // off the right edge
    }

    #[test]
    fn preamp_jacks_live_in_their_section() {
        let (x, y) = out_jack_pos(out::ENV);
        let s = &SECTIONS[11];
        assert_eq!(s.title, "PREAMP");
        assert!(x > s.x && x < s.x + s.w);
        assert_eq!(out_jack_at(x, y), Some(out::ENV));
        let (x, y) = in_jack_pos(inp::PRE_IN);
        assert!(x > s.x && x < s.x + s.w);
        assert_eq!(in_jack_at(x, y), Some(inp::PRE_IN));
    }

    #[test]
    fn jack_hit_testing() {
        let (x, y) = out_jack_pos(0);
        assert_eq!(out_jack_at(x, y), Some(0));
        let (x, y) = in_jack_pos(28);
        assert_eq!(in_jack_at(x, y), Some(28));
        // Jacks now live in their modules: VCO1 SAW sits in the VCO1
        // section, and the MODULATION section holds the ring inputs.
        let (mx, my) = out_jack_pos(out::VCO2_PLS);
        assert!(mx > SECTIONS[1].x && mx < SECTIONS[1].x + SECTIONS[1].w);
        let _ = my;
        assert_eq!(out_jack_at(640, 600), None);
    }

    #[test]
    fn slider_hit_testing() {
        let sl = sliders()[0];
        assert_eq!(slider_at(sl.x + 9, sl.y + 75), Some(0));
        assert_eq!(slider_at(640, 600), None);
    }
}
