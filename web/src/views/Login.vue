<template>
  <div class="login-container">
    <div class="login-wrapper">
      <div class="login-left">
        <div class="login-brand">
          <svg width="32" height="32" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
            <path d="M12 2L2 7L12 12L22 7L12 2Z" stroke="#165DFF" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
            <path d="M2 17L12 22L22 17" stroke="#165DFF" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
            <path d="M2 12L12 17L22 12" stroke="#165DFF" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
          </svg>
          <span style="font-size: 24px; font-weight: 700; color: #1d2129; margin-left: 10px;">RecchDockerPanel</span>
        </div>
        <div class="login-slogan">
          Enterprise Container<br/>
          Management System.
        </div>
        <div class="login-sub-slogan">
          高效、专业、极简的 Docker 企业级可视化运维面板
        </div>
      </div>
      <div class="login-right">
        <div class="login-form-wrapper">
          <h2 style="font-size: 24px; font-weight: 600; color: #1d2129; margin-bottom: 8px;">欢迎登录</h2>
          <p style="color: #86909c; font-size: 14px; margin-bottom: 32px;">请输入您的管理员账号信息</p>
          
          <n-form ref="formRef" :model="form" :rules="rules">
            <n-form-item path="username" label="用户名">
              <n-input
                v-model:value="form.username"
                placeholder="请输入用户名"
                size="large"
                clearable
                @keyup.enter="handleLogin"
              />
            </n-form-item>
            <n-form-item path="password" label="密码">
              <n-input
                v-model:value="form.password"
                type="password"
                show-password-on="click"
                placeholder="请输入密码"
                size="large"
                clearable
                @keyup.enter="handleLogin"
              />
            </n-form-item>
            <n-button
              type="primary"
              block
              size="large"
              :loading="loading"
              @click="handleLogin"
              style="margin-top: 16px; height: 40px;"
            >
              登录
            </n-button>
          </n-form>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { useMessage } from 'naive-ui'
import api from '../utils/api'

const router = useRouter()
const message = useMessage()
const loading = ref(false)
const formRef = ref(null)

const form = ref({ username: '', password: '' })
const rules = {
  username: { required: true, message: '请输入用户名', trigger: 'blur' },
  password: { required: true, message: '请输入密码', trigger: 'blur' },
}

async function handleLogin() {
  formRef.value?.validate(async (errors) => {
    if (!errors) {
      try {
        loading.value = true
        const res = await api.post('/auth/login', form.value)
        if (res.success) {
          localStorage.setItem('dockpanel_token', res.data.token)
          localStorage.setItem('dockpanel_user', res.data.username)
          message.success('登录成功')
          router.push('/dashboard')
        } else {
          message.error(res.message || '登录失败')
        }
      } catch (err) {
        message.error('用户名或密码错误')
      } finally {
        loading.value = false
      }
    }
  })
}
</script>

<style scoped>
.login-container {
  min-height: 100vh;
  background-color: #f2f3f5;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 20px;
}

.login-wrapper {
  width: 1000px;
  max-width: 100%;
  height: 600px;
  background: #fff;
  border-radius: 8px;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.05);
  display: flex;
  overflow: hidden;
}

.login-left {
  flex: 1;
  background: linear-gradient(145deg, #e8f3ff 0%, #f2f6ff 100%);
  padding: 60px;
  display: flex;
  flex-direction: column;
}

.login-brand {
  display: flex;
  align-items: center;
  margin-bottom: auto;
}

.login-slogan {
  font-size: 40px;
  font-weight: 700;
  line-height: 1.2;
  color: #1d2129;
  margin-bottom: 24px;
}

.login-sub-slogan {
  font-size: 16px;
  color: #4e5969;
  line-height: 1.6;
  margin-bottom: 40px;
}

.login-right {
  width: 480px;
  padding: 60px;
  display: flex;
  flex-direction: column;
  justify-content: center;
}

.login-form-wrapper {
  width: 100%;
}

.n-form-item-label {
  color: #1d2129 !important;
  font-weight: 500;
}
</style>
