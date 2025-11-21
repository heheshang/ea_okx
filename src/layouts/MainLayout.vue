<template>
  <el-container class="main-layout">
    <!-- Sidebar with responsive behavior -->
    <el-aside 
      :width="sidebarWidth" 
      class="sidebar"
      v-show="!isMobile"
    >
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
        <el-menu-item index="/templates">
          <el-icon><Document /></el-icon>
          <span>Templates</span>
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
          <el-icon class="header-icon" v-if="!isMobile"><Bell /></el-icon>
          <el-icon class="header-icon" v-if="!isMobile"><User /></el-icon>
        </div>
      </el-header>
      <el-main class="main-content" :class="{ 'mobile-content': isMobile }">
        <router-view v-slot="{ Component, route }">
          <transition name="fade-slide" mode="out-in">
            <keep-alive :include="cachedViews">
              <component :is="Component" :key="route.path" />
            </keep-alive>
          </transition>
        </router-view>
      </el-main>
    </el-container>
    
    <!-- Mobile Bottom Navigation -->
    <MobileBottomNav />
  </el-container>
</template>

<script setup lang="ts">
import { computed, onMounted, ref, onUnmounted } from 'vue'
import { useRoute } from 'vue-router'
import { useTheme } from '@/composables/useTheme'
import { useResponsive } from '@/composables/useResponsive'
import MobileBottomNav from '@/components/MobileBottomNav.vue'

const route = useRoute()
const { isDark, toggleTheme, initTheme } = useTheme()
const { isMobile } = useResponsive()

const activeMenu = computed(() => route.path)
const currentTitle = computed(() => route.meta.title as string || 'Dashboard')

// Responsive sidebar width
const sidebarWidth = computed(() => '200px')

// Cache main views for faster switching (exclude detail pages)
const cachedViews = ref([
  'Dashboard',
  'Strategies',
  'Templates',
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
@import '@/styles/variables.scss';
@import '@/styles/utilities.scss';

.main-layout {
  height: 100vh;
  background-color: var(--bg-primary);
  position: relative;
}

.sidebar {
  background-color: var(--bg-secondary);
  border-right: 1px solid var(--border-color);
  transition: all $transition-base;
  z-index: $z-sticky;
  overflow-y: auto;
  overflow-x: hidden;
  @include custom-scrollbar;

  .logo {
    height: $header-height;
    display: flex;
    align-items: center;
    justify-content: center;
    border-bottom: 1px solid var(--border-color);
    
    h2 {
      color: var(--accent-color);
      font-size: $font-size-xl;
      font-weight: $font-weight-bold;
      margin: 0;
    }
  }
}

.sidebar-menu {
  border: none;
  padding: $spacing-md 0;
  
  :deep(.el-menu-item) {
    margin: $spacing-xs $spacing-md;
    border-radius: $radius-md;
    transition: all $transition-fast;
    
    &:hover {
      background-color: var(--hover-bg);
    }
    
    &.is-active {
      background-color: var(--info-bg);
      color: var(--accent-color);
      font-weight: $font-weight-medium;
    }
  }
}

.header {
  background-color: var(--bg-secondary);
  border-bottom: 1px solid var(--border-color);
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 $spacing-2xl;
  height: $header-height;
  z-index: $z-sticky;

  @include mobile {
    padding: 0 $spacing-lg;
    height: $header-height-mobile;
  }

  .header-left {
    display: flex;
    align-items: center;
    gap: $spacing-lg;
    
    .breadcrumb {
      font-size: $font-size-lg;
      font-weight: $font-weight-medium;
      color: var(--text-primary);
      
      @include mobile {
        font-size: $font-size-base;
      }
    }
  }

  .header-right {
    display: flex;
    gap: $spacing-xl;
    
    @include mobile {
      gap: $spacing-lg;
    }

    .header-icon {
      font-size: 20px;
      cursor: pointer;
      color: var(--text-secondary);
      transition: all $transition-fast;
      @include focus-visible;
      
      @include mobile {
        font-size: 18px;
      }
      
      &:hover {
        color: var(--accent-color);
        transform: translateY(-1px);
      }

      &.theme-switcher {
        font-size: 22px;
        
        @include mobile {
          font-size: 20px;
        }
        
        &:hover {
          transform: rotate(20deg);
        }
      }
    }
  }
}

.main-content {
  background-color: var(--bg-primary);
  padding: $spacing-2xl;
  overflow-y: auto;
  overflow-x: hidden;
  @include custom-scrollbar;
  
  @include mobile {
    padding: $spacing-lg;
    padding-bottom: calc($bottom-nav-height + $spacing-lg);
  }
  
  &.mobile-content {
    // Additional mobile-specific adjustments
    height: calc(100vh - $header-height-mobile - $bottom-nav-height);
  }
}

// Route transition animations
.fade-slide-enter-active,
.fade-slide-leave-active {
  transition: all $transition-base;
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
