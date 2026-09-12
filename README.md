# simple-synth

Einfacher Synthesizer in Rust — CLI und GUI mit QWERTZ/QWERTY-Klavier.

Signalweg:

```text
Note → Osc A + Osc B → Mix|AM|Ring|FM|Sync → ADSR A[/B] → Gain → Noise (+|×1±|s×prev) → cpal
                                                                              ↘ WAV
```

- Zwei Oszillatoren: Mix, AM, Ring, FM, Hard-Sync
- Wellenformen: `sine`, `square`, `saw`, `triangle`
- Gemeinsame oder getrennte ADSR-Hüllkurven (Checkbox)
- Zufallsabweichung: Additiv (`sample + n`), multiplikativ (`sample × (1 + n)`) oder Walk (`sample × (val_prev × (1 + n))`), `n ∈ [-val, +val]`
- 8-stimmige Polyphonie
- Realtime-Ausgabe über [cpal](https://crates.io/crates/cpal) 0.18.2
- WAV-Export über [hound](https://crates.io/crates/hound)
- GUI über [eframe](https://crates.io/crates/eframe) 0.36.2

## Voraussetzungen

- Rust 1.95+

Arch Linux:

```bash
sudo pacman -S --needed rust alsa-lib libxcb libxkbcommon vulkan-icd-loader mesa
```

Debian/Ubuntu:

```bash
sudo apt install libasound2-dev libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev
```

Lautstärke vor dem ersten Start runterdrehen.

## Start

```bash
git clone https://github.com/17xbaphomet/simple-synth.git
cd simple-synth
git checkout feature/dual-osc

cargo run --release -- gui
cargo run --release -- play --note C4 --wave square --duration 1.2
cargo run --release -- wav --note E3 --wave triangle --duration 2 --out bass.wav
cargo run --release -- melody --wave square
```

## GUI

Zwei Oszillatoren A/B, Interaktionsmodus und Mix/Ratio/Detune/Tiefe per Slider.

- Checkbox **Getrennte ADSR** (aus = eine gemeinsame Hüllkurve für A und B; an = unabhängige ADSR A und ADSR B nebeneinander).
- Slider **Zufall ±** plus Modus:
  - **Additiv**: `sample + n` mit `n ∈ [-val, +val]`
  - **×(1±val)**: `sample × (1 + n)` mit `n ∈ [-val, +val]`
  - **s×prev(1±)**: `sample × (val_prev × (1 + n))` — `val_prev` ist ein laufender Faktor (Start 1, Reset wenn keine Stimme mehr live)
  Nach Gain, vor dem Clamp auf ±1. Bei `val = 0` kein Rauschen. CLI bleibt ohne Noise.

Layout-Schalter QWERTZ (Standard) / QWERTY. On-Screen-Tasten folgen der physischen Position.

### Tastatur (QWERTZ)

| Taste | Note |
| --- | --- |
| Y S X D C V G B H N J M | C4 C# D D# E F F# G G# A A# B4 |
| Q 2 W 3 E R 5 T 6 Z 7 U I | C5 C# D D# E F F# G G# A A# B C6 |
| ← / → | Oktave − / + |
| F1 F2 F3 F4 | sine / square / saw / triangle (Osc A) |
| Esc | Panic — alle Töne sofort aus |

Auf QWERTY ist die untere C4-Taste `Z` und die obere A5-Taste `Y`.

Mausklick auf die eingezeichneten Tasten spielt ebenfalls. Bis zu 8 Stimmen gleichzeitig.

### Osc-Modi

| Modus | Wirkung |
| --- | --- |
| Mix | linearer Mix A↔B |
| AM | Amplitudenmodulation A × (1 + Tiefe·B) |
| Ring | Ringmodulation A × B |
| FM | B moduliert die Frequenz von A |
| Sync | Hard-Sync: B-Phase reset bei A-Wrap |

Getrennte ADSR: Osc A und Osc B haben eigene Hüllkurven (vor dem Modus; bei FM moduliert B·EnvB, Carrier A·EnvA).

CLI bleibt unverändert: Default ist Mix mit Mix=0 (nur Osc A).

## CLI

```text
simple-synth gui
simple-synth play [--freq 440 | --note A4] [--wave sine] [--duration 1.5]
                 [--gain 0.2] [--attack 0.01] [--decay 0.1] [--sustain 0.7] [--release 0.2]
simple-synth wav  … --out tone.wav
simple-synth melody [--wave square]
```

Noten: `A4`, `C#5`, `Bb3`, `H4` (deutsches H = englisches B).

Wellenform-Aliase: `sin`/`sinus`, `sqr`/`rechteck`, `saege`/`säge`, `tri`/`dreieck`.

## Projektstruktur

```text
src/
  main.rs      CLI (play / wav / melody / gui)
  gui.rs       eframe-Fenster, Dual-Osc, Sliders, On-Screen-Klavier
  keys.rs      QWERTZ/QWERTY → MIDI
  wave.rs      Sine / Square / Saw / Triangle
  osc.rs       Phasenakkumulator + OscInteract
  env.rs       ADSR
  note.rs      Notenname → Frequenz
  engine.rs    8 Stimmen: OscA/OscB * Mode * Env * Gain * Noise
  audio.rs     cpal-Realtime-Ausgabe
  wav.rs       WAV-Export
  error.rs     Domain-Fehler
```
