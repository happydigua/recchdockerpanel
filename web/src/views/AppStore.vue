<template>
  <div class="arco-card arco-shadow">
    <div class="page-header" style="margin-bottom: 24px;">
      <h2 class="page-title">应用商店</h2>
      <n-input
        v-model:value="search"
        placeholder="搜索应用..."
        style="width: 250px;"
        clearable
      />
    </div>

    <div class="app-grid">
      <div
        v-for="app in filteredApps"
        :key="app.id"
        class="app-card"
        @click="openInstall(app)"
      >
        <div class="app-header">
          <div class="app-icon">{{ app.icon }}</div>
          <n-tag size="small" :bordered="false" type="info">{{ app.category }}</n-tag>
        </div>
        <div class="app-body">
          <div class="app-name">{{ app.name }}</div>
          <div class="app-desc">{{ app.description }}</div>
          <div class="app-version">{{ app.image }}</div>
        </div>
      </div>
    </div>

    <!-- 安装弹窗 -->
    <n-modal v-model:show="showInstall" preset="card" :title="`安装 ${currentApp?.name || ''}`" style="width: 500px;">
      <n-form v-if="currentApp" label-placement="left" label-width="120" style="margin-top: 10px;">
        <n-form-item label="实例名称">
          <n-input v-model:value="installForm.name" placeholder="例如: my-app" />
        </n-form-item>
        <n-form-item label="主机映射端口">
          <n-input-number v-model:value="installForm.port" :min="1" :max="65535" style="width: 100%;" />
        </n-form-item>
        <n-form-item label="镜像版本">
          <n-input v-model:value="installForm.image_tag" :placeholder="getDefaultTag(currentApp)">
            <template #prefix>
              <span style="color: #86909c;">{{ getImageBase(currentApp) }}:</span>
            </template>
          </n-input>
        </n-form-item>
        <n-divider dashed style="margin: 12px 0 24px;" />
        <n-form-item v-for="env in currentApp.env_vars" :key="env.key" :label="env.label">
          <n-input
            v-model:value="installForm.env_vars[env.key]"
            :type="env.is_password ? 'password' : 'text'"
            :placeholder="env.default || `请输入`"
            show-password-on="click"
          />
        </n-form-item>
      </n-form>
      <template #footer>
        <n-space justify="end" style="margin-top: 10px;">
          <n-button @click="showInstall = false">取消</n-button>
          <n-button type="primary" :loading="installing" @click="doInstall">
            一键部署
          </n-button>
        </n-space>
      </template>
    </n-modal>
  </div>
</template>

<script setup>
import { ref, computed, onMounted } from 'vue'
import { useMessage } from 'naive-ui'
import api from '../utils/api'

const message = useMessage()
const apps = ref([])
const search = ref('')
const showInstall = ref(false)
const installing = ref(false)
const currentApp = ref(null)
const installForm = ref({ name: '', port: 0, env_vars: {}, image_tag: '' })

function getImageBase(app) {
  if (!app) return ''
  return app.image.split(':')[0]
}

function getDefaultTag(app) {
  if (!app) return 'latest'
  const parts = app.image.split(':')
  return parts.length > 1 ? parts[1] : 'latest'
}

const filteredApps = computed(() => {
  if (!search.value) return apps.value
  const q = search.value.toLowerCase()
  return apps.value.filter(a =>
    a.name.toLowerCase().includes(q) ||
    a.description.toLowerCase().includes(q) ||
    a.category.toLowerCase().includes(q)
  )
})

function openInstall(app) {
  currentApp.value = app
  const envDefaults = {}
  app.env_vars.forEach(e => {
    envDefaults[e.key] = e.default || ''
  })
  installForm.value = {
    name: app.id + '-1',
    port: app.default_port,
    env_vars: envDefaults,
    image_tag: '',
  }
  showInstall.value = true
}

async function doInstall() {
  if (!installForm.value.name) {
    message.warning('请输入实例名称')
    return
  }
  for (const env of currentApp.value.env_vars) {
    if (env.required && !installForm.value.env_vars[env.key]) {
      message.warning(`请填写必填项: ${env.label}`)
      return
    }
  }
  installing.value = true
  try {
    const payload = { ...installForm.value }
    // 如果没填版本标签则不传（使用模板默认值）
    if (!payload.image_tag) delete payload.image_tag
    const res = await api.post(`/apps/${currentApp.value.id}/install`, payload)
    if (res.success) {
      message.success(`${currentApp.value.name} 部署成功！`)
      showInstall.value = false
    } else {
      message.error(res.message || '部署失败')
    }
  } catch (err) {
    message.error('部署失败: ' + (err.response?.data?.message || err.message))
  } finally {
    installing.value = false
  }
}

async function loadApps() {
  try {
    const res = await api.get('/apps')
    if (res.success) apps.value = res.data
  } catch (err) {
    message.error('获取应用列表失败')
  }
}

onMounted(loadApps)
</script>

<style scoped>
.app-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 20px;
}

.app-card {
  border: 1px solid #e5e6eb;
  border-radius: 4px;
  padding: 20px;
  cursor: pointer;
  transition: all 0.2s;
  background: #fff;
  display: flex;
  flex-direction: column;
}

.app-card:hover {
  border-color: #165dff;
  box-shadow: 0 4px 10px rgba(22, 93, 255, 0.1);
  transform: translateY(-2px);
}

.app-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 16px;
}

.app-icon {
  font-size: 36px;
  line-height: 1;
}

.app-name {
  font-size: 16px;
  font-weight: 500;
  color: #1d2129;
  margin-bottom: 8px;
}

.app-desc {
  font-size: 13px;
  color: #86909c;
  line-height: 1.5;
  flex: 1;
}

.app-version {
  margin-top: 8px;
  font-size: 12px;
  color: #165dff;
  background: #e8f3ff;
  display: inline-block;
  padding: 2px 8px;
  border-radius: 3px;
  font-family: monospace;
}
</style>
