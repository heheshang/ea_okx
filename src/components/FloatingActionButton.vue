<template>
  <div
    ref="fabContainerRef"
    class="fab-container"
    :class="fabClasses"
    :style="fabStyle"
  >
    <!-- 主按钮 -->
    <el-button
      :type="buttonType"
      :size="buttonSize"
      :icon="mainIcon"
      circle
      class="fab-main-button"
      :class="{ 'fab-expanded': isExpanded, 'fab-pulse': pulse }"
      @click="handleMainClick"
    >
      <span v-if="showText && !isMobile" class="fab-text">{{ mainText }}</span>
    </el-button>

    <!-- 展开的操作按钮 -->
    <transition-group name="fab-item" tag="div" class="fab-actions">
      <el-button
        v-for="(action, index) in visibleActions"
        :key="action.key"
        :type="action.type || 'default'"
        :size="action.size || 'small'"
        :icon="action.icon"
        circle
        class="fab-action-button"
        :style="getActionStyle(index)"
        @click="handleActionClick(action)"
      >
        <el-tooltip
          v-if="action.tooltip && !isMobile"
          :content="action.tooltip"
          placement="left"
          :disabled="isExpanded"
        >
          <template #content>
            <span class="fab-tooltip">{{ action.tooltip }}</span>
          </template>
        </el-tooltip>
      </el-button>
    </transition-group>

    <!-- 遮罩层（移动端） -->
    <div
      v-if="isMobile && isExpanded"
      class="fab-backdrop"
      @click="handleBackdropClick"
    />

    <!-- 标签（移动端展开时显示） -->
    <div
      v-for="(action, index) in visibleActions"
      :key="`label-${action.key}`"
      v-show="isMobile && isExpanded"
      class="fab-action-label"
      :style="getLabelStyle(index)"
    >
      {{ action.label || action.tooltip }}
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from 'vue'
import { ElButton, ElTooltip } from 'element-plus'
import { useResponsive } from '@/composables/useResponsive'

interface Action {
  key: string
  label?: string
  tooltip?: string
  icon: any
  type?: 'primary' | 'success' | 'warning' | 'danger' | 'info' | 'default'
  size?: 'large' | 'default' | 'small'
  disabled?: boolean
  badge?: number
  onClick: () => void
}

interface Props {
  actions?: Action[]
  mainIcon?: any
  mainText?: string
  position?: 'bottom-right' | 'bottom-left' | 'top-right' | 'top-left'
  showText?: boolean
  expandable?: boolean
  autoClose?: boolean
  pulse?: boolean
  offset?: number
  zIndex?: number
  disabled?: boolean
  hideOnScroll?: boolean
  scrollThreshold?: number
}

const props = withDefaults(defineProps<Props>(), {
  actions: () => [],
  mainText: '操作',
  position: 'bottom-right',
  showText: false,
  expandable: true,
  autoClose: true,
  pulse: false,
  offset: 20,
  zIndex: 1000,
  disabled: false,
  hideOnScroll: false,
  scrollThreshold: 100
})

const emit = defineEmits<{
  mainClick: []
  actionClick: [action: Action]
  expand: []
  collapse: []
}>()

// 响应式引用
const fabContainerRef = ref<HTMLElement>()
const isExpanded = ref(false)
const isScrolling = ref(false)
const scrollY = ref(0)
let scrollTimer: number | null = null

const { isMobile, isTablet } = useResponsive()

// 计算属性
const visibleActions = computed(() => {
  return props.actions.filter(action => !action.disabled)
})

const buttonSize = computed(() => {
  if (isMobile.value) return 'large'
  if (isTablet.value) return 'default'
  return 'large'
})

const buttonType = computed(() => {
  if (props.disabled) return 'info'
  if (isExpanded.value) return 'danger'
  return 'primary'
})

const fabClasses = computed(() => ({
  'fab-mobile': isMobile.value,
  'fab-tablet': isTablet.value,
  'fab-expanded': isExpanded.value,
  'fab-disabled': props.disabled,
  'fab-hidden': props.hideOnScroll && isScrolling.value,
  [`fab-position-${props.position}`]: true
}))

const fabStyle = computed(() => ({
  '--fab-offset': `${props.offset}px`,
  '--fab-z-index': props.zIndex,
  '--fab-animation-duration': isMobile.value ? '0.3s' : '0.25s'
}))

// 获取操作按钮样式
const getActionStyle = (index: number) => {
  const baseOffset = 60 // 主按钮直径 + 间距
  const spacing = isMobile.value ? 50 : 45
  const offset = baseOffset + (index * spacing)

  const positions: Record<string, { bottom?: number; top?: number; left?: number; right?: number }> = {}

  switch (props.position) {
    case 'bottom-right':
      positions.bottom = offset
      positions.right = 0
      break
    case 'bottom-left':
      positions.bottom = offset
      positions.left = 0
      break
    case 'top-right':
      positions.top = offset
      positions.right = 0
      break
    case 'top-left':
      positions.top = offset
      positions.left = 0
      break
  }

  return positions
}

// 获取标签样式
const getLabelStyle = (index: number) => {
  const positions: Record<string, { bottom?: number; top?: number; left?: number; right?: number }> = {}

  switch (props.position) {
    case 'bottom-right':
      positions.bottom = 60 + (index * 50)
      positions.right = 60
      break
    case 'bottom-left':
      positions.bottom = 60 + (index * 50)
      positions.left = 60
      break
    case 'top-right':
      positions.top = 60 + (index * 50)
      positions.right = 60
      break
    case 'top-left':
      positions.top = 60 + (index * 50)
      positions.left = 60
      break
  }

  return {
    ...positions,
    transform: 'translateY(-50%)'
  }
}

// 事件处理
const handleMainClick = () => {
  if (props.disabled) return

  if (props.expandable && visibleActions.value.length > 0) {
    toggleExpanded()
  } else {
    emit('mainClick')
  }
}

const handleActionClick = (action: Action) => {
  if (action.disabled) return

  action.onClick()
  emit('actionClick', action)

  if (props.autoClose) {
    collapse()
  }
}

const handleBackdropClick = () => {
  if (isExpanded.value) {
    collapse()
  }
}

const expand = () => {
  isExpanded.value = true
  emit('expand')
}

const collapse = () => {
  isExpanded.value = false
  emit('collapse')
}

const toggleExpanded = () => {
  if (isExpanded.value) {
    collapse()
  } else {
    expand()
  }
}

// 滚动处理
const handleScroll = () => {
  scrollY.value = window.scrollY

  if (!isScrolling.value) {
    isScrolling.value = true
  }

  // 防抖
  if (scrollTimer) {
    clearTimeout(scrollTimer)
  }

  scrollTimer = window.setTimeout(() => {
    isScrolling.value = false
  }, 150)

  // 检查滚动阈值
  if (props.hideOnScroll && Math.abs(scrollY.value - scrollY.value) > props.scrollThreshold) {
    // 在滚动时自动收起
    if (isExpanded.value) {
      collapse()
    }
  }
}

// 键盘快捷键支持
const handleKeydown = (event: KeyboardEvent) => {
  if (event.key === 'Escape' && isExpanded.value) {
    collapse()
  }
}

// 点击外部收起
const handleClickOutside = (event: MouseEvent) => {
  if (fabContainerRef.value && !fabContainerRef.value.contains(event.target as Node)) {
    if (isExpanded.value && props.autoClose) {
      collapse()
    }
  }
}

// 监听滚动
watch(() => props.hideOnScroll, (newValue) => {
  if (newValue) {
    window.addEventListener('scroll', handleScroll, { passive: true })
  } else {
    window.removeEventListener('scroll', handleScroll)
  }
}, { immediate: true })

// 监听键盘事件
watch(() => isExpanded.value, (newValue) => {
  if (newValue) {
    document.addEventListener('keydown', handleKeydown)
    document.addEventListener('click', handleClickOutside)
  } else {
    document.removeEventListener('keydown', handleKeydown)
    document.removeEventListener('click', handleClickOutside)
  }
})

// 获取焦点元素
const getFocusableElements = () => {
  const container = fabContainerRef.value
  if (!container) return []

  return Array.from(container.querySelectorAll(
    'button:not([disabled]), [tabindex]:not([tabindex="-1"])'
  )) as HTMLElement[]
}

// 焦点管理
const manageFocus = () => {
  if (isExpanded.value) {
    const focusableElements = getFocusableElements()
    if (focusableElements.length > 1) {
      // 聚焦到第一个操作按钮
      focusableElements[1].focus()
    }
  }
}

// 监听展开状态变化
watch(isExpanded, (newValue) => {
  if (newValue) {
    // 在下一个动画帧后管理焦点
    requestAnimationFrame(manageFocus)
  }
})

onMounted(() => {
  if (props.hideOnScroll) {
    window.addEventListener('scroll', handleScroll, { passive: true })
  }
})

onUnmounted(() => {
  window.removeEventListener('scroll', handleScroll)
  document.removeEventListener('keydown', handleKeydown)
  document.removeEventListener('click', handleClickOutside)

  if (scrollTimer) {
    clearTimeout(scrollTimer)
  }
})

// 暴露方法给父组件
defineExpose({
  expand,
  collapse,
  toggleExpanded,
  isExpanded: () => isExpanded.value
})
</script>

<style lang="scss" scoped>
@import '@/styles/variables.scss';

.fab-container {
  position: fixed;
  z-index: var(--fab-z-index);
  --fab-offset: 20px;

  &.fab-position-bottom-right {
    bottom: var(--fab-offset);
    right: var(--fab-offset);
  }

  &.fab-position-bottom-left {
    bottom: var(--fab-offset);
    left: var(--fab-offset);
  }

  &.fab-position-top-right {
    top: var(--fab-offset);
    right: var(--fab-offset);
  }

  &.fab-position-top-left {
    top: var(--fab-offset);
    left: var(--fab-offset);
  }

  &.fab-mobile {
    --fab-offset: 16px;
  }

  &.fab-hidden {
    transform: scale(0.8);
    opacity: 0;
    pointer-events: none;
  }
}

.fab-main-button {
  width: 56px;
  height: 56px;
  font-size: 20px;
  box-shadow: $shadow-lg;
  transition: all var(--fab-animation-duration, 0.25s) cubic-bezier(0.4, 0, 0.2, 1);
  position: relative;
  z-index: 2;

  &:hover {
    transform: scale(1.05);
    box-shadow: $shadow-xl;
  }

  &:active {
    transform: scale(0.95);
  }

  &.fab-expanded {
    transform: rotate(45deg);
    background-color: var(--danger-color);
    border-color: var(--danger-color);
  }

  &.fab-pulse {
    animation: fab-pulse 2s infinite;
  }

  .fab-text {
    margin-left: $spacing-sm;
  }

  @include mobile {
    width: 60px;
    height: 60px;
    font-size: 24px;
  }
}

.fab-actions {
  position: absolute;
  pointer-events: none;
}

.fab-action-button {
  position: absolute;
  width: 44px;
  height: 44px;
  font-size: 16px;
  box-shadow: $shadow-md;
  opacity: 0;
  transform: scale(0.3);
  pointer-events: all;
  transition: all var(--fab-animation-duration) cubic-bezier(0.34, 1.56, 0.64, 1);

  @include mobile {
    width: 48px;
    height: 48px;
  }
}

.fab-expanded .fab-action-button {
  opacity: 1;
  transform: scale(1);
}

.fab-action-label {
  position: absolute;
  background-color: var(--bg-secondary);
  color: var(--text-primary);
  padding: $spacing-xs $spacing-sm;
  border-radius: $radius-md;
  font-size: $font-size-sm;
  white-space: nowrap;
  box-shadow: $shadow-md;
  opacity: 0;
  transform: scale(0.8) translateY(-50%);
  transition: all var(--fab-animation-duration) cubic-bezier(0.34, 1.56, 0.64, 1);
  pointer-events: none;
}

.fab-expanded .fab-action-label {
  opacity: 1;
  transform: scale(1) translateY(-50%);
}

.fab-backdrop {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: rgba(0, 0, 0, 0.3);
  z-index: 1;
  opacity: 0;
  animation: fab-backdrop-fade-in var(--fab-animation-duration) forwards;
}

.fab-tooltip {
  font-size: $font-size-xs;
  line-height: 1.4;
}

// 动画
.fab-item-enter-active {
  transition: all var(--fab-animation-duration) cubic-bezier(0.34, 1.56, 0.64, 1);
}

.fab-item-leave-active {
  transition: all var(--fab-animation-duration) cubic-bezier(0.4, 0, 1, 1);
}

.fab-item-enter-from {
  opacity: 0;
  transform: scale(0.3) translateY(20px);
}

.fab-item-leave-to {
  opacity: 0;
  transform: scale(0.3) translateY(-20px);
}

.fab-item-move {
  transition: transform var(--fab-animation-duration) cubic-bezier(0.4, 0, 0.2, 1);
}

@keyframes fab-pulse {
  0% {
    box-shadow: 0 0 0 0 rgba(var(--accent-color-rgb, 83, 155, 245), 0.4);
  }

  50% {
    box-shadow: 0 0 0 10px rgba(var(--accent-color-rgb, 83, 155, 245), 0);
  }

  100% {
    box-shadow: 0 0 0 0 rgba(var(--accent-color-rgb, 83, 155, 245), 0);
  }
}

@keyframes fab-backdrop-fade-in {
  to {
    opacity: 1;
  }
}

// 移动端优化
@include mobile {
  .fab-container {
    // 触摸优化
    touch-action: manipulation;

    // 避免与底部导航冲突
    &.fab-position-bottom-right,
    &.fab-position-bottom-left {
      bottom: calc(var(--bottom-nav-height, 56px) + var(--fab-offset));
    }
  }

  .fab-action-button {
    // 增大触摸区域
    &::before {
      content: '';
      position: absolute;
      top: -8px;
      left: -8px;
      right: -8px;
      bottom: -8px;
    }
  }
}

// 高对比度模式
@media (prefers-contrast: high) {
  .fab-container {
    .fab-main-button,
    .fab-action-button {
      border: 2px solid var(--text-primary);
    }
  }
}

// 减少动画模式
@media (prefers-reduced-motion: reduce) {
  .fab-container {
    * {
      animation-duration: 0.01ms !important;
      animation-iteration-count: 1 !important;
      transition-duration: 0.01ms !important;
    }
  }
}
</style>