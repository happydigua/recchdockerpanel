<template>
  <div>
    <div class="page-header">
      <h2 class="page-title">项目部署</h2>
      <n-space>
        <n-button type="primary" @click="showDeploy = true">新建部署</n-button>
        <n-button @click="refresh" :loading="loading">刷新</n-button>
      </n-space>
    </div>

    <!-- 项目列表 -->
    <div v-if="projects.length === 0 && !loading" class="arco-card arco-shadow" style="padding: 60px; text-align: center;">
      <div style="font-size: 48px; margin-bottom: 16px;">🚀</div>
      <h3 style="font-size: 18px; font-weight: 600; color: #1d2129; margin-bottom: 8px;">还没有部署任何项目</h3>
      <p style="color: #86909c; margin-bottom: 24px;">支持从 GitHub 仓库一键部署，或直接使用 Docker 镜像</p>
      <n-button type="primary" @click="showDeploy = true">创建第一个项目</n-button>
    </div>

    <div v-else style="display: grid; grid-template-columns: repeat(auto-fill, minmax(340px, 1fr)); gap: 20px;">
      <div v-for="p in projects" :key="p.name" class="arco-card arco-shadow" style="padding: 24px;">
        <div style="display: flex; justify-content: space-between; align-items: flex-start; margin-bottom: 16px;">
          <div>
            <div style="font-size: 16px; font-weight: 600; color: #1d2129;">{{ p.name }}</div>
            <div style="font-size: 13px; color: #86909c; margin-top: 4px;">{{ p.repo_url || p.image }}</div>
          </div>
          <n-tag :type="statusType(p.status)" size="small" :bordered="false">{{ statusLabel(p.status) }}</n-tag>
        </div>
        
        <div style="display: flex; gap: 24px; margin-bottom: 16px; font-size: 13px; color: #4e5969;">
          <div>端口：<span style="color: #165dff; font-family: monospace;">:{{ p.port }}</span></div>
          <div>来源：{{ p.source === 'github' ? 'GitHub' : '镜像' }}</div>
        </div>

        <div v-if="p.message" style="background: #f7f8fa; border-radius: 4px; padding: 8px 12px; font-size: 12px; color: #4e5969; margin-bottom: 16px; word-break: break-all;">
          {{ p.message }}
        </div>

        <n-space size="small">
          <n-button size="small" type="primary" secondary :loading="p.status === 'building'" @click="redeploy(p.name)">重新部署</n-button>
          <n-button size="small" secondary @click="viewLogs(p)">查看日志</n-button>
          <n-button size="small" type="error" secondary @click="deleteProject(p)">删除</n-button>
        </n-space>
      </div>
    </div>

    <!-- 新建部署弹窗 -->
    <n-modal v-model:show="showDeploy" preset="card" title="新建项目部署" style="width: 600px;">
      <n-form ref="formRef" :model="form" label-placement="left" label-width="120" style="margin-top: 10px;">
        
        <n-form-item label="部署方式">
          <n-radio-group v-model:value="form.source">
            <n-space>
              <n-radio value="github">GitHub 仓库</n-radio>
              <n-radio value="image">Docker 镜像</n-radio>
            </n-space>
          </n-radio-group>
        </n-form-item>

        <n-form-item label="项目名称">
          <n-input v-model:value="form.name" placeholder="例如: my-api (用作容器名)" />
        </n-form-item>

        <!-- GitHub 方式 -->
        <template v-if="form.source === 'github'">
          <n-form-item label="仓库地址">
            <n-input v-model:value="form.repo_url" placeholder="https://github.com/user/repo.git" />
          </n-form-item>
          <n-form-item label="访问令牌">
            <n-input
              v-model:value="form.token"
              type="password"
              show-password-on="click"
              placeholder="私有仓库需要填写 GitHub Personal Access Token（公开仓库可留空）"
            />
          </n-form-item>
          <n-form-item label="分支">
            <n-input v-model:value="form.branch" placeholder="默认: main" />
          </n-form-item>
          <n-form-item label="Dockerfile">
            <n-space vertical style="width: 100%;">
              <n-radio-group v-model:value="dockerfileMode" size="small">
                <n-radio-button value="auto">🔍 自动检测</n-radio-button>
                <n-radio-button value="custom">✏️ 自定义</n-radio-button>
              </n-radio-group>
              <n-input
                v-if="dockerfileMode === 'custom'"
                v-model:value="form.dockerfile"
                type="textarea"
                :rows="8"
                placeholder="在这里粘贴您的 Dockerfile 内容。如果留空，系统将自动检测项目类型（Go/Node/Python/Rust）并生成合适的 Dockerfile。"
                style="font-family: monospace; font-size: 13px;"
              />
              <n-alert v-else type="info" :bordered="false" style="font-size: 13px;">
                系统将自动扫描你的仓库，检测项目类型（Go / Node.js / Python / Rust / 静态站点），并自动生成最佳的 Dockerfile。无需任何配置！
              </n-alert>
            </n-space>
          </n-form-item>
        </template>

        <!-- 镜像方式 -->
        <template v-if="form.source === 'image'">
          <n-form-item label="镜像名称">
            <n-input v-model:value="form.image" placeholder="例如: nginx:latest 或 your-registry/app:v1" />
          </n-form-item>
        </template>

        <n-divider dashed style="margin: 12px 0;" />

        <n-form-item label="主机端口">
          <n-input-number v-model:value="form.port" :min="1" :max="65535" style="width: 100%;" placeholder="映射到主机的端口" />
        </n-form-item>
        <n-form-item label="容器端口">
          <n-input-number v-model:value="form.container_port" :min="1" :max="65535" style="width: 100%;" placeholder="容器内服务监听端口（默认同主机端口）" />
        </n-form-item>

        <!-- 环境变量 -->
        <n-form-item label="环境变量">
          <n-dynamic-input v-model:value="envList" :on-create="() => ({ key: '', value: '' })">
            <template #default="{ value }">
              <div style="display: flex; gap: 8px; width: 100%;">
                <n-input v-model:value="value.key" placeholder="KEY" style="width: 40%;" />
                <n-input v-model:value="value.value" placeholder="VALUE" style="width: 60%;" />
              </div>
            </template>
          </n-dynamic-input>
        </n-form-item>
      </n-form>

      <template #footer>
        <n-space justify="end">
          <n-button @click="showDeploy = false">取消</n-button>
          <n-button type="primary" :loading="deploying" @click="doDeploy">
            🚀 开始部署
          </n-button>
        </n-space>
      </template>
    </n-modal>

    <!-- 日志弹窗 -->
    <n-modal v-model:show="showLogs" preset="card" title="容器日志" style="width: 800px;">
      <div style="background: #1d2129; padding: 16px; border-radius: 4px; max-height: 60vh; overflow-y: auto; font-family: monospace; font-size: 13px; line-height: 1.5; color: #e5e6eb;">
        <div v-for="(line, i) in logs" :key="i" style="word-break: break-all; margin-bottom: 2px;">{{ line }}</div>
        <div v-if="logs.length === 0" style="color: #86909c;">暂无日志</div>
      </div>
    </n-modal>
  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue'
import { useMessage, useDialog } from 'naive-ui'
import api from '../utils/api'

const message = useMessage()
const dialog = useDialog()
const loading = ref(false)
const projects = ref([])
const showDeploy = ref(false)
const deploying = ref(false)
const showLogs = ref(false)
const logs = ref([])
const dockerfileMode = ref('auto')
const envList = ref([])

const form = ref({
  name: '',
  source: 'github',
  repo_url: '',
  image: '',
  branch: 'main',
  port: 8080,
  container_port: null,
  dockerfile: '',
  token: '',
})

function statusType(s) {
  if (s === 'running') return 'success'
  if (s === 'building') return 'info'
  if (s === 'error') return 'error'
  return 'warning'
}

function statusLabel(s) {
  const map = { running: '运行中', building: '构建中', stopped: '已停止', error: '异常' }
  return map[s] || s
}

async function refresh() {
  loading.value = true
  try {
    const res = await api.get('/projects')
    if (res.success) projects.value = res.data
  } catch (err) {
    message.error('获取项目列表失败')
  } finally {
    loading.value = false
  }
}

async function doDeploy() {
  if (!form.value.name) { message.warning('请输入项目名称'); return }
  if (form.value.source === 'github' && !form.value.repo_url) { message.warning('请输入 GitHub 仓库地址'); return }
  if (form.value.source === 'image' && !form.value.image) { message.warning('请输入镜像名称'); return }

  deploying.value = true
  try {
    // 构建环境变量 map
    const envVars = {}
    envList.value.forEach(e => {
      if (e.key) envVars[e.key] = e.value
    })

    const payload = {
      name: form.value.name,
      source: form.value.source,
      repo_url: form.value.source === 'github' ? form.value.repo_url : undefined,
      image: form.value.source === 'image' ? form.value.image : undefined,
      branch: form.value.branch || 'main',
      port: form.value.port,
      container_port: form.value.container_port || undefined,
      env_vars: Object.keys(envVars).length > 0 ? envVars : undefined,
      dockerfile: dockerfileMode.value === 'custom' && form.value.dockerfile ? form.value.dockerfile : undefined,
      token: form.value.token || undefined,
    }

    const res = await api.post('/projects', payload)
    if (res.success) {
      message.success('部署成功！')
      showDeploy.value = false
      form.value = { name: '', source: 'github', repo_url: '', image: '', branch: 'main', port: 8080, container_port: null, dockerfile: '', token: '' }
      envList.value = []
      refresh()
    } else {
      message.error(res.message || '部署失败')
    }
  } catch (err) {
    message.error('部署失败: ' + (err.response?.data?.message || err.message))
  } finally {
    deploying.value = false
  }
}

async function redeploy(name) {
  try {
    const res = await api.post(`/projects/${name}/redeploy`)
    if (res.success) {
      message.success('重新部署成功')
      refresh()
    } else {
      message.error(res.message || '重新部署失败')
    }
  } catch (err) {
    message.error('重新部署失败')
  }
}

async function viewLogs(p) {
  if (!p.container_id) {
    message.warning('该项目暂无运行中的容器')
    return
  }
  try {
    const res = await api.get(`/containers/${p.container_id}/logs`)
    if (res.success) {
      logs.value = res.data
      showLogs.value = true
    }
  } catch (err) {
    message.error('获取日志失败')
  }
}

function deleteProject(p) {
  dialog.warning({
    title: '确认删除',
    content: `确定要删除项目 "${p.name}" 吗？容器和构建文件都会被清除。`,
    positiveText: '确认删除',
    negativeText: '取消',
    onPositive: async () => {
      try {
        await api.delete(`/projects/${p.name}`)
        message.success('项目已删除')
        refresh()
      } catch (err) {
        message.error('删除失败')
      }
    }
  })
}

onMounted(refresh)
</script>
