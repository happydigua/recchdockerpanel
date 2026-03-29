<template>
  <n-config-provider :theme-overrides="arcoTheme">
    <n-dialog-provider>
      <n-message-provider>
        <router-view />
      </n-message-provider>
    </n-dialog-provider>
  </n-config-provider>
</template>

<script setup>
import { NConfigProvider, NMessageProvider, NDialogProvider } from 'naive-ui'

// 深度覆盖 Naive UI 样式以完全贴合 Arco Design
const arcoTheme = {
  common: {
    primaryColor: '#165dff',
    primaryColorHover: '#4080ff',
    primaryColorPressed: '#0e42d2',
    primaryColorSuppl: '#165dff',
    infoColor: '#165dff',
    successColor: '#00b42a',
    warningColor: '#ff7d00',
    errorColor: '#f53f3f',
    textColor1: '#1d2129', // 主标题
    textColor2: '#4e5969', // 正文
    textColor3: '#86909c', // 辅助提示
    borderRadius: '4px',   // Arco 默认小圆角
    fontFamily: '"Inter", -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, "Noto Sans", sans-serif',
    bodyColor: '#f2f3f5',  // Arco 标志性浅灰背景
    cardColor: '#ffffff'
  },
  Card: {
    paddingMedium: '20px',
    borderRadius: '4px',
    borderColor: '#e5e6eb',
    titleTextColor: '#1d2129',
    titleFontSizeMedium: '16px',
    titleFontWeight: '500'
  },
  Button: {
    borderRadiusMedium: '4px',
    textColor: '#4e5969',
    fontWeight: '400'
  },
  DataTable: {
    thColor: '#f2f3f5',
    thTextColor: '#4e5969',
    tdTextColor: '#1d2129',
    borderColor: '#e5e6eb',
    borderRadius: '4px',
    thFontWeight: '500'
  },
  Input: {
    borderRadius: '4px',
    colorFocus: '#ffffff',
    borderColor: '#e5e6eb',
    borderColorHover: '#165dff',
    borderColorFocus: '#165dff',
    boxShadowFocus: '0 0 0 2px rgba(22, 93, 255, 0.2)'
  }
}
</script>

<style>
/* 全局基础样式 */
html, body, #app {
  height: 100%;
  margin: 0;
  padding: 0;
  background-color: #f2f3f5;
  color: #1d2129;
  font-family: var(--n-font-family);
  -webkit-font-smoothing: antialiased;
}
</style>
