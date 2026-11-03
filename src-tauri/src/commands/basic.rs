use crate::paths;

#[tauri::command]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

// ============ Logging ============
#[tauri::command]
pub fn get_log_path() -> Result<String, String> {
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    Ok(paths::log_dir().join(format!("wenxun-{}.log", today)).to_string_lossy().to_string())
}

#[tauri::command]
pub fn open_log_dir() -> Result<(), String> {
    let log_dir = paths::log_dir();
    std::fs::create_dir_all(&log_dir).map_err(|e| format!("创建日志目录失败: {}", e))?;

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(&log_dir)
            .spawn()
            .map_err(|e| format!("打开日志目录失败: {}", e))?;
        Ok(())
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&log_dir)
            .spawn()
            .map_err(|e| format!("打开日志目录失败: {}", e))?;
        Ok(())
    }

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&log_dir)
            .spawn()
            .map_err(|e| format!("打开日志目录失败: {}", e))?;
        Ok(())
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        let _ = log_dir;
        Err("当前系统暂不支持自动打开目录".to_string())
    }
}

#[tauri::command]
pub fn open_url(url: String) -> Result<(), String> {
    let url = url.trim().to_string();
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err("仅支持 http/https 链接".to_string());
    }

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("rundll32")
            .args(["url.dll,FileProtocolHandler", &url])
            .spawn()
            .map_err(|e| format!("打开链接失败: {}", e))?;
        Ok(())
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&url)
            .spawn()
            .map_err(|e| format!("打开链接失败: {}", e))?;
        Ok(())
    }

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&url)
            .spawn()
            .map_err(|e| format!("打开链接失败: {}", e))?;
        Ok(())
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        Err("当前系统暂不支持自动打开链接".to_string())
    }
}

#[tauri::command]
pub async fn get_public_ip() -> Result<String, String> {
    let urls = [
        "https://api.ipify.org",
        "https://ifconfig.me/ip",
        "https://ip.sb",
        "https://httpbin.org/ip",
    ];

    // 直连出口 IP 检测：绕过系统代理，否则测到的是代理的 IP
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .no_proxy()
        .build()
        .map_err(|e| format!("创建请求失败: {}", e))?;

    for url in &urls {
        match client.get(*url).send().await {
            Ok(resp) => {
                let body = resp.text().await.map_err(|e| format!("读取响应失败: {}", e))?;
                let ip = body.trim();
                // httpbin returns JSON, extract "origin" field
                if let Ok(obj) = serde_json::from_str::<serde_json::Value>(ip) {
                    if let Some(origin) = obj["origin"].as_str() {
                        return Ok(origin.to_string());
                    }
                }
                if !ip.is_empty() {
                    return Ok(ip.to_string());
                }
            }
            Err(e) => {
                log::warn!("获取公网 IP 失败 ({}): {}", url, e);
                continue;
            }
        }
    }

    Err("无法获取公网 IP，请检查网络连接".to_string())
}
