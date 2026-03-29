import { createRouter, createWebHistory } from 'vue-router'

// 自动推导 base path
// 开发模式 → /
// 生产模式 → /{prefix}/（如 /xYzAbCdE/）
function getBase() {
  if (import.meta.env.DEV) return '/'
  const segments = window.location.pathname.split('/').filter(Boolean)
  // 如果 URL 是 /prefix/dashboard，则 base 是 /prefix/
  if (segments.length > 0) {
    return `/${segments[0]}/`
  }
  return '/'
}

const routes = [
  {
    path: '/login',
    name: 'Login',
    component: () => import('../views/Login.vue'),
    meta: { requiresAuth: false }
  },
  {
    path: '/',
    component: () => import('../layout/AppLayout.vue'),
    meta: { requiresAuth: true },
    children: [
      { path: '', redirect: '/dashboard' },
      { path: 'dashboard', name: 'Dashboard', component: () => import('../views/Dashboard.vue') },
      { path: 'containers', name: 'Containers', component: () => import('../views/Containers.vue') },
      { path: 'images', name: 'Images', component: () => import('../views/Images.vue') },
      { path: 'apps', name: 'AppStore', component: () => import('../views/AppStore.vue') },
      { path: 'projects', name: 'Projects', component: () => import('../views/Projects.vue') },
    ]
  }
]

const router = createRouter({
  history: createWebHistory(getBase()),
  routes
})

// 路由守卫
router.beforeEach((to, from, next) => {
  const token = localStorage.getItem('dockpanel_token')
  if (to.meta.requiresAuth !== false && !token) {
    next('/login')
  } else if (to.path === '/login' && token) {
    next('/dashboard')
  } else {
    next()
  }
})

export default router
