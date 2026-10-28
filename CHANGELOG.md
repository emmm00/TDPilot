# Changelog

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
