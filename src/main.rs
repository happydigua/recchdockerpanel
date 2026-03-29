mod api;
mod app_store;
mod auth;
mod db;
mod docker;
mod models;
mod projects;

use anyhow::Result;
use axum::{
    http::{header, StatusCode, Uri},
    response::{IntoResponse, Redirect},
    routing::get,
    Router,
};
use rust_embed::Embed;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// 前端静态资源嵌入
#[derive(Embed)]
#[folder = "web/dist/"]
struct Asset;

/// 应用全局状态
pub struct AppState {
    pub docker: Option<bollard::Docker>,
    pub db: db::Database,
    pub jwt_secret: String,
    pub project_mgr: projects::ProjectManager,
    pub base_path: String,
}

/// 生成随机路径前缀（8 位字母数字）
fn generate_random_path() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let chars: Vec<char> = "abcdefghijkmnopqrstuvwxyzABCDEFGHJKLMNPQRSTUVWXYZ23456789"
        .chars()
        .collect();
    (0..8).map(|_| chars[rng.gen_range(0..chars.len())]).collect()
}

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "dockpanel=info,tower_http=info".into()),
        ))
        .init();

    tracing::info!("🚀 RecchDockerPanel 正在启动...");

    // 连接 Docker（可选，连接失败进入演示模式）
    let docker = match bollard::Docker::connect_with_socket_defaults() {
        Ok(d) => {
            match d.version().await {
                Ok(version) => {
                    tracing::info!(
                        "✅ Docker 连接成功 - 版本: {}",
                        version.version.unwrap_or_default()
                    );
                    Some(d)
                }
                Err(e) => {
                    tracing::warn!("⚠️ Docker 连接失败: {}，进入演示模式", e);
                    None
                }
            }
        }
        Err(e) => {
            tracing::warn!("⚠️ 未找到 Docker: {}，进入演示模式", e);
            None
        }
    };

    // 初始化数据库
    let database = db::Database::new("dockpanel.db")?;
    database.init_tables()?;
    tracing::info!("✅ 数据库初始化完成");

    // 生成或读取安全路径前缀
    let base_path = if let Ok(p) = std::env::var("DOCKPANEL_PATH") {
        // 环境变量优先
        p
    } else if let Ok(Some(p)) = database.get_setting("base_path") {
        // 其次数据库
        p
    } else {
        // 首次启动，生成随机路径并存入数据库
        let p = generate_random_path();
        database.set_setting("base_path", &p)?;
        p
    };

    // 生成或读取 JWT 密钥
    let jwt_secret = std::env::var("DOCKPANEL_SECRET")
        .unwrap_or_else(|_| uuid::Uuid::new_v4().to_string());

    // 初始化默认管理员账户
    database.ensure_admin_exists()?;

    let state = Arc::new(AppState {
        docker,
        db: database,
        jwt_secret,
        project_mgr: projects::ProjectManager::new(),
        base_path: base_path.clone(),
    });

    // CORS 配置（开发模式）
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // 构建带前缀的路由
    let prefix = format!("/{}", base_path);

    let panel_routes = Router::new()
        .nest("/api", api::routes())
        .fallback(static_handler)
        .with_state(state);

    let app = Router::new()
        // 根路径重定向到安全路径
        .route("/", get(move || async move {
            Redirect::permanent(&format!("/{}/", base_path))
        }))
        .nest(&prefix, panel_routes)
        .layer(cors);

    let port = std::env::var("DOCKPANEL_PORT").unwrap_or_else(|_| "3001".into());
    let addr = format!("0.0.0.0:{}", port);
    tracing::info!("🌐 RecchDockerPanel 已启动: http://0.0.0.0:{}/{}/", port, prefix.trim_start_matches('/'));
    tracing::info!("📖 默认账户: admin / admin123（请及时修改密码）");
    tracing::info!("🔒 安全路径: {} （可通过 DOCKPANEL_PATH 环境变量自定义）", prefix);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// 处理前端静态资源请求
async fn static_handler(uri: Uri) -> impl IntoResponse {
    let path = uri.path().trim_start_matches('/');

    // 尝试找到对应文件
    if let Some(content) = Asset::get(path) {
        let mime = mime_guess::from_path(path).first_or_octet_stream();
        let body: Vec<u8> = content.data.to_vec();
        (
            StatusCode::OK,
            [(header::CONTENT_TYPE, mime.as_ref())],
            body,
        )
            .into_response()
    } else {
        // SPA 回退：返回 index.html
        match Asset::get("index.html") {
            Some(content) => {
                let body: Vec<u8> = content.data.to_vec();
                (
                    StatusCode::OK,
                    [(header::CONTENT_TYPE, "text/html")],
                    body,
                )
                    .into_response()
            }
            None => (StatusCode::NOT_FOUND, "前端资源未找到，请先构建前端: cd web && npm run build").into_response(),
        }
    }
}
