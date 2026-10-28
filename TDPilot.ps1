# =====================================================================
#  TDPilot - entry point
#  AMD Ryzen power-limit tuner (RyzenAdj front-end)
#  Usage:  powershell -NoProfile -ExecutionPolicy Bypass -File TDPilot.ps1
# =====================================================================

param(
  [switch]$SelfTest,          # render the UI offscreen, print a report, exit
  [switch]$NoPoll
)

$ErrorActionPreference = 'Stop'
$here = $PSScriptRoot
$src  = Join-Path $here 'src'
if (-not (Test-Path (Join-Path $src 'core.ps1'))) { $src = $here }

. (Join-Path $src 'core.ps1')
. (Join-Path $src 'Ui.ps1')

# ---------------------------------------------------------------- startup
$script:AccessOk    = $false
$script:AccessError = $null
$cfg = Get-Config
if ($cfg.Lang) { Set-Language $cfg.Lang }

Add-Type -AssemblyName PresentationFramework, PresentationCore, WindowsBase

$xamlPath = Join-Path $src 'MainWindow.xaml'
if (-not (Test-Path $xamlPath)) { throw "MainWindow.xaml not found: $xamlPath" }
$xamlText = [System.IO.File]::ReadAllText($xamlPath, [System.Text.Encoding]::UTF8)

$reader = New-Object System.Xml.XmlNodeReader (([xml]$xamlText))
$script:UC = [System.Windows.Markup.XamlReader]::Load($reader)

# ---------------------------------------------------------------- nav maps
$script:Nav = @{
  'dash'  = (UI 'NavDash')
  'prof'  = (UI 'NavProf')
  'auto'  = (UI 'NavAuto')
  'hw'    = (UI 'NavInfo')
  'set'   = (UI 'NavSet')
  'about' = (UI 'NavAbout')
}
$script:Nav['dash'].IsChecked = $true

foreach ($k in @('dash','prof','auto','hw','set','about')) {
  $rb = $script:Nav[$k]
  $key = $k
  $rb.Add_Click({
    param($s, $e)
    Show-Page $key
  }.GetNewClosure())
}

# ---------------------------------------------------------------- slider
$sld = UI 'SldPower'
$sld.Add_ValueChanged({
  param($s, $e)
  $v = [double]$s.Value
  Set-Text 'ValSetW' ('{0:0.0}' -f $v)
})

$sld.Add_MouseLeftButtonUp({
  param($s, $e)
  Set-Text 'TxtApplyState' (T 'power.pending')
  (UI 'TxtApplyState').Foreground = Br '#FBBF24'
})

$sldRefresh = UI 'SldRefresh'
if ($sldRefresh) {
  $sldRefresh.Add_ValueChanged({
    param($s, $e)
    Set-Text 'TxtRefreshVal' ('{0} s' -f [int]$s.Value)
  })
}

# ---------------------------------------------------------------- apply
(UI 'BtnApply').Add_Click({
  $mw = [int]([math]::Round([double](UI 'SldPower').Value * 1000))
  [void](Apply-Power $mw)
})

# ---------------------------------------------------------------- presets
$presets = @(
  @{ Name = 'Pr15';  MW = 15000 },
  @{ Name = 'Pr25';  MW = 25000 },
  @{ Name = 'Pr30';  MW = 30000 },
  @{ Name = 'Pr45';  MW = 45000 }
)
foreach ($p in $presets) {
  $btn = UI $p.Name
  $mw  = $p.MW
  $btn.Add_Click({
    param($s, $e)
    (UI 'SldPower').Value = ($mw / 1000.0)
    Set-Text 'ValSetW' ('{0:0.0}' -f ($mw / 1000.0))
    [void](Apply-Power $mw)
  }.GetNewClosure())
}

(UI 'PrMax').Add_Click({
  param($s, $e)
  $r = Reset-ToDefaults
  (UI 'SldPower').Value = ($script:DefaultMW / 1000.0)
  Set-Text 'ValSetW' ('{0:0.0}' -f ($script:DefaultMW / 1000.0))
  if ($r.Written) {
    Add-Log $(if ($script:Lang -eq 'zh') { ('已恢复默认：{0:0.0} W' -f ($r.MilliWatts / 1000.0)) } else { ('Restored default: {0:0.0} W' -f ($r.MilliWatts / 1000.0)) }) 'ok'
  } else {
    Add-Log $(if ($script:Lang -eq 'zh') { '恢复默认失败：无法写入功耗' } else { 'Restore failed: could not write the limit' }) 'err'
  }
  if ($r.AutoRemoved) { Add-Log $(if ($script:Lang -eq 'zh') { '自动套用已一并移除' } else { 'Auto-apply removed as well' }) 'ok' }
  Refresh-Data -Quiet
  Refresh-Automation
})
(UI 'PrCustom').Add_Click({
  param($s, $e)
  (UI 'SldPower').Focus() | Out-Null
  Add-Log $(if ($script:Lang -eq 'zh') { '拖动滑杆选择自定义瓦数，然后点「应用」' } else { 'Drag the slider to pick a value, then press Apply' })
})

# ---------------------------------------------------------------- topbar
(UI 'BtnRefresh').Add_Click({
  param($s, $e)
  Refresh-Data
})

function Switch-Language {
  $next = if ($script:Lang -eq 'zh') { 'en' } else { 'zh' }
  Set-Language $next
  $cfg = Get-Config; $cfg.Lang = $next; Save-Config $cfg | Out-Null
  Apply-Lang
  Update-PageTitle
  Update-StatusBlock
  Show-Page (Get-CurrentPageKey)
  Add-Log $(if ($next -eq 'zh') { '界面语言：简体中文' } else { 'UI language: English' })
}

(UI 'BtnLang').Add_Click({ param($s, $e) Switch-Language })
(UI 'BtnLangZh').Add_Click({ param($s, $e) Set-Language 'zh'; $c = Get-Config; $c.Lang='zh'; Save-Config $c | Out-Null; Apply-Lang; Update-PageTitle; Show-Page (Get-CurrentPageKey) })
(UI 'BtnLangEn').Add_Click({ param($s, $e) Set-Language 'en'; $c = Get-Config; $c.Lang='en'; Save-Config $c | Out-Null; Apply-Lang; Update-PageTitle; Show-Page (Get-CurrentPageKey) })

# ---------------------------------------------------------------- log page
(UI 'BtnClearLog').Add_Click({
  param($s, $e)
  $tb = UI 'TxtLog'; if ($tb) { $tb.Text = '' }
  Set-Text 'TxtLogCount' '0 lines'
})

# ---------------------------------------------------------------- automation
# ---------------------------------------------------------------- automation
(UI 'BtnAutoInstall').Add_Click({
  param($s, $e)
  $mw = [int]([math]::Round([double](UI 'SldPower').Value * 1000))
  if (Install-Autostart -MilliWatts $mw) {
    $wv = '{0:0.0}' -f ($mw / 1000.0)
    Add-Log $(if ($script:Lang -eq 'zh') { "已启用自动套用：下次登录会套用 $wv W" } else { "Auto-apply enabled: $wv W will be applied at next logon" }) 'ok'
    if (-not (Test-UacSilent)) {
      Add-Log $(if ($script:Lang -eq 'zh') { '提示：本机 UAC 会弹窗，登录后请点一次确认' } else { 'Note: this machine will show a UAC prompt at logon' }) 'warn'
    }
  } else {
    Add-Log $(if ($script:Lang -eq 'zh') { '启用失败：无法写入启动文件夹' } else { 'Failed - cannot write to the Startup folder' }) 'err'
  }
  Refresh-Automation
})
(UI 'BtnAutoRemove').Add_Click({
  param($s, $e)
  if (Remove-Autostart) {
    Add-Log $(if ($script:Lang -eq 'zh') { '已移除自动套用' } else { 'Auto-apply removed' }) 'ok'
  } else {
    Add-Log $(if ($script:Lang -eq 'zh') { '移除失败' } else { 'Remove failed' }) 'err'
  }
  Refresh-Automation
})
if (UI 'BtnAutoRunNow') {
  (UI 'BtnAutoRunNow').Add_Click({
    param($s, $e)
    if (-not (Test-Path (Get-ApplyMwPath))) {
      Add-Log $(if ($script:Lang -eq 'zh') { '还没有保存过功耗值，先点「应用功耗上限」' } else { 'No saved value yet - apply a limit first' }) 'warn'
      return
    }
    Add-Log $(if ($script:Lang -eq 'zh') { '正在立即套用保存的功耗值…' } else { 'Re-applying the saved value now...' })
    [void](Invoke-AutostartNow)
    $lw = Get-LastApplyLine
    if ($lw) { Add-Log $lw $(if ($lw -match '\bOK\b') { 'ok' } else { 'err' }) }
    Refresh-Data -Quiet
    Refresh-Automation
  })
}

# ---------------------------------------------------------------- misc pages
if (UI 'BtnOpenData') {
  (UI 'BtnOpenData').Add_Click({ param($s, $e)
    if (-not (Test-Path $script:DataDir)) { New-Item -ItemType Directory -Path $script:DataDir -Force | Out-Null }
    Start-Process explorer.exe $script:DataDir
  })
}
if (UI 'BtnCopyDiag') {
  (UI 'BtnCopyDiag').Add_Click({ param($s, $e)
    $cpu = Get-CpuIdentity; $dg = Test-DeviceGuard
    $txt = @(
      "TDPilot diag $(Get-Date -Format s)",
      "CPU: $($cpu.Name) ($($cpu.Vendor)) threads=$($cpu.Logical)",
      "OS: $([System.Environment]::OSVersion.VersionString)",
      "Elevated: $(Test-IsAdmin)  HVCI: $($dg.Hvci)  Blocklist: $($dg.Blocklist)",
      "ryzenadj: $script:RyzenExe",
      "access: ok=$($script:AccessOk) err=$($script:AccessError)"
    ) -join "`r`n"
    [System.Windows.Clipboard]::SetText($txt)
    Add-Log 'diagnostics copied to clipboard' 'ok'
  })
}

if (UI 'BtnProfNew') {
  (UI 'BtnProfNew').Add_Click({ param($s, $e)
    $mw = [int]([math]::Round([double](UI 'SldPower').Value * 1000))
    $name = '{0:0.0} W' -f ($mw / 1000.0)
    [void]$script:Profiles.Add([pscustomobject]@{ Name = $name; MW = $mw })
    Render-Profiles
    Add-Log ("profile saved: {0}" -f $name) 'ok'
  })
}

# ---------------------------------------------------------------- boot
Apply-Lang
Show-Page 'dash'
Update-StatusBlock
Add-Log 'TDPilot started'
Add-Log ("ryzenadj: {0}" -f $script:RyzenExe)
if (-not (Test-Path $script:RyzenExe)) {
  Add-Log 'ryzenadj.exe not found next to the script' 'err'
}
if (-not (Test-IsAdmin)) {
  Add-Log $(if ($script:Lang -eq 'zh') { '当前不是管理员，写功耗会失败' } else { 'Not elevated - writing limits will fail' }) 'warn'
}

# ---------------------------------------------------------------- self test
if ($SelfTest) {
  $script:UC.Measure((New-Object System.Windows.Size(1020, 700)))
  $script:UC.Arrange((New-Object System.Windows.Rect(0, 0, 1020, 700)))
  $script:UC.UpdateLayout()
  Draw-Chart

  $names = @('NavDash','NavProf','NavAuto','NavInfo','NavSet','NavAbout','SldPower','BtnApply','Pr15','Pr25','Pr30','Pr45','PrMax','PrCustom','ChartCanvas','TxtLog','ValPower','ValTemp','ValFreq','TxtPageTitle','TxtPageSub','DotDriver','TxtDriverState','TxtCpuShort')
  foreach ($n in $names) {
    $e = UI $n
    if ($e) {
      $p = $e.TranslatePoint((New-Object System.Windows.Point(0, 0)), $script:UC)
      "  {0,-16} ok  ({1},{2} {3}x{4})" -f $n, [int]$p.X, [int]$p.Y, [int]$e.ActualWidth, [int]$e.ActualHeight
    } else {
      "  {0,-16} MISSING" -f $n
    }
  }
  "TITLE: " + (UI 'TxtPageTitle').Text
  "SUB:   " + (UI 'TxtPageSub').Text
  "LOG:   " + ((UI 'TxtLog').Text -split "`r`n" | Select-Object -First 4) -join ' | '
  return
}

# ---------------------------------------------------------------- show
$win = New-Object System.Windows.Window
$win.Title = (T 'app.title')
$win.Width = 1020
$win.Height = 700
$win.WindowStartupLocation = 'CenterScreen'
$win.Background = (Br '#0A0D13')
$win.MinWidth = 940
$win.MinHeight = 640
$win.Content = $script:UC

$win.Add_Loaded({
  param($s, $e)
  Refresh-Data
  if (-not $NoPoll) {
    $script:PollTimer = New-Object System.Windows.Threading.DispatcherTimer
    $script:PollTimer.Interval = (New-Object System.TimeSpan(0, 0, 0, 0, [int](UI 'SldRefresh').Value * 1000))
    $script:PollTimer.Add_Tick({
      param($ts, $te)
      Refresh-Data -Quiet
    })
    $script:PollTimer.Start()
  }
})

$app = New-Object System.Windows.Application
$app.Add_DispatcherUnhandledException({
  param($s, $e)
  Add-Log ("unhandled: " + $e.Exception.Message) 'err'
  $e.Handled = $true
})
$app.Run($win) | Out-Null
