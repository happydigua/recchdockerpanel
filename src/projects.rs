use anyhow::Result;
use bollard::Docker;
use bollard::image::BuildImageOptions;
use bollard::container::{CreateContainerOptions, Config, StartContainerOptions};
use bollard::models::{HostConfig, PortBinding as DockerPortBinding, RestartPolicy, RestartPolicyNameEnum};
use futures_util::StreamExt;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::models::*;

/// 项目管理器（内存存储，简单直接）
pub struct ProjectManager {
    pub projects: Arc<RwLock<Vec<ProjectInfo>>>,
}

impl ProjectManager {
    pub fn new() -> Self {
        Self {
            projects: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

// ============ Dockerfile 自动生成器 ============

/// 检测仓库类型并生成对应 Dockerfile
pub fn detect_and_generate_dockerfile(repo_path: &str) -> DetectResult {
    let path = Path::new(repo_path);

    // 1. 已有 Dockerfile
    if path.join("Dockerfile").exists() {
        let content = std::fs::read_to_string(path.join("Dockerfile")).unwrap_or_default();
        return DetectResult {
            language: detect_language_from_dockerfile(&content),
            has_dockerfile: true,
            suggested_dockerfile: content,
        };
    }

    // 2. Go 项目（检测 go.mod）
    if path.join("go.mod").exists() {
        let module_name = extract_go_module(path);
        return DetectResult {
            language: "go".into(),
            has_dockerfile: false,
            suggested_dockerfile: generate_go_dockerfile(&module_name),
        };
    }

    // 3. Node.js 项目（检测 package.json）
    if path.join("package.json").exists() {
        let has_next = path.join("next.config.js").exists() || path.join("next.config.mjs").exists();
        return DetectResult {
            language: "node".into(),
            has_dockerfile: false,
            suggested_dockerfile: if has_next {
                generate_nextjs_dockerfile()
            } else {
                generate_node_dockerfile()
            },
        };
    }

    // 4. Python 项目（检测 requirements.txt 或 pyproject.toml）
    if path.join("requirements.txt").exists() || path.join("pyproject.toml").exists() {
        return DetectResult {
            language: "python".into(),
            has_dockerfile: false,
            suggested_dockerfile: generate_python_dockerfile(),
        };
    }

    // 5. Rust 项目（检测 Cargo.toml）
    if path.join("Cargo.toml").exists() {
        return DetectResult {
            language: "rust".into(),
            has_dockerfile: false,
            suggested_dockerfile: generate_rust_dockerfile(),
        };
    }

    // 6. 静态文件（检测 index.html）
    if path.join("index.html").exists() || path.join("dist").exists() || path.join("public").exists() {
        return DetectResult {
            language: "static".into(),
            has_dockerfile: false,
            suggested_dockerfile: generate_static_dockerfile(),
        };
    }

    // 未识别
    DetectResult {
        language: "unknown".into(),
        has_dockerfile: false,
        suggested_dockerfile: generate_generic_dockerfile(),
    }
}

fn detect_language_from_dockerfile(content: &str) -> String {
    let lower = content.to_lowercase();
    if lower.contains("golang") || lower.contains("go build") { return "go".into(); }
    if lower.contains("node") || lower.contains("npm") { return "node".into(); }
    if lower.contains("python") || lower.contains("pip") { return "python".into(); }
    if lower.contains("cargo") || lower.contains("rustc") { return "rust".into(); }
    "unknown".into()
}

fn extract_go_module(path: &Path) -> String {
    if let Ok(content) = std::fs::read_to_string(path.join("go.mod")) {
        for line in content.lines() {
            if line.starts_with("module ") {
                return line.trim_start_matches("module ").trim().to_string();
            }
        }
    }
    "app".into()
}

fn generate_go_dockerfile(module_name: &str) -> String {
    let binary = module_name.rsplit('/').next().unwrap_or("app");
    format!(r#"# 自动生成的 Go Dockerfile
FROM golang:1.22-alpine AS builder
WORKDIR /app
COPY go.mod go.sum ./
RUN go mod download
COPY . .
RUN CGO_ENABLED=0 go build -ldflags="-s -w" -o /app/{binary} .

FROM alpine:3.19
RUN apk --no-cache add ca-certificates tzdata
WORKDIR /app
COPY --from=builder /app/{binary} .
EXPOSE 8080
CMD ["./{binary}"]
"#)
}

fn generate_node_dockerfile() -> String {
    r#"# 自动生成的 Node.js Dockerfile
FROM node:20-alpine AS builder
WORKDIR /app
COPY package*.json ./
RUN npm ci --production=false
COPY . .
RUN npm run build 2>/dev/null || true

FROM node:20-alpine
WORKDIR /app
COPY package*.json ./
RUN npm ci --production
COPY . .
COPY --from=builder /app/dist ./dist 2>/dev/null || true
EXPOSE 3000
CMD ["node", "index.js"]
"#.into()
}

fn generate_nextjs_dockerfile() -> String {
    r#"# 自动生成的 Next.js Dockerfile
FROM node:20-alpine AS builder
WORKDIR /app
COPY package*.json ./
RUN npm ci
COPY . .
RUN npm run build

FROM node:20-alpine
WORKDIR /app
COPY --from=builder /app/.next ./.next
COPY --from=builder /app/node_modules ./node_modules
COPY --from=builder /app/package*.json ./
COPY --from=builder /app/public ./public
EXPOSE 3000
CMD ["npm", "start"]
"#.into()
}

fn generate_python_dockerfile() -> String {
    r#"# 自动生成的 Python Dockerfile
FROM python:3.12-slim
WORKDIR /app
COPY requirements.txt ./  2>/dev/null || true
RUN pip install --no-cache-dir -r requirements.txt 2>/dev/null || true
COPY . .
EXPOSE 8000
CMD ["python", "main.py"]
"#.into()
}

fn generate_rust_dockerfile() -> String {
    r#"# 自动生成的 Rust Dockerfile
FROM rust:1.77-alpine AS builder
RUN apk add --no-cache musl-dev
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release

FROM alpine:3.19
RUN apk --no-cache add ca-certificates
WORKDIR /app
COPY --from=builder /app/target/release/* /app/ 2>/dev/null || true
EXPOSE 8080
CMD ["./app"]
"#.into()
}

fn generate_static_dockerfile() -> String {
    r#"# 自动生成的静态文件 Dockerfile
FROM nginx:alpine
COPY . /usr/share/nginx/html/
EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]
"#.into()
}

fn generate_generic_dockerfile() -> String {
    r#"# 通用 Dockerfile - 请根据您的项目修改
FROM ubuntu:22.04
WORKDIR /app
COPY . .
EXPOSE 8080
CMD ["./app"]
"#.into()
}

// ============ 部署核心逻辑 ============

/// 从 GitHub 克隆仓库（支持私有仓库 Token 认证）
pub async fn clone_repo(url: &str, branch: &str, target_dir: &str, token: Option<&str>) -> Result<()> {
    // 确保目标目录存在
    std::fs::create_dir_all(target_dir)?;

    // 如果提供了 token，将其嵌入 HTTPS URL 进行认证
    // https://github.com/user/repo → https://{token}@github.com/user/repo
    let auth_url = match token {
        Some(t) if !t.is_empty() => {
            if url.starts_with("https://") {
                url.replacen("https://", &format!("https://{}@", t), 1)
            } else {
                url.to_string()
            }
        }
        _ => url.to_string(),
    };

    let output = tokio::process::Command::new("git")
        .args(["clone", "--depth", "1", "--branch", branch, &auth_url, target_dir])
        .output()
        .await?;

    if !output.status.success() {
        // 如果指定分支失败，尝试默认分支
        let _ = std::fs::remove_dir_all(target_dir);
        std::fs::create_dir_all(target_dir)?;
        let output2 = tokio::process::Command::new("git")
            .args(["clone", "--depth", "1", &auth_url, target_dir])
            .output()
            .await?;
        if !output2.status.success() {
            let stderr = String::from_utf8_lossy(&output2.stderr);
            // 脱敏：不要在错误信息里暴露 token
            let safe_err = stderr.replace(&auth_url, url);
            return Err(anyhow::anyhow!("Git 克隆失败: {}", safe_err));
        }
    }
    Ok(())
}

/// 使用 Docker 构建镜像（从目录）
pub async fn build_image_from_dir(
    docker: &Docker,
    build_dir: &str,
    image_tag: &str,
    dockerfile_content: Option<&str>,
) -> Result<String> {
    // 如果提供了自定义 Dockerfile 内容，把它写入构建目录
    if let Some(content) = dockerfile_content {
        std::fs::write(format!("{}/Dockerfile", build_dir), content)?;
    }

    // 创建 tar 归档
    let tar_bytes = create_tar_archive(build_dir)?;

    let build_options = BuildImageOptions {
        t: image_tag,
        rm: true,
        forcerm: true,
        ..Default::default()
    };

    let mut stream = docker.build_image(build_options, None, Some(tar_bytes.into()));
    let mut build_log = String::new();

    while let Some(result) = stream.next().await {
        match result {
            Ok(output) => {
                if let Some(stream) = output.stream {
                    build_log.push_str(&stream);
                    tracing::info!("构建: {}", stream.trim());
                }
                if let Some(error) = output.error {
                    return Err(anyhow::anyhow!("构建失败: {}", error));
                }
            }
            Err(e) => return Err(anyhow::anyhow!("构建失败: {}", e)),
        }
    }

    Ok(build_log)
}

/// 创建 tar 归档（Docker build 需要）
fn create_tar_archive(dir: &str) -> Result<Vec<u8>> {
    use std::io::Write;

    let mut tar_builder = tar::Builder::new(Vec::new());
    tar_builder.append_dir_all(".", dir)?;
    tar_builder.finish()?;
    let tar_bytes = tar_builder.into_inner()?;

    // gzip 压缩
    let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
    encoder.write_all(&tar_bytes)?;
    Ok(encoder.finish()?)
}

/// 为项目创建并启动容器
pub async fn deploy_container(
    docker: &Docker,
    image: &str,
    name: &str,
    host_port: u16,
    container_port: u16,
    env_vars: Option<Vec<String>>,
) -> Result<String> {
    // 先尝试删除同名旧容器
    let _ = docker.remove_container(name, Some(bollard::container::RemoveContainerOptions {
        force: true,
        ..Default::default()
    })).await;

    let container_port_str = format!("{}/tcp", container_port);
    let mut port_bindings: HashMap<String, Option<Vec<DockerPortBinding>>> = HashMap::new();
    let mut exposed_ports: HashMap<String, HashMap<(), ()>> = HashMap::new();

    exposed_ports.insert(container_port_str.clone(), HashMap::new());
    port_bindings.insert(
        container_port_str,
        Some(vec![DockerPortBinding {
            host_ip: Some("0.0.0.0".into()),
            host_port: Some(host_port.to_string()),
        }]),
    );

    let host_config = HostConfig {
        port_bindings: Some(port_bindings),
        restart_policy: Some(RestartPolicy {
            name: Some(RestartPolicyNameEnum::UNLESS_STOPPED),
            maximum_retry_count: None,
        }),
        ..Default::default()
    };

    let config = Config {
        image: Some(image.to_string()),
        env: env_vars,
        exposed_ports: Some(exposed_ports),
        host_config: Some(host_config),
        ..Default::default()
    };

    let options = CreateContainerOptions {
        name,
        platform: None,
    };

    let response = docker.create_container(Some(options), config).await?;
    docker.start_container(&response.id, None::<StartContainerOptions<String>>).await?;

    Ok(response.id)
}
