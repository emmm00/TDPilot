# Third-party components

TDPilot is a front-end. The files in `bin/` are the work of other people and are
distributed here unmodified, under their own licenses.

## RyzenAdj

| | |
|---|---|
| Project | [FlyGoat/RyzenAdj](https://github.com/FlyGoat/RyzenAdj) |
| Version | v0.19.0 (released 2026-05-13) |
| Files | `bin/ryzenadj.exe`, `bin/libryzenadj.dll` |
| License | **LGPL-3.0** |
| Source | <https://github.com/FlyGoat/RyzenAdj> (tag `v0.19.0`) |

> RyzenAdj is free software: you can redistribute it and/or modify it under the
> terms of the GNU Lesser General Public License as published by the Free
> Software Foundation, either version 3 of the License, or (at your option) any
> later version.
>
> RyzenAdj is distributed in the hope that it will be useful, but WITHOUT ANY
> WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR
> A PARTICULAR PURPOSE. See the GNU Lesser General Public License for more
> details.

The binaries are shipped **unmodified**; TDPilot only invokes `ryzenadj.exe` as
a separate process. Because RyzenAdj is LGPL-3.0 and is distributed here in
binary form, you are entitled to the corresponding source. It is available at
the GitHub URL above, and the exact release is `v0.19.0`.

If you fork TDPilot and modify `bin/ryzenadj.exe` itself (rather than wrapping
it), that modified binary must be released under the LGPL-3.0 as well.

## Kernel-mode I/O drivers

RyzenAdj requires privileged access to the SMU mailbox, which it obtains through
one of two userspace I/O drivers:

| File | Origin | Notes |
|---|---|---|
| `bin/inpoutx64.sys`, `bin/inpoutx64.dll` | [InpOut32/InpOutx64](https://www.highrez.co.uk/downloads/inpout32/) by Phil Gibbons / Red Fox UK Limited | Freeware, closed source. Authenticode-signed, but the cross-signature dates from 2008. |
| `bin/WinRing0x64.sys`, `bin/WinRing0x64.dll` | [WinRing0](https://github.com/GermanAizek/WinRing0) by Noriyuki MIYAZAKI | BSD-3-Clause style permissive license. Present on Microsoft's vulnerable-driver blocklist. |

These drivers are what make the tool work — and what make it fail on hardened
systems. See the "Driver notes" section of the README before opening an issue
about "driver unavailable".

## Runtime

TDPilot itself needs nothing beyond Windows PowerShell 5.1 and .NET Framework
4.x, both of which ship with Windows 10 and 11. The UI is WPF, loaded from XAML
at runtime, so there is no build step and no package manager involved.

## Fonts

The UI requests `Microsoft YaHei UI` with a `Segoe UI` fallback, both of which
are part of Windows. No fonts are redistributed.
