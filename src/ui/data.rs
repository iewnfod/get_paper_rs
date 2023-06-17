use std::{path::{Path, PathBuf}};

pub const KINDS: &[&str] = &[
    "0413 - Physical Education (IGCSE)",
    "0450 - Business Studies (IGCSE)",
    "0452 - Accounting (IGCSE)",
    "0455 - Economics (IGCSE)",
    "0460 - Geography (IGCSE)",
    "0470 - History (IGCSE)",
    "0478 - Computer Science (IGCSE)",
    "0500 - English First Language (IGCSE)",
    "0509 - Chinese First Language (IGCSE)",
    "0510 - English Speaking endorsement (IGCSE)",
    "0511 - English Count-in speaking (IGCSE)",
    "0580 - Mathematics (IGCSE)",
    "0606 - Mathematics Additional (IGCSE)",
    "0610 - Biology (IGCSE)",
    "0620 - Chemistry (IGCSE)",
    "0625 - Physics (IGCSE)",
    "9093 - English Language AS and A Level (AS-A2)",
    "9231 - Further Mathematics (AS-A2)",
    "9389 - History (AS-A2)",
    "9396 - Physical Education (AS-A2)",
    "9489 - History (AS-A2)",
    "9608 - Computer Science (AS-A2)",
    "9609 - Business (AS-A2)",
    "9618 - Computer Science (AS-A2)",
    "9696 - Geography (AS-A2)",
    "9698 - Psychology (AS-A2)",
    "9700 - Biology (AS-A2)",
    "9701 - Chemistry (AS-A2)",
    "9702 - Physics (AS-A2)",
    "9706 - Accounting (AS-A2)",
    "9707 - Business Studies (AS-A2)",
    "9708 - Economics (AS-A2)",
    "9709 - Mathematics (AS-A2)",
    "9713 - Applied ICT (AS-A2)",
    "9715 - Chinese First Language (AS-A2)",
    "9990 - Psychology (AS-A2)"
];

pub const SEARCH_URL: &str = "https://cie.fraft.cn/obj/Fetch/renum";
pub const FETCH_URL: &str = "https://cie.fraft.cn/obj/Fetch/redir/";

pub const SEASONS: &[&str] = &[
    "Jun",
    "Nov",
    "Mar",
    "Gen"
];

pub fn base_dir() -> PathBuf {
    if let Some(home) = dirs::home_dir() {
        if cfg!(target_os = "windows") {
            return home.join("AppData\\Local\\get-paper-rs").to_path_buf();
        } else if cfg!(target_os = "macos") {
            return home.join("Library/Application Support/get-paper-rs").to_path_buf();
        } else {
            return home.join("get-paper-rs").to_path_buf();
        }
    } else {
        Path::new("get-paper-rs").to_path_buf()
    }
}

pub const CONFIG_PATH: &str = "config.txt";
pub static mut SAVE_DIR: Option<String> = None;
pub const DEFAULT_SAVE_DIR: &str = "PastPapers";
pub fn get_save_dir() -> String {
    let p = unsafe { SAVE_DIR.clone() };
    match p {
        Some(val) => val,
        None => base_dir().join(DEFAULT_SAVE_DIR.to_string()).to_str().unwrap().to_string()
    }
}

pub const DOUBLE_CLICK_INTERVAL: f32 = 0.5;  // 秒为单位

pub fn default_config_content() -> String {
    unsafe {
        format!("
            save_dir={}
            width={}
            height={}
            ",
            WIDTH,
            HEIGHT,
            get_save_dir()
        )
    }
}

pub static mut WIDTH: i32 = 1000;
pub static mut HEIGHT: i32 = 950;
