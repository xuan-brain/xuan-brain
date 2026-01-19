use tauri::Manager;
use tracing::{debug, error, info};

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    info!("greet 命令被调用，参数: {}", name);
    format!("Hello, {}! You've been greeted from Rust!", name)
}

/// 应用目录结构
#[derive(serde::Serialize, Debug)]
pub struct AppDirs {
    /// 配置文件目录
    pub config: String,
    /// 数据文件目录（数据库、文档等）
    pub data: String,
    /// 缓存目录
    pub cache: String,
    /// 日志目录
    pub logs: String,
}

/// 初始化应用数据目录
///
/// 检测并创建用户数据文件夹结构，包括：
/// - config/：配置文件
/// - data/：数据库和文档
/// - cache/：缓存文件
/// - logs/：应用日志
///
/// 返回各个目录的路径
#[tauri::command]
async fn init_app_dirs(app_handle: tauri::AppHandle) -> Result<AppDirs, String> {
    // 获取应用数据目录
    let data_dir = app_handle.path().app_data_dir().map_err(|e| {
        error!("无法获取应用数据目录: {}", e);
        format!("无法获取应用数据目录: {}", e)
    })?;

    info!("应用数据目录: {:?}", data_dir);

    // 定义子目录结构
    let dirs = vec![
        ("config", "配置文件"),
        ("data", "数据文件"),
        ("cache", "缓存文件"),
        ("logs", "日志文件"),
    ];

    // 创建所有子目录
    for (dir_name, description) in dirs {
        let dir_path = data_dir.join(dir_name);

        match tokio::fs::metadata(&dir_path).await {
            Ok(_) => {
                debug!("{} 目录已存在: {:?}", description, dir_path);
            }
            Err(_) => {
                info!("创建 {} 目录: {:?}", description, dir_path);
                tokio::fs::create_dir_all(&dir_path).await.map_err(|e| {
                    error!("创建目录失败: {} ({})", dir_path.display(), e);
                    format!("创建目录失败: {} ({})", dir_path.display(), e)
                })?;
                info!("{} 目录创建成功: {:?}", description, dir_path);
            }
        }
    }

    // 返回所有目录路径
    Ok(AppDirs {
        config: data_dir.join("config").to_string_lossy().to_string(),
        data: data_dir.join("data").to_string_lossy().to_string(),
        cache: data_dir.join("cache").to_string_lossy().to_string(),
        logs: data_dir.join("logs").to_string_lossy().to_string(),
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 初始化 tracing subscriber
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                tracing_subscriber::EnvFilter::new("xuan_brain=debug,tauri=debug")
            }),
        )
        .init();

    info!("应用启动中...");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // 在应用启动时初始化数据目录
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                info!("正在初始化应用数据目录...");
                match init_app_dirs(app_handle).await {
                    Ok(dirs) => {
                        info!("应用数据目录初始化成功:");
                        info!("  配置目录: {}", dirs.config);
                        info!("  数据目录: {}", dirs.data);
                        info!("  缓存目录: {}", dirs.cache);
                        info!("  日志目录: {}", dirs.logs);
                    }
                    Err(e) => {
                        error!("初始化数据目录失败: {}", e);
                    }
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![greet, init_app_dirs])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
