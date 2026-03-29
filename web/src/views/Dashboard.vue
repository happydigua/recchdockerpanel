<template>
  <div>
    <!-- Docker 未安装引导 -->
    <div v-if="!dockerInstalled && !loading" style="max-width: 720px; margin: 40px auto;">
      <div class="arco-card arco-shadow" style="padding: 48px; text-align: center;">
        <div style="font-size: 64px; margin-bottom: 16px;">🐳</div>
        <h2 style="font-size: 24px; font-weight: 600; color: #1d2129; margin-bottom: 8px;">未检测到 Docker</h2>
        <p style="color: #86909c; margin-bottom: 32px; font-size: 15px;">DockPanel 需要 Docker Engine 才能运行。请在服务器上安装 Docker 后刷新此页面。</p>

        <div style="text-align: left; background: #f7f8fa; border-radius: 8px; padding: 24px; margin-bottom: 24px;">
          <div style="font-size: 15px; font-weight: 600; color: #1d2129; margin-bottom: 16px;">📦 一键安装 Docker（Linux 服务器）</div>
          <div style="background: #1d2129; border-radius: 6px; padding: 16px; font-family: monospace; font-size: 13px; color: #e5e6eb; line-height: 1.8; margin-bottom: 16px; position: relative;">
            <div style="color: #86909c;"># 方式一：官方安装脚本（推荐）</div>
            <div>curl -fsSL https://get.docker.com | sh</div>
            <div style="color: #86909c; margin-top: 8px;"># 方式二：国内加速安装</div>
            <div>curl -fsSL https://get.docker.com | sh -s -- --mirror Aliyun</div>
            <div style="color: #86909c; margin-top: 8px;"># 安装完成后启动 Docker</div>
            <div>systemctl enable docker && systemctl start docker</div>
          </div>

          <div style="font-size: 15px; font-weight: 600; color: #1d2129; margin-bottom: 16px;">🍎 macOS / Windows</div>
          <p style="color: #4e5969; font-size: 14px; margin: 0;">
            请下载并安装 <a href="https://www.docker.com/products/docker-desktop/" target="_blank" style="color: #165dff; text-decoration: none; font-weight: 500;">Docker Desktop</a>，安装后启动即可。
          </p>
        </div>

        <n-button type="primary" size="large" @click="refresh" :loading="loading">
          🔄 重新检测
        </n-button>
      </div>
    </div>

    <!-- 正常仪表盘 -->
    <template v-else-if="dockerInstalled">
      <div class="page-header">
        <h2 class="page-title">工作台概览</h2>
        <n-button type="primary" size="medium" @click="refresh" :loading="loading">
          刷新数据
        </n-button>
      </div>

      <!-- 统计卡片网格 -->
      <div class="grid-4" style="display: grid; grid-template-columns: repeat(4, 1fr); gap: 20px; margin-bottom: 20px;">
        <div class="arco-card arco-shadow" style="padding: 24px;">
          <div style="color: #4e5969; font-size: 14px; margin-bottom: 12px;">运行中的容器</div>
          <div style="display: flex; align-items: baseline; gap: 8px;">
            <span style="font-size: 32px; font-weight: 600; color: #1d2129;">{{ info.containers_running }}</span>
            <span style="font-size: 14px; color: #86909c;">/ {{ info.containers_total }}</span>
          </div>
        </div>
        
        <div class="arco-card arco-shadow" style="padding: 24px;">
          <div style="color: #4e5969; font-size: 14px; margin-bottom: 12px;">镜像数量</div>
          <div style="font-size: 32px; font-weight: 600; color: #1d2129;">{{ info.images_total }}</div>
        </div>
        
        <div class="arco-card arco-shadow" style="padding: 24px;">
          <div style="color: #4e5969; font-size: 14px; margin-bottom: 12px;">逻辑处理器 (CPU)</div>
          <div style="font-size: 32px; font-weight: 600; color: #1d2129;">{{ info.cpu_count }}</div>
        </div>
        
        <div class="arco-card arco-shadow" style="padding: 24px;">
          <div style="color: #4e5969; font-size: 14px; margin-bottom: 12px;">总内存</div>
          <div style="font-size: 32px; font-weight: 600; color: #1d2129;">{{ formatBytes(info.memory_total) }}</div>
        </div>
      </div>

      <!-- 系统信息面板 -->
      <div class="arco-card arco-shadow" style="padding: 24px;">
        <div class="arco-card-title">系统信息</div>
        <n-descriptions :column="2" label-placement="left" size="large" bordered>
          <n-descriptions-item label="Docker Engine">
            {{ info.docker_version }}
          </n-descriptions-item>
          <n-descriptions-item label="操作系统">
            {{ info.os }}
          </n-descriptions-item>
          <n-descriptions-item label="系统架构">
            {{ info.arch }}
          </n-descriptions-item>
          <n-descriptions-item label="停止状态容器数">
            <n-badge :value="info.containers_stopped" type="warning" show-zero />
          </n-descriptions-item>
        </n-descriptions>
      </div>
    </template>
  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue'
import { useMessage } from 'naive-ui'
import api from '../utils/api'

const message = useMessage()
const loading = ref(false)
const dockerInstalled = ref(true)
const info = ref({
  docker_version: '-',
  containers_running: 0,
  containers_stopped: 0,
  containers_total: 0,
  images_total: 0,
  cpu_count: 0,
  memory_total: 0,
  os: '-',
  arch: '-',
})

function formatBytes(bytes) {
  if (!bytes) return '0 B'
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB']
  const i = Math.floor(Math.log(bytes) / Math.log(1024))
  return (bytes / Math.pow(1024, i)).toFixed(1) + ' ' + sizes[i]
}

async function refresh() {
  loading.value = true
  try {
    const res = await api.get('/system/info')
    if (res.success) {
      if (res.data.docker_installed === false) {
        dockerInstalled.value = false
      } else {
        dockerInstalled.value = true
        info.value = res.data
      }
    }
  } catch (err) {
    dockerInstalled.value = false
  } finally {
    loading.value = false
  }
}

onMounted(refresh)
</script>

<style scoped>
.n-descriptions {
  --n-border-color: #e5e6eb !important;
  --n-th-color: #f7f8fa !important;
  --n-td-color: #ffffff !important;
  --n-th-text-color: #4e5969 !important;
  --n-td-text-color: #1d2129 !important;
}
</style>
