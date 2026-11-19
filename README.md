# TDPilot

**A native GUI for tuning AMD Ryzen package power limits on Windows.**

TDPilot is a small desktop app that puts a friendly face on
[RyzenAdj](https://github.com/FlyGoat/RyzenAdj). Instead of typing
`ryzenadj --stapm-limit=25000 --slow-limit=25000 ...` into a terminal, you type a
number (or drag the slider, or pick a preset) and watch the effect live on a chart.

Written in Rust with [egui](https://github.com/emilk/egui) — one 4.4 MB executable,
no runtime, no installer.

> **English** below · **中文** at the bottom · [中文说明](#中文说明)

---

## Why

Laptop OEMs usually give you two power modes and nothing in between — a "quiet"
profile that sags under load, and a "performance" profile that runs hot and loud.
The useful middle ground (say 25 W on a chip whose stock limit is 45 W) is
normally locked behind the vendor's control centre, which may not even exist on
your SKU.

TDPilot talks to the SMU directly through RyzenAdj and lets you pick any value
between 3 W and 60 W — typed straight into the box, decimals included — with
one-click presets, session energy statistics, and an optional logon entry that
re-applies your value every time you sign in.

## Requirements

| | |
|---|---|
| CPU | **AMD Ryzen** with a supported SMU interface (Zen 2 and newer: 4000U/H, 5000, 6000, 7040, 8040, AI 300, ...) |
| OS | Windows 10 / 11 (x64) |
| Runtime | None — a single static executable |
| Rights | Administrator (the app re-launches itself through UAC) |
| Kernel | `WinRing0` or `inpoutx64` must be loadable — see [Driver notes](#driver-notes) |

**Not for Intel, ARM/Snapdragon, or pre-Zen AMD.** Intel users want ThrottleStop
or XTU; those chips use a completely different mailbox.

## Quick start

1. Download / unzip the folder and keep its layout intact (`bin\` next to
   `tdpilot.exe`).
2. Double-click **`tdpilot.exe`**. Windows asks for administrator rights — say
   yes. That is required to talk to the SMU.
3. Type the wattage you want (or drag the slider, or hit a preset) and press
   **Apply**. Typing is the fastest route: click the number, key in `25`, press
   `Ctrl+Enter`.
4. Optional: go to **Automation → Enable**, which drops a small `.cmd` into your
   Startup folder. It runs at every logon and silently re-applies the saved value.

There is nothing to install and nothing to uninstall: delete the folder when you
are done. Settings live in `%ProgramData%\TDPilot\` and are shared with the
1.1.x PowerShell line, so upgrading keeps your profiles.

## What the screens do

| Page | Purpose |
|---|---|
| **Dashboard** | Ring gauge for live package power, power/temp/clock tiles, STAPM/FAST/SLOW/THM strip, dual-axis trend chart, session statistics (average / peak / Wh), the wattage box, slider and presets |
| **Profiles** | Save the current setting under a name, then apply, edit or delete any saved profile |
| **Automation** | Enable / disable the logon entry, choose the logon wattage, re-apply on demand, restart the kernel driver |
| **Hardware** | CPU model, vendor, cores/threads, rated clock, architecture, board, BIOS, OS, resolved `ryzenadj.exe` path, driver state |
| **Settings** | Refresh interval, language, open log / data folder, start minimised, thermal limit, reset |
| **About** | Version, license, upstream credits |

The **EN / 中文** button in the top bar flips the entire interface. The choice is
remembered in `config.json`.

### Keyboard shortcuts

| Keys | Action |
|---|---|
| `Ctrl+1` … `Ctrl+6` | Jump to page |
| `Ctrl+Enter` | Apply the current cap |
| `F5` | Force a refresh |

## Command line

The GUI is a thin layer over a scriptable core:

```
tdpilot.exe                    open the dashboard
tdpilot.exe --page profiles    open a specific page
tdpilot.exe --apply-at-logon   headless: apply the saved cap and exit
```

`--apply-at-logon` is what the Startup entry calls, so auto-apply never opens a
window. Everything else goes through `ryzenadj.exe` directly:

```
bin\ryzenadj.exe --info
bin\ryzenadj.exe --stapm-limit=25000 --fast-limit=25000 --slow-limit=25000
```

## How it works

```
tdpilot.exe          the whole UI (Rust + egui, ~4.4 MB, statically linked)
src/main.rs          entry point, UAC relaunch, headless logon mode
src/app/mod.rs       application state, poll/apply worker threads, rail + top bar
src/app/theme.rs     palette, egui style, font fallback (Microsoft YaHei etc.)
src/app/widgets.rs   card, ring gauge, readout, chart, nav item, buttons
src/app/pages/*.rs   the six pages
src/core/backend.rs  runs ryzenadj.exe, parses the PM table, verifies write-back
src/core/config.rs   config.json (%ProgramData%\TDPilot), atomic save
src/core/autostart.rs Startup-folder entry
src/core/hwinfo.rs   CPU / board / BIOS / OS detection
src/core/power.rs    live clock via CallNtPowerInformation
src/core/i18n.rs     Chinese + English strings
bin/ryzenadj.exe     RyzenAdj v0.19.0 (LGPL-3.0) — the actual SMU interface
bin/*.sys, *.dll     the kernel I/O drivers RyzenAdj loads
```

Three details are worth knowing:

- **Reads run on a worker thread.** `ryzenadj --info` costs 250–900 ms, so the UI
  thread never blocks on it; a background thread samples on the configured
  interval and posts snapshots over a channel.
- **Every write is verified by read-back.** RyzenAdj on some machines returns a
  bogus exit code (even crashing with `0xC0000005`) *after* the write already
  succeeded. TDPilot therefore ignores the exit code and only reports success
  when `STAPM`/`PPT FAST`/`PPT SLOW` read back as the requested value.
- **Clock comes from the OS, not the PM table.** On this class of chip
  `CCLK BUSY VALUE` reads `nan`, so the live clock is queried through
  `CallNtPowerInformation(ProcessorInformation)`.

## Driver notes

RyzenAdj needs a kernel driver to poke the SMU. Two things can go wrong:

- **The service is stuck.** After a resume from sleep the driver object can die
  while the service still reports `RUNNING`; every read then fails with
  `Unable to get os_access Obj`. A plain `sc start` does not fix it — TDPilot
  runs a full `sc stop`, waits, then `sc start` (the **Restart kernel driver**
  button does the same thing).
- **The driver is blocked.** On systems with **HVCI / Memory Integrity**
  enabled, a driver carrying an expired cross-signature can be refused at load
  time. The bundled `inpoutx64.sys` is signed with a 2008 certificate; it loads
  on many machines but is rejected on some. If the rail shows **Driver
  unavailable** permanently, check *Windows Security → Device security → Core
  isolation → Memory integrity*. Turning that off is a security decision — make
  it yourself, don't let a tool make it for you.

## Safety

- The limit is a **ceiling, not a lock**. At 25 W the CPU still boosts freely for
  short bursts; it only gets capped when sustained load exceeds the budget. Light
  use feels identical, sustained all-core work gets slower but dramatically
  cooler.
- Values outside 3–60 W are rejected. Don't exceed your chassis' cooling.
- RyzenAdj writes are **volatile** — they do not survive a reboot unless
  auto-apply is enabled. That is by design.
- Watch the temperature tile for the first few minutes after a new value.

## Building from source

```
rustup target add x86_64-pc-windows-gnu
# a mingw-w64 toolchain on PATH is required for the GNU target
cargo build --release --target x86_64-pc-windows-gnu
```

The binary lands in `target\x86_64-pc-windows-gnu\release\tdpilot.exe`; copy it
next to `bin\` and you are done.

## License

TDPilot's own code is released under the **MIT License** — see [LICENSE](LICENSE).

It bundles **RyzenAdj** by FlyGoat and contributors, licensed under
**LGPL-3.0**. The unmodified binary is shipped in `bin\`; the corresponding
source is available at <https://github.com/FlyGoat/RyzenAdj> (v0.19.0, tag
`v0.19.0`). See [THIRD-PARTY.md](THIRD-PARTY.md) for details, including the I/O
driver licenses.

## Contributing

Issues and pull requests are welcome. The codebase is deliberately small: about
3 000 lines of Rust across 18 files, no build scripts beyond Cargo, no generated
code. Edit a file, `cargo run`, done.

To add a language, extend the `strings!` table in `src/core/i18n.rs` — every key
carries both a Chinese and an English string, and the `Lang` enum decides which
one is drawn.

---

## 中文说明

**TDPilot 是一个给 AMD 锐龙笔记本 / 台式机调功耗上限的本地小工具**，本质是给
[RyzenAdj](https://github.com/FlyGoat/RyzenAdj) 套了个原生界面。用 Rust + egui
写成，单个 4.4 MB 可执行文件，不需要运行库、不需要安装。

厂商通常只给「安静」和「性能」两档，中间那段最舒服的区间（比如默认 45W 的 CPU
限到 25W）藏在自家控制中心里，有些机器甚至没有控制中心。这个工具让你用
**3W ~ 60W 之间的任意值**，直接敲键盘输进去就行（支持小数），也有一键预设、
本次运行的能耗统计，还能开机自动套用。

### 用法

1. 把整个文件夹放在一起（`bin\` 和 `tdpilot.exe` 要在同一层，别拆开）。
2. 双击 **`tdpilot.exe`**，Windows 会弹 UAC 要管理员权限，点是。
3. 在大数字那里直接输入瓦数，或者拖滑块、点预设，然后点「应用」。
   快捷键 `Ctrl+Enter` 也能直接应用。
4. 想要长期生效：进「自动化」页点「启用」，会在你的启动文件夹里放一个小
   `.cmd`，每次登录时静默提权、把保存的功耗值重新写一遍。
5. 不想要了：删掉文件夹即可，本工具不写系统目录。设置存在
   `%ProgramData%\TDPilot\`，和 1.1.x 的 PowerShell 版本共用，升级不会丢预设。

### 六个页面

| 页面 | 作用 |
|---|---|
| **仪表盘** | 环形实时功耗、功耗 / 温度 / 频率卡片、STAPM/FAST/SLOW/THM 条、双轴趋势曲线、本次运行统计（平均 / 峰值 / 累计 Wh）、瓦数输入框、滑块与预设 |
| **预设方案** | 把当前设置存成一条，之后可以套用、编辑、删除 |
| **自动化** | 开关登录自动套用、设置登录时的瓦数、手动立即套用、重启内核驱动 |
| **硬件信息** | CPU 型号、厂商、核心 / 线程、标称频率、架构、主板、BIOS、系统、ryzenadj 路径、驱动状态 |
| **设置** | 刷新间隔、界面语言、打开日志 / 数据目录、启动时最小化、温度墙、恢复默认 |
| **关于** | 版本、许可、上游致谢 |

顶栏的 **EN / 中文** 按钮切换整个界面，选择会被记住。

### 快捷键

| 按键 | 作用 |
|---|---|
| `Ctrl+1` … `Ctrl+6` | 切换页面 |
| `Ctrl+Enter` | 应用当前功耗上限 |
| `F5` | 立即刷新 |

### 三个实现细节

- **读数据在后台线程。** `ryzenadj --info` 单次要 250 ~ 900ms，UI 线程绝不阻塞，
  后台线程按设置的间隔采样，通过 channel 把快照丢回界面。
- **每次写入都用回读校验。** RyzenAdj 在部分机器上会在**写入成功之后**返回错误
  退出码（甚至直接 `0xC0000005` 崩溃），所以程序不看退出码，只看回读的
  `STAPM` / `PPT FAST` / `PPT SLOW` 是否等于目标值。
- **频率取自系统 API。** 这类芯片的 PM 表里 `CCLK BUSY VALUE` 读出来是 `nan`，
  所以实时频率走 `CallNtPowerInformation(ProcessorInformation)`。

### 关于驱动（重要）

RyzenAdj 需要内核驱动才能访问 SMU，有两种常见故障：

- **服务卡死**：睡眠唤醒后驱动对象可能已经死了，但服务仍然显示 `RUNNING`，此时
  每次读取都报 `Unable to get os_access Obj`。单独 `sc start` 修不好，程序做的是
  完整的 `sc stop` → 等待 → `sc start`（「重启内核驱动」按钮同理）。
- **驱动被拦**：开了 **HVCI / 内存完整性** 的机器，可能直接拒绝加载签名过期
  （本包里的 `inpoutx64.sys` 是 2008 年的交叉签名）的驱动。如果左侧一直显示
  「驱动异常」，去看 *Windows 安全中心 → 设备安全性 → 内核隔离 → 内存完整性*。
  关掉它是个安全取舍，工具不该替你做决定。

### 安全提示

- 这个限制是**上限，不是锁频**。设 25W 后 CPU 短时间爆发照样能冲高，只有在持续
  满载时才会被按住。日常轻载感觉不到差别，满载会慢一些但凉很多。
- 超出 3 ~ 60W 的值会被拒绝。别超过你机器散热能承受的。
- RyzenAdj 的写入**重启后失效**，不开自动套用的话重启就回默认了，这是刻意设计。
- 改完新值后，头几分钟留意温度卡片。

### 从源码构建

```
rustup target add x86_64-pc-windows-gnu
# GNU target 需要一个在 PATH 里的 mingw-w64 工具链
cargo build --release --target x86_64-pc-windows-gnu
```

产物在 `target\x86_64-pc-windows-gnu\release\tdpilot.exe`，和 `bin\` 放一起即可。

### 许可

TDPilot 自身代码用 **MIT**；随包分发的 **RyzenAdj** 是 **LGPL-3.0**，源码在
<https://github.com/FlyGoat/RyzenAdj>。详见 [LICENSE](LICENSE) 和
[THIRD-PARTY.md](THIRD-PARTY.md)。
