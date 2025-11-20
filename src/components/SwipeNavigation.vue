<template>
  <div
    ref="pageContainerRef"
    class="swipe-navigation-container"
    :class="{ 'swipe-active': isSwiping }"
  >
    <!-- 滑动指示器 -->
    <div
      v-if="isSwiping && isMobile"
      class="swipe-indicator"
      :class="swipeIndicatorClass"
    >
      <el-icon><ArrowRight /></el-icon>
    </div>

    <!-- 页面内容 -->
    <slot />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRoute } from 'vue-router'
import { useSwipe, useNavigationSwipe } from '@/composables/useSwipe'
import { useResponsive } from '@/composables/useResponsive'
import { ArrowRight } from '@element-plus/icons-vue'

interface Props {
  navigationEnabled?: boolean
  visualFeedback?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  navigationEnabled: true,
  visualFeedback: true
})

const emit = defineEmits<{
  swipeLeft: []
  swipeRight: []
  navigationChanged: [direction: 'forward' | 'backward']
}>()

// 响应式和导航
const { isMobile } = useResponsive()
const route = useRoute()
const pageContainerRef = ref<HTMLElement | null>(null)

// 导航配置
const navigationOrder = [
  '/dashboard',
  '/strategies',
  '/backtest',
  '/trading',
  '/risk',
  '/settings'
]

// 获取下一个/上一个页面路径
const getAdjacentPages = (currentPath: string) => {
  const currentIndex = navigationOrder.indexOf(currentPath)
  return {
    next: currentIndex < navigationOrder.length - 1 ? navigationOrder[currentIndex + 1] : null,
    prev: currentIndex > 0 ? navigationOrder[currentIndex - 1] : null
  }
}

// 处理导航滑动
const handleNavigationSwipe = (direction: 'left' | 'right') => {
  if (!props.navigationEnabled) return

  const adjacent = getAdjacentPages(route.path)

  if (direction === 'left' && adjacent.next) {
    emit('navigationChanged', 'forward')
    // 这里可以添加路由导航逻辑
  } else if (direction === 'right' && adjacent.prev) {
    emit('navigationChanged', 'backward')
    // 这里可以添加路由导航逻辑
  }

  // 触发自定义事件
  if (direction === 'left') {
    emit('swipeLeft')
  } else {
    emit('swipeRight')
  }
}

// 使用滑动手势
const { isSwiping, swipeDirection } = useSwipe(pageContainerRef, {
  threshold: isMobile.value ? 80 : 120,
  preventDefault: isMobile.value,
  onSwipeLeft: () => handleNavigationSwipe('left'),
  onSwipeRight: () => handleNavigationSwipe('right')
})

// 滑动指示器样式
const swipeIndicatorClass = computed(() => ({
  'swipe-left': swipeDirection.value.left,
  'swipe-right': swipeDirection.value.right,
  'swipe-up': swipeDirection.value.up,
  'swipe-down': swipeDirection.value.down
}))

// 检查是否可以导航
const canNavigateForward = computed(() => {
  const adjacent = getAdjacentPages(route.path)
  return !!adjacent.next
})

const canNavigateBackward = computed(() => {
  const adjacent = getAdjacentPages(route.path)
  return !!adjacent.prev
})

// 暴露给父组件的方法
defineExpose({
  canNavigateForward,
  canNavigateBackward,
  isSwiping
})

onMounted(() => {
  // 添加页面可见性变化监听
  document.addEventListener('visibilitychange', () => {
    if (document.hidden) {
      // 页面隐藏时重置滑动状态
    }
  })
})
</script>

<style lang="scss" scoped>
@import '@/styles/variables.scss';

.swipe-navigation-container {
  position: relative;
  width: 100%;
  height: 100%;
  touch-action: pan-y; // 允许垂直滚动，但禁用水平滚动
  overflow: hidden;

  &.swipe-active {
    // 滑动时的视觉反馈
    background-color: var(--hover-bg);
    transition: background-color $transition-fast;
  }

  // 移动端滑动区域扩大
  @include mobile {
    touch-action: none; // 移动端完全控制触摸行为
  }
}

.swipe-indicator {
  position: fixed;
  top: 50%;
  transform: translateY(-50%);
  width: 60px;
  height: 60px;
  background-color: var(--accent-color);
  color: white;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  opacity: 0.8;
  z-index: $z-tooltip;
  pointer-events: none;
  animation: swipe-pulse 1s infinite;

  &.swipe-left {
    right: 20px;
    transform: translateY(-50%) rotate(180deg);
  }

  &.swipe-right {
    left: 20px;
    transform: translateY(-50%);
  }

  @include mobile {
    width: 48px;
    height: 48px;

    &.swipe-left {
      right: 16px;
    }

    &.swipe-right {
      left: 16px;
    }
  }
}

@keyframes swipe-pulse {
  0%, 100% {
    opacity: 0.8;
    transform: translateY(-50%) scale(1);
  }

  50% {
    opacity: 0.6;
    transform: translateY(-50%) scale(0.9);
  }
}

// 滑动进度条
.swipe-progress {
  position: fixed;
  bottom: 0;
  left: 0;
  height: 3px;
  background-color: var(--accent-color);
  transition: width $transition-fast;
  z-index: $z-sticky;

  &.swipe-left {
    right: 0;
    left: auto;
    transform-origin: right;
  }

  &.swipe-right {
    transform-origin: left;
  }
}

// 触摸反馈动画
@keyframes swipe-feedback {
  0% {
    transform: scale(1);
    opacity: 0.8;
  }

  50% {
    transform: scale(1.1);
    opacity: 1;
  }

  100% {
    transform: scale(1);
    opacity: 0.8;
  }
}

// 边缘发光效果
.swipe-navigation-container::before,
.swipe-navigation-container::after {
  content: '';
  position: absolute;
  top: 0;
  width: 4px;
  height: 100%;
  opacity: 0;
  transition: opacity $transition-fast;
  pointer-events: none;
}

.swipe-navigation-container::before {
  left: 0;
  background: linear-gradient(90deg, var(--accent-color), transparent);
}

.swipe-navigation-container::after {
  right: 0;
  background: linear-gradient(90deg, transparent, var(--accent-color));
}

.swipe-navigation-container.swipe-active::before,
.swipe-navigation-container.swipe-active::after {
  opacity: 0.3;
}

// 在浅色模式下的调整
:root[data-theme='light'] {
  .swipe-indicator {
    background-color: var(--accent-color);
    color: white;
  }
}

// 深色模式下的调整
:root[data-theme='dark'] {
  .swipe-indicator {
    background-color: var(--accent-color);
    color: var(--text-primary);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
  }
}
</style>