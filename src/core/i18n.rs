//! Bilingual strings. Every user-visible string goes through `t()`.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    Zh,
    En,
}

impl Lang {
    pub fn from_code(s: &str) -> Self {
        match s {
            "en" => Lang::En,
            _ => Lang::Zh,
        }
    }
    pub fn code(self) -> &'static str {
        match self {
            Lang::Zh => "zh",
            Lang::En => "en",
        }
    }
    pub fn flip(self) -> Self {
        match self {
            Lang::Zh => Lang::En,
            Lang::En => Lang::Zh,
        }
    }
}

#[derive(Clone, Copy)]
pub struct Strings;

macro_rules! strings {
    ($($key:ident => ($zh:expr, $en:expr)),* $(,)?) => {
        impl Strings {
            $(
                pub fn $key(self, lang: Lang) -> &'static str {
                    match lang { Lang::Zh => $zh, Lang::En => $en }
                }
            )*
        }
    };
}

strings! {
    tab_dashboard      => ("仪表盘",                            "Dashboard"),
    tab_profiles       => ("预设方案",                          "Profiles"),
    tab_auto           => ("自动化",                            "Automation"),
    tab_hardware       => ("硬件信息",                          "Hardware"),
    tab_settings       => ("设置",                              "Settings"),
    tab_about          => ("关于",                              "About"),

    dash_sub           => ("实时功耗与温度监控",                "Live power and temperature"),
    card_power         => ("封装功耗",                          "Package power"),
    card_temp          => ("核心温度",                          "Core temp"),
    card_clock         => ("当前频率",                          "Current clock"),
    card_limit         => ("当前上限",                          "Active cap"),
    chart_title        => ("功耗 / 温度趋势",                   "Power / temperature trend"),
    chart_power        => ("功耗",                              "Power"),
    chart_temp         => ("温度",                              "Temp"),

    cap_title          => ("功耗上限",                          "Power cap"),
    cap_hint           => ("拖动滑块、直接输入数字，或点选下方预设",  "Drag, type, or pick a preset below"),
    cap_apply          => ("应用",                              "Apply"),
    cap_applying       => ("正在应用…",                         "Applying…"),
    cap_quick          => ("快速预设",                          "Quick presets"),

    prof_sub           => ("为不同场景保存常用功率",            "Saved power levels for different scenarios"),
    prof_add           => ("新增预设",                          "New profile"),
    prof_name          => ("名称",                              "Name"),
    prof_value         => ("功耗",                              "Power"),
    prof_save          => ("保存",                              "Save"),
    prof_cancel        => ("取消",                              "Cancel"),
    prof_delete        => ("删除",                              "Delete"),
    prof_empty         => ("还没有预设，点「新增预设」创建一个",  "No profiles yet — create one"),

    auto_sub           => ("登录时自动把保存的功耗上限写回处理器",  "Re-apply your saved cap at logon"),
    auto_enable        => ("开机自动套用",                      "Apply at logon"),
    auto_desc          => ("通过「启动」文件夹实现，无需计划任务",  "Uses the Startup folder — no scheduled task"),
    auto_value         => ("登录时的功耗上限",                  "Power cap at logon"),
    auto_install       => ("启用",                              "Enable"),
    auto_uninstall     => ("停用",                              "Disable"),
    auto_active        => ("已启用",                            "Active"),
    auto_inactive      => ("未启用",                            "Not active"),
    auto_restore       => ("立即套用一次",                      "Apply now"),
    auto_note          => ("启用后会在下次登录时自动生效；随时可以停用。",
                          "Takes effect at your next logon; you can disable it any time."),

    hw_sub             => ("处理器与固件详情",                  "Processor and firmware"),
    hw_refresh         => ("重新检测",                          "Refresh"),
    hw_vendor          => ("厂商",                              "Vendor"),
    hw_cores           => ("核心 / 线程",                       "Cores / Threads"),
    hw_maxclock        => ("最高频率",                          "Max clock"),
    hw_arch            => ("架构",                              "Architecture"),
    hw_model           => ("型号",                              "Model"),
    hw_product         => ("产品",                              "Product"),
    hw_board           => ("主板",                              "Board"),
    hw_bios            => ("BIOS",                              "BIOS"),
    hw_os              => ("系统",                              "OS"),
    hw_path            => ("ryzenadj.exe",                      "ryzenadj.exe"),

    set_sub            => ("刷新频率、日志与维护",              "Refresh rate, logs and maintenance"),
    set_refresh        => ("刷新",                              "Refresh"),
    set_interval       => ("刷新间隔",                          "Refresh interval"),
    set_lang           => ("界面语言",                          "Language"),
    set_log            => ("运行日志",                          "Log"),
    set_openlog        => ("打开日志文件",                      "Open log file"),
    set_opendata       => ("打开数据目录",                      "Open data folder"),
    set_driverrestart  => ("重启内核驱动",                      "Restart kernel driver"),
    set_drvhint        => ("功耗读取失败时用它恢复",            "Use this if readings stop working"),

    about_sub          => ("版本与开源许可",                    "Version and license"),
    about_version      => ("版本",                              "Version"),
    about_license      => ("许可",                              "License"),
    about_source       => ("源码",                              "Source"),

    st_ready           => ("驱动就绪",                          "Driver ready"),
    st_dead            => ("驱动异常",                          "Driver unavailable"),
    st_unknown         => ("检测中…",                           "Checking…"),

    msg_applied        => ("已应用",                            "Applied"),
    msg_failed         => ("应用失败",                          "Apply failed"),
    msg_ok             => ("完成",                              "Done"),

    unit_w             => ("W",                                 "W"),
    unit_c             => ("°C",                                "°C"),
    unit_mhz           => ("MHz",                               "MHz"),

    stat_title         => ("本次运行统计",                      "Session statistics"),
    stat_avg           => ("平均功耗",                          "Average"),
    stat_peak          => ("峰值功耗",                          "Peak"),
    stat_energy        => ("累计能耗",                          "Energy used"),
    stat_note          => ("按采样间隔估算，仅供参考",           "Estimated from the sampling interval"),
}