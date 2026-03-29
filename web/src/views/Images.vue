<template>
  <div class="arco-card arco-shadow">
    <div class="page-header" style="margin-bottom: 20px;">
      <h2 class="page-title">镜像列表</h2>
      <n-space>
        <n-button type="primary" @click="showPull = true">拉取镜像</n-button>
        <n-button @click="refresh" :loading="loading">刷新</n-button>
      </n-space>
    </div>

    <n-data-table
      :columns="columns"
      :data="images"
      :loading="loading"
      :row-key="row => row.id"
      :bordered="true"
    />

    <!-- 拉取镜像弹窗 -->
    <n-modal v-model:show="showPull" preset="card" title="拉取新镜像" style="width: 480px;">
      <n-form style="margin-top: 10px;">
        <n-form-item label="加速源">
          <n-select v-model:value="pullForm.mirror" :options="mirrorOptions" />
        </n-form-item>
        <n-form-item label="镜像名称">
          <n-select
            v-model:value="pullForm.image"
            filterable
            remote
            placeholder="输入名称搜索 Docker Hub... (例如: nginx, mysql)"
            :options="searchOptions"
            :loading="searching"
            @search="handleSearch"
            :render-label="renderLabel"
            clearable
          />
        </n-form-item>
        <n-form-item label="版本标签">
          <n-select
            v-model:value="pullForm.tag"
            filterable
            tag
            placeholder="选择版本（先选上方镜像名自动加载，也可手动输入）"
            :options="tagOptions"
            :loading="loadingTags"
            clearable
          />
        </n-form-item>
        <div v-if="pullForm.mirror" style="background: #f7f8fa; border-radius: 4px; padding: 8px 12px; font-size: 12px; color: #4e5969; margin-bottom: 12px; font-family: monospace;">
          实际拉取：{{ computedImageName }}
        </div>
        <n-button type="primary" block size="large" :loading="pulling" @click="pullImage" style="margin-top: 10px;">
          开始拉取
        </n-button>
      </n-form>
    </n-modal>
  </div>
</template>

<script setup>
import { ref, h, computed, watch, onMounted } from 'vue'
import { useMessage, useDialog, NButton, NTag } from 'naive-ui'
import api from '../utils/api'

const message = useMessage()
const dialog = useDialog()
const loading = ref(false)
const images = ref([])
const showPull = ref(false)
const pulling = ref(false)
const searching = ref(false)
const searchOptions = ref([])
const tagOptions = ref([])
const loadingTags = ref(false)
let searchTimer = null

const mirrorOptions = [
  { label: '🇨🇳 腾讯云加速', value: 'mirror.ccs.tencentyun.com' },
  { label: '🇨🇳 网易云加速', value: 'hub-mirror.c.163.com' },
  { label: '🇨🇳 百度云加速', value: 'mirror.baidubce.com' },
  { label: '🇨🇳 Docker 官方中国', value: 'registry.docker-cn.com' },
  { label: '🇳🇱 官方 Docker Hub', value: '' },
]

const pullForm = ref({ image: '', tag: '', mirror: 'mirror.ccs.tencentyun.com' })

const computedImageName = computed(() => {
  const img = pullForm.value.image || 'nginx'
  const tag = pullForm.value.tag || 'latest'
  const mirror = pullForm.value.mirror
  if (mirror) {
    // 官方镜像加 library/ 前缀，第三方镜像保持原样
    const name = img.includes('/') ? img : `library/${img}`
    return `${mirror}/${name}:${tag}`
  }
  return `${img}:${tag}`
})

// 选中镜像后自动加载可用版本标签
watch(() => pullForm.value.image, async (newImage) => {
  tagOptions.value = []
  pullForm.value.tag = ''
  if (!newImage) return
  loadingTags.value = true
  try {
    const mirror = pullForm.value.mirror
    const res = await api.get(`/images/tags?image=${encodeURIComponent(newImage)}&mirror=${encodeURIComponent(mirror)}`)
    if (res.success && res.data) {
      tagOptions.value = res.data.map(t => ({ label: t, value: t }))
    }
  } catch (err) {
    tagOptions.value = [{ label: 'latest', value: 'latest' }]
  } finally {
    loadingTags.value = false
  }
})

async function handleSearch(query) {
  if (!query) {
    searchOptions.value = []
    return
  }
  searching.value = true
  clearTimeout(searchTimer)
  searchTimer = setTimeout(async () => {
    try {
      const res = await api.get(`/images/search?q=${encodeURIComponent(query)}`)
      if (res.success) {
        searchOptions.value = res.data.map(item => ({
          label: item.name,
          value: item.name,
          description: item.description,
          stars: item.stars,
          is_official: item.is_official
        }))
      }
    } catch (err) {
      console.error('Search failed', err)
    } finally {
      searching.value = false
    }
  }, 500)
}

function renderLabel(option) {
  return h('div', { style: 'padding: 4px 0;' }, [
    h('div', { style: 'display: flex; justify-content: space-between; align-items: flex-start;' }, [
      h('div', { style: 'display: flex; align-items: center; gap: 8px;' }, [
        h('span', { style: 'font-weight: 600; color: #1d2129;' }, option.label),
        option.is_official ? h(NTag, { type: 'success', size: 'small', bordered: false }, () => '官方') : null
      ]),
      h('span', { style: 'color: #86909c; font-size: 12px; margin-left: 12px; white-space: nowrap;' }, `⭐ ${option.stars}`)
    ]),
    h('div', { style: 'font-size: 13px; color: #86909c; margin-top: 4px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; max-width: 360px;' }, option.description || '暂无描述')
  ])
}

function formatSize(bytes) {
  if (!bytes) return '0 B'
  const sizes = ['B', 'KB', 'MB', 'GB']
  const i = Math.floor(Math.log(Math.abs(bytes)) / Math.log(1024))
  return (bytes / Math.pow(1024, i)).toFixed(2) + ' ' + sizes[i]
}

function formatDate(ts) {
  if (!ts) return '-'
  return new Date(ts * 1000).toLocaleString('zh-CN', { 
    year: 'numeric', month: '2-digit', day: '2-digit', 
    hour: '2-digit', minute: '2-digit'
  })
}

const columns = [
  {
    title: '镜像标签 (Tags)',
    key: 'tags',
    render: (row) => {
      if (!row.tags || row.tags.length === 0) return h('span', { style: 'color: #c9cdd4;' }, '<无>')
      return row.tags.map(t => h(NTag, { size: 'small', type: 'info', bordered: false, style: 'margin-right: 6px;' }, () => t))
    }
  },
  { 
    title: '镜像 ID', 
    key: 'id', 
    width: 200, 
    ellipsis: { tooltip: true },
    render: (row) => h('span', { style: 'font-family: monospace; color: #86909c;' }, row.id.replace('sha256:', '').substring(0, 12))
  },
  {
    title: '大小',
    key: 'size',
    width: 120,
    render: (row) => h('span', { style: 'font-weight: 500; color: #1d2129;' }, formatSize(row.size))
  },
  {
    title: '创建时间',
    key: 'created',
    width: 160,
    render: (row) => h('span', { style: 'font-size: 13px; color: #4e5969;' }, formatDate(row.created))
  },
  {
    title: '操作',
    key: 'actions',
    width: 120,
    render: (row) => {
      return h(NButton, {
        size: 'small',
        type: 'error',
        secondary: true,
        onClick: () => removeImage(row)
      }, () => '删除')
    }
  },
]

async function refresh() {
  loading.value = true
  try {
    const res = await api.get('/images')
    if (res.success) images.value = res.data
  } catch (err) {
    message.error('获取镜像列表失败')
  } finally {
    loading.value = false
  }
}

async function pullImage() {
  if (!pullForm.value.image) {
    message.warning('请输入镜像名称')
    return false
  }
  pulling.value = true
  try {
    // 根据加速源构建实际拉取的镜像名
    const mirror = pullForm.value.mirror
    let actualImage = pullForm.value.image
    if (mirror) {
      const name = actualImage.includes('/') ? actualImage : `library/${actualImage}`
      actualImage = `${mirror}/${name}`
    }
    await api.post('/images/pull', { image: actualImage, tag: pullForm.value.tag })
    message.success('镜像拉取成功')
    pullForm.value = { image: '', tag: '', mirror: 'mirror.ccs.tencentyun.com' }
    showPull.value = false
    refresh()
  } catch (err) {
    message.error('镜像拉取失败')
  } finally {
    pulling.value = false
  }
}

function removeImage(row) {
  const label = row.tags?.[0] || row.id.substring(0, 12)
  dialog.warning({
    title: '确认删除',
    content: `确定要删除镜像 "${label}" 吗？`,
    positiveText: '确认删除',
    negativeText: '取消',
    onPositive: async () => {
      try {
        await api.delete(`/images/${encodeURIComponent(row.id)}`)
        message.success('镜像已删除')
        refresh()
      } catch (err) {
        message.error('删除镜像失败，可能是因为有容器正在使用它')
      }
    }
  })
}

onMounted(refresh)
</script>
