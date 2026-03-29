#!/bin/bash
set -e

# ============================================================
#  RecchDockerPanel 一键安装脚本
#  用法: curl -fsSL https://raw.githubusercontent.com/happydigua/recchdockerpanel/main/install.sh | bash
# ============================================================

REPO="happydigua/recchdockerpanel"
INSTALL_DIR="/usr/local/bin"
DATA_DIR="/opt/dockpanel"
SERVICE_NAME="dockpanel"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

info()  { echo -e "${GREEN}[INFO]${NC} $1"; }
warn()  { echo -e "${YELLOW}[WARN]${NC} $1"; }
error() { echo -e "${RED}[ERROR]${NC} $1"; exit 1; }

# ---- 检测架构 ----
ARCH=$(uname -m)
case "$ARCH" in
    x86_64|amd64) ASSET="dockpanel-linux-amd64" ;;
    *) error "暂不支持的架构: $ARCH（目前仅支持 x86_64）" ;;
esac

# ---- 检测和安装 Docker ----
if command -v docker &>/dev/null; then
    info "Docker 已安装: $(docker --version)"
else
    warn "未检测到 Docker，正在自动安装..."
    if command -v curl &>/dev/null; then
        curl -fsSL https://get.docker.com | sh -s -- --mirror Aliyun
    elif command -v wget &>/dev/null; then
        wget -qO- https://get.docker.com | sh -s -- --mirror Aliyun
    else
        error "请先安装 curl 或 wget"
    fi
    systemctl enable docker
    systemctl start docker
    info "Docker 安装完成"
fi

# ---- 获取最新版本 ----
info "正在获取最新版本..."
LATEST=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name"' | sed -E 's/.*"tag_name": *"([^"]+)".*/\1/')
if [ -z "$LATEST" ]; then
    error "无法获取最新版本号，请检查网络连接"
fi
info "最新版本: ${LATEST}"

# ---- 下载二进制 ----
DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${LATEST}/${ASSET}"
info "正在下载 ${DOWNLOAD_URL} ..."
curl -fsSL -o /tmp/dockpanel "$DOWNLOAD_URL" || error "下载失败，请检查网络"
chmod +x /tmp/dockpanel
mv /tmp/dockpanel "${INSTALL_DIR}/dockpanel"
info "二进制已安装到 ${INSTALL_DIR}/dockpanel"

# ---- 创建数据目录 ----
mkdir -p "$DATA_DIR"

# ---- 生成随机密码和路径 ----
RANDOM_PATH=$(tr -dc 'a-zA-Z0-9' < /dev/urandom | head -c 8)
PORT=${DOCKPANEL_PORT:-3001}

# ---- 创建 systemd 服务 ----
cat > /etc/systemd/system/${SERVICE_NAME}.service <<EOF
[Unit]
Description=RecchDockerPanel - Docker Management Panel
After=network.target docker.service
Requires=docker.service

[Service]
Type=simple
WorkingDirectory=${DATA_DIR}
ExecStart=${INSTALL_DIR}/dockpanel
Environment=DOCKPANEL_PATH=${RANDOM_PATH}
Environment=DOCKPANEL_PORT=${PORT}
Environment=DOCKPANEL_SECRET=$(tr -dc 'A-Za-z0-9' < /dev/urandom | head -c 32)
Environment=RUST_LOG=dockpanel=info
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
EOF

systemctl daemon-reload
systemctl enable ${SERVICE_NAME}
systemctl start ${SERVICE_NAME}

# ---- 等待服务启动 ----
sleep 2

# ---- 获取服务器 IP ----
SERVER_IP=$(curl -fsSL -4 https://ifconfig.me 2>/dev/null || hostname -I | awk '{print $1}')

# ---- 输出结果 ----
echo ""
echo -e "${CYAN}╔══════════════════════════════════════════════════════╗${NC}"
echo -e "${CYAN}║${NC}  ${GREEN}✅ RecchDockerPanel 安装完成！${NC}                              ${CYAN}║${NC}"
echo -e "${CYAN}╠══════════════════════════════════════════════════════╣${NC}"
echo -e "${CYAN}║${NC}                                                      ${CYAN}║${NC}"
echo -e "${CYAN}║${NC}  访问地址: ${YELLOW}http://${SERVER_IP}:${PORT}/${RANDOM_PATH}/${NC}"
echo -e "${CYAN}║${NC}  用户名:   ${GREEN}admin${NC}"
echo -e "${CYAN}║${NC}  密码:     ${GREEN}admin123${NC}"
echo -e "${CYAN}║${NC}                                                      ${CYAN}║${NC}"
echo -e "${CYAN}║${NC}  ${RED}⚠️  请登录后立即修改默认密码！${NC}                      ${CYAN}║${NC}"
echo -e "${CYAN}║${NC}                                                      ${CYAN}║${NC}"
echo -e "${CYAN}║${NC}  管理命令:                                            ${CYAN}║${NC}"
echo -e "${CYAN}║${NC}    systemctl status dockpanel   # 查看状态            ${CYAN}║${NC}"
echo -e "${CYAN}║${NC}    systemctl restart dockpanel  # 重启服务            ${CYAN}║${NC}"
echo -e "${CYAN}║${NC}    journalctl -u dockpanel -f   # 查看日志            ${CYAN}║${NC}"
echo -e "${CYAN}╚══════════════════════════════════════════════════════╝${NC}"
echo ""
