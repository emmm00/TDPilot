# TDPilot

**A local GUI for tuning AMD Ryzen package power limits on Windows.**

TDPilot is a small, dependency-free desktop app that puts a friendly face on
[RyzenAdj](https://github.com/FlyGoat/RyzenAdj). Instead of typing
`ryzenadj --stapm-limit=25000 ...` into a terminal, you type a number (or drag
the slider, or pick a preset) and watch the effect live on a chart.

> **English** below · **中文** at the bottom · [中文说明](#中文说明)

---

## Why

Laptop OEMs usually give you two power modes and nothing in between — a "quiet"
profile that sags under load, and a "performance" profile that runs hot and loud.
The useful middle ground (say 25 W on a chip whose stock limit is 45 W) is
normally locked behind the vendor's control centre, which may not even exist on
your SKU.

TDPilot talks to the SMU directly through RyzenAdj and lets you pick **any value
between 5 W and 54 W** — typed straight into the box, decimals included — with
one-click presets and an optional logon entry that re-applies your value every
time you sign in.

## Requirements

| | |
|---|---|
| CPU | **AMD Ryzen** with a supported SMU interface (Zen 2 and newer mobile/desktop parts: 4000U/H, 5000, 6000, 7040, 8040, AI 300, ...) |
| OS | Windows 10 / 11 (x64) |
| Runtime | Windows PowerShell 5.1 (preinstalled) — no .NET SDK, no Python, no Node |
| Rights | Administrator (the app re-launches itself through UAC) |
| Kernel | `WinRing0` or `inpoutx64` must be loadable — see [Driver notes](#driver-notes) |

**Not for Intel, ARM/Snapdragon, or pre-Zen AMD.** Intel users want ThrottleStop
or XTU; those chips use a completely different mailbox.

## Quick start

1. Download / clone this folder and keep its layout intact (`bin\` next to
   `src\`).
2. Double-click **`TDPilot.cmd`**. Windows asks for administrator rights — say
   yes. That is required to talk to the SMU.
3. Type the wattage you want (or drag the slider, or hit a preset) and press
   **Apply power cap**. Typing is the fastest route: click the big number, key in
   `25`, press Enter. The `−` / `+` buttons nudge it by one watt, and `↑` / `↓`
   work too.
4. Optional: go to **Automation → Enable auto-apply**. TDPilot drops a small
   .cmd into your Startup folder; it runs at every logon and silently
   re-applies the saved value through an elevated helper.

There is nothing to install and nothing to uninstall: delete the folder when
you are done. Your settings live in `%ProgramData%\TDPilot\`.

## What the screens do

| Page | Purpose |
|---|---|
| **Dashboard** | Live package power, core temperature, clock; power/temp trend chart; the wattage box, slider and presets |
| **Profiles** | Save the current setting under a name, then apply, rename or delete any saved profile |
| **Automation** | Enable / disable the logon entry, see what it will apply, re-apply on demand |
| **Hardware** | CPU model, vendor, thread count, nominal clock, and all live SMU limits (`STAPM`, `PPT FAST/SLOW/APU`, `THM`, `TDC`, `EDC`) |
| **Settings** | Refresh interval, language, open the data folder, copy a diagnostics block |
| **About** | Version, license, environment report (HVCI state, driver blocklist, resolved ryzenadj path) |

The **EN / CH** button in the top bar flips the entire interface. The choice is
remembered in `%ProgramData%\TDPilot\config.json`.

## Command line

The GUI is a thin layer over a scriptable core, so you can skip it:

```powershell
# read the current limits
powershell -NoProfile -ExecutionPolicy Bypass -File .\TDPilot.ps1 -SelfTest

# or use RyzenAdj directly
.\bin\ryzenadj.exe --stapm-limit=25000 --fast-limit=25000 --slow-limit=25000
.\bin\ryzenadj.exe --info
```

`ryzenadj.exe --help` lists every knob (temperature limit, graphics clock,
slow-PPT time constant, ...). TDPilot only exposes the three power limits.

## How it works

```
TDPilot.cmd          launcher, requests UAC elevation
TDPilot.ps1          entry point: loads src/core.ps1 + src/Ui.ps1, builds the window
src/MainWindow.xaml  the whole UI, pure XAML (1020x700, six pages)
src/core.ps1         backend: i18n, SMU access, config, Startup-folder entry
src/Ui.ps1           UI layer: binding, chart drawing, page rendering
bin/ryzenadj.exe     RyzenAdj v0.19.0 (LGPL-3.0) - the actual SMU interface
bin/*.sys, *.dll     the kernel I/O drivers RyzenAdj loads
```

Every write is followed by a `--info` read-back; the value is only reported as
applied when all three limits actually match. That matters because RyzenAdj on
some machines returns a bogus exit code while the write itself succeeds.

## Driver notes

RyzenAdj needs a kernel driver to poke the SMU. Two things can go wrong:

- **The service is stuck.** Right after a resume from sleep, `sc start` may
  return `1450` ("insufficient system resources"). The write still fails until
  the service is restarted. TDPilot retries automatically (up to five attempts,
  including a stop/start and recreating the service); the log line records
  `tries=N`.
- **The driver is blocked.** On systems with **HVCI / Memory Integrity** and
  Microsoft's vulnerable-driver blocklist enabled, an unsigned or
  certificate-expired driver can be refused at load time. `inpoutx64.sys` in
  `bin\` carries a 2008 cross-signature; it loads fine on many machines but is
  rejected on some. If the dashboard shows **Driver unavailable** permanently,
  check *Windows Security → Device security → Core isolation → Memory integrity*
  and the **About** page's environment block. Turning Memory Integrity off is a
  security decision — make it yourself, don't let a tool make it for you.

## Safety

- The limit is a **ceiling, not a lock**. At 25 W the CPU still boosts freely
  for short bursts; it only gets capped when sustained load exceeds the budget.
  Light use feels identical, sustained all-core work gets slower but
  dramatically cooler.
- Values outside 5–54 W are rejected. Don't exceed your chassis' cooling.
- Press **Restore default** to undo everything: it writes the fallback value
  back and removes the logon entry in one go. Settings do not survive a reboot
  unless auto-apply is enabled — that is by design, since RyzenAdj writes are
  volatile.
- Watch the temperature card for the first few minutes after a new value.

## License

TDPilot's own code is released under the **MIT License** — see [LICENSE](LICENSE).

It bundles **RyzenAdj** by FlyGoat and contributors, which is licensed under
**LGPL-3.0**. The unmodified binary is shipped in `bin\`; the corresponding
source is available at <https://github.com/FlyGoat/RyzenAdj> (v0.19.0,
tag `v0.19.0`). See [THIRD-PARTY.md](THIRD-PARTY.md) for details, including the
I/O driver licenses.

## Contributing

Issues and pull requests are welcome. The codebase is deliberately boring:
three files of PowerShell plus one XAML file, no build step, no package manager.
Edit a file, double-click `TDPilot.cmd`, done.

If you add a language, copy the `zh` block in `src/core.ps1`'s `$script:I18N`
table and translate the values — nothing else needs to change.

---

## 中文说明

**TDPilot 是一个给 AMD 锐龙笔记本/台式机调功耗上限的本地小工具**，本质是给
[RyzenAdj](https://github.com/FlyGoat/RyzenAdj) 套了个界面。

厂商通常只给「安静」和「性能」两档，中间那段最舒服的区间（比如默认 45W 的
CPU 限到 25W）藏在自家控制中心里，有些机器甚至没有控制中心。这个工具让你用
**5W ~ 54W 之间的任意值**，直接敲键盘输进去就行（支持小数），也有一键预设，
还能开机 / 唤醒后自动套用。

### 用法

1. 把整个文件夹放在一起（`bin\` 和 `src\` 要在同一层，别拆开）。
2. 双击 **`TDPilot.cmd`**，Windows 会弹 UAC 要管理员权限，点是。
3. 在大数字那里直接输入瓦数，回车就应用；也可以拖滑块、点预设。`−` / `+` 每次
   调 1W，`↑` / `↓` 也行。
4. 想要长期生效：进「自动化」页 →「启用自动套用」，会在你的启动文件夹里放一个
   小 .cmd，每次登录时静默提权、把保存的功耗值重新写一遍。
5. 不想要了：删掉文件夹即可，本工具不写系统目录。设置存在 `%ProgramData%\TDPilot\`。

### 六个页面

| 页面 | 作用 |
|---|---|
| **仪表盘** | 实时功耗 / 温度 / 频率、趋势曲线、瓦数输入框、滑块与预设 |
| **预设方案** | 把当前设置存成一条，之后可以套用、改名、删除 |
| **自动化** | 开关登录自动套用，查看会套用多少瓦，也可以手动立即套用一次 |
| **硬件信息** | CPU 型号、厂商、线程数、标称频率，以及全部 SMU 限制值 |
| **设置** | 刷新间隔、界面语言、打开数据目录、复制诊断信息 |
| **关于** | 版本、许可、环境自检（HVCI 是否开启、驱动黑名单、ryzenadj 实际路径） |

顶栏的 **EN / CH** 按钮切换整个界面的中英文，选择会被记住。

### 关于驱动（重要）

RyzenAdj 需要内核驱动才能访问 SMU，有两种常见故障：

- **服务卡死**：睡眠唤醒后 `sc start` 可能报 1450「系统资源不足」，这时写入会
  一直失败。程序会自动重试最多 5 次（停起、重建服务都试），日志里会写
  `tries=N`。
- **驱动被拦**：开了 **HVCI / 内存完整性** 和微软漏洞驱动黑名单的机器，可能
  直接拒绝加载签名过期（本包里的 `inpoutx64.sys` 是 2008 年的交叉签名）的
  驱动。如果仪表盘一直显示「驱动不可用」，去看「关于」页的环境信息，自己判断
  要不要关内存完整性——这是个安全取舍，工具不该替你做决定。

### 安全提示

- 这个限制是**上限，不是锁频**。设 25W 后 CPU 短时间爆发照样能冲高，只有在
  持续满载时才会被按住。日常轻载感觉不到差别，满载会慢一些但凉很多。
- 超出 5~54W 的值会被拒绝。别超过你机器散热能承受的。
- 想还原就点「恢复默认」，它会一次把功耗写回默认值并移除登录自动套用。
  RyzenAdj 的写入重启后失效，所以不开自动套用的话，重启就自动回默认了。
- 改完新值后，头几分钟留意温度。

### 许可

TDPilot 自身代码用 **MIT**；随包分发的 **RyzenAdj** 是 **LGPL-3.0**，源码在
<https://github.com/FlyGoat/RyzenAdj>。详见 [LICENSE](LICENSE) 和
[THIRD-PARTY.md](THIRD-PARTY.md)。
