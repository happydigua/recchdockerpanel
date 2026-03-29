<template>
  <div class="layout-wrapper">
    <!-- 顶部导航 -->
    <header class="navbar">
      <div class="nav-left">
        <svg width="24" height="24" viewBox="0 0 24 24" fill="none" style="margin-right: 12px;">
          <path d="M12 2L2 7L12 12L22 7L12 2Z" fill="#165DFF"/>
          <path d="M2 17L12 22L22 17M2 12L12 17L22 12" stroke="#165DFF" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
        </svg>
        <div class="nav-brand">DockPanel</div>
      </div>
      
      <div class="nav-menu">
        <router-link
          v-for="item in navItems"
          :key="item.path"
          :to="item.path"
          class="nav-menu-item"
          :class="{ active: $route.path === item.path }"
        >
          {{ item.label }}
        </router-link>
      </div>
      
      <div class="nav-right">
        <div class="user-profile">
          <div class="avatar">{{ username.charAt(0).toUpperCase() }}</div>
          <span>{{ username }}</span>
        </div>
        <n-button text style="color: #4e5969; margin-left: 20px;" @click="logout" title="退出登录">
          退出登录
        </n-button>
      </div>
    </header>

    <!-- 主内容区 -->
    <main class="layout-content">
      <div class="page-container">
        <router-view />
      </div>
    </main>
  </div>
</template>

<script setup>
import { ref } from 'vue'
import { useRouter } from 'vue-router'

const router = useRouter()
const username = ref(localStorage.getItem('dockpanel_user') || 'admin')

const navItems = [
  { path: '/dashboard', label: '工作台' },
  { path: '/containers', label: '容器管理' },
  { path: '/images', label: '镜像列表' },
  { path: '/projects', label: '项目部署' },
  { path: '/apps', label: '应用商店' },
]

function logout() {
  localStorage.removeItem('dockpanel_token')
  localStorage.removeItem('dockpanel_user')
  router.push('/login')
}
</script>

<style scoped>
.layout-wrapper {
  min-height: 100vh;
  background-color: #f2f3f5;
  display: flex;
  flex-direction: column;
}

.navbar {
  height: 60px;
  background-color: #ffffff;
  border-bottom: 1px solid #e5e6eb;
  padding: 0 40px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  position: sticky;
  top: 0;
  z-index: 100;
  box-shadow: 0 2px 5px rgba(0,0,0,0.02);
}

.nav-left {
  display: flex;
  align-items: center;
  width: 200px;
}

.nav-brand {
  font-size: 18px;
  font-weight: 600;
  color: #1d2129;
}

.nav-menu {
  display: flex;
  gap: 32px;
  height: 100%;
}

.nav-menu-item {
  color: #4e5969;
  text-decoration: none;
  font-size: 15px;
  font-weight: 500;
  position: relative;
  display: flex;
  align-items: center;
  height: 100%;
  transition: color 0.2s;
}

.nav-menu-item:hover {
  color: #1d2129;
}

.nav-menu-item.active {
  color: #165dff;
}

.nav-menu-item.active::after {
  content: '';
  position: absolute;
  bottom: 0;
  left: 0;
  right: 0;
  height: 3px;
  background-color: #165dff;
  border-radius: 3px 3px 0 0;
}

.nav-right {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  width: 200px;
}

.user-profile {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 14px;
  color: #1d2129;
  font-weight: 500;
}

.avatar {
  width: 28px;
  height: 28px;
  background-color: #165dff;
  color: #fff;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 12px;
}

.layout-content {
  flex: 1;
  padding: 24px 40px;
}

.page-container {
  max-width: 1400px;
  margin: 0 auto;
}
</style>
