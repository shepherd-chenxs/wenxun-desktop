#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::{Arc, Mutex};
use wenxun::database::Database;
use wenxun::paths;
use wenxun::scheduler::TaskScheduler;
use wenxun::state::AppState;

fn main() {
    // 旧版（安装目录/exe 目录）数据库迁移到用户数据目录后再打开新库
    paths::migrate_legacy_database();
    let data_dir = paths::app_data_dir();
    let log_dir = paths::log_dir();
    if let Err(e) = std::fs::create_dir_all(&data_dir) {
        eprintln!("创建数据目录失败: {}", e);
    }
    std::fs::create_dir_all(&log_dir).ok();
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let log_filename = format!("wenxun-{}.log", today);

    let db_path = paths::db_path();
    let db_path_str = db_path.to_string_lossy().to_string();
    eprintln!("数据目录: {}", data_dir.display());

    // 单一数据库连接：AppState 命令层与调度器共享（std Mutex 串行化，配合 WAL）
    let db = match Database::new(&db_path_str) {
        Ok(db) => db,
        Err(e) => {
            eprintln!("Database initialization failed: {}", e);
            std::process::exit(1);
        }
    };

    let db_arc = Arc::new(Mutex::new(db));

    let log_path = log_dir.join(&log_filename);
    eprintln!("日志文件: {}", log_path.display());
    let scheduler = TaskScheduler::new(Arc::clone(&db_arc));

    tauri::Builder::default()
        // 页面加载完成后显示窗口（配合 visible:false）：不依赖前端脚本时序
        .on_page_load(|webview_window, payload| {
            log::info!("[window] on_page_load event: {:?}", payload.event());
            if payload.event() == tauri::webview::PageLoadEvent::Finished {
                let shown = webview_window.show();
                log::info!("[window] show result: {:?}", shown);
                let _ = webview_window.set_focus();
            }
        })
        // 单实例：第二次启动时聚焦已打开的窗口，而不是静默退出
        .plugin(tauri_plugin_single_instance::init(|_app, _args, _cwd| {
            use tauri::Manager;
            if let Some(window) = _app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.unminimize();
                let _ = window.set_focus();
            }
        }))
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(
            tauri_plugin_log::Builder::new()
                .targets([
                    tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::Folder {
                        path: log_dir.clone(),
                        file_name: Some(log_filename.clone()),
                    }),
                    tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::Stdout),
                ])
                .level(log::LevelFilter::Debug)
                .max_file_size(10_485_760) // 10MB 单文件上限
                .rotation_strategy(tauri_plugin_log::RotationStrategy::KeepOne)
                .build(),
        )
        .manage(AppState {
            db: Arc::clone(&db_arc),
        })
        .setup(move |_app| {
            let s = scheduler.clone();
            std::thread::spawn(move || {
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .expect("Failed to create Tokio runtime for scheduler");
                rt.block_on(async move {
                    log::info!("[scheduler] 调度器线程已启动");
                    s.start().await;
                });
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Basic
            wenxun::commands::greet,
            // Tasks
            wenxun::commands::get_tasks,
            wenxun::commands::add_task,
            wenxun::commands::update_task,
            wenxun::commands::delete_task,
            wenxun::commands::execute_task,
            // Articles
            wenxun::commands::get_articles,
            wenxun::commands::get_article,
            wenxun::commands::save_article,
            wenxun::commands::delete_article,
            wenxun::commands::save_articles_batch,
            // Topic Categories
            wenxun::commands::get_topic_categories,
            wenxun::commands::add_topic_category,
            wenxun::commands::update_topic_category,
            wenxun::commands::delete_topic_category,
            // Bloggers
            wenxun::commands::get_bloggers,
            wenxun::commands::add_blogger,
            wenxun::commands::update_blogger,
            wenxun::commands::save_blogger,
            wenxun::commands::delete_blogger,
            // Accounts
            wenxun::commands::get_accounts,
            wenxun::commands::add_account,
            wenxun::commands::update_account,
            wenxun::commands::save_wechat_account,
            wenxun::commands::delete_account,
            // Topic Packs
            wenxun::commands::get_topic_packs,
            wenxun::commands::add_topic_pack,
            wenxun::commands::update_topic_pack,
            wenxun::commands::save_topic_pack,
            wenxun::commands::delete_topic_pack,
            // AI API Keys
            wenxun::commands::get_ai_api_key,
            wenxun::commands::save_ai_api_key,
            wenxun::commands::get_ai_config,
            wenxun::commands::save_ai_config,
            // Task Logs
            wenxun::commands::get_task_logs,
            wenxun::commands::add_task_log,
            wenxun::commands::clear_task_logs,
            // AI Generation
            wenxun::commands::generate_article,
            wenxun::commands::generate_title,
            wenxun::commands::generate_summary,
            // Wechat
            wenxun::commands::wechat_publish,
            wenxun::commands::wechat_upload_media,
            wenxun::commands::wechat_upload_media_with_app,
            // Generation History
            wenxun::commands::get_generation_history,
            wenxun::commands::save_generation_history,
            wenxun::commands::clear_generation_history,
            // Topics
            wenxun::commands::get_topics,
            wenxun::commands::add_topic,
            wenxun::commands::update_topic,
            wenxun::commands::delete_topic,
            // Publish History
            wenxun::commands::get_publish_history,
            wenxun::commands::add_publish_history,
            wenxun::commands::clear_publish_history,
            // IP
            wenxun::commands::get_public_ip,
            // Verify WeChat
            wenxun::commands::verify_wechat_account,
            // Logging
            wenxun::commands::get_log_path,
            wenxun::commands::open_log_dir,
            wenxun::commands::open_url,
            // 爆文工坊
            wenxun::commands::fetch_wechat_article,
            wenxun::commands::ai_chat,
            // 热点雷达
            wenxun::hot::get_hot_list,
            wenxun::hot::get_hot_sources,
            // 质量体检生成 + 人设沉淀
            wenxun::commands::generate_with_quality_gate,
            wenxun::commands::generate_blogger_article,
            wenxun::commands::create_blogger_from_analysis,
            wenxun::commands::recheck_quality,
            wenxun::commands::revise_with_report,
            wenxun::commands::add_lesson_to_blogger,
            // 质检库
            wenxun::commands::get_quality_report_list,
            wenxun::commands::delete_quality_report,
            wenxun::commands::analyze_selected_quality_reports,
            // 智能体检：创作归属检测 + 去 AI 味
            wenxun::commands::detect_ai_authorship,
            wenxun::commands::humanize_ai_text,
            wenxun::commands::humanize_ai_text_with_check,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
