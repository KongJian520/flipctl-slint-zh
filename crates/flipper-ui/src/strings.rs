//! The panel's copy in each language, and which one is being drawn.
//!
//! The English text is the key. That is deliberate and it is the whole design: the
//! menu tables, the status strings and every screen's labels are `&'static str`
//! literals that also serve as identifiers (`row.label == "5G Modem"`,
//! `card_menu(title)`), so turning them into an enum would mean rewriting the data
//! and every comparison with it. Looking the translation up by the string it
//! replaces keeps those comparisons working on the language-independent English and
//! leaves one table to maintain.
//!
//! A string that is not in the table comes back unchanged, so a missing entry shows
//! English rather than nothing — the same failure the panel had before there was a
//! table at all. `tests/strings.rs` checks the table for duplicate keys.
//!
//! Which language is drawn is a runtime value, not a compile-time one: the faces
//! carry both scripts, so switching costs a redraw and nothing else.

use std::sync::atomic::{AtomicU8, Ordering};

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Lang {
    En,
    Zh,
}

static LANG: AtomicU8 = AtomicU8::new(Lang::Zh as u8);

pub fn lang() -> Lang {
    match LANG.load(Ordering::Relaxed) {
        1 => Lang::Zh,
        _ => Lang::En,
    }
}

/// Switch the language. The caller redraws; nothing here knows about screens.
pub fn set_lang(lang: Lang) {
    LANG.store(lang as u8, Ordering::Relaxed);
}

/// The copy for `text` in the current language, or `text` itself when the table has
/// no entry for it.
pub fn tr(text: &'static str) -> &'static str {
    if lang() == Lang::En {
        return text;
    }
    ZH.binary_search_by(|(en, _)| (*en).cmp(text))
        .map(|at| ZH[at].1)
        .unwrap_or(text)
}

/// The same lookup for a string that is not necessarily `'static`: a row's value, a
/// device name, an SSID. Not found means the input comes back, so passing a value
/// that happens to be dynamic costs nothing.
pub fn tr_any<'a>(text: &'a str) -> &'a str {
    if lang() == Lang::En {
        return text;
    }
    ZH.binary_search_by(|(en, _)| (*en).cmp(text))
        .map(|at| ZH[at].1)
        .unwrap_or(text)
}

/// Simplified Chinese, sorted by the English key so `tr` can binary-search it.
pub const ZH: &[(&str, &str)] = &[
    ("5G Modem", "5G 模块"),
    ("Airplane mode", "飞行模式"),
    ("App Manager", "应用管理"),
    ("Apps", "应用"),
    ("Back", "返回"),
    ("Battery", "电池"),
    ("Battery info", "电池信息"),
    ("CPU", "CPU"),
    ("CPU serial", "CPU 序列号"),
    ("Capacity", "容量"),
    ("Channel", "通道"),
    ("Checking the build server...", "正在检查构建服务器…"),
    ("Commit", "提交"),
    ("Cycles", "循环次数"),
    ("Default gateways:", "默认网关："),
    ("Desktop Computer", "桌面电脑"),
    ("Disk", "磁盘"),
    ("Disk info", "磁盘信息"),
    ("Do not power off", "请勿断电"),
    ("Duplex", "双工"),
    ("Ethernet", "以太网"),
    ("Files", "文件"),
    ("Health", "健康度"),
    ("Info", "信息"),
    ("Install type", "安装类型"),
    ("Kernel", "内核"),
    ("LED Brightness", "LED 亮度"),
    ("Life used", "已用寿命"),
    ("Link", "链路"),
    ("Link speed", "链路速率"),
    ("MCU", "MCU"),
    ("Maskrom", "Maskrom"),
    ("Memory", "内存"),
    ("Model", "型号"),
    ("Modem", "模块"),
    ("Network", "网络"),
    ("No default routes", "无默认路由"),
    ("Not available", "不可用"),
    ("Not found", "未找到"),
    ("Not mounted", "未挂载"),
    ("OFF", "关"),
    ("ON", "开"),
    ("OS", "系统"),
    ("OS Profiles", "系统配置"),
    ("Power", "电源"),
    ("Power Management", "电源管理"),
    ("Profile", "配置"),
    ("QR", "QR"),
    ("Reboot", "重启"),
    ("Reboots into the installer", "重启进入安装器"),
    ("Reinstall FlipperOS", "重装系统"),
    ("Routing info", "路由信息"),
    ("Run", "运行"),
    ("SD Card", "SD 卡"),
    ("SD card", "SD 卡"),
    ("Self Check", "自检"),
    ("Settings", "设置"),
    ("Shutdown", "关机"),
    ("Signal", "信号"),
    ("Size", "大小"),
    ("Slot", "槽位"),
    ("SoC", "SoC"),
    ("Starting the installer", "正在启动安装器"),
    ("System files changed", "系统文件已变更"),
    ("System info", "系统信息"),
    ("TAC", "TAC"),
    ("Temp", "温度"),
    ("Testing", "测试"),
    ("UFS", "UFS"),
    ("USB-C1 power out", "USB-C1 供电"),
    ("Unknown", "未知"),
    ("Update FlipperOS", "更新系统"),
    ("Updated automatically", "自动更新"),
    ("Uptime", "运行时长"),
    ("Version", "版本"),
    ("Voltage", "电压"),
    ("Wattmeter", "功率计"),
    ("connected", "已连接"),
    ("metric", "度量值"),
    ("not connected", "未连接"),
    ("via", "经"),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_table_is_sorted_and_unique() {
        for pair in ZH.windows(2) {
            assert!(pair[0].0 < pair[1].0, "{:?} is out of order", pair[1].0);
        }
    }

    #[test]
    fn a_missing_key_comes_back_as_itself() {
        set_lang(Lang::Zh);
        assert_eq!(tr("Something nobody translated"), "Something nobody translated");
        set_lang(Lang::En);
        assert_eq!(tr("Network"), "Network");
        set_lang(Lang::Zh);
        assert_eq!(tr("Network"), "网络");
    }
}
