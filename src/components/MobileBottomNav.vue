<template>
  <div class="mobile-bottom-nav" v-if="isMobile">
    <div class="nav-items">
      <router-link
        v-for="item in navItems"
        :key="item.path"
        :to="item.path"
        class="nav-item"
        :class="{ active: isActive(item.path) }"
      >
        <el-icon :size="22">
          <component :is="item.icon" />
        </el-icon>
        <span class="nav-label">{{ item.label }}</span>
      </router-link>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useRoute } from 'vue-router'
import { useResponsive } from '@/composables/useResponsive'
import {
  DataAnalysis,
  TrendCharts,
  Timer,
  Money,
  Warning,
} from '@element-plus/icons-vue'

const route = useRoute()
const { isMobile } = useResponsive()

const navItems = [
  { path: '/dashboard', label: 'Dashboard', icon: DataAnalysis },
  { path: '/strategies', label: 'Strategies', icon: TrendCharts },
  { path: '/backtest', label: 'Backtest', icon: Timer },
  { path: '/trading', label: 'Trading', icon: Money },
  { path: '/risk', label: 'Risk', icon: Warning },
]

const isActive = (path: string) => {
  return route.path === path || route.path.startsWith(path + '/')
}
</script>

<style lang="scss" scoped>
@import '@/styles/variables.scss';

.mobile-bottom-nav {
  position: fixed;
  bottom: 0;
  left: 0;
  right: 0;
  height: $bottom-nav-height;
  background-color: var(--bg-secondary);
  border-top: 1px solid var(--border-color);
  z-index: $z-fixed;
  padding-bottom: env(safe-area-inset-bottom);
  box-shadow: 0 -2px 8px rgba(0, 0, 0, 0.1);
  
  .nav-items {
    display: flex;
    justify-content: space-around;
    align-items: center;
    height: 100%;
    max-width: 100%;
    margin: 0 auto;
  }
  
  .nav-item {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    flex: 1;
    height: 100%;
    color: var(--text-secondary);
    text-decoration: none;
    transition: all $transition-fast;
    position: relative;
    
    &::before {
      content: '';
      position: absolute;
      top: 0;
      left: 50%;
      transform: translateX(-50%) scaleX(0);
      width: 40px;
      height: 3px;
      background-color: var(--accent-color);
      border-radius: 0 0 3px 3px;
      transition: transform $transition-base;
    }
    
    &.active {
      color: var(--accent-color);
      
      &::before {
        transform: translateX(-50%) scaleX(1);
      }
      
      .nav-label {
        font-weight: $font-weight-medium;
      }
    }
    
    &:active {
      transform: scale(0.95);
      background-color: var(--hover-bg);
    }
    
    .nav-label {
      font-size: 11px;
      margin-top: 4px;
      white-space: nowrap;
    }
  }
}
</style>
