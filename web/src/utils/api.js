import axios from 'axios'

// 自动检测 base path：
// 开发模式下 Vite proxy 直接用 /api
// 生产模式下从当前 URL 路径推导（如 /xYzAbCdE/dashboard → baseURL=/xYzAbCdE/api）
function getBasePath() {
  // 开发模式
  if (import.meta.env.DEV) return '/api'
  // 生产模式：取 URL 中的第一段作为 prefix
  const segments = window.location.pathname.split('/').filter(Boolean)
  if (segments.length > 0) {
    return `/${segments[0]}/api`
  }
  return '/api'
}

const api = axios.create({
  baseURL: getBasePath(),
  timeout: 30000,
})

// 请求拦截器：自动添加 Token
api.interceptors.request.use(config => {
  const token = localStorage.getItem('dockpanel_token')
  if (token) {
    config.headers.Authorization = `Bearer ${token}`
  }
  return config
})

// 响应拦截器：处理 401
api.interceptors.response.use(
  response => response.data,
  error => {
    if (error.response?.status === 401) {
      localStorage.removeItem('dockpanel_token')
      // 跳转到当前 base 下的 login 页
      const segments = window.location.pathname.split('/').filter(Boolean)
      const base = segments.length > 0 ? `/${segments[0]}` : ''
      window.location.href = `${base}/login`
    }
    return Promise.reject(error)
  }
)

export default api
