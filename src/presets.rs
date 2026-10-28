//! Plain-text presets: 8 slots in `~/.arp2600/presets.txt`, one
//! `[n]` section per slot, `key=value` lines. Unknown keys and bad
//! values are ignored on load, so older files stay readable.

use crate::osc::Wave;
use crate::voice::{NoiseType, Params};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

pub const SLOTS: usize = 8;

pub fn presets_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    PathBuf::from(home).join(".arp2600").join("presets.txt")
}

fn wave_to_str(w: Wave) -> &'static str {
    match w {
        Wave::Saw => "Saw",
        Wave::Square => "Square",
        Wave::Pulse => "Pulse",
        Wave::Sine => "Sine",
        Wave::Triangle => "Triangle",
    }
}

fn wave_from_str(s: &str) -> Option<Wave> {
    match s {
        "Saw" => Some(Wave::Saw),
        "Square" => Some(Wave::Square),
        "Pulse" => Some(Wave::Pulse),
        "Sine" => Some(Wave::Sine),
        "Triangle" => Some(Wave::Triangle),
        _ => None,
    }
}

fn noise_to_str(n: NoiseType) -> &'static str {
    match n {
        NoiseType::White => "White",
        NoiseType::Pink => "Pink",
    }
}

fn noise_from_str(s: &str) -> Option<NoiseType> {
    match s {
        "White" => Some(NoiseType::White),
        "Pink" => Some(NoiseType::Pink),
        _ => None,
    }
}

pub fn params_to_map(p: &Params) -> Vec<(String, String)> {
    let f = |v: f32| v.to_string();
    vec![
        ("vco1_wave".into(), wave_to_str(p.vco1_wave).into()),
        ("vco1_coarse".into(), f(p.vco1_coarse)),
        ("vco1_fine".into(), f(p.vco1_fine)),
        ("vco1_fm_amt".into(), f(p.vco1_fm_amt)),
        ("vco1_level".into(), f(p.vco1_level)),
        ("vco2_wave".into(), wave_to_str(p.vco2_wave).into()),
        ("vco2_coarse".into(), f(p.vco2_coarse)),
        ("vco2_fine".into(), f(p.vco2_fine)),
        ("vco2_pw".into(), f(p.vco2_pw)),
        ("vco2_fm_amt".into(), f(p.vco2_fm_amt)),
        ("vco2_level".into(), f(p.vco2_level)),
        ("vco3_wave".into(), wave_to_str(p.vco3_wave).into()),
        ("vco3_coarse".into(), f(p.vco3_coarse)),
        ("vco3_fine".into(), f(p.vco3_fine)),
        ("vco3_pw".into(), f(p.vco3_pw)),
        ("vco3_fm_amt".into(), f(p.vco3_fm_amt)),
        ("vco3_level".into(), f(p.vco3_level)),
        ("noise_type".into(), noise_to_str(p.noise_type).into()),
        ("noise_level".into(), f(p.noise_level)),
        ("ring_level".into(), f(p.ring_level)),
        ("sh_rate".into(), f(p.sh_rate)),
        ("lag_time".into(), f(p.lag_time)),
        ("glide".into(), f(p.glide)),
        ("vcf_cutoff".into(), f(p.vcf_cutoff)),
        ("vcf_reso".into(), f(p.vcf_reso)),
        ("vcf_keytrack".into(), f(p.vcf_keytrack)),
        ("vcf_cv1_amt".into(), f(p.vcf_cv1_amt)),
        ("vcf_cv2_amt".into(), f(p.vcf_cv2_amt)),
        ("vca_gain".into(), f(p.vca_gain)),
        ("vca_cv1_amt".into(), f(p.vca_cv1_amt)),
        ("vca_cv2_amt".into(), f(p.vca_cv2_amt)),
        ("adsr_a".into(), f(p.adsr_a)),
        ("adsr_d".into(), f(p.adsr_d)),
        ("adsr_s".into(), f(p.adsr_s)),
        ("adsr_r".into(), f(p.adsr_r)),
        ("ar_a".into(), f(p.ar_a)),
        ("ar_r".into(), f(p.ar_r)),
        ("mix0".into(), f(p.mix_levels[0])),
        ("mix1".into(), f(p.mix_levels[1])),
        ("mix2".into(), f(p.mix_levels[2])),
        ("mix3".into(), f(p.mix_levels[3])),
        ("rev_mix".into(), f(p.rev_mix)),
        ("pre_gain".into(), f(p.pre_gain)),
    ]
}

pub fn apply_map(p: &mut Params, map: &HashMap<String, String>) {
    let g = |k: &str| map.get(k).and_then(|v| v.parse::<f32>().ok());
    if let Some(w) = map.get("vco1_wave").and_then(|v| wave_from_str(v)) {
        p.vco1_wave = w;
    }
    if let Some(w) = map.get("vco2_wave").and_then(|v| wave_from_str(v)) {
        p.vco2_wave = w;
    }
    if let Some(w) = map.get("vco3_wave").and_then(|v| wave_from_str(v)) {
        p.vco3_wave = w;
    }
    if let Some(n) = map.get("noise_type").and_then(|v| noise_from_str(v)) {
        p.noise_type = n;
    }
    macro_rules! set {
        ($field:ident, $key:literal) => {
            if let Some(v) = g($key) {
                p.$field = v;
            }
        };
    }
    set!(vco1_coarse, "vco1_coarse");
    set!(vco1_fine, "vco1_fine");
    set!(vco1_fm_amt, "vco1_fm_amt");
    set!(vco1_level, "vco1_level");
    set!(vco2_coarse, "vco2_coarse");
    set!(vco2_fine, "vco2_fine");
    set!(vco2_pw, "vco2_pw");
    set!(vco2_fm_amt, "vco2_fm_amt");
    set!(vco2_level, "vco2_level");
    set!(vco3_coarse, "vco3_coarse");
    set!(vco3_fine, "vco3_fine");
    set!(vco3_pw, "vco3_pw");
    set!(vco3_fm_amt, "vco3_fm_amt");
    set!(vco3_level, "vco3_level");
    set!(noise_level, "noise_level");
    set!(ring_level, "ring_level");
    set!(sh_rate, "sh_rate");
    set!(lag_time, "lag_time");
    set!(glide, "glide");
    set!(vcf_cutoff, "vcf_cutoff");
    set!(vcf_reso, "vcf_reso");
    set!(vcf_keytrack, "vcf_keytrack");
    set!(vcf_cv1_amt, "vcf_cv1_amt");
    set!(vcf_cv2_amt, "vcf_cv2_amt");
    set!(vca_gain, "vca_gain");
    set!(vca_cv1_amt, "vca_cv1_amt");
    set!(vca_cv2_amt, "vca_cv2_amt");
    set!(adsr_a, "adsr_a");
    set!(adsr_d, "adsr_d");
    set!(adsr_s, "adsr_s");
    set!(adsr_r, "adsr_r");
    set!(ar_a, "ar_a");
    set!(ar_r, "ar_r");
    set!(rev_mix, "rev_mix");
    set!(pre_gain, "pre_gain");
    for (i, k) in ["mix0", "mix1", "mix2", "mix3"].iter().enumerate() {
        if let Some(v) = g(k) {
            p.mix_levels[i] = v;
        }
    }
}

fn parse_file(text: &str) -> Vec<(usize, HashMap<String, String>)> {
    let mut slots = Vec::new();
    let mut cur: Option<(usize, HashMap<String, String>)> = None;
    for line in text.lines() {
        let line = line.trim();
        if line.starts_with('[') && line.ends_with(']') {
            if let Some(c) = cur.take() {
                slots.push(c);
            }
            if let Ok(n) = line[1..line.len() - 1].trim().parse::<usize>() {
                cur = Some((n, HashMap::new()));
            }
        } else if let Some((_, map)) = cur.as_mut() {
            if let Some((k, v)) = line.split_once('=') {
                map.insert(k.trim().to_string(), v.trim().to_string());
            }
        }
    }
    if let Some(c) = cur.take() {
        slots.push(c);
    }
    slots
}

pub fn load_slot(slot: usize) -> Option<Params> {
    let text = fs::read_to_string(presets_path()).ok()?;
    for (n, map) in parse_file(&text) {
        if n == slot {
            let mut p = Params::default();
            apply_map(&mut p, &map);
            return Some(p);
        }
    }
    None
}

pub fn save_slot(slot: usize, p: &Params) -> std::io::Result<()> {
    let path = presets_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut slots: Vec<(usize, Vec<(String, String)>)> = Vec::new();
    if let Ok(text) = fs::read_to_string(&path) {
        for (n, map) in parse_file(&text) {
            let entries: Vec<(String, String)> = map.into_iter().collect();
            slots.push((n, entries));
        }
    }
    let mut out = String::new();
    let mut written = false;
    for (n, entries) in &slots {
        if *n == slot {
            out.push_str(&format!("[{slot}]\n"));
            for (k, v) in params_to_map(p) {
                out.push_str(&format!("{k}={v}\n"));
            }
            written = true;
        } else {
            out.push_str(&format!("[{n}]\n"));
            for (k, v) in entries {
                out.push_str(&format!("{k}={v}\n"));
            }
        }
    }
    if !written {
        out.push_str(&format!("[{slot}]\n"));
        for (k, v) in params_to_map(p) {
            out.push_str(&format!("{k}={v}\n"));
        }
    }
    fs::write(path, out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_map() {
        let mut p = Params::default();
        p.vco2_wave = Wave::Pulse;
        p.vco2_pw = 0.33;
        p.vcf_cutoff = 1234.0;
        let map: HashMap<String, String> = params_to_map(&p).into_iter().collect();
        let mut q = Params::default();
        apply_map(&mut q, &map);
        assert_eq!(q.vco2_wave, Wave::Pulse);
        assert!((q.vco2_pw - 0.33).abs() < 1e-6);
        assert!((q.vcf_cutoff - 1234.0).abs() < 1e-3);
    }

    #[test]
    fn round_trip_file() {
        let tmp = std::env::temp_dir().join("arp2600-preset-test");
        let _ = fs::remove_dir_all(&tmp);
        let old_home = std::env::var("HOME").ok();
        std::env::set_var("HOME", &tmp);
        let mut p = Params::default();
        p.vco3_wave = Wave::Triangle;
        p.adsr_s = 0.42;
        save_slot(3, &p).expect("save");
        let q = load_slot(3).expect("load");
        assert_eq!(q.vco3_wave, Wave::Triangle);
        assert!((q.adsr_s - 0.42).abs() < 1e-6);
        assert!(load_slot(5).is_none());
        if let Some(h) = old_home {
            std::env::set_var("HOME", h);
        }
        let _ = fs::remove_dir_all(&tmp);
    }
}
