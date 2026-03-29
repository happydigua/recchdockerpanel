use serde::{Deserialize, Serialize};

/// 容器信息
#[derive(Debug, Serialize, Deserialize)]
pub struct ContainerInfo {
    pub id: String,
    pub name: String,
    pub image: String,
    pub status: String,
    pub state: String,
    pub created: i64,
    pub ports: Vec<PortBinding>,
}

/// 端口映射
#[derive(Debug, Serialize, Deserialize)]
pub struct PortBinding {
    pub container_port: u16,
    pub host_port: Option<u16>,
    pub protocol: String,
}

/// 镜像信息
#[derive(Debug, Serialize, Deserialize)]
pub struct ImageInfo {
    pub id: String,
    pub tags: Vec<String>,
    pub size: i64,
    pub created: i64,
}

/// 网络信息
#[derive(Debug, Serialize, Deserialize)]
pub struct NetworkInfo {
    pub id: String,
    pub name: String,
    pub driver: String,
    pub scope: String,
}

/// 存储卷信息
#[derive(Debug, Serialize, Deserialize)]
pub struct VolumeInfo {
    pub name: String,
    pub driver: String,
    pub mountpoint: String,
}

/// 系统信息
#[derive(Debug, Serialize, Deserialize)]
pub struct SystemInfo {
    pub docker_version: String,
    pub containers_running: i64,
    pub containers_stopped: i64,
    pub containers_total: i64,
    pub images_total: i64,
    pub cpu_count: i64,
    pub memory_total: i64,
    pub os: String,
    pub arch: String,
}

/// 创建容器请求
#[derive(Debug, Deserialize)]
pub struct CreateContainerRequest {
    pub name: String,
    pub image: String,
    pub ports: Option<Vec<PortMappingRequest>>,
    pub env: Option<Vec<String>>,
    pub volumes: Option<Vec<String>>,
    pub restart_policy: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PortMappingRequest {
    pub host: u16,
    pub container: u16,
    pub protocol: Option<String>,
}

/// 拉取镜像请求
#[derive(Debug, Deserialize)]
pub struct PullImageRequest {
    pub image: String,
    pub tag: Option<String>,
}

/// 用户信息
#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub username: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub created_at: String,
}

/// 登录请求
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

/// 登录响应
#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub username: String,
}

/// 应用模板
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub icon: String,
    pub category: String,
    pub image: String,
    pub default_port: u16,
    pub env_vars: Vec<EnvVarTemplate>,
    pub volumes: Vec<VolumeTemplate>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EnvVarTemplate {
    pub key: String,
    pub label: String,
    pub default: String,
    pub required: bool,
    pub is_password: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VolumeTemplate {
    pub container_path: String,
    pub description: String,
}

/// 安装应用请求
#[derive(Debug, Deserialize)]
pub struct InstallAppRequest {
    pub name: String,
    pub port: u16,
    pub env_vars: std::collections::HashMap<String, String>,
}

/// API 通用响应
#[derive(Debug, Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub success: bool,
    pub data: Option<T>,
    pub message: Option<String>,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn ok(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            message: None,
        }
    }

    pub fn error(msg: &str) -> Self {
        Self {
            success: false,
            data: None,
            message: Some(msg.to_string()),
        }
    }
}

// ============ 项目部署相关模型 ============

/// 部署来源类型
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum DeploySource {
    Github,   // GitHub 仓库地址
    Image,    // Docker 镜像名
}

/// 创建项目部署请求
#[derive(Debug, Deserialize)]
pub struct DeployProjectRequest {
    pub name: String,                  // 项目名（也作为容器名）
    pub source: DeploySource,          // 来源类型
    pub repo_url: Option<String>,      // GitHub 仓库地址
    pub image: Option<String>,         // Docker 镜像名
    pub branch: Option<String>,        // 分支名，默认 main
    pub port: u16,                     // 主机端口
    pub container_port: Option<u16>,   // 容器内部端口（默认同 port）
    pub env_vars: Option<std::collections::HashMap<String, String>>,
    pub dockerfile: Option<String>,    // 用户自定义 Dockerfile 内容（可选）
    pub token: Option<String>,         // GitHub Personal Access Token（私有仓库）
}

/// 项目信息（返回给前端）
#[derive(Debug, Serialize, Clone)]
pub struct ProjectInfo {
    pub name: String,
    pub source: DeploySource,
    pub repo_url: Option<String>,
    pub image: String,
    pub status: String,            // building / running / stopped / error
    pub container_id: Option<String>,
    pub port: u16,
    pub created_at: String,
    pub message: Option<String>,   // 构建日志或错误信息
}

/// 自动检测项目类型的返回
#[derive(Debug, Serialize)]
pub struct DetectResult {
    pub language: String,          // go / node / python / rust / static / unknown
    pub has_dockerfile: bool,
    pub suggested_dockerfile: String,
}
