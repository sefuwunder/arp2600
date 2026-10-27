//! Minimal WAV writer (16-bit PCM, stereo) for `--render`.

use std::fs::File;
use std::io::Write;

pub fn write_wav_stereo(path: &str, samples: &[f32], sr: u32) -> std::io::Result<()> {
    let mut f = File::create(path)?;
    let data_bytes = (samples.len() * 2) as u32;
    let chunk_size = 36 + data_bytes;
    f.write_all(b"RIFF")?;
    f.write_all(&chunk_size.to_le_bytes())?;
    f.write_all(b"WAVE")?;
    f.write_all(b"fmt ")?;
    f.write_all(&16u32.to_le_bytes())?;
    f.write_all(&1u16.to_le_bytes())?; // PCM
    f.write_all(&2u16.to_le_bytes())?; // stereo
    f.write_all(&sr.to_le_bytes())?;
    f.write_all(&(sr * 2 * 2).to_le_bytes())?; // byte rate
    f.write_all(&4u16.to_le_bytes())?; // block align
    f.write_all(&16u16.to_le_bytes())?; // bits
    f.write_all(b"data")?;
    f.write_all(&data_bytes.to_le_bytes())?;
    for s in samples {
        let v = (s.clamp(-1.0, 1.0) * 32767.0) as i16;
        f.write_all(&v.to_le_bytes())?;
    }
    Ok(())
}
