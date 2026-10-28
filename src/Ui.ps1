# =====================================================================
#  TDPilot - UI layer (dot-sourced by Tdpilot.ps1)
#  Pure ASCII source; all user-facing strings come from the i18n table.
# =====================================================================

Add-Type -AssemblyName PresentationFramework, PresentationCore, WindowsBase, System.Windows.Forms, System.Drawing
[System.Windows.Application]::Current | Out-Null

$script:UC          = $null      # root UserControl
$script:Nav         = @{}
$script:Page        = @{}
$script:HistPower   = New-Object System.Collections.ArrayList
$script:HistTemp    = New-Object System.Collections.ArrayList
$script:MaxPoints   = 60
$script:PollTimer   = $null
$script:Busy        = $false
$script:LastSnap    = $null
$script:Profiles    = New-Object System.Collections.ArrayList

# ------------------------------------------------------------ helpers
function Br {
  param([string]$Hex)
  return [System.Windows.Media.BrushConverter]::new().ConvertFromString($Hex)
}

function UI([string]$Name) { return $script:UC.FindName($Name) }

# Condense "AMD Ryzen 7 8845HS w/ Radeon 780M Graphics" -> "Ryzen 7 8845HS" (+ thread count)
function Format-CpuShort($Cpu) {
  $n = [string]$Cpu.Name
  if (-not $n) { return '' }
  # cut trailing graphics / with-clause
  $n = $n -replace '\s+w/\s+.*$', ''
  $n = $n -replace '\s+with\s+.*$', ''
  # drop the leading vendor word
  $n = $n -replace '^(AMD|Intel|Apple)\s+', ''
  # drop a trailing "Processor"/"CPU"
  $n = $n -replace '\s+(Processor|CPU)$', ''
  $n = $n.Trim()
  if ($Cpu.Logical -gt 0) { $n = ('{0}  {1}T' -f $n, $Cpu.Logical) }
  return $n
}

function Set-Text([string]$Name, [string]$Value) {
  $e = UI $Name
  if ($e -and $Value -ne $null) { $e.Text = $Value }
}

function Set-Vis([string]$Name, [bool]$Visible) {
  $e = UI $Name
  if ($e) { $e.Visibility = $(if ($Visible) { 'Visible' } else { 'Collapsed' }) }
}

function Set-Brush([string]$Name, [string]$Hex) {
  $e = UI $Name
  if ($e) {
    if ($e -is [System.Windows.Shapes.Shape]) { $e.Fill = (Br $Hex) }
    elseif ($e -is [System.Windows.Controls.TextBlock]) { $e.Foreground = (Br $Hex) }
    elseif ($e -is [System.Windows.Controls.Border]) { $e.Background = (Br $Hex) }
  }
}

function Add-Log([string]$Msg, [string]$Level = 'info') {
  $tb = UI 'TxtLog'
  if (-not $tb) { return }
  $stamp = (Get-Date).ToString('HH:mm:ss')
  $prefix = switch ($Level) {
    'ok'    { 'OK  ' }
    'warn'  { '!!  ' }
    'err'   { 'ERR ' }
    default { '    ' }
  }
  $line = "[$stamp] $prefix$Msg"
  $tb.AppendText($line + "`r`n")
  $tb.ScrollToEnd()
  $n = ($tb.Text -split "`r`n").Count - 1
  Set-Text 'TxtLogCount' ("{0} lines" -f $n)
  # keep log bounded
  if ($tb.Text.Length -gt 24000) {
    $tb.Text = $tb.Text.Substring($tb.Text.Length - 12000)
  }
}

# ------------------------------------------------------------ i18n apply
function Apply-Lang {
  $L = $script:Lang
  $title = T 'app.title'
  $win = [System.Windows.Window]::GetWindow($script:UC)
  if ($win) { $win.Title = $title }

  # sidebar
  Set-Text 'TxtNavHead' (T 'nav.control')
  Set-Text 'TxtNavSys'  (T 'nav.system')
  (UI 'NavDash').Content = T 'nav.dashboard'
  (UI 'NavProf').Content = T 'nav.profiles'
  (UI 'NavAuto').Content = T 'nav.auto'
  (UI 'NavInfo').Content = T 'nav.hardware'
  (UI 'NavSet').Content  = T 'nav.settings'
  (UI 'NavAbout').Content = T 'nav.about'

  # topbar
  (UI 'BtnRefresh').Content = T 'btn.refresh'
  (UI 'BtnLang').Content = $(if ($L -eq 'zh') { 'EN' } else { 'CH' })

  # dashboard
  Set-Text 'LblPower' (T 'card.power')
  Set-Text 'LblTemp'  (T 'card.temp')
  Set-Text 'LblFreq'  (T 'card.freq')
  Set-Text 'LblChart' (T 'chart.title')
  Set-Text 'LblSetPower' (T 'power.limit')
  Set-Text 'TxtSetHint' (T 'power.hint')
  (UI 'BtnApply').Content = T 'power.apply'
  Set-Text 'LblPresets' (T 'power.presets')
  (UI 'Pr15').Content = '15W ' + (T 'preset.power')
  (UI 'Pr25').Content = '25W ' + (T 'preset.balanced')
  (UI 'Pr30').Content = '30W ' + (T 'preset.perf')
  (UI 'Pr45').Content = '45W ' + (T 'preset.full')
  (UI 'PrMax').Content = T 'preset.default'
  (UI 'PrCustom').Content = T 'preset.custom'
  Set-Text 'LblLog' (T 'log.title')
  (UI 'BtnClearLog').Content = T 'log.clear'

  # profiles page
  Set-Text 'TxtProfHead' (T 'page.profiles')
  Set-Text 'TxtProfDesc' (T 'page.profiles.sub')
  (UI 'BtnProfNew').Content = $(if ($L -eq 'zh') { '新建预设' } else { 'New profile' })

  # automation page
  Set-Text 'TxtAutoHead' (T 'page.auto')
  Set-Text 'TxtAutoDesc' (T 'page.auto.sub')
  (UI 'BtnAutoInstall').Content = $(if ($L -eq 'zh') { '启用自动套用' } else { 'Enable auto-apply' })
  (UI 'BtnAutoRemove').Content  = $(if ($L -eq 'zh') { '移除' } else { 'Remove' })
  if (UI 'BtnAutoRunNow') { (UI 'BtnAutoRunNow').Content = $(if ($L -eq 'zh') { '立即套用一次' } else { 'Apply now' }) }
  Set-Text 'TxtTrigHead' $(if ($L -eq 'zh') { '触发器' } else { 'Triggers' })
  Set-Text 'TxtTaskHead' $(if ($L -eq 'zh') { '启动项详情' } else { 'Startup entry details' })

  # hardware page
  Set-Text 'TxtHwCpuHead' $(if ($L -eq 'zh') { '处理器' } else { 'Processor' })
  Set-Text 'TxtHwLimitHead' $(if ($L -eq 'zh') { '当前限制' } else { 'Current limits' })

  # settings page
  Set-Text 'TxtSetHead' $(if ($L -eq 'zh') { '常规' } else { 'General' })
  Set-Text 'LblSetRefresh' $(if ($L -eq 'zh') { '刷新间隔' } else { 'Refresh interval' })
  Set-Text 'TxtSetRefreshDesc' $(if ($L -eq 'zh') { '读取功耗数据的频率；读取一次约需 1 秒' } else { 'How often to poll. One read takes about 1 s' })
  Set-Text 'LblSetLang' $(if ($L -eq 'zh') { '界面语言' } else { 'Interface language' })
  Set-Text 'TxtSetLangDesc' $(if ($L -eq 'zh') { '立即生效' } else { 'Applies immediately' })
  Set-Text 'LblSetLog' $(if ($L -eq 'zh') { '日志与数据' } else { 'Logs and data' })
  Set-Text 'TxtSetLogDesc' $(if ($L -eq 'zh') { '配置、日志保存在程序数据目录' } else { 'Config and logs live in the app data folder' })
  (UI 'BtnOpenData').Content = $(if ($L -eq 'zh') { '打开目录' } else { 'Open folder' })
  (UI 'BtnCopyDiag').Content = $(if ($L -eq 'zh') { '复制诊断' } else { 'Copy diagnostics' })

  # about page
  Set-Text 'TxtAboutVer'  ('v1.0.0')
  Set-Text 'TxtAboutDesc' $(if ($L -eq 'zh') { '面向 AMD Ryzen 处理器的功耗墙调节工具。通过 RyzenAdj 直接与 SMU 通信，可在 5~54 W 之间任意设定封装功耗上限。' } else { 'A power-limit tuner for AMD Ryzen processors. Talks to the SMU through RyzenAdj so you can set any package power cap between 5 and 54 W.' })
  Set-Text 'TxtAboutLicHead' $(if ($L -eq 'zh') { '开源许可' } else { 'License' })
  Set-Text 'TxtAboutLic' $(if ($L -eq 'zh') {
      "本程序以 MIT 许可证发布。随附的 RyzenAdj 以 LGPL-3.0 授权，源码见 https://github.com/FlyGoat/RyzenAdj。内核驱动 inpoutx64 / WinRing0 属各自作者所有。"
    } else {
      "This program is released under the MIT license. Bundled RyzenAdj is LGPL-3.0, sources at https://github.com/FlyGoat/RyzenAdj. Kernel drivers inpoutx64 / WinRing0 belong to their respective authors."
    })
  Set-Text 'TxtAboutEnvHead' $(if ($L -eq 'zh') { '运行环境' } else { 'Environment' })

  # refresh dynamic content
  Update-PageTitle
  Update-StatusBlock
  if ($script:LastSnap) { Render-Snapshot $script:LastSnap }
}

# ------------------------------------------------------------ navigation
function Get-PageMeta {
  param([string]$Key)
  switch ($Key) {
    'dash'  { return @{ Title = (T 'page.dashboard'); Sub = (T 'page.dash.sub');      Page = 'PageDash'  } }
    'prof'  { return @{ Title = (T 'page.profiles');  Sub = (T 'page.profiles.sub');  Page = 'PageProf'  } }
    'auto'  { return @{ Title = (T 'page.auto');      Sub = (T 'page.auto.sub');      Page = 'PageAuto'  } }
    'hw'    { return @{ Title = (T 'page.hardware');  Sub = (T 'page.hardware.sub');  Page = 'PageHw'    } }
    'set'   { return @{ Title = (T 'page.settings');  Sub = (T 'page.settings.sub');  Page = 'PageSet'   } }
    'about' { return @{ Title = (T 'page.about');     Sub = (T 'page.about.sub');     Page = 'PageAbout' } }
  }
  return @{ Title = ''; Sub = ''; Page = 'PageDash' }
}

function Get-CurrentPageKey {
  foreach ($k in @('dash','prof','auto','hw','set','about')) {
    $r = $script:Nav[$k]
    if ($r -and $r.IsChecked) { return $k }
  }
  return 'dash'
}

function Update-PageTitle {
  param([string]$Key)
  if (-not $Key) { $Key = Get-CurrentPageKey }
  $m = Get-PageMeta $Key
  Set-Text 'TxtPageTitle' $m.Title
  Set-Text 'TxtPageSub'   $m.Sub
}

function Show-Page {
  param([string]$Key)
  $m = Get-PageMeta $Key
  foreach ($k in @('dash','prof','auto','hw','set','about')) {
    $pm = Get-PageMeta $k
    Set-Vis $pm.Page ($k -eq $Key)
  }
  $rb = $script:Nav[$Key]
  if ($rb -and -not $rb.IsChecked) { $rb.IsChecked = $true }
  Update-PageTitle -Key $Key
  Add-Log ("page -> {0}" -f $Key)
  switch ($Key) {
    'prof' { Render-Profiles }
    'auto' { Refresh-Automation }
    'hw'   { Render-Hardware }
    'about'{ Render-AboutEnv }
  }
}

# ------------------------------------------------------------ status block
function Update-StatusBlock {
  $dot = UI 'DotDriver'
  $txt = UI 'TxtDriverState'
  if (-not $dot -or -not $txt) { return }
  $cpu = Get-CpuIdentity
  Set-Text 'TxtCpuShort' (Format-CpuShort $cpu)

  if ($script:AccessOk) {
    $dot.Fill = Br '#34D399'
    $txt.Text = T 'status.driverok'
    $txt.Foreground = Br '#34D399'
  } elseif ($script:AccessError) {
    $dot.Fill = Br '#F43F5E'
    $txt.Text = T 'status.drivermissing'
    $txt.Foreground = Br '#F43F5E'
  } else {
    $dot.Fill = Br '#5E6B80'
    $txt.Text = T 'status.checking'
    $txt.Foreground = Br '#9AA6B8'
  }
}

# ------------------------------------------------------------ snapshot
function Render-Snapshot {
  param($Snap)
  if (-not $Snap) { return }
  $d = $Snap.Data

  $pw    = if ($d.ContainsKey('stapmValue'))  { $d['stapmValue'] }  else { $null }
  $lim   = if ($d.ContainsKey('stapmLimit'))  { $d['stapmLimit'] }  else { $null }
  $pf    = if ($d.ContainsKey('fastValue'))   { $d['fastValue'] }   else { $null }
  $tmp   = if ($d.ContainsKey('thmValue'))    { $d['thmValue'] }    else { $null }
  $tlim  = if ($d.ContainsKey('thmLimit'))    { $d['thmLimit'] }    else { $null }

  if ($pw -ne $null)  { Set-Text 'ValPower' ('{0:0.0}' -f $pw) }
  if ($tmp -ne $null) { Set-Text 'ValTemp'  ('{0:0}' -f $tmp) }

  # bar widths (relative to sane maxima)
  if ($pw -ne $null)  { (UI 'BarPower').Width = [math]::Min(1.0, [math]::Max(0.0, $pw / 60.0)) * 190 }
  if ($tmp -ne $null) { (UI 'BarTemp').Width  = [math]::Min(1.0, [math]::Max(0.0, $tmp / 100.0)) * 190 }

  if ($lim -ne $null -and $pf -ne $null) {
    Set-Text 'SubPower' ('{0} {1:0.0} / {2} {3:0.0}' -f (T 'power.limit'), $lim, 'peak', $pf)
  }
  if ($tlim -ne $null) {
    Set-Text 'SubTemp' ('{0} {1:0.0} {2}' -f 'limit', $tlim, (T 'unit.c'))
  }

  # clock from registry (cheap, no WMI)
  $cpu = Get-CpuIdentity
  if ($cpu.MaxMHz -gt 0) {
    Set-Text 'ValFreq' ('{0}' -f $cpu.MaxMHz)
    Set-Text 'SubFreq' ('{0} threads' -f $cpu.Logical)
    (UI 'BarFreq').Width = [math]::Min(1.0, $cpu.MaxMHz / 5200.0) * 190
  }

  # push history
  if ($pw -ne $null)  { [void]$script:HistPower.Add([double]$pw) }
  if ($tmp -ne $null) { [void]$script:HistTemp.Add([double]$tmp) }
  while ($script:HistPower.Count -gt $script:MaxPoints) { $script:HistPower.RemoveAt(0) }
  while ($script:HistTemp.Count  -gt $script:MaxPoints) { $script:HistTemp.RemoveAt(0)  }

  Draw-Chart
}

function Draw-Chart {
  $c = UI 'ChartCanvas'
  if (-not $c) { return }
  $c.Children.Clear()
  $w = $c.ActualWidth; $h = $c.ActualHeight
  if ($w -le 10 -or $h -le 10) { return }

  $padL = 30; $padB = 16
  $iw = $w - $padL; $ih = $h - $padB
  if ($iw -le 10 -or $ih -le 10) { return }

  # grid
  $gridBr = Br '#1B2230'
  foreach ($i in 0..4) {
    $y = $ih * $i / 4.0
    $ln = New-Object System.Windows.Shapes.Line
    $ln.X1 = $padL; $ln.X2 = $w; $ln.Y1 = $y; $ln.Y2 = $y
    $ln.Stroke = $gridBr; $ln.StrokeThickness = 1
    [void]$c.Children.Add($ln)
  }

  # y-axis labels (power 0..60 W)
  foreach ($i in 0..2) {
    $v = 60 - 30 * $i
    $t = New-Object System.Windows.Controls.TextBlock
    $t.Text = "$v"
    $t.FontSize = 9
    $t.Foreground = Br '#5E6B80'
    $t.Width = 26
    $t.TextAlignment = 'Right'
    [System.Windows.Controls.Canvas]::SetLeft($t, 0)
    [System.Windows.Controls.Canvas]::SetTop($t, ($ih * $i / 2.0) - 6)
    [void]$c.Children.Add($t)
  }

  function New-Line($values, $scale, $max, $color, $thick) {
    if ($values.Count -lt 2) { return $null }
    $pl = New-Object System.Windows.Shapes.Polyline
    $pl.Stroke = (Br $color)
    $pl.StrokeThickness = $thick
    $pl.StrokeLineJoin = 'Round'
    $pts = New-Object System.Windows.Media.PointCollection
    $n = $values.Count
    for ($i = 0; $i -lt $n; $i++) {
      $x = $padL + ($iw * $i / [math]::Max(1, ($script:MaxPoints - 1)))
      $yv = [math]::Min($max, [math]::Max(0, $values[$i] * $scale))
      $y = $ih - ($ih * $yv / $max)
      $pts.Add([System.Windows.Point]::new($x, $y))
    }
    $pl.Points = $pts
    return $pl
  }

  $pLine = New-Line $script:HistPower 1.0 60 '#22D3EE' 2
  if ($pLine) { [void]$c.Children.Add($pLine) }
  $tLine = New-Line $script:HistTemp 1.0 100 '#FBBF24' 1.6
  if ($tLine) { [void]$c.Children.Add($tLine) }

  $empty = ($script:HistPower.Count -lt 2)
  Set-Vis 'TxtChartEmpty' $empty
  if (-not $empty) { Set-Text 'TxtChartEmpty' '' }
}

# ------------------------------------------------------------ refresh
function Refresh-Data {
  param([switch]$Quiet)
  if ($script:Busy) { return }
  $script:Busy = $true
  try {
    $snap = Get-Snapshot
    if ($snap) {
      $script:AccessOk = $true
      $script:AccessError = $null
      $script:LastSnap = $snap
      Render-Snapshot $snap
      Render-Hardware
      Update-StatusBlock
      if (-not $Quiet) { Add-Log ('read ok  pw={0:0.0}W  tmp={1:0.0}C' -f $snap.Data['stapmValue'], $snap.Data['thmValue']) 'ok' }
    } else {
      $script:AccessOk = $false
      $script:AccessError = 'init failed'
      Update-StatusBlock
      if (-not $Quiet) { Add-Log 'hardware access failed (--info returned non-zero)' 'err' }
    }
  } catch {
    Add-Log ("refresh error: " + $_.Exception.Message) 'err'
  } finally {
    $script:Busy = $false
  }
}

# ------------------------------------------------------------ apply
function Apply-Power {
  param([int]$MilliWatts)
  if ($MilliWatts -lt 5000 -or $MilliWatts -gt 54000) {
    Add-Log ("value out of range: {0} mW" -f $MilliWatts) 'err'
    return $false
  }
  Set-Text 'TxtApplyState' (T 'msg.applying')
  (UI 'BtnApply').IsEnabled = $false
  Add-Log ("apply {0} mW ..." -f $MilliWatts)

  $r = Invoke-RyzenAdj @("--stapm-limit=$MilliWatts", "--fast-limit=$MilliWatts", "--slow-limit=$MilliWatts")
  Add-Log ("write raw_exit={0}" -f $r.Code) 'warn'

  $chk = Invoke-RyzenAdj @('--info')
  $p = Parse-InfoText $chk.Text
  $ok = 0
  if ($p.ContainsKey('stapmLimit') -and [math]::Abs($p['stapmLimit'] * 1000 - $MilliWatts) -lt 60) { $ok++ }
  if ($p.ContainsKey('fastLimit')  -and [math]::Abs($p['fastLimit']  * 1000 - $MilliWatts) -lt 60) { $ok++ }
  if ($p.ContainsKey('slowLimit')  -and [math]::Abs($p['slowLimit']  * 1000 - $MilliWatts) -lt 60) { $ok++ }

  (UI 'BtnApply').IsEnabled = $true
  if ($ok -eq 3) {
    $script:AccessOk = $true
    Set-Text 'TxtApplyState' (T 'msg.applied').Replace('{0}', ('{0:0.0}' -f ($MilliWatts / 1000.0)))
    (UI 'TxtApplyState').Foreground = Br '#34D399'
    Add-Log ("verified 3/3 -> {0} mW" -f $MilliWatts) 'ok'
    $cfg = Get-Config
    $cfg.PowerMW = $MilliWatts
    $cfg.LastApplied = (Get-Date).ToString('s')
    Save-Config $cfg | Out-Null
    Refresh-Data -Quiet
    return $true
  } else {
    Set-Text 'TxtApplyState' ((T 'msg.applyfail').Replace('{0}', ("{0}/3" -f $ok)))
    (UI 'TxtApplyState').Foreground = Br '#F43F5E'
    Add-Log ("verify failed {0}/3" -f $ok) 'err'
    return $false
  }
}

# ------------------------------------------------------------ profiles page
function Render-Profiles {
  $sp = UI 'ProfList'
  if (-not $sp) { return }
  $sp.Children.Clear()
  if ($script:Profiles.Count -eq 0) {
    $t = New-Object System.Windows.Controls.TextBlock
    $t.Text = $(if ($script:Lang -eq 'zh') { '还没有预设。点右上角「新建预设」，把当前设置存下来。' } else { 'No profiles yet. Click "New profile" to save the current setting.' })
    $t.FontSize = 12; $t.Foreground = Br '#5E6B80'; $t.Margin = '12,14,12,14'
    $t.TextWrapping = 'Wrap'
    [void]$sp.Children.Add($t)
    return
  }
  foreach ($pf in $script:Profiles) {
    $b = New-Object System.Windows.Controls.Border
    $b.CornerRadius = 10
    $b.Background = Br '#141A24'
    $b.BorderBrush = Br '#232A38'
    $b.BorderThickness = 1
    $b.Padding = '14,12'
    $b.Margin = '6,4,6,4'
    $g = New-Object System.Windows.Controls.Grid
    $c1 = New-Object System.Windows.Controls.ColumnDefinition; $c1.Width = '*'
    $c2 = New-Object System.Windows.Controls.ColumnDefinition; $c2.Width = 'Auto'
    $g.ColumnDefinitions.Add($c1); $g.ColumnDefinitions.Add($c2)
    $st = New-Object System.Windows.Controls.StackPanel
    $n1 = New-Object System.Windows.Controls.TextBlock
    $n1.Text = $pf.Name; $n1.FontSize = 13; $n1.Foreground = Br '#E8EDF5'
    $n2 = New-Object System.Windows.Controls.TextBlock
    $n2.Text = ('{0:0.0} W' -f ($pf.MW / 1000.0)); $n2.FontSize = 11; $n2.Foreground = Br '#22D3EE'
    $n2.Margin = '0,3,0,0'
    [void]$st.Children.Add($n1); [void]$st.Children.Add($n2)
    $btn = New-Object System.Windows.Controls.Button
    $btn.Content = $(if ($script:Lang -eq 'zh') { '应用' } else { 'Apply' })
    $btn.Width = 66; $btn.Height = 30
    $btn.Style = $script:UC.FindResource('BtnGhost')
    $btn.Tag = $pf.MW
    $btn.Add_Click({
      $mw = [int]$this.Tag
      Set-Text 'ValSetW' ('{0:0.0}' -f ($mw / 1000.0))
      (UI 'SldPower').Value = ($mw / 1000.0)
      [void](Apply-Power $mw)
    })
    [System.Windows.Controls.Grid]::SetColumn($btn, 1)
    [void]$g.Children.Add($st); [void]$g.Children.Add($btn)
    $b.Child = $g
    [void]$sp.Children.Add($b)
  }
}

# ------------------------------------------------------------ automation page
function Refresh-Automation {
  $zh = ($script:Lang -eq 'zh')
  $installed = Test-AutostartInstalled

  $dot = UI 'DotAuto'; $txt = UI 'TxtAutoState'
  if ($dot -and $txt) {
    if ($installed) {
      $dot.Fill = Br '#34D399'
      $txt.Text = $(if ($zh) { '已启用' } else { 'Enabled' })
      $txt.Foreground = Br '#34D399'
    } else {
      $dot.Fill = Br '#5E6B80'
      $txt.Text = $(if ($zh) { '未启用' } else { 'Not enabled' })
      $txt.Foreground = Br '#9AA6B8'
    }
  }
  if (UI 'BtnAutoInstall') { (UI 'BtnAutoInstall').IsEnabled = (-not $installed) }
  if (UI 'BtnAutoRemove')  { (UI 'BtnAutoRemove').IsEnabled  = $installed }

  $savedMW = 0
  $mwPath = Get-ApplyMwPath
  if (Test-Path $mwPath) {
    try { $savedMW = [int](([System.IO.File]::ReadAllText($mwPath)).Trim()) } catch { $savedMW = 0 }
  }
  if ($savedMW -le 0) {
    $c0 = Get-Config
    if ($c0.PowerMW) { $savedMW = [int]$c0.PowerMW }
  }

  # trigger list
  $tl = UI 'TrigList'
  if ($tl) {
    $tl.Children.Clear()
    $rows = if ($zh) {
      @(
        @('登录后 2 秒', '启动文件夹里的 TDPilot AutoApply 立即运行'),
        @('静默提权', $(if (Test-UacSilent) { '本机不弹 UAC，全程无感' } else { '会弹出一次 UAC 确认' })),
        @('套用数值', $(if ($savedMW -gt 0) { ('{0:0.0} W（上次保存的值）' -f ($savedMW / 1000.0)) } else { '尚未保存过' })),
        @('失败自愈', '驱动服务卡死时自动 stop / start 重试')
      )
    } else {
      @(
        @('2 s after logon', 'TDPilot AutoApply in the Startup folder runs immediately'),
        @('Silent elevation', $(if (Test-UacSilent) { 'no UAC prompt on this machine' } else { 'one UAC prompt will appear' })),
        @('Saved value', $(if ($savedMW -gt 0) { ('{0:0.0} W (last saved)' -f ($savedMW / 1000.0)) } else { 'nothing saved yet' })),
        @('Self-healing', 'stops and restarts the driver service when it is stuck')
      )
    }
    foreach ($r in $rows) {
      $row = New-Object System.Windows.Controls.StackPanel
      $row.Orientation = 'Horizontal'
      $row.Margin = '0,5,0,5'
      $e = New-Object System.Windows.Shapes.Ellipse
      $e.Width = 6; $e.Height = 6; $e.Fill = Br '#22D3EE'
      $e.Margin = '2,0,10,0'; $e.VerticalAlignment = 'Center'
      $k = New-Object System.Windows.Controls.TextBlock
      $k.Text = $r[0]; $k.FontSize = 12; $k.Foreground = Br '#E8EDF5'; $k.Width = 128
      $v = New-Object System.Windows.Controls.TextBlock
      $v.Text = $r[1]; $v.FontSize = 12; $v.Foreground = Br '#9AA6B8'
      [void]$row.Children.Add($e); [void]$row.Children.Add($k); [void]$row.Children.Add($v)
      [void]$tl.Children.Add($row)
    }
  }

  $tb2 = UI 'TxtTaskBody'
  if ($tb2) { $tb2.Text = (Get-AutostartInfo) }
}

function Get-AutostartInfo {
  $zh = ($script:Lang -eq 'zh')
  $entry = Get-AutostartEntryPath
  $cmd   = Get-ApplyCmdPath
  $l = @()
  $l += $(if ($zh) { '启动项' } else { 'Startup entry' }) + ' : ' + $entry
  $l += $(if ($zh) { '存在       ' } else { 'exists     ' }) + ' : ' + (Test-Path $entry)
  $l += $(if ($zh) { '执行脚本' } else { 'Helper     ' }) + ' : ' + $cmd
  $l += $(if ($zh) { 'UAC 静默   ' } else { 'Silent UAC ' }) + ' : ' + (Test-UacSilent)
  if (Test-LegacyTaskInstalled) {
    $l += ''
    $l += $(if ($zh) { '检测到旧版计划任务，点「移除」可一并清理。' } else { 'An older scheduled task exists; press Remove to clean it up.' })
  }
  return ($l -join "`r`n")
}

# ------------------------------------------------------------ hardware page
function Render-Hardware {
  $cl = UI 'HwCpuList'
  if ($cl) {
    $cl.Children.Clear()
    $cpu = Get-CpuIdentity
    $rows = @(
      @{ k = $(if ($script:Lang -eq 'zh') { '型号' } else { 'Model' });        v = $cpu.Name }
      @{ k = $(if ($script:Lang -eq 'zh') { '厂商' } else { 'Vendor' });       v = $cpu.Vendor }
      @{ k = $(if ($script:Lang -eq 'zh') { '逻辑处理器' } else { 'Threads' }); v = "$($cpu.Logical)" }
      @{ k = $(if ($script:Lang -eq 'zh') { '标称频率' } else { 'Nominal' });  v = ("{0} MHz" -f $cpu.MaxMHz) }
    )
    foreach ($r in $rows) {
      $wrap = ($r.k -eq '型号' -or $r.k -eq 'Model')
      [void]$cl.Children.Add((New-KV $r.k $r.v -Wrap:$wrap))
    }
  }

  $ll = UI 'HwLimitList'
  if ($ll) {
    $ll.Children.Clear()
    $snap = $script:LastSnap
    if ($snap) {
      $d = $snap.Data
      $pairs = @(
        @('STAPM LIMIT (sustained)', 'stapmLimit'),
        @('PPT LIMIT FAST', 'fastLimit'),
        @('PPT LIMIT SLOW', 'slowLimit'),
        @('PPT LIMIT APU', 'apuLimit'),
        @('THM LIMIT CORE', 'thmLimit'),
        @('TDC LIMIT VDD', 'tdcVdd'),
        @('EDC LIMIT VDD', 'edcVdd')
      )
      foreach ($pr in $pairs) {
        if ($d.ContainsKey($pr[1])) {
          [void]$ll.Children.Add((New-KV $pr[0] ('{0:0.00}' -f $d[$pr[1]])))
        }
      }
    } else {
      $t = New-Object System.Windows.Controls.TextBlock
      $t.Text = $(if ($script:Lang -eq 'zh') { '（暂无数据）' } else { '(no data)' })
      $t.FontSize = 12; $t.Foreground = Br '#5E6B80'
      [void]$ll.Children.Add($t)
    }
  }
}

function New-KV([string]$Key, [string]$Value, [switch]$Wrap) {
  $g = New-Object System.Windows.Controls.Grid
  $g.Margin = '0,6,0,6'
  $c1 = New-Object System.Windows.Controls.ColumnDefinition; $c1.Width = 'Auto'
  $c2 = New-Object System.Windows.Controls.ColumnDefinition; $c2.Width = '*'
  $g.ColumnDefinitions.Add($c1); $g.ColumnDefinitions.Add($c2)
  $a = New-Object System.Windows.Controls.TextBlock
  $a.Text = $Key; $a.FontSize = 11.5; $a.Foreground = Br '#5E6B80'
  $a.Width = 116
  $a.VerticalAlignment = 'Top'
  $b = New-Object System.Windows.Controls.TextBlock
  $b.Text = $Value; $b.FontSize = 11.5; $b.Foreground = Br '#E8EDF5'
  $b.TextAlignment = 'Right'
  if ($Wrap) {
    $b.TextWrapping = 'Wrap'
    $b.LineHeight = 16
    $b.LineStackingStrategy = 'BlockLineHeight'
  } else {
    $b.TextTrimming = 'CharacterEllipsis'
  }
  [System.Windows.Controls.Grid]::SetColumn($a, 0)
  [System.Windows.Controls.Grid]::SetColumn($b, 1)
  [void]$g.Children.Add($a); [void]$g.Children.Add($b)
  return $g
}

# ------------------------------------------------------------ about env
function Render-AboutEnv {
  $tb = UI 'TxtAboutEnv'
  if (-not $tb) { return }
  $cpu = Get-CpuIdentity
  $os = [System.Environment]::OSVersion.VersionString
  $adm = Test-IsAdmin
  $dg = Test-DeviceGuard
  $lines = @(
    ('CPU        : {0}' -f $cpu.Name),
    ('OS         : {0}' -f $os),
    ('PowerShell : {0}' -f $PSVersionTable.PSVersion),
    ('Elevated   : {0}' -f $adm),
    ('HVCI       : {0}' -f $dg.Hvci),
    ('DriverBlocklist : {0}' -f $dg.Blocklist),
    ('ryzenadj   : {0}' -f $script:RyzenExe)
  )
  $tb.Text = ($lines -join "`r`n")
}
