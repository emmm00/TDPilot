# Changelog

## v1.2.0 — 2026-11-19

**Complete rewrite in Rust.** The PowerShell + WPF front end is gone; TDPilot is
now a single 4.4 MB native executable that starts in about 1.6 seconds instead of
ten, and every screen was redesigned from scratch.

### Changed

- **Rewritten in Rust with egui.** No PowerShell, no XAML, no runtime to install.
  The release build is one statically linked `tdpilot.exe` (4.4 MB) plus the
  `bin\` folder RyzenAdj needs.
- **Startup went from 10–16 s to ~1.6 s.** The old version spent 3.3 s parsing
  XAML and 3–4 s waiting for a cold PowerShell host; both costs are gone. Memory
  footprint is about 22 MB.
- **New visual design.** Deep-slate dark theme with a cyan accent, a fixed
  navigation rail instead of a tab strip, card-based layout, a ring gauge for
  live power, and a dual-axis trend chart (watts left, °C right) that
  auto-scales to your cap.
- **Wider range: 3 W – 60 W** (was 5 W – 54 W).
- Settings moved into the GUI as proper controls: refresh interval is now
  selectable and there is a start-minimised option.

### Added

- **Session statistics card.** Average power, peak power and accumulated energy
  (Wh) for the current run, integrated over the real sampling interval — plus
  elapsed time and sample count. Resets when you restart the app.
- **Keyboard shortcuts.** `Ctrl+1`…`Ctrl+6` jump between pages, `Ctrl+Enter`
  applies the current cap, `F5` forces a refresh.
- **`--page <name>`** so you can launch straight into a page (`dashboard`,
  `profiles`, `automation`, `hardware`, `settings`, `about`).
- **`--apply-at-logon`** — the headless mode the Startup entry uses. It applies
  the saved cap and exits without ever drawing a window.
- **Thermal limit (TCTL) control** on the Settings page. `0` leaves it untouched;
  any other value between 60 °C and 105 °C is written together with the power cap.
- **Live clock via the OS.** The PM table reports `CCLK BUSY VALUE` as `nan` on
  this chip generation, so the frequency readout now comes from
  `CallNtPowerInformation(ProcessorInformation)`.
- **Driver restart button** on the dashboard and the automation page, for when
  the `inpoutx64` service wedges after a resume.

### Fixed

- **Writes are now verified by reading back.** RyzenAdj on some machines exits
  with a bogus code — including `0xC0000005` — *after* the write has already
  taken effect. TDPilot ignores the exit code and reports success only when
  `STAPM` / `PPT FAST` / `PPT SLOW` read back at the requested value, so a
  crash-after-success no longer shows up as a failure.
- **The PM table is parsed correctly.** The previous parser split on the leading
  `|` and therefore matched nothing; the field names were also wrong (`PPT LIMIT
  FAST`, not `FAST LIMIT`). Real numbers now appear in every tile.
- **Units.** Values from `--info` are printed in watts; they are now converted to
  milliwatts before being stored, instead of being divided by 1000 twice.
- **Charts no longer sit empty.** egui only repaints on input by default, so the
  history never grew; the app now schedules a repaint at the sampling interval.
- **`config.json` with a UTF-8 BOM loads.** The 1.1.x PowerShell version wrote a
  BOM via `ConvertTo-Json`; the parser used to reject it and silently fall back to
  defaults. The BOM is stripped on read, and writes stay BOM-free.
- **Elevated relaunch keeps its arguments.** The UAC re-launch passed a null
  parameter string, so `--page` was lost; the command line is now forwarded.
- **Hardware page correctness.** The rated clock was shown in raw hex (`0xed1`)
  because `reg.exe` prints `REG_DWORD` in hex; the OS line was garbled because
  `cmd /c ver` emits GBK; and every field label on that page was hard-coded in
  English regardless of the language setting. All three are fixed.
- Subtitles were clipped on every page — the top bar was 10 px too short.

## v1.1.0 — 2026-10-28

Usability pass driven by real use: the power cap is now a text field you can
type into, and saved profiles are no longer write-once.

### Added

- **Type the wattage.** The big number on the dashboard is an editable box.
  Click it, key in a value (`25`, `22.5`, `18,5` all work), press **Enter** to
  apply. `↑` / `↓` nudge by one watt; the box and the slider stay in sync.
- **`−` / `+` buttons** next to the box for single-watt steps.
- **Manage profiles.** Each saved profile now has **Apply / Edit / Delete**.
  Edit opens a small dialog that lets you rename it and change its wattage;
  Delete asks for confirmation. New profiles are seeded from the current
  wattage instead of a fixed default.
- **CHANGELOG.md** (this file).

### Changed

- The hint under the slider now reads "type a number, drag the slider or pick a
  preset, then click Apply" instead of implying the slider was the only way in.
- Invalid input (letters, out-of-range numbers) is clamped to 5–54 W instead of
  being silently ignored, so the box never lies about what will be applied.
- Bumped the version shown on the About page to v1.1.0.

### Fixed

- Profiles could not be renamed or removed once created — the only option was
  Apply.
- `ValSetW` used to be read-only text, which made precise values (e.g. 26.5 W)
  awkward to reach with a mouse.

## v1.0.0 — 2026-10-28

First release.

- Six-page WPF interface (Dashboard, Profiles, Automation, Hardware, Settings,
  About), Chinese and English, one click to switch.
- Live package power / temperature / clock plus a rolling trend chart, polled
  through `ryzenadj.exe --info`.
- Any package power cap from 5 W to 54 W.
- One-click presets: 15 W / 25 W / 30 W / 45 W / restore default / custom.
- Optional logon auto-apply: a small `.cmd` in your Startup folder silently
  re-applies the saved value through an elevated helper. Includes driver
  self-healing (`sc stop` → `sc start`, then re-create the service if needed).
- Hardware page lists every SMU limit (`STAPM`, `PPT FAST/SLOW/APU`, `THM`,
  `TDC`, `EDC`).
- Settings page: refresh interval, interface language, open data folder, copy a
  diagnostics block.
- Bundles RyzenAdj v0.19.0 (LGPL-3.0) plus the `inpoutx64` / `WinRing0` drivers.
