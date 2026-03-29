<template>
  <div class="arco-card arco-shadow">
    <div class="page-header" style="margin-bottom: 20px;">
      <h2 class="page-title">容器管理</h2>
      <n-space>
        <n-button type="primary" @click="refresh" :loading="loading">刷新</n-button>
      </n-space>
    </div>

    <n-data-table
      :columns="columns"
      :data="containers"
      :loading="loading"
      :row-key="row => row.id"
      :bordered="true"
    />

    <!-- 日志弹窗 -->
    <n-modal v-model:show="showLogs" preset="card" title="容器日志" style="width: 800px;">
      <div style="background: #1d2129; padding: 16px; border-radius: 4px; max-height: 60vh; overflow-y: auto; font-family: 'Consolas', monospace; font-size: 13px; line-height: 1.5; color: #e5e6eb;">
        <div v-for="(line, i) in logs" :key="i" style="word-break: break-all; margin-bottom: 2px;">{{ line }}</div>
        <div v-if="logs.length === 0" style="color: #86909c;">暂无日志输出</div>
      </div>
    </n-modal>
  </div>
</template>

<script setup>
import { ref, h, onMounted } from 'vue'
import { useMessage, useDialog, NButton, NSpace, NTag } from 'naive-ui'
import api from '../utils/api'

const message = useMessage()
const dialog = useDialog()
const loading = ref(false)
const containers = ref([])
const showLogs = ref(false)
const logs = ref([])

function stateType(state) {
  if (state === 'running') return 'success'
  if (state === 'exited') return 'error'
  if (state === 'paused') return 'warning'
  return 'default'
}

const columns = [
  {
    title: '容器名称',
    key: 'name',
    width: 180,
    ellipsis: { tooltip: true },
    render: (row) => h('span', { style: 'font-weight: 500; color: #1d2129;' }, row.name || '-')
  },
  {
    title: '镜像',
    key: 'image',
    width: 200,
    ellipsis: { tooltip: true },
    render: (row) => h('span', { style: 'color: #4e5969;' }, row.image)
  },
  {
    title: '状态',
    key: 'state',
    width: 100,
    render: (row) => h(NTag, { type: stateType(row.state), size: 'small', bordered: false }, () => row.state.toUpperCase())
  },
  {
    title: '运行时长',
    key: 'status',
    width: 200,
    ellipsis: { tooltip: true },
    render: (row) => h('span', { style: 'font-size: 13px; color: #86909c;' }, row.status)
  },
  {
    title: '端口映射',
    key: 'ports',
    width: 160,
    render: (row) => {
      const mapped = row.ports.filter(p => p.host_port)
      if (mapped.length === 0) return h('span', { style: 'color: #c9cdd4;' }, '-')
      return h('span', { style: 'color: #165dff; font-family: monospace; font-size: 13px;' }, mapped.map(p => `${p.host_port}→${p.container_port}`).join(', '))
    }
  },
  {
    title: '操作',
    key: 'actions',
    width: 280,
    render: (row) => {
      return h(NSpace, { size: 'small' }, () => [
        row.state !== 'running'
          ? h(NButton, { size: 'small', type: 'primary', secondary: true, onClick: () => startContainer(row.id) }, () => '启动')
          : h(NButton, { size: 'small', type: 'warning', secondary: true, onClick: () => stopContainer(row.id) }, () => '停止'),
        h(NButton, { size: 'small', type: 'default', secondary: true, onClick: () => restartContainer(row.id) }, () => '重启'),
        h(NButton, { size: 'small', type: 'default', onClick: () => viewLogs(row.id) }, () => '日志'),
        h(NButton, { size: 'small', type: 'error', secondary: true, onClick: () => removeContainer(row) }, () => '删除'),
      ])
    }
  },
]

async function refresh() {
  loading.value = true
  try {
    const res = await api.get('/containers')
    if (res.success) containers.value = res.data
  } catch (err) {
    message.error('获取容器列表失败')
  } finally {
    loading.value = false
  }
}

async function startContainer(id) {
  try {
    await api.post(`/containers/${id}/start`)
    message.success('容器已启动')
    refresh()
  } catch (err) {
    message.error('容器启动失败')
  }
}

async function stopContainer(id) {
  try {
    await api.post(`/containers/${id}/stop`)
    message.success('容器已停止')
    refresh()
  } catch (err) {
    message.error('容器停止失败')
  }
}

async function restartContainer(id) {
  try {
    await api.post(`/containers/${id}/restart`)
    message.success('容器已重启')
    refresh()
  } catch (err) {
    message.error('容器重启失败')
  }
}

async function viewLogs(id) {
  try {
    const res = await api.get(`/containers/${id}/logs`)
    if (res.success) {
      logs.value = res.data
      showLogs.value = true
    }
  } catch (err) {
    message.error('获取日志失败')
  }
}

function removeContainer(row) {
  dialog.warning({
    title: '确认删除',
    content: `确定要删除容器 "${row.name}" 吗？此操作不可恢复。`,
    positiveText: '确认删除',
    negativeText: '取消',
    onPositive: async () => {
      try {
        await api.delete(`/containers/${row.id}`)
        message.success('容器已删除')
        refresh()
      } catch (err) {
        message.error('删除容器失败')
      }
    }
  })
}

onMounted(refresh)
</script>
