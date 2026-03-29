use crate::models::*;

/// 获取内置应用模板列表
pub fn get_app_templates() -> Vec<AppTemplate> {
    vec![
        AppTemplate {
            id: "mysql".into(),
            name: "MySQL".into(),
            description: "最流行的开源关系型数据库".into(),
            icon: "🗄️".into(),
            category: "数据库".into(),
            image: "mysql:8".into(),
            default_port: 3306,
            env_vars: vec![
                EnvVarTemplate {
                    key: "MYSQL_ROOT_PASSWORD".into(),
                    label: "Root 密码".into(),
                    default: "".into(),
                    required: true,
                    is_password: true,
                },
                EnvVarTemplate {
                    key: "MYSQL_DATABASE".into(),
                    label: "默认数据库名".into(),
                    default: "mydb".into(),
                    required: false,
                    is_password: false,
                },
            ],
            volumes: vec![VolumeTemplate {
                container_path: "/var/lib/mysql".into(),
                description: "数据存储目录".into(),
            }],
        },
        AppTemplate {
            id: "postgres".into(),
            name: "PostgreSQL".into(),
            description: "最先进的开源关系型数据库".into(),
            icon: "🐘".into(),
            category: "数据库".into(),
            image: "postgres:16".into(),
            default_port: 5432,
            env_vars: vec![
                EnvVarTemplate {
                    key: "POSTGRES_PASSWORD".into(),
                    label: "管理员密码".into(),
                    default: "".into(),
                    required: true,
                    is_password: true,
                },
                EnvVarTemplate {
                    key: "POSTGRES_DB".into(),
                    label: "默认数据库名".into(),
                    default: "mydb".into(),
                    required: false,
                    is_password: false,
                },
            ],
            volumes: vec![VolumeTemplate {
                container_path: "/var/lib/postgresql/data".into(),
                description: "数据存储目录".into(),
            }],
        },
        AppTemplate {
            id: "redis".into(),
            name: "Redis".into(),
            description: "高性能内存键值数据库".into(),
            icon: "⚡".into(),
            category: "数据库".into(),
            image: "redis:7-alpine".into(),
            default_port: 6379,
            env_vars: vec![],
            volumes: vec![VolumeTemplate {
                container_path: "/data".into(),
                description: "数据持久化目录".into(),
            }],
        },
        AppTemplate {
            id: "nginx".into(),
            name: "Nginx".into(),
            description: "高性能 Web 服务器和反向代理".into(),
            icon: "🌐".into(),
            category: "Web 服务".into(),
            image: "nginx:alpine".into(),
            default_port: 80,
            env_vars: vec![],
            volumes: vec![
                VolumeTemplate {
                    container_path: "/usr/share/nginx/html".into(),
                    description: "网站文件目录".into(),
                },
                VolumeTemplate {
                    container_path: "/etc/nginx/conf.d".into(),
                    description: "配置文件目录".into(),
                },
            ],
        },
        AppTemplate {
            id: "wordpress".into(),
            name: "WordPress".into(),
            description: "全球最流行的建站系统".into(),
            icon: "📝".into(),
            category: "Web 应用".into(),
            image: "wordpress:latest".into(),
            default_port: 8080,
            env_vars: vec![
                EnvVarTemplate {
                    key: "WORDPRESS_DB_HOST".into(),
                    label: "数据库地址".into(),
                    default: "mysql:3306".into(),
                    required: true,
                    is_password: false,
                },
                EnvVarTemplate {
                    key: "WORDPRESS_DB_USER".into(),
                    label: "数据库用户".into(),
                    default: "root".into(),
                    required: true,
                    is_password: false,
                },
                EnvVarTemplate {
                    key: "WORDPRESS_DB_PASSWORD".into(),
                    label: "数据库密码".into(),
                    default: "".into(),
                    required: true,
                    is_password: true,
                },
                EnvVarTemplate {
                    key: "WORDPRESS_DB_NAME".into(),
                    label: "数据库名".into(),
                    default: "wordpress".into(),
                    required: true,
                    is_password: false,
                },
            ],
            volumes: vec![VolumeTemplate {
                container_path: "/var/www/html".into(),
                description: "WordPress 文件目录".into(),
            }],
        },
        AppTemplate {
            id: "mongo".into(),
            name: "MongoDB".into(),
            description: "灵活的文档型 NoSQL 数据库".into(),
            icon: "🍃".into(),
            category: "数据库".into(),
            image: "mongo:7".into(),
            default_port: 27017,
            env_vars: vec![
                EnvVarTemplate {
                    key: "MONGO_INITDB_ROOT_USERNAME".into(),
                    label: "管理员用户名".into(),
                    default: "admin".into(),
                    required: true,
                    is_password: false,
                },
                EnvVarTemplate {
                    key: "MONGO_INITDB_ROOT_PASSWORD".into(),
                    label: "管理员密码".into(),
                    default: "".into(),
                    required: true,
                    is_password: true,
                },
            ],
            volumes: vec![VolumeTemplate {
                container_path: "/data/db".into(),
                description: "数据存储目录".into(),
            }],
        },
        AppTemplate {
            id: "portainer-agent".into(),
            name: "Portainer Agent".into(),
            description: "远程 Docker 管理代理".into(),
            icon: "🐳".into(),
            category: "工具".into(),
            image: "portainer/agent:latest".into(),
            default_port: 9001,
            env_vars: vec![],
            volumes: vec![
                VolumeTemplate {
                    container_path: "/var/run/docker.sock".into(),
                    description: "Docker Socket".into(),
                },
                VolumeTemplate {
                    container_path: "/var/lib/docker/volumes".into(),
                    description: "Docker 卷目录".into(),
                },
            ],
        },
        AppTemplate {
            id: "minio".into(),
            name: "MinIO".into(),
            description: "高性能对象存储，兼容 S3 API".into(),
            icon: "📦".into(),
            category: "存储".into(),
            image: "minio/minio:latest".into(),
            default_port: 9000,
            env_vars: vec![
                EnvVarTemplate {
                    key: "MINIO_ROOT_USER".into(),
                    label: "管理员用户".into(),
                    default: "admin".into(),
                    required: true,
                    is_password: false,
                },
                EnvVarTemplate {
                    key: "MINIO_ROOT_PASSWORD".into(),
                    label: "管理员密码".into(),
                    default: "".into(),
                    required: true,
                    is_password: true,
                },
            ],
            volumes: vec![VolumeTemplate {
                container_path: "/data".into(),
                description: "数据存储目录".into(),
            }],
        },
    ]
}
