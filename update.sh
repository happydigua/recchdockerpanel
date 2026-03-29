#!/bin/bash
set -e

# ============================================================
#  DockPanel 更新脚本
#  用法: curl -fsSL https://raw.githubusercontent.com/happydigua/recchdockerpanel/main/update.sh | bash
# ============================================================

REPO="happydigua/recchdockerpanel"
INSTALL_DIR="/usr/local/bin"

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

info()  { echo -e "${GREEN}[INFO]${NC} $1"; }
error() { echo -e "${RED}[ERROR]${NC} $1"; exit 1; }

# 检测架构
ARCH=$(uname -m)
case "$ARCH" in
    x86_64|amd64) ASSET="dockpanel-linux-amd64" ;;
    *) error "暂不支持的架构: $ARCH" ;;
esac

# 检测是否已安装
if [ -f "${INSTALL_DIR}/dockpanel" ]; then
    info "检测到已安装的 DockPanel"
else
    error "未检测到 DockPanel，请先使用 install.sh 安装"
fi

# 获取最新版本
info "正在检查最新版本..."
LATEST=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name"' | sed -E 's/.*"tag_name": *"([^"]+)".*/\1/')
if [ -z "$LATEST" ]; then
    error "无法获取最新版本号"
fi
info "最新版本: ${LATEST}"

# 下载新版本
DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${LATEST}/${ASSET}"
info "正在下载 ${LATEST} ..."
curl -fsSL -o /tmp/dockpanel "$DOWNLOAD_URL" || error "下载失败"
chmod +x /tmp/dockpanel

# 停止服务 → 替换二进制 → 重启服务
info "正在更新..."
systemctl stop dockpanel 2>/dev/null || true
mv /tmp/dockpanel "${INSTALL_DIR}/dockpanel"
systemctl start dockpanel 2>/dev/null || true

sleep 2
info "✅ DockPanel 已更新到 ${LATEST} 并重新启动！"
echo ""
echo -e "${YELLOW}提示: 访问地址和密码不变，安全路径不变。${NC}"
echo -e "查看状态: ${GREEN}systemctl status dockpanel${NC}"
