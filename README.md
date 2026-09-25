# Zweiton

Dual-Osc Synthesizer in Rust — Standalone (CLI + GUI) und VST3/CLAP-Instrument.

Arbeitsname für Phase 1. Repo bleibt `simple-synth`, Plugin-ID ist `Zweiton`.

```text
Note → Osc A + Osc B → Mix|AM|Ring|FM|Sync → ADSR A[/B] → Gain → Noise → Host
                                                                              ↘ WAV
```

- Zwei Oszillatoren: Mix, AM, Ring, FM, Hard-Sync
- Wellenformen: `sine`, `square`, `saw`, `triangle`
- Gemeinsame oder getrennte ADSR-Hüllkurven (Checkbox)
- Zufallsabweichung: Additiv (`sample + n`), multiplikativ (`sample × (1 + n)`), Walk (`sample × (val_prev × (1 + n))`) oder Parabol (`sample × v(2 + n − v)`); `n` gleichverteilt in `[-val, +val]` oder gaußverteilt `N(0, val)`
- 8-stimmige Polyphonie
- Standalone: [cpal](https://crates.io/crates/cpal) + [hound](https://crates.io/crates/hound) + [eframe](https://crates.io/crates/eframe)
- Plugin: [nice-plug](https://crates.io/crates/nice-plug) 0.4 — VST3 + CLAP, kein Mutex/Alloc auf dem Audio-Thread

## Workspace

```text
crates/engine   zweiton-engine      DSP (kein cpal, kein eframe)
crates/plugin   zweiton             VST3 + CLAP
src/            zweiton-standalone  CLI + eframe-GUI
```

## Voraussetzungen

- Rust 1.88+

Arch Linux:

```bash
sudo pacman -S --needed rust alsa-lib libxcb libxkbcommon vulkan-icd-loader mesa
```

Debian/Ubuntu:

```bash
sudo apt install libasound2-dev libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev
```

Lautstärke vor dem ersten Start runterdrehen.

## Standalone

```bash
git clone https://github.com/17xbaphomet/simple-synth.git
cd simple-synth
git checkout feature/vst3

cargo run --release --bin zweiton -- gui
cargo run --release --bin zweiton -- play --note C4 --wave square --duration 1.2
cargo run --release --bin zweiton -- wav --note E3 --wave triangle --duration 2 --out bass.wav
cargo run --release --bin zweiton -- melody --wave square
```

## Plugin (VST3 / CLAP) — Phase 1

Ableton Live 12 lädt **VST3 64-bit**. Phase 1 nutzt die Generic-Parameter-UI des Hosts (noch kein eigenes In-Host-Editor-Fenster).

Win + Mac from day 1. MIDI vom Host, Stereo-Ausgang. Parameter sind automatisierbar und werden mit dem Liveset gespeichert.

```bash
cargo install cargo-nice-plug
cargo nice-plug bundle -p zweiton --release
```

Artefakt unter `target/bundled/zweiton.vst3` bzw. `zweiton.clap`.

Kopieren:

- Windows VST3: `C:\Program Files\Common Files\VST3\Zweiton.vst3`
- Windows CLAP: `C:\Program Files\Common Files\CLAP\`
- macOS VST3: `~/Library/Audio/Plug-Ins/VST3/Zweiton.vst3`
- macOS CLAP: `~/Library/Audio/Plug-Ins/CLAP/`

Danach in Live: Preferences → Plug-ins → VST3 system folder an → Rescan.

Signierte Installer und In-Host-egui sind **nicht** Teil von Phase 1.

## GUI

Zwei Oszillatoren A/B, Interaktionsmodus und Mix/Ratio/Detune/Tiefe per Slider.

- Checkbox **Getrennte ADSR** (aus = eine gemeinsame Hüllkurve für A und B; an = unabhängige ADSR A und ADSR B nebeneinander).
- Slider **Zufall ±** plus Modus:
  - **Additiv**: `sample + n` mit `n ∈ [-val, +val]`
  - **×(1±val)**: `sample × (1 + n)` mit `n ∈ [-val, +val]`
  - **s×prev(1±)**: `sample × (val_prev × (1 + n))` — `val_prev` ist ein laufender Faktor (Start 1, Reset wenn keine Stimme mehr live)
  - **s×v(2+n−v)**: `sample × (2·val_prev + val_prev·n − val_prev²)` — `val_prev` wie Walk (Start 1, Reset bei Stille). Ohne Rauschen läuft `v` gegen 1.
  - **Verteilung**: **Gleich** (`n ∈ [-val, +val]`) oder **Gauß** (`n ~ N(0, val)`), gilt für alle vier Modi.
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
zweiton gui
zweiton play [--freq 440 | --note A4] [--wave sine] [--duration 1.5]
             [--gain 0.2] [--attack 0.01] [--decay 0.1] [--sustain 0.7] [--release 0.2]
zweiton wav  … --out tone.wav
zweiton melody [--wave square]
```

Noten: `A4`, `C#5`, `Bb3`, `H4` (deutsches H = englisches B).

Wellenform-Aliase: `sin`/`sinus`, `sqr`/`rechteck`, `saege`/`säge`, `tri`/`dreieck`.

## Projektstruktur

```text
crates/engine/src/   DSP: wave, osc, env, note, engine
crates/plugin/src/   nice-plug VST3 + CLAP Wrapper
src/
  main.rs            CLI (play / wav / melody / gui)
  gui.rs             eframe-Fenster, Dual-Osc, Sliders, On-Screen-Klavier
  keys.rs            QWERTZ/QWERTY → MIDI
  audio.rs           cpal-Realtime-Ausgabe
  wav.rs             WAV-Export
  error.rs           HostError (cpal + hound + SynthError)
```

## Lizenz

MIT OR Apache-2.0. Engine-Snapshot auf GitHub bleibt öffentlich. Plugin-Wrapper kann später proprietär werden.
