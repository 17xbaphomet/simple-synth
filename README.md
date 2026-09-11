# simple-synth

Einfacher monophoner Synthesizer in Rust.

Signalweg:

```text
Note/Freq → Oscillator (Phase) → Waveform → ADSR → Gain → cpal-Stream
                                                      ↘ hound WAV
```

- Wellenformen: `sine`, `square`, `saw`, `triangle`
- ADSR-Hüllkurve
- Realtime-Ausgabe über [cpal](https://crates.io/crates/cpal) 0.18.2
- WAV-Export über [hound](https://crates.io/crates/hound) (funktioniert auch ohne Soundkarte)

## Voraussetzungen

- Rust 1.85+
- Linux: `sudo apt install libasound2-dev`

Lautstärke vor dem ersten Start runterdrehen.

## Start

```bash
git clone https://github.com/17xbaphomet/simple-synth.git
cd simple-synth

cargo run --release -- play --freq 440 --wave sine
cargo run --release -- play --note C4 --wave square --duration 1.2
cargo run --release -- play --note A4 --wave saw --attack 0.05 --release 0.4
cargo run --release -- wav --note E3 --wave triangle --duration 2 --out bass.wav
cargo run --release -- melody --wave square
```

Noten: `A4`, `C#5`, `Bb3`, `H4` (deutsches H = englisches B).

Wellenform-Aliase: `sin`/`sinus`, `sqr`/`rechteck`, `saege`/`säge`, `tri`/`dreieck`.

## CLI

```text
simple-synth play [--freq 440 | --note A4] [--wave sine] [--duration 1.5]
                 [--gain 0.2] [--attack 0.01] [--decay 0.1] [--sustain 0.7] [--release 0.2]
simple-synth wav  … --out tone.wav
simple-synth melody [--wave square]
```

## Projektstruktur

```text
src/
  main.rs      CLI (play / wav / melody)
  wave.rs      Sine / Square / Saw / Triangle
  osc.rs       Phasenakkumulator
  env.rs       ADSR
  note.rs      Notenname → Frequenz
  engine.rs    eine Stimme: Osc * Env * Gain
  audio.rs     cpal-Realtime-Ausgabe
  wav.rs       WAV-Export
  error.rs     Domain-Fehler
```

Naive Square- und Saw-Wellen aliasen. Als Nächstes: PolyBLEP, Polyphonie, MIDI.
