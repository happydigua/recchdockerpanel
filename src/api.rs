use axum::{
    extract::{Json, Path, State, Query},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post},
    Router,
};
use std::sync::Arc;

use crate::auth;
use crate::docker;
use crate::app_store;
use crate::projects;
use crate::models::*;
use crate::AppState;

/// 获取 Docker 客户端，不可用时返回错误响应
fn require_docker(state: &AppState) -> Result<&bollard::Docker, impl IntoResponse> {
    state.docker.as_ref().ok_or((
        StatusCode::SERVICE_UNAVAILABLE,
        Json(ApiResponse::<()>::error("Docker 未连接，请安装并启动 Docker Desktop")),
    ))
}

/// 注册所有 API 路由
pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        // 公开路由
        .route("/auth/login", post(login))
        // 系统信息
        .route("/system/info", get(system_info))
        // 容器管理
        .route("/containers", get(list_containers))
        .route("/containers", post(create_container))
        .route("/containers/{id}/start", post(start_container))
        .route("/containers/{id}/stop", post(stop_container))
        .route("/containers/{id}/restart", post(restart_container))
        .route("/containers/{id}", delete(remove_container))
        .route("/containers/{id}/logs", get(container_logs))
        // 镜像管理
        .route("/images", get(list_images))
        .route("/images/search", get(search_images))
        .route("/images/tags", get(list_image_tags))
        .route("/images/pull", post(pull_image))
        .route("/images/{id}", delete(remove_image))
        // 网络和存储卷
        .route("/networks", get(list_networks))
        .route("/volumes", get(list_volumes))
        // 应用商店
        .route("/apps", get(list_apps))
        .route("/apps/{id}/install", post(install_app))
        // 项目部署
        .route("/projects", get(list_projects))
        .route("/projects", post(deploy_project))
        .route("/projects/{name}", delete(delete_project))
        .route("/projects/{name}/redeploy", post(redeploy_project))
}

// ============ 认证 API ============

async fn login(
    State(state): State<Arc<AppState>>,
    Json(req): Json<LoginRequest>,
) -> impl IntoResponse {
    match state.db.verify_user(&req.username, &req.password) {
        Ok(Some(user)) => {
            match auth::create_token(&user.username, &state.jwt_secret) {
                Ok(token) => {
                    let resp = LoginResponse {
                        token,
                        username: user.username,
                    };
                    (StatusCode::OK, Json(ApiResponse::ok(resp))).into_response()
                }
                Err(_) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::<()>::error("Token 生成失败")),
                )
                    .into_response(),
            }
        }
        Ok(None) => (
            StatusCode::UNAUTHORIZED,
            Json(ApiResponse::<()>::error("用户名或密码错误")),
        )
            .into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error("服务器内部错误")),
        )
            .into_response(),
    }
}

// ============ 系统信息 API ============

async fn system_info(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    match &state.docker {
        Some(d) => {
            match docker::get_system_info(d).await {
                Ok(info) => {
                    // 在返回的 JSON 中注入 docker_installed 标志
                    (StatusCode::OK, Json(ApiResponse::ok(serde_json::json!({
                        "docker_installed": true,
                        "docker_version": info.docker_version,
                        "containers_running": info.containers_running,
                        "containers_stopped": info.containers_stopped,
                        "containers_total": info.containers_total,
                        "images_total": info.images_total,
                        "cpu_count": info.cpu_count,
                        "memory_total": info.memory_total,
                        "os": info.os,
                        "arch": info.arch,
                    })))).into_response()
                }
                Err(e) => (
                    StatusCode::OK,
                    Json(ApiResponse::ok(serde_json::json!({
                        "docker_installed": false,
                        "error": e.to_string(),
                    }))),
                ).into_response(),
            }
        }
        None => {
            (StatusCode::OK, Json(ApiResponse::ok(serde_json::json!({
                "docker_installed": false,
            })))).into_response()
        }
    }
}

// ============ 容器管理 API ============

async fn list_containers(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let d = match require_docker(&state) {
        Ok(d) => d,
        Err(e) => return e.into_response(),
    };
    match docker::list_containers(d).await {
        Ok(containers) => (StatusCode::OK, Json(ApiResponse::ok(containers))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(&e.to_string())),
        )
            .into_response(),
    }
}

async fn create_container(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateContainerRequest>,
) -> impl IntoResponse {
    let d = match require_docker(&state) {
        Ok(d) => d,
        Err(e) => return e.into_response(),
    };
    match docker::create_container(d, &req).await {
        Ok(id) => (StatusCode::CREATED, Json(ApiResponse::ok(id))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(&e.to_string())),
        )
            .into_response(),
    }
}

async fn start_container(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let d = match require_docker(&state) {
        Ok(d) => d,
        Err(e) => return e.into_response(),
    };
    match docker::start_container(d, &id).await {
        Ok(_) => (StatusCode::OK, Json(ApiResponse::ok("容器已启动"))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(&e.to_string())),
        )
            .into_response(),
    }
}

async fn stop_container(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let d = match require_docker(&state) {
        Ok(d) => d,
        Err(e) => return e.into_response(),
    };
    match docker::stop_container(d, &id).await {
        Ok(_) => (StatusCode::OK, Json(ApiResponse::ok("容器已停止"))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(&e.to_string())),
        )
            .into_response(),
    }
}

async fn restart_container(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let d = match require_docker(&state) {
        Ok(d) => d,
        Err(e) => return e.into_response(),
    };
    match docker::restart_container(d, &id).await {
        Ok(_) => (StatusCode::OK, Json(ApiResponse::ok("容器已重启"))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(&e.to_string())),
        )
            .into_response(),
    }
}

async fn remove_container(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let d = match require_docker(&state) {
        Ok(d) => d,
        Err(e) => return e.into_response(),
    };
    match docker::remove_container(d, &id).await {
        Ok(_) => (StatusCode::OK, Json(ApiResponse::ok("容器已删除"))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(&e.to_string())),
        )
            .into_response(),
    }
}

async fn container_logs(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let d = match require_docker(&state) {
        Ok(d) => d,
        Err(e) => return e.into_response(),
    };
    match docker::get_container_logs(d, &id, 100).await {
        Ok(logs) => (StatusCode::OK, Json(ApiResponse::ok(logs))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(&e.to_string())),
        )
            .into_response(),
    }
}

// ============ 镜像管理 API ============

async fn list_images(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let d = match require_docker(&state) {
        Ok(d) => d,
        Err(e) => return e.into_response(),
    };
    match docker::list_images(d).await {
        Ok(images) => (StatusCode::OK, Json(ApiResponse::ok(images))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(&e.to_string())),
        )
            .into_response(),
    }
}

async fn search_images(
    State(state): State<Arc<AppState>>,
    Query(query): Query<std::collections::HashMap<String, String>>,
) -> impl IntoResponse {
    let d = match require_docker(&state) {
        Ok(d) => d,
        Err(e) => return e.into_response(),
    };

    let q = query.get("q").cloned().unwrap_or_default();
    if q.is_empty() {
        return (StatusCode::BAD_REQUEST, Json(ApiResponse::<()>::error("搜索词不能为空"))).into_response();
    }
    
    let options = bollard::image::SearchImagesOptions {
        term: q,
        limit: Some(20),
        ..Default::default()
    };
    
    match d.search_images(options).await {
        Ok(results) => {
            let mut items = Vec::new();
            for item in results {
                items.push(serde_json::json!({
                    "name": item.name.unwrap_or_default(),
                    "description": item.description.unwrap_or_default(),
                    "stars": item.star_count.unwrap_or(0),
                    "is_official": item.is_official.unwrap_or(false),
                }));
            }
            (StatusCode::OK, Json(ApiResponse::ok(items))).into_response()
        }
        Err(e) => {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::<()>::error(&format!("Docker 搜索失败: {}", e)))).into_response()
        }
    }
}

async fn list_image_tags(
    Query(query): Query<std::collections::HashMap<String, String>>,
) -> impl IntoResponse {
    let image = query.get("image").cloned().unwrap_or_default();
    if image.is_empty() {
        return (StatusCode::BAD_REQUEST, Json(ApiResponse::<()>::error("请提供镜像名称"))).into_response();
    }
    let mirror = query.get("mirror").cloned().unwrap_or_default();
    
    // 构建 Registry V2 tags 查询 URL
    let repo = if image.contains('/') { image.clone() } else { format!("library/{}", image) };
    let base = if mirror.is_empty() { "https://registry.hub.docker.com".to_string() } else { format!("https://{}", mirror) };
    let url = format!("{}/v2/{}/tags/list", base, repo);
    
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(8))
        .build()
        .unwrap_or_default();
    
    match client.get(&url).send().await {
        Ok(res) if res.status().is_success() => {
            if let Ok(json) = res.json::<serde_json::Value>().await {
                if let Some(tags) = json.get("tags").and_then(|t| t.as_array()) {
                    let mut tag_list: Vec<String> = tags.iter()
                        .filter_map(|t| t.as_str().map(|s| s.to_string()))
                        .collect();
                    // latest 置顶，其余倒序（新版本在前）
                    tag_list.sort_by(|a, b| {
                        if a == "latest" { return std::cmp::Ordering::Less; }
                        if b == "latest" { return std::cmp::Ordering::Greater; }
                        b.cmp(a)
                    });
                    tag_list.truncate(50);
                    return (StatusCode::OK, Json(ApiResponse::ok(tag_list))).into_response();
                }
            }
            (StatusCode::OK, Json(ApiResponse::ok(Vec::<String>::new()))).into_response()
        }
        _ => {
            let fallback = vec!["latest".to_string()];
            (StatusCode::OK, Json(ApiResponse::ok(fallback))).into_response()
        }
    }
}

async fn pull_image(
    State(state): State<Arc<AppState>>,
    Json(req): Json<PullImageRequest>,
) -> impl IntoResponse {
    let d = match require_docker(&state) {
        Ok(d) => d,
        Err(e) => return e.into_response(),
    };
    let tag = req.tag.as_deref().unwrap_or("latest");
    match docker::pull_image(d, &req.image, tag).await {
        Ok(_) => (StatusCode::OK, Json(ApiResponse::ok("镜像拉取完成"))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(&e.to_string())),
        )
            .into_response(),
    }
}

async fn remove_image(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let d = match require_docker(&state) {
        Ok(d) => d,
        Err(e) => return e.into_response(),
    };
    match docker::remove_image(d, &id).await {
        Ok(_) => (StatusCode::OK, Json(ApiResponse::ok("镜像已删除"))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(&e.to_string())),
        )
            .into_response(),
    }
}

// ============ 网络和存储卷 API ============

async fn list_networks(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let d = match require_docker(&state) {
        Ok(d) => d,
        Err(e) => return e.into_response(),
    };
    match docker::list_networks(d).await {
        Ok(networks) => (StatusCode::OK, Json(ApiResponse::ok(networks))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(&e.to_string())),
        )
            .into_response(),
    }
}

async fn list_volumes(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let d = match require_docker(&state) {
        Ok(d) => d,
        Err(e) => return e.into_response(),
    };
    match docker::list_volumes(d).await {
        Ok(volumes) => (StatusCode::OK, Json(ApiResponse::ok(volumes))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(&e.to_string())),
        )
            .into_response(),
    }
}

// ============ 应用商店 API ============

async fn list_apps() -> impl IntoResponse {
    let apps = app_store::get_app_templates();
    (StatusCode::OK, Json(ApiResponse::ok(apps))).into_response()
}

async fn install_app(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(req): Json<InstallAppRequest>,
) -> impl IntoResponse {
    let d = match require_docker(&state) {
        Ok(d) => d,
        Err(e) => return e.into_response(),
    };

    let templates = app_store::get_app_templates();
    let template = templates.iter().find(|t| t.id == id);

    match template {
        Some(tmpl) => {
            let env: Vec<String> = req
                .env_vars
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect();

            let volumes: Vec<String> = tmpl
                .volumes
                .iter()
                .map(|v| format!("/opt/dockpanel/{}/{}", req.name, v.container_path.trim_start_matches('/')) + ":" + &v.container_path)
                .collect();

            let create_req = CreateContainerRequest {
                name: req.name,
                image: tmpl.image.clone(),
                ports: Some(vec![PortMappingRequest {
                    host: req.port,
                    container: tmpl.default_port,
                    protocol: Some("tcp".into()),
                }]),
                env: Some(env),
                volumes: Some(volumes),
                restart_policy: Some("unless-stopped".into()),
            };

            match docker::create_container(d, &create_req).await {
                Ok(id) => (StatusCode::CREATED, Json(ApiResponse::ok(id))).into_response(),
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::<()>::error(&format!("安装失败: {}", e))),
                )
                    .into_response(),
            }
        }
        None => (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<()>::error("应用模板不存在")),
        )
            .into_response(),
    }
}

// ============ 项目部署 API ============

async fn list_projects(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let projects = state.project_mgr.projects.read().await;
    (StatusCode::OK, Json(ApiResponse::ok(projects.clone()))).into_response()
}

async fn deploy_project(
    State(state): State<Arc<AppState>>,
    Json(req): Json<DeployProjectRequest>,
) -> impl IntoResponse {
    let d = match require_docker(&state) {
        Ok(d) => d,
        Err(e) => return e.into_response(),
    };

    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let container_port = req.container_port.unwrap_or(req.port);

    // 构建环境变量
    let env_vars: Option<Vec<String>> = req.env_vars.as_ref().map(|vars| {
        vars.iter().map(|(k, v)| format!("{}={}", k, v)).collect()
    });

    match req.source {
        DeploySource::Image => {
            // ===== 自定义镜像模式 =====
            let image = match &req.image {
                Some(img) => img.clone(),
                None => return (
                    StatusCode::BAD_REQUEST,
                    Json(ApiResponse::<()>::error("请提供 Docker 镜像名")),
                ).into_response(),
            };

            // 拉取镜像
            if let Err(e) = docker::pull_image(d, &image, "latest").await {
                tracing::warn!("镜像拉取失败(可能本地已有): {}", e);
            }

            // 创建并启动容器
            match projects::deploy_container(d, &image, &req.name, req.port, container_port, env_vars).await {
                Ok(container_id) => {
                    let project = ProjectInfo {
                        name: req.name.clone(),
                        source: DeploySource::Image,
                        repo_url: None,
                        image,
                        status: "running".into(),
                        container_id: Some(container_id),
                        port: req.port,
                        created_at: now,
                        message: Some("部署成功".into()),
                    };
                    state.project_mgr.projects.write().await.push(project.clone());
                    (StatusCode::CREATED, Json(ApiResponse::ok(project))).into_response()
                }
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::<()>::error(&format!("部署失败: {}", e))),
                ).into_response(),
            }
        }
        DeploySource::Github => {
            // ===== GitHub 仓库模式 =====
            let repo_url = match &req.repo_url {
                Some(url) => url.clone(),
                None => return (
                    StatusCode::BAD_REQUEST,
                    Json(ApiResponse::<()>::error("请提供 GitHub 仓库地址")),
                ).into_response(),
            };

            let branch = req.branch.clone().unwrap_or_else(|| "main".into());
            let clone_dir = format!("/tmp/dockpanel-build/{}", req.name);
            let image_tag = format!("dockpanel-{}", req.name);

            // 先添加一个 building 状态的记录
            {
                let project = ProjectInfo {
                    name: req.name.clone(),
                    source: DeploySource::Github,
                    repo_url: Some(repo_url.clone()),
                    image: image_tag.clone(),
                    status: "building".into(),
                    container_id: None,
                    port: req.port,
                    created_at: now.clone(),
                    message: Some("正在克隆仓库并构建...".into()),
                };
                state.project_mgr.projects.write().await.push(project);
            }

            // Step 1: 克隆仓库
            let _ = std::fs::remove_dir_all(&clone_dir);
            if let Err(e) = projects::clone_repo(&repo_url, &branch, &clone_dir, req.token.as_deref()).await {
                update_project_status(&state, &req.name, "error", None, &format!("克隆失败: {}", e)).await;
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::<()>::error(&format!("克隆失败: {}", e))),
                ).into_response();
            }

            // Step 2: 检测语言并生成 Dockerfile
            let detect = projects::detect_and_generate_dockerfile(&clone_dir);
            tracing::info!("🔍 检测到项目类型: {} (已有 Dockerfile: {})", detect.language, detect.has_dockerfile);

            // 如果用户提供了自定义 Dockerfile，使用用户的；否则用自动生成的
            let dockerfile_content = req.dockerfile.as_deref().unwrap_or(&detect.suggested_dockerfile);

            // Step 3: 构建镜像
            match projects::build_image_from_dir(d, &clone_dir, &image_tag, Some(dockerfile_content)).await {
                Ok(log) => {
                    // Step 4: 启动容器
                    match projects::deploy_container(d, &image_tag, &req.name, req.port, container_port, env_vars).await {
                        Ok(container_id) => {
                            update_project_status(&state, &req.name, "running", Some(&container_id), &format!("部署成功 ({})", detect.language)).await;

                            let project = ProjectInfo {
                                name: req.name.clone(),
                                source: DeploySource::Github,
                                repo_url: Some(repo_url),
                                image: image_tag,
                                status: "running".into(),
                                container_id: Some(container_id),
                                port: req.port,
                                created_at: now,
                                message: Some(format!("部署成功 - 检测类型: {}", detect.language)),
                            };
                            (StatusCode::CREATED, Json(ApiResponse::ok(project))).into_response()
                        }
                        Err(e) => {
                            update_project_status(&state, &req.name, "error", None, &format!("容器启动失败: {}", e)).await;
                            (
                                StatusCode::INTERNAL_SERVER_ERROR,
                                Json(ApiResponse::<()>::error(&format!("容器启动失败: {}", e))),
                            ).into_response()
                        }
                    }
                }
                Err(e) => {
                    update_project_status(&state, &req.name, "error", None, &format!("构建失败: {}", e)).await;
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ApiResponse::<()>::error(&format!("构建失败: {}", e))),
                    ).into_response()
                }
            }
        }
    }
}

async fn delete_project(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    let d = match require_docker(&state) {
        Ok(d) => d,
        Err(e) => return e.into_response(),
    };

    // 删除容器
    let _ = docker::remove_container(d, &name).await;

    // 从列表中移除
    let mut projects = state.project_mgr.projects.write().await;
    projects.retain(|p| p.name != name);

    // 清理构建目录
    let _ = std::fs::remove_dir_all(format!("/tmp/dockpanel-build/{}", name));

    (StatusCode::OK, Json(ApiResponse::ok("项目已删除"))).into_response()
}

async fn redeploy_project(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    let project = {
        let projects = state.project_mgr.projects.read().await;
        projects.iter().find(|p| p.name == name).cloned()
    };

    match project {
        Some(p) => {
            // 构建重新部署请求
            let req = DeployProjectRequest {
                name: p.name,
                source: p.source,
                repo_url: p.repo_url,
                image: Some(p.image),
                branch: None,
                port: p.port,
                container_port: None,
                env_vars: None,
                dockerfile: None,
                token: None,
            };
            deploy_project(State(state), Json(req)).await.into_response()
        }
        None => (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<()>::error("项目不存在")),
        ).into_response(),
    }
}

/// 辅助函数：更新内存中项目的状态
async fn update_project_status(state: &AppState, name: &str, status: &str, container_id: Option<&str>, msg: &str) {
    let mut projects = state.project_mgr.projects.write().await;
    if let Some(p) = projects.iter_mut().find(|p| p.name == name) {
        p.status = status.into();
        if let Some(cid) = container_id {
            p.container_id = Some(cid.into());
        }
        p.message = Some(msg.into());
    }
}

