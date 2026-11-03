use std::path::PathBuf;

/// 应用级数据目录名（与 tauri identifier 保持一致，卸载时一并清理）。
const APP_DATA_DIR_NAME: &str = "com.wenxun.app";

/// 可执行文件所在目录（安装版是 Program Files\wenxun，开发版是 target/debug）。
pub fn current_exe_dir() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."))
}

fn os_app_base_dir() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        return std::env::var_os("APPDATA").map(PathBuf::from);
    }

    #[cfg(target_os = "macos")]
    {
        return std::env::var_os("HOME")
            .map(PathBuf::from)
            .map(|h| h.join("Library/Application Support"));
    }

    #[cfg(target_os = "linux")]
    {
        if let Some(d) = std::env::var_os("XDG_DATA_HOME") {
            return Some(PathBuf::from(d));
        }
        return std::env::var_os("HOME")
            .map(PathBuf::from)
            .map(|h| h.join(".local/share"));
    }

    #[allow(unreachable_code)]
    None
}

/// 用户可写的数据根目录：Windows 为 %APPDATA%\com.wenxun.app。
/// 读不到系统环境变量时回退到 exe 目录，保证开发/异常环境仍可启动。
pub fn app_data_dir() -> PathBuf {
    os_app_base_dir()
        .map(|base| base.join(APP_DATA_DIR_NAME))
        .unwrap_or_else(current_exe_dir)
}

pub fn db_path() -> PathBuf {
    app_data_dir().join("wenxun.db")
}

pub fn log_dir() -> PathBuf {
    app_data_dir().join("log")
}

/// 兼容旧版：数据库曾放在安装目录/exe 目录下。
/// 新版首次启动时如果用户目录还没有数据库，就把旧库（含 WAL）复制过来。
pub fn migrate_legacy_database() {
    let src_dir = current_exe_dir();
    let src_db = src_dir.join("wenxun.db");
    let dest_dir = app_data_dir();

    if !src_db.exists() || dest_dir.join("wenxun.db").exists() {
        return;
    }
    if let Err(e) = std::fs::create_dir_all(&dest_dir) {
        eprintln!("[storage] 创建数据目录失败: {}", e);
        return;
    }

    for suffix in ["", "-wal", "-shm"] {
        let src = src_dir.join(format!("wenxun.db{}", suffix));
        if !src.exists() {
            continue;
        }
        let dest = dest_dir.join(format!("wenxun.db{}", suffix));
        if let Err(e) = std::fs::copy(&src, &dest) {
            eprintln!("[storage] 迁移 {} 失败: {}", src.display(), e);
            return;
        }
    }
    eprintln!(
        "[storage] 已把旧版数据库从 {} 迁移到 {}",
        src_db.display(),
        dest_dir.display()
    );
}
