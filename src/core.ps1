# =====================================================================
#  TDPilot - 核心逻辑模块 (Core backend, dot-sourced by Tdpilot.ps1)
#  纯 ASCII 源文件；界面文案走 i18n 表，避免本地化编码问题
# =====================================================================

Set-StrictMode -Off
$ErrorActionPreference = 'Stop'

# ---------------------------------------------------------------- paths
$script:RootDir   = $PSScriptRoot
$script:BinRoot   = $script:RootDir
foreach ($cand in @((Join-Path $script:RootDir 'bin'), (Join-Path (Split-Path $script:RootDir -Parent) 'bin'))) {
  if (Test-Path (Join-Path $cand 'ryzenadj.exe')) { $script:BinRoot = $cand; break }
}
$script:BinDir    = $script:BinRoot
$script:RyzenExe  = Join-Path $script:BinRoot 'ryzenadj.exe'
$script:DataDir   = Join-Path $env:ProgramData 'TDPilot'
$script:CfgFile   = Join-Path $script:DataDir 'config.json'
$script:LogFile   = Join-Path $script:DataDir 'tdpilot.log'
# The scheduled task used by older TDPilot builds; kept so that the
# uninstaller can clean it up after an in-place upgrade.
$script:LegacyTaskName = 'TDPilotApply'
$script:DefaultMW = 45000

# ---------------------------------------------------------------- i18n
$script:Lang = 'zh'

$script:I18N = @{
  zh = @{
    'app.title'          = 'TDPilot - AMD 功耗调节'
    'nav.control'        = '控制'
    'nav.system'         = '系统'
    'nav.dashboard'      = '仪表盘'
    'nav.profiles'       = '预设方案'
    'nav.auto'           = '自动化'
    'nav.hardware'       = '硬件信息'
    'nav.settings'       = '设置'
    'nav.about'          = '关于'
    'page.dashboard'     = '仪表盘'
    'page.dash.sub'      = '实时功耗与温度监控'
    'page.profiles'      = '预设方案'
    'page.profiles.sub'  = '为不同场景保存常用功率组合'
    'page.auto'          = '自动化'
    'page.auto.sub'      = '登录时自动把保存的功耗上限写回处理器'
    'page.hardware'      = '硬件信息'
    'page.hardware.sub'  = '处理器与固件详情'
    'page.settings'      = '设置'
    'page.settings.sub'  = '刷新频率、日志与维护'
    'page.about'         = '关于'
    'page.about.sub'     = '版本与开源许可'
    'card.power'         = '封装功耗'
    'card.temp'          = '核心温度'
    'card.freq'          = '当前频率'
    'chart.title'        = '功耗 / 温度趋势'
    'power.limit'        = '功耗上限'
    'power.hint'         = '拖动滑块或点选预设，然后点「应用」'
    'power.apply'        = '应用功耗上限'
    'power.pending'      = '已修改，点「应用」生效'
    'power.presets'      = '快速预设'
    'preset.power'       = '省电'
    'preset.balanced'    = '均衡'
    'preset.perf'        = '性能'
    'preset.full'        = '满血'
    'preset.default'     = '恢复默认'
    'preset.custom'      = '自定义…'
    'preset.customask'   = '输入功耗上限（5 ~ 54 W，可带小数）：'
    'log.title'          = '运行日志'
    'log.clear'          = '清空'
    'btn.refresh'        = '刷新'
    'btn.lang'           = 'EN'
    'status.checking'    = '检测中…'
    'status.driverok'    = '驱动就绪'
    'status.drivermissing' = '驱动不可用'
    'status.noCpuPower'  = '硬件访问失败'
    'msg.applied'        = '已应用到 {0} W'
    'msg.applying'       = '正在应用…'
    'msg.applyfail'      = '应用失败：{0}'
    'msg.needadmin'      = '需要管理员权限，正在请求提升…'
    'msg.done'           = '完成'
    'msg.reading'        = '读取中…'
    'msg.nochart'        = '采集数据中…'
    'unit.w'             = 'W'
    'unit.c'             = '°C'
    'unit.mhz'           = 'MHz'
  }
  en = @{
    'app.title'          = 'TDPilot - AMD Power Tuner'
    'nav.control'        = 'CONTROL'
    'nav.system'         = 'SYSTEM'
    'nav.dashboard'      = 'Dashboard'
    'nav.profiles'       = 'Profiles'
    'nav.auto'           = 'Automation'
    'nav.hardware'        = 'Hardware'
    'nav.settings'       = 'Settings'
    'nav.about'          = 'About'
    'page.dashboard'     = 'Dashboard'
    'page.dash.sub'      = 'Live power and temperature monitor'
    'page.profiles'      = 'Profiles'
    'page.profiles.sub'  = 'Save power combinations for different scenarios'
    'page.auto'          = 'Automation'
    'page.auto.sub'      = 'Re-apply the saved power cap automatically at logon'
    'page.hardware'      = 'Hardware'
    'page.hardware.sub'  = 'Processor and firmware details'
    'page.settings'      = 'Settings'
    'page.settings.sub'  = 'Refresh rate, logs and maintenance'
    'page.about'         = 'About'
    'page.about.sub'     = 'Version and open-source license'
    'card.power'         = 'Package power'
    'card.temp'          = 'Core temp'
    'card.freq'          = 'Current clock'
    'chart.title'        = 'Power / Temperature trend'
    'power.limit'        = 'Power cap'
    'power.hint'         = 'Drag the slider or pick a preset, then click Apply'
    'power.apply'        = 'Apply power cap'
    'power.pending'      = 'Changed - press Apply'
    'power.presets'      = 'Quick presets'
    'preset.power'       = 'Eco'
    'preset.balanced'    = 'Balanced'
    'preset.perf'        = 'Perf'
    'preset.full'        = 'Full'
    'preset.default'     = 'Reset'
    'preset.custom'      = 'Custom…'
    'preset.customask'   = 'Enter power cap (5 ~ 54 W, decimals allowed):'
    'log.title'          = 'Activity log'
    'log.clear'          = 'Clear'
    'btn.refresh'        = 'Refresh'
    'btn.lang'           = 'CH'
    'status.checking'    = 'Checking…'
    'status.driverok'    = 'Driver ready'
    'status.drivermissing' = 'Driver unavailable'
    'status.noCpuPower'  = 'Hardware access failed'
    'msg.applied'        = 'Applied {0} W'
    'msg.applying'       = 'Applying…'
    'msg.applyfail'      = 'Apply failed: {0}'
    'msg.needadmin'      = 'Administrator required, elevating…'
    'msg.done'           = 'Done'
    'msg.reading'        = 'Reading…'
    'msg.nochart'        = 'Collecting…'
    'unit.w'             = 'W'
    'unit.c'             = '°C'
    'unit.mhz'           = 'MHz'
  }
}

function T {
  param([string]$Key)
  $tbl = $script:I18N[$script:Lang]
  if ($null -eq $tbl) { $tbl = $script:I18N['en'] }
  if ($tbl.ContainsKey($Key)) { return $tbl[$Key] }
  return $Key
}

function Set-Language {
  param([string]$Code)
  if ($script:I18N.ContainsKey($Code)) { $script:Lang = $Code; return $true }
  return $false
}

# ------------------------------------------------------------- elevation
function Test-IsAdmin {
  try {
    $id = [System.Security.Principal.WindowsIdentity]::GetCurrent()
    $p  = New-Object System.Security.Principal.WindowsPrincipal($id)
    return $p.IsInRole([System.Security.Principal.WindowsBuiltInRole]::Administrator)
  } catch { return $false }
}

function Test-DeviceGuard {
  # Returns hashtable with HVCI / blocklist state (best effort, no WMI needed)
  $r = @{ Hvci = $false; Blocklist = $false; WhqlEnforce = $false }
  try {
    $k = 'HKLM:\SYSTEM\CurrentControlSet\Control\DeviceGuard\Scenarios\HypervisorEnforcedCodeIntegrity'
    $v = Get-ItemProperty -Path $k -Name Enabled -ErrorAction SilentlyContinue
    if ($v -and $v.Enabled -eq 1) { $r.Hvci = $true }
  } catch { }
  try {
    $k = 'HKLM:\SYSTEM\CurrentControlSet\Control\CI\Config'
    $v = Get-ItemProperty -Path $k -Name VulnerableDriverBlocklistEnable -ErrorAction SilentlyContinue
    if ($v -and $v.VulnerableDriverBlocklistEnable -eq 1) { $r.Blocklist = $true }
  } catch { }
  return $r
}

# ------------------------------------------------------- ryzenadj wrapper
function Invoke-RyzenAdj {
  param([string[]]$Arguments = @('--info'))
  $result = @{ Code = -1; Text = '' }
  $exe = $script:RyzenExe
  if (-not (Test-Path $exe)) { $result.Text = "ryzenadj.exe not found at $exe"; return $result }
  $line = '"' + $exe + '" ' + ($Arguments -join ' ')
  $old = $ErrorActionPreference
  $ErrorActionPreference = 'Continue'
  try {
    Push-Location $script:RootDir
    $txt = (@(& cmd.exe /c $line 2>&1 | ForEach-Object { [string]$_ }) -join "`r`n")
    $code = $LASTEXITCODE
    Pop-Location
  } catch {
    $txt = $_.Exception.Message
    $code = -1
    try { Pop-Location } catch { }
  } finally {
    $ErrorActionPreference = $old
  }
  $result.Text = $txt
  $result.Code = $code
  return $result
}

# Parse "| STAPM LIMIT               |        25.000 |" style rows
function Parse-InfoText {
  param([string]$Text)
  $o = @{}
  if (-not $Text) { return $o }
  $map = @{
    'STAPM LIMIT'       = 'stapmLimit'
    'STAPM VALUE'       = 'stapmValue'
    'PPT LIMIT FAST'    = 'fastLimit'
    'PPT VALUE FAST'    = 'fastValue'
    'PPT LIMIT SLOW'    = 'slowLimit'
    'PPT VALUE SLOW'    = 'slowValue'
    'PPT LIMIT APU'     = 'apuLimit'
    'TDC LIMIT VDD'     = 'tdcVdd'
    'TDC LIMIT SOC'     = 'tdcSoc'
    'EDC LIMIT VDD'     = 'edcVdd'
    'EDC LIMIT SOC'     = 'edcSoc'
    'THM LIMIT CORE'    = 'thmLimit'
    'THM VALUE CORE'    = 'thmValue'
    'StapmTimeConst'    = 'stapmTime'
    'SlowPPTTimeConst'  = 'slowTime'
  }
  foreach ($line in ($Text -split "`r?`n")) {
    if ($line -notmatch '\|') { continue }
    $parts = $line -split '\|'
    if ($parts.Count -lt 3) { continue }
    $name = $parts[1].Trim()
    $val  = $parts[2].Trim()
    foreach ($k in $map.Keys) {
      if ($name -ieq $k) {
        $d = 0.0
        if ([double]::TryParse($val, [ref]$d)) { $o[$map[$k]] = $d }
        break
      }
    }
  }
  return $o
}

function Get-Snapshot {
  <#
    One call = one --info (~0.9s). Returns $null when hardware is inaccessible.
  #>
  $r = Invoke-RyzenAdj @('--info')
  if ($r.Code -ne 0) { return $null }
  $p = Parse-InfoText $r.Text
  if ($p.Count -eq 0) { return $null }
  return @{
    Raw       = $r.Text
    Data      = $p
    When      = Get-Date
  }
}

# ---------------------------------------------------------- driver health
function Get-DriverServices {
  # Any service that points at one of our known drivers
  $out = @()
  $roots = @('HKLM:\SYSTEM\CurrentControlSet\Services')
  foreach ($root in $roots) {
    Get-ChildItem $root -ErrorAction SilentlyContinue | ForEach-Object {
      $ip = $null
      try { $ip = (Get-ItemProperty -Path $_.PSPath -Name ImagePath -ErrorAction SilentlyContinue).ImagePath } catch { }
      if ($ip -and ($ip -match 'inpoutx64|WinRing0|TDPilot')) {
        $exit = $null
        try { $exit = (Get-ItemProperty -Path $_.PSPath -Name ExitCode -ErrorAction SilentlyContinue).ExitCode } catch { }
        $out += [pscustomobject]@{ Name = $_.PSChildName; ImagePath = $ip; ExitCode = $exit }
      }
    }
  }
  return $out
}

function Test-HardwareAccess {
  # exit 0 from --info is the only reliable signal
  $r = Invoke-RyzenAdj @('--info')
  return @{
    Ok     = ($r.Code -eq 0)
    Code   = $r.Code
    Detail = $r.Text
  }
}

# ---------------------------------------------------------------- config
function Get-Config {
  $d = @{
    Lang        = 'zh'
    PowerMW     = 25000
    RefreshMs   = 3000
    AutoApply   = $false
    LastApplied = $null
  }
  if (Test-Path $script:CfgFile) {
    try {
      $j = Get-Content $script:CfgFile -Raw -Encoding UTF8 | ConvertFrom-Json
      foreach ($k in @($d.Keys)) { if ($null -ne $j.$k) { $d[$k] = $j.$k } }
    } catch { }
  }
  return $d
}

function Save-Config {
  param($Cfg)
  if (-not (Test-Path $script:DataDir)) {
    New-Item -ItemType Directory -Path $script:DataDir -Force -ErrorAction SilentlyContinue | Out-Null
  }
  try {
    ($Cfg | ConvertTo-Json -Depth 5) | Set-Content -Path $script:CfgFile -Encoding UTF8 -ErrorAction Stop
    return $true
  } catch { return $false }
}

# ------------------------------------------------------------ cpu identity
function Get-CpuIdentity {
  $r = @{ Name = 'Unknown'; Vendor = ''; Cores = 0; Logical = 0; MaxMHz = 0; IsAmd = $false }
  try {
    $k = Get-ItemProperty 'HKLM:\HARDWARE\DESCRIPTION\System\CentralProcessor\0' -ErrorAction Stop
    $r.Name = $k.ProcessorNameString
    $r.Vendor = $k.VendorIdentifier
    $r.MaxMHz = $k.'~MHz'
    if ($r.Vendor -match 'AuthenticAMD') { $r.IsAmd = $true }
  } catch { }
  try {
    $k2 = Get-ItemProperty 'HKLM:\HARDWARE\DESCRIPTION\System\CentralProcessor' -ErrorAction SilentlyContinue
    $r.Logical = [System.Environment]::ProcessorCount
    if ($k2) { $n = @($k2.PSObject.Properties | Where-Object { $_.Name -match '^\d+$' }).Count; if ($n -gt 0) { $r.Logical = $n } }
  } catch { }
  return $r
}

# ---------------------------------------------------------------- autostart
# The SMU power limit is volatile: it disappears after a reboot, so TDPilot
# re-applies the saved value at logon.
#
# A logon entry in the user's Startup folder is used instead of a Task
# Scheduler job, because a task-scheduler-hosted process cannot reach the
# SMU on some machines: on a Mechrevo WUJIE14 every combination of identity
# (SYSTEM / current user), integrity level, session and working directory
# made ryzenadj fail with "Unable to init ryzenadj", while the very same
# binary worked from any interactively elevated process.  The Startup folder
# needs no administrator rights to install, and the helper it launches asks
# for elevation through the AppInfo service, which is silent when the host
# policy allows it (ConsentPromptBehaviorAdmin = 0).

function Get-StartupDir {
  $p = $null
  try { $p = [System.Environment]::GetFolderPath('Startup') } catch { }
  if (-not $p -or -not (Test-Path $p)) {
    $p = Join-Path $env:APPDATA 'Microsoft\Windows\Start Menu\Programs\Startup'
  }
  return $p
}

function Get-AutostartEntryPath { return (Join-Path (Get-StartupDir) 'TDPilot AutoApply.cmd') }
function Get-ApplyCmdPath { return (Join-Path $script:DataDir 'ApplyLimit.cmd') }
function Get-ApplyMwPath  { return (Join-Path $script:DataDir 'limit-mw.txt') }

function Test-LegacyTaskInstalled {
  if (-not $script:LegacyTaskName) { return $false }
  $null = & cmd.exe /c "schtasks /query /tn `"$($script:LegacyTaskName)`" >nul 2>&1"
  return ($LASTEXITCODE -eq 0)
}

function Test-AutostartInstalled {
  if (Test-Path (Get-AutostartEntryPath)) { return $true }
  return (Test-LegacyTaskInstalled)
}

# True when the machine elevates administrators without showing a UAC prompt,
# which is what makes the logon helper completely silent.
function Test-UacSilent {
  try {
    $k = Get-ItemProperty -Path 'HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\System' -ErrorAction Stop
    return ([int]$k.ConsentPromptBehaviorAdmin -eq 0)
  } catch { return $false }
}

# Writes the ASCII helper that re-applies the limit. Runs unprivileged-safe.
function Write-ApplyCmd {
  $dir  = $script:DataDir
  $exe  = $script:RyzenExe
  $cmd  = Join-Path $dir 'ApplyLimit.cmd'
  $mwf  = Join-Path $dir 'limit-mw.txt'
  $log  = Join-Path $dir 'apply.log'
  $lines = @(
    '@echo off',
    'setlocal EnableExtensions EnableDelayedExpansion',
    ('set "DIR=' + $dir + '\"'),
    ('set "EXE=' + $exe + '"'),
    ('set "MWF=' + $mwf + '"'),
    ('set "LOG=' + $log + '"'),
    ('set "INFOF=' + (Join-Path $dir 'info.tmp') + '"'),
    'set "MW=25000"',
    'if exist "%MWF%" set /p MW=<"%MWF%"',
    'if defined MW set "MW=%MW: =%"',
    'if not defined MW set "MW=25000"',
    'call :valid "%MW%" || set "MW=25000"',
    'set "EXP=%MW:~0,-3%.%MW:~-3%"',
    '"%EXE%" --info >nul 2>&1',
    'if errorlevel 1 (',
    '  call :heal',
    '  "%EXE%" --info >nul 2>&1',
    ')',
    '"%EXE%" --stapm-limit=%MW% --fast-limit=%MW% --slow-limit=%MW% >nul 2>&1',
    'set "RC=!ERRORLEVEL!"',
    'set "OK=0"',
    'rem Verify by reading the table back from a file: piping a quoted exe path',
    'rem straight into for /f makes cmd mangle the quotes.',
    '"%EXE%" --info >"%INFOF%" 2>&1',
    'findstr /c:"STAPM LIMIT" "%INFOF%" 2>nul | findstr /c:"%EXP%" >nul 2>&1 && set /a OK+=1',
    'findstr /c:"PPT LIMIT FAST" "%INFOF%" 2>nul | findstr /c:"%EXP%" >nul 2>&1 && set /a OK+=1',
    'findstr /c:"PPT LIMIT SLOW" "%INFOF%" 2>nul | findstr /c:"%EXP%" >nul 2>&1 && set /a OK+=1',
    'if exist "%INFOF%" del /q "%INFOF%" >nul 2>&1',
    'if "!OK!"=="3" (',
    '  >>"%LOG%" echo [!TIME!] OK !MW! mW verified 3/3 raw_exit=!RC!',
    '  endlocal & exit /b 0',
    ')',
    '>>"%LOG%" echo [!TIME!] FAIL !MW! mW verified !OK!/3 raw_exit=!RC!',
    'endlocal & exit /b 2',
    '',
    ':valid',
    'set "V=%~1"',
    'if not defined V exit /b 1',
    'set /a "N=V+0" 2>nul || exit /b 1',
    'if %N% LSS 5000 exit /b 1',
    'if %N% GTR 54000 exit /b 1',
    'exit /b 0',
    '',
    ':heal',
    'rem A driver that reports RUNNING can still hold a dead device object,',
    'rem so a plain "sc start" is not enough: bounce the service completely.',
    'sc stop inpoutx64 >nul 2>&1',
    'powershell -NoProfile -Command "Start-Sleep -Milliseconds 700" >nul 2>&1',
    'sc start inpoutx64 >nul 2>&1',
    'powershell -NoProfile -Command "Start-Sleep -Milliseconds 900" >nul 2>&1',
    '"%EXE%" --info >nul 2>&1',
    'if not errorlevel 1 exit /b 0',
    'rem second pass: recreate the service entry as a last resort',
    'sc stop inpoutx64 >nul 2>&1',
    'sc delete inpoutx64 >nul 2>&1',
    'powershell -NoProfile -Command "Start-Sleep -Milliseconds 500" >nul 2>&1',
    'sc create inpoutx64 type= kernel start= demand binPath= "\SystemRoot\System32\drivers\inpoutx64.sys" >nul 2>&1',
    'sc start inpoutx64 >nul 2>&1',
    'powershell -NoProfile -Command "Start-Sleep -Milliseconds 900" >nul 2>&1',
    'exit /b 0'
  )
  if (-not (Test-Path $dir)) { New-Item -ItemType Directory -Path $dir -Force | Out-Null }
  $ascii = New-Object System.Text.ASCIIEncoding
  [System.IO.File]::WriteAllText($cmd, (($lines -join "`r`n") + "`r`n"), $ascii)
  return $cmd
}

function Write-ApplyValue {
  param([int]$MilliWatts)
  if (-not (Test-Path $script:DataDir)) { New-Item -ItemType Directory -Path $script:DataDir -Force | Out-Null }
  [System.IO.File]::WriteAllText((Get-ApplyMwPath), [string]$MilliWatts, (New-Object System.Text.ASCIIEncoding))
}

# The logon helper: a tiny ASCII batch file in the Startup folder that asks
# for elevation through Start-Process -Verb RunAs and runs ApplyLimit.cmd.
function New-AutostartEntry {
  $cmd   = Get-ApplyCmdPath
  $entry = Get-AutostartEntryPath
  $sp    = Get-StartupDir
  if (-not (Test-Path $sp)) { New-Item -ItemType Directory -Path $sp -Force | Out-Null }
  $run = 'powershell -NoProfile -WindowStyle Hidden -ExecutionPolicy Bypass -Command "Start-Process -FilePath ''cmd.exe'' -ArgumentList ''/c'',''\"' + $cmd + '\"'' -Verb RunAs -WindowStyle Hidden -Wait"'
  $lines = @(
    '@echo off',
    'rem TDPilot - re-applies the saved power limit after logon.',
    'rem Generated by TDPilot. Delete this file to disable auto-apply.',
    $run,
    'exit /b 0'
  )
  [System.IO.File]::WriteAllText($entry, (($lines -join "`r`n") + "`r`n"), (New-Object System.Text.ASCIIEncoding))
  return $entry
}

function Install-Autostart {
  param([int]$MilliWatts = 0)
  if (-not (Test-Path $script:DataDir)) { New-Item -ItemType Directory -Path $script:DataDir -Force | Out-Null }
  [void](Write-ApplyCmd)
  if ($MilliWatts -gt 0) { Write-ApplyValue $MilliWatts }
  elseif (-not (Test-Path (Get-ApplyMwPath))) { Write-ApplyValue $script:DefaultMW }
  [void](New-AutostartEntry)
  return (Test-Path (Get-AutostartEntryPath))
}

function Remove-Autostart {
  $entry = Get-AutostartEntryPath
  if (Test-Path $entry) { Remove-Item $entry -Force -ErrorAction SilentlyContinue }
  # clean up the scheduled task left behind by earlier builds
  $null = & cmd.exe /c "schtasks /delete /tn `"$($script:LegacyTaskName)`" /f >nul 2>&1"
  return (-not (Test-Path $entry))
}

# Runs ApplyLimit.cmd right now, exactly the way the logon entry would.
function Invoke-AutostartNow {
  $cmd = Get-ApplyCmdPath
  if (-not (Test-Path $cmd)) { return $false }
  if (Test-IsAdmin) {
    $null = & cmd.exe /c "`"$cmd`""
    return ($LASTEXITCODE -eq 0)
  }
  $ps = 'Start-Process -FilePath ''cmd.exe'' -ArgumentList ''/c'',''\"' + $cmd + '\"'' -Verb RunAs -WindowStyle Hidden -Wait'
  try {
    & cmd.exe /c ('powershell -NoProfile -ExecutionPolicy Bypass -Command "' + $ps + '"')
    return $true
  } catch { return $false }
}
function Get-LastApplyLine {
  $log = Join-Path $script:DataDir 'apply.log'
  if (-not (Test-Path $log)) { return '' }
  try {
    $tail = @(Get-Content -Path $log -Tail 4 -ErrorAction Stop)
  } catch { return '' }
  for ($i = $tail.Count - 1; $i -ge 0; $i--) {
    if ($tail[$i] -match '(OK|FAIL)') { return ([string]$tail[$i]).Trim() }
  }
  return ''
}

# Restores the vendor defaults: writes the fallback cap back and removes the
# logon entry. Both steps are best effort so one failure does not mask the other.
function Reset-ToDefaults {
  param([int]$MilliWatts = 0)
  if ($MilliWatts -le 0) { $MilliWatts = $script:DefaultMW }
  $okWrite = $false
  if (Test-HardwareAccess) {
    $r = Invoke-RyzenAdj @("--stapm-limit=$MilliWatts", "--fast-limit=$MilliWatts", "--slow-limit=$MilliWatts")
    $chk = Get-Snapshot
    if ($chk -and $chk.Data -and $chk.Data.ContainsKey('stapmLimit')) {
      $d = $chk.Data
      $m = 0
      foreach ($k in @('stapmLimit', 'fastLimit', 'slowLimit')) {
        if ($d.ContainsKey($k) -and [math]::Abs($d[$k] * 1000 - $MilliWatts) -lt 60) { $m++ }
      }
      $okWrite = ($m -eq 3)
    }
  }
  $okAuto = $false
  if (Test-AutostartInstalled) { $okAuto = Remove-Autostart } else { $okAuto = $true }
  try {
    $cfg = Get-Config
    $cfg.PowerMW = $MilliWatts
    Save-Config $cfg | Out-Null
  } catch { }
  return [pscustomobject]@{ Written = $okWrite; AutoRemoved = $okAuto; MilliWatts = $MilliWatts }
}