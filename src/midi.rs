//! MIDI input via midir: first available port, auto-reconnect, note
//! on/off, pitch bend (+/-2 semitones), sustain pedal (CC64). Messages
//! arrive on a channel; the main loop drains it each frame.

use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::{Duration, Instant};

pub enum MidiMsg {
    NoteOn(u8),
    NoteOff(u8),
    Bend(f32), // semitones
    Pedal(bool),
}

pub struct Midi {
    rx: Receiver<MidiMsg>,
    _conn: Option<midir::MidiInputConnection<Sender<MidiMsg>>>,
    last_try: Instant,
    pub port_name: Option<String>,
}

fn open_first(tx: Sender<MidiMsg>) -> Option<(midir::MidiInputConnection<Sender<MidiMsg>>, String)> {
    let input = midir::MidiInput::new("arp2600").ok()?;
    let ports = input.ports();
    if ports.is_empty() {
        return None;
    }
    let name = input.port_name(&ports[0]).unwrap_or_else(|_| "?".into());
    let conn = input
        .connect(
            &ports[0],
            "arp2600-in",
            move |_, msg, tx| {
                if msg.len() < 3 {
                    return;
                }
                let status = msg[0] & 0xF0;
                match status {
                    0x90 => {
                        if msg[2] > 0 {
                            let _ = tx.send(MidiMsg::NoteOn(msg[1]));
                        } else {
                            let _ = tx.send(MidiMsg::NoteOff(msg[1]));
                        }
                    }
                    0x80 => {
                        let _ = tx.send(MidiMsg::NoteOff(msg[1]));
                    }
                    0xE0 => {
                        let v = ((msg[2] as i32) << 7) | msg[1] as i32;
                        let bend = (v as f32 - 8192.0) / 8192.0 * 2.0;
                        let _ = tx.send(MidiMsg::Bend(bend));
                    }
                    0xB0 => {
                        if msg[1] == 64 {
                            let _ = tx.send(MidiMsg::Pedal(msg[2] >= 64));
                        }
                    }
                    _ => {}
                }
            },
            tx,
        )
        .ok()?;
    Some((conn, name))
}

impl Midi {
    pub fn new() -> Self {
        let (tx, rx) = channel();
        let (conn, name) = match open_first(tx) {
            Some((c, n)) => (Some(c), Some(n)),
            None => (None, None),
        };
        Self {
            rx,
            _conn: conn,
            last_try: Instant::now(),
            port_name: name,
        }
    }

    /// Drain queued messages; reconnect every 2s when unconnected.
    pub fn poll(&mut self) -> Vec<MidiMsg> {
        if self._conn.is_none() && self.last_try.elapsed() > Duration::from_secs(2) {
            self.last_try = Instant::now();
            let (tx2, rx2) = channel();
            if let Some((conn, name)) = open_first(tx2) {
                let _ = std::mem::replace(&mut self.rx, rx2);
                self._conn = Some(conn);
                self.port_name = Some(name);
            }
        }
        let mut out = Vec::new();
        while let Ok(m) = self.rx.try_recv() {
            out.push(m);
        }
        out
    }

    pub fn list_devices() -> Vec<String> {
        match midir::MidiInput::new("arp2600") {
            Ok(input) => input
                .ports()
                .iter()
                .map(|p| input.port_name(p).unwrap_or_else(|_| "?".into()))
                .collect(),
            Err(_) => Vec::new(),
        }
    }
}
