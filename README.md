# arp2600

An ARP 2600-inspired semi-modular synthesizer in Rust + SDL2: three
VCOs, a 4-pole ladder filter, ADSR + AR transient generators, a ring
modulator, white/pink noise, sample & hold, a lag processor, a utility
mixer, a spring-style reverb, and a patch bay with normalized routing.
All continuous params are vertical sliders, the 2600's own idiom.

This is an inspired variant, not a circuit clone: the module set and
signal flow follow the 2600, but the implementation is original DSP
(naive waveforms, a Moog-style ladder, a Schroeder spring network),
and a few conveniences are added where the hardware would make you
patch (a VCF key-track knob, a glide knob, a master soft clip).

## Build and run

```sh
cargo build --release
./target/release/arp2600
```

Needs SDL2 + ALSA dev libraries on Linux
(`libsdl2-dev`, `libasound2-dev`; after a VM replacement, re-run
`apt-get update` first). MIDI input uses midir (ALSA); the first
available port is opened, with auto-reconnect.

Headless checks:

```sh
./target/release/arp2600 --smoke 3        # render 3 s, report peak/rms
./target/release/arp2600 --render out.wav 3
./target/release/arp2600 --shot shot.bmp  # offscreen faceplate
./target/release/arp2600 --list-devices   # MIDI ports
```

## The default voice (no patching needed)

Keyboard CV -> all three VCOs; the voice bus (VCO levels + noise +
ring) -> VCF input; VCF -> VCA input; ADSR -> VCA CV; AR -> VCF CV.
Play the on-screen keyboard, the computer keys, or a MIDI keyboard.

## Patch bay

Click an OUT jack, then an IN jack to patch; click a patched IN jack
to unplug. Patching an input breaks its normalized connection. One
cable per input. Every patch read uses the previous sample's jack
value, so feedback patches (VCF out -> VCF CV, and so on) are stable
with a single sample of latency.

Normalized connections:

| Input | Default source |
|---|---|
| VCO1/2/3 CV | KB CV |
| VCF IN1 | VOICE (the VCO/noise/ring bus) |
| VCF CV1 | AR |
| VCA IN | VCF |
| VCA CV1 | ADSR |
| RING 1 / RING 2 | VCO1 SAW / VCO2 SINE |
| S&H IN | NOISE W |
| LAG IN | KB CV |
| MIX 1..4 | VCO1 SAW / VCO2 SAW / VCO3 SAW / NOISE W |
| REV 1 | VCA |

CV jacks carry volts (1 V/oct); audio jacks carry -1..1 signals. VCO
FM inputs are bipolar audio scaled by the FM slider (octaves). The
VCO pitch reference is A440: the note arrives only through the CV
jacks, so patching a VCO CV input replaces keyboard tracking instead
of adding to it.

## Controls

- Sliders: drag vertically, or hover + mouse wheel (0.05 steps).
- Wave switches: click SAW/SQR (VCO1), SAW/PLS/SIN/TRI (VCO2/3),
  WHT/PNK (noise).
- Keys 1-8 load presets, Shift+1-8 save (plain text, 8 slots in
  `~/.arp2600/presets.txt`).
- Computer keys: Z-row = lower octave (Z=C), Q-row = upper octave
  (Q=C), arrow keys shift octaves, Space = all notes off (panic),
  Esc quits.
- MIDI: note on/off, pitch bend (+/-2 semitones), sustain pedal
  (CC64). The footer shows the connected port.

## v1 omissions (future work)

No sequencer, no preamp/envelope follower (no audio input yet), no
electronic switch, monophonic only (last-note priority, retrigger).
The patch bay, engine, and preset format are built to take them.
