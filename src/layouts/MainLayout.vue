<template>
  <el-container class="main-layout">
    <el-aside width="200px" class="sidebar">
      <div class="logo">
        <h2>EA OKX</h2>
      </div>
      <el-menu
        :default-active="activeMenu"
        class="sidebar-menu"
        :background-color="menuBgColor"
        :text-color="menuTextColor"
        :active-text-color="menuActiveColor"
        router
      >
        <el-menu-item index="/dashboard">
          <el-icon><DataAnalysis /></el-icon>
          <span>Dashboard</span>
        </el-menu-item>
        <el-menu-item index="/strategies">
          <el-icon><TrendCharts /></el-icon>
          <span>Strategies</span>
        </el-menu-item>
        <el-menu-item index="/backtest">
          <el-icon><Timer /></el-icon>
          <span>Backtest</span>
        </el-menu-item>
        <el-menu-item index="/trading">
          <el-icon><Money /></el-icon>
          <span>Trading</span>
        </el-menu-item>
        <el-menu-item index="/risk">
          <el-icon><Warning /></el-icon>
          <span>Risk</span>
        </el-menu-item>
        <el-menu-item index="/settings">
          <el-icon><Setting /></el-icon>
          <span>Settings</span>
        </el-menu-item>
      </el-menu>
    </el-aside>
    <el-container>
      <el-header class="header">
        <div class="header-left">
          <span class="breadcrumb">{{ currentTitle }}</span>
        </div>
        <div class="header-right">
          <el-tooltip :content="isDark ? 'Switch to Light Mode' : 'Switch to Dark Mode'" placement="bottom">
            <el-icon class="header-icon theme-switcher" @click="toggleTheme">
              <Sunny v-if="isDark" />
              <Moon v-else />
            </el-icon>
          </el-tooltip>
          <el-icon class="header-icon"><Bell /></el-icon>
          <el-icon class="header-icon"><User /></el-icon>
        </div>
      </el-header>
      <el-main class="main-content">
        <router-view v-slot="{ Component, route }">
          <transition name="fade-slide" mode="out-in">
            <keep-alive :include="cachedViews">
              <component :is="Component" :key="route.path" />
            </keep-alive>
          </transition>
        </router-view>
      </el-main>
    </el-container>
  </el-container>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useRoute } from 'vue-router'
import { useTheme } from '@/composables/useTheme'

const route = useRoute()
const { isDark, toggleTheme, initTheme } = useTheme()

const activeMenu = computed(() => route.path)
const currentTitle = computed(() => route.meta.title as string || 'Dashboard')

// Cache main views for faster switching (exclude detail pages)
const cachedViews = ref([
  'Dashboard',
  'Strategies', 
  'Backtest',
  'Trading',
  'Risk',
  'Settings'
])

// Dynamic menu colors based on theme
const menuBgColor = computed(() => isDark.value ? '#1c2128' : '#f6f8fa')
const menuTextColor = computed(() => isDark.value ? '#adbac7' : '#59636e')
const menuActiveColor = computed(() => isDark.value ? '#539bf5' : '#0969da')

// Initialize theme on mount
onMounted(() => {
  initTheme()
})
</script>

<style lang="scss" scoped>
.main-layout {
  height: 100vh;
  background-color: var(--bg-primary);
}

.sidebar {
  background-color: var(--bg-secondary);
  border-right: 1px solid var(--border-color);

  .logo {
    height: 60px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-bottom: 1px solid var(--border-color);
    
    h2 {
      color: var(--accent-color);
      font-size: 20px;
      font-weight: bold;
    }
  }
}

.sidebar-menu {
  border: none;
}

.header {
  background-color: var(--bg-secondary);
  border-bottom: 1px solid var(--border-color);
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 20px;

  .header-left {
    .breadcrumb {
      font-size: 18px;
      font-weight: 500;
      color: var(--text-primary);
    }
  }

  .header-right {
    display: flex;
    gap: 20px;

    .header-icon {
      font-size: 20px;
      cursor: pointer;
      color: var(--text-secondary);
      
      &:hover {
        color: var(--accent-color);
      }

      &.theme-switcher {
        font-size: 22px;
        
        &:hover {
          transform: rotate(20deg);
        }
      }
    }
  }
}

.main-content {
  background-color: var(--bg-primary);
  padding: 20px;
  overflow-y: auto;
}

// Route transition animations
.fade-slide-enter-active,
.fade-slide-leave-active {
  transition: all 0.15s ease;
}

.fade-slide-enter-from {
  opacity: 0;
  transform: translateX(10px);
}

.fade-slide-leave-to {
  opacity: 0;
  transform: translateX(-10px);
}
</style>
