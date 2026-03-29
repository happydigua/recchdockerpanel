use anyhow::Result;
use bollard::container::{
    CreateContainerOptions, ListContainersOptions, LogsOptions, RemoveContainerOptions,
    StartContainerOptions, StopContainerOptions,
};
use bollard::image::{CreateImageOptions, ListImagesOptions, RemoveImageOptions};
use bollard::network::ListNetworksOptions;
use bollard::volume::ListVolumesOptions;
use bollard::Docker;
use futures_util::StreamExt;
use std::collections::HashMap;

use crate::models::*;

/// 获取系统信息
pub async fn get_system_info(docker: &Docker) -> Result<SystemInfo> {
    let info = docker.info().await?;
    let version = docker.version().await?;

    Ok(SystemInfo {
        docker_version: version.version.unwrap_or_default(),
        containers_running: info.containers_running.unwrap_or(0) as i64,
        containers_stopped: info.containers_stopped.unwrap_or(0) as i64,
        containers_total: info.containers.unwrap_or(0) as i64,
        images_total: info.images.unwrap_or(0) as i64,
        cpu_count: info.ncpu.unwrap_or(0) as i64,
        memory_total: info.mem_total.unwrap_or(0),
        os: info.operating_system.unwrap_or_default(),
        arch: info.architecture.unwrap_or_default(),
    })
}

/// 列出所有容器
pub async fn list_containers(docker: &Docker) -> Result<Vec<ContainerInfo>> {
    let options = Some(ListContainersOptions::<String> {
        all: true,
        ..Default::default()
    });

    let containers = docker.list_containers(options).await?;

    let result: Vec<ContainerInfo> = containers
        .into_iter()
        .map(|c| {
            let ports = c
                .ports
                .unwrap_or_default()
                .into_iter()
                .map(|p| PortBinding {
                    container_port: p.private_port,
                    host_port: p.public_port,
                    protocol: p.typ.map(|t| format!("{:?}", t)).unwrap_or_else(|| "tcp".into()),
                })
                .collect();

            let names = c.names.unwrap_or_default();
            let name = names
                .first()
                .map(|n| n.trim_start_matches('/').to_string())
                .unwrap_or_default();

            ContainerInfo {
                id: c.id.unwrap_or_default(),
                name,
                image: c.image.unwrap_or_default(),
                status: c.status.unwrap_or_default(),
                state: c.state.unwrap_or_default(),
                created: c.created.unwrap_or(0),
                ports,
            }
        })
        .collect();

    Ok(result)
}

/// 创建并启动容器
pub async fn create_container(
    docker: &Docker,
    req: &CreateContainerRequest,
) -> Result<String> {
    use bollard::models::{HostConfig, PortBinding as DockerPortBinding, RestartPolicy, RestartPolicyNameEnum};
    use bollard::container::Config;

    let mut port_bindings: HashMap<String, Option<Vec<DockerPortBinding>>> = HashMap::new();
    let mut exposed_ports: HashMap<String, HashMap<(), ()>> = HashMap::new();

    if let Some(ports) = &req.ports {
        for p in ports {
            let proto = p.protocol.as_deref().unwrap_or("tcp");
            let container_port = format!("{}/{}", p.container, proto);
            exposed_ports.insert(container_port.clone(), HashMap::new());
            port_bindings.insert(
                container_port,
                Some(vec![DockerPortBinding {
                    host_ip: Some("0.0.0.0".to_string()),
                    host_port: Some(p.host.to_string()),
                }]),
            );
        }
    }

    let binds = req.volumes.clone();

    let restart_policy = req.restart_policy.as_deref().map(|p| {
        let name = match p {
            "always" => Some(RestartPolicyNameEnum::ALWAYS),
            "unless-stopped" => Some(RestartPolicyNameEnum::UNLESS_STOPPED),
            "on-failure" => Some(RestartPolicyNameEnum::ON_FAILURE),
            _ => Some(RestartPolicyNameEnum::NO),
        };
        RestartPolicy {
            name,
            maximum_retry_count: None,
        }
    });

    let host_config = HostConfig {
        port_bindings: Some(port_bindings),
        binds,
        restart_policy,
        ..Default::default()
    };

    let config = Config {
        image: Some(req.image.clone()),
        env: req.env.clone(),
        exposed_ports: Some(exposed_ports),
        host_config: Some(host_config),
        ..Default::default()
    };

    let options = CreateContainerOptions {
        name: &req.name,
        platform: None,
    };

    let response = docker.create_container(Some(options), config).await?;

    // 自动启动容器
    docker
        .start_container(&response.id, None::<StartContainerOptions<String>>)
        .await?;

    Ok(response.id)
}

/// 启动容器
pub async fn start_container(docker: &Docker, id: &str) -> Result<()> {
    docker
        .start_container(id, None::<StartContainerOptions<String>>)
        .await?;
    Ok(())
}

/// 停止容器
pub async fn stop_container(docker: &Docker, id: &str) -> Result<()> {
    docker
        .stop_container(id, Some(StopContainerOptions { t: 10 }))
        .await?;
    Ok(())
}

/// 重启容器
pub async fn restart_container(docker: &Docker, id: &str) -> Result<()> {
    docker.restart_container(id, Some(bollard::container::RestartContainerOptions { t: 10 })).await?;
    Ok(())
}

/// 删除容器
pub async fn remove_container(docker: &Docker, id: &str) -> Result<()> {
    docker
        .remove_container(
            id,
            Some(RemoveContainerOptions {
                force: true,
                ..Default::default()
            }),
        )
        .await?;
    Ok(())
}

/// 获取容器日志
pub async fn get_container_logs(docker: &Docker, id: &str, tail: usize) -> Result<Vec<String>> {
    let options = LogsOptions::<String> {
        stdout: true,
        stderr: true,
        tail: tail.to_string(),
        ..Default::default()
    };

    let mut stream = docker.logs(id, Some(options));
    let mut logs = Vec::new();

    while let Some(log) = stream.next().await {
        match log {
            Ok(output) => logs.push(output.to_string()),
            Err(_) => break,
        }
    }

    Ok(logs)
}

/// 列出所有镜像
pub async fn list_images(docker: &Docker) -> Result<Vec<ImageInfo>> {
    let images = docker
        .list_images(Some(ListImagesOptions::<String> {
            all: false,
            ..Default::default()
        }))
        .await?;

    let result: Vec<ImageInfo> = images
        .into_iter()
        .map(|i| ImageInfo {
            id: i.id.chars().take(19).collect(),
            tags: i.repo_tags,
            size: i.size,
            created: i.created,
        })
        .collect();

    Ok(result)
}

/// 拉取镜像
pub async fn pull_image(docker: &Docker, image: &str, tag: &str) -> Result<()> {
    let options = CreateImageOptions {
        from_image: image,
        tag,
        ..Default::default()
    };

    let mut stream = docker.create_image(Some(options), None, None);

    while let Some(result) = stream.next().await {
        match result {
            Ok(info) => {
                if let Some(status) = info.status {
                    tracing::info!("拉取镜像: {}", status);
                }
            }
            Err(e) => {
                return Err(anyhow::anyhow!("拉取镜像失败: {}", e));
            }
        }
    }

    Ok(())
}

/// 删除镜像
pub async fn remove_image(docker: &Docker, id: &str) -> Result<()> {
    docker
        .remove_image(
            id,
            Some(RemoveImageOptions {
                force: true,
                ..Default::default()
            }),
            None,
        )
        .await?;
    Ok(())
}

/// 列出所有网络
pub async fn list_networks(docker: &Docker) -> Result<Vec<NetworkInfo>> {
    let networks = docker
        .list_networks(None::<ListNetworksOptions<String>>)
        .await?;

    let result: Vec<NetworkInfo> = networks
        .into_iter()
        .map(|n| NetworkInfo {
            id: n.id.unwrap_or_default(),
            name: n.name.unwrap_or_default(),
            driver: n.driver.unwrap_or_default(),
            scope: n.scope.unwrap_or_default(),
        })
        .collect();

    Ok(result)
}

/// 列出所有存储卷
pub async fn list_volumes(docker: &Docker) -> Result<Vec<VolumeInfo>> {
    let volumes = docker
        .list_volumes(None::<ListVolumesOptions<String>>)
        .await?;

    let result: Vec<VolumeInfo> = volumes
        .volumes
        .unwrap_or_default()
        .into_iter()
        .map(|v| VolumeInfo {
            name: v.name,
            driver: v.driver,
            mountpoint: v.mountpoint,
        })
        .collect();

    Ok(result)
}
