// Windows release 下隐藏控制台窗口。
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    eratw_next_desktop_lib::run();
}
