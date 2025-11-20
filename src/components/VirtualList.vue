<template>
  <div
    ref="containerRef"
    class="virtual-list-container"
    :style="{ height: containerHeight }"
    @scroll="handleScroll"
  >
    <!-- 总高度占位符 -->
    <div
      class="virtual-list-phantom"
      :style="{ height: `${totalHeight}px` }"
    />

    <!-- 可见项容器 -->
    <div
      class="virtual-list-content"
      :style="{
        transform: `translateY(${offsetY}px)`,
        height: `${visibleHeight}px`
      }"
    >
      <div
        v-for="item in visibleItems"
        :key="getItemKey(item)"
        :ref="setItemRef"
        class="virtual-list-item"
        :style="{
          height: `${getItemHeight(item)}px`
        }"
      >
        <slot name="item" :item="item" :index="getItemIndex(item)">
          <div class="default-item">{{ item }}</div>
        </slot>
      </div>
    </div>

    <!-- 加载更多指示器 -->
    <div
      v-if="showLoadingMore"
      class="loading-more"
      :style="{ top: `${totalHeight}px` }"
    >
      <el-icon class="loading-icon">
        <Loading />
      </el-icon>
      <span>加载更多...</span>
    </div>

    <!-- 空状态 -->
    <div v-if="items.length === 0 && !loading" class="virtual-list-empty">
      <el-icon :size="48">
        <DocumentRemove />
      </el-icon>
      <span>{{ emptyText }}</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted, nextTick } from 'vue'
import { Loading, DocumentRemove } from '@element-plus/icons-vue'
import { useResponsive } from '@/composables/useResponsive'

interface Props<T = any> {
  items: T[]
  itemHeight?: number | ((item: T) => number)
  containerHeight?: string | number
  keyField?: string
  bufferSize?: number
  threshold?: number
  loading?: boolean
  hasMore?: boolean
  emptyText?: string
  preloadNextPage?: boolean
  estimatedItemHeight?: number
}

const props = withDefaults(defineProps<Props>(), {
  itemHeight: 50,
  containerHeight: '400px',
  keyField: 'id',
  bufferSize: 5,
  threshold: 0.1,
  loading: false,
  hasMore: false,
  emptyText: '暂无数据',
  preloadNextPage: true,
  estimatedItemHeight: 50
})

const emit = defineEmits<{
  loadMore: []
  scroll: [event: Event]
  itemClick: [item: any, index: number]
}>()

// 响应式引用
const containerRef = ref<HTMLElement>()
const scrollTop = ref(0)
const itemsHeight = ref<Map<any, number>>(new Map())
const itemRefs = ref<HTMLElement[]>([])

const { isMobile } = useResponsive()

// 计算属性
const totalHeight = computed(() => {
  return props.items.reduce((height, item) => {
    return height + getItemHeight(item)
  }, 0)
})

const visibleHeight = computed(() => {
  return typeof props.containerHeight === 'number'
    ? props.containerHeight
    : parseInt(props.containerHeight) || 400
})

const containerHeightPx = computed(() => {
  return typeof props.containerHeight === 'number'
    ? `${props.containerHeight}px`
    : props.containerHeight
})

const getItemHeight = (item: any): number => {
  if (typeof props.itemHeight === 'function') {
    return props.itemHeight(item)
  }
  return props.itemHeight
}

const getItemKey = (item: any): string | number => {
  if (props.keyField && item[props.keyField] !== undefined) {
    return item[props.keyField]
  }
  return props.items.indexOf(item)
}

const getItemIndex = (item: any): number => {
  return props.items.indexOf(item)
}

// 计算可见范围
const visibleRange = computed(() => {
  let start = 0
  let accumulatedHeight = 0
  let end = props.items.length

  // 找到开始索引
  for (let i = 0; i < props.items.length; i++) {
    const itemHeight = getItemHeight(props.items[i])
    if (accumulatedHeight + itemHeight > scrollTop.value) {
      start = Math.max(0, i - props.bufferSize)
      break
    }
    accumulatedHeight += itemHeight
  }

  // 找到结束索引
  accumulatedHeight = 0
  for (let i = start; i < props.items.length; i++) {
    const itemHeight = getItemHeight(props.items[i])
    accumulatedHeight += itemHeight
    if (accumulatedHeight > visibleHeight.value + scrollTop.value + 100) {
      end = Math.min(props.items.length, i + props.bufferSize)
      break
    }
  }

  return { start, end }
})

const visibleItems = computed(() => {
  const { start, end } = visibleRange.value
  return props.items.slice(start, end)
})

const offsetY = computed(() => {
  let offset = 0
  for (let i = 0; i < visibleRange.value.start; i++) {
    offset += getItemHeight(props.items[i])
  }
  return offset
})

// 是否显示加载更多
const showLoadingMore = computed(() => {
  return props.hasMore && props.loading
})

// 滚动处理
const handleScroll = (event: Event) => {
  const target = event.target as HTMLElement
  scrollTop.value = target.scrollTop

  emit('scroll', event)

  // 检查是否需要加载更多
  if (props.hasMore && !props.loading) {
    const scrollThreshold = target.scrollHeight - target.clientHeight - target.scrollTop

    if (scrollThreshold < (props.estimatedItemHeight * props.bufferSize)) {
      emit('loadMore')
    }
  }
}

// 设置元素引用
const setItemRef = (el: HTMLElement | null) => {
  if (el && !itemRefs.value.includes(el)) {
    itemRefs.value.push(el)
  }
}

// 更新项目高度
const updateItemHeights = async () => {
  await nextTick()

  itemRefs.value.forEach((el, index) => {
    if (el) {
      const actualIndex = visibleRange.value.start + index
      const item = props.items[actualIndex]
      if (item) {
        const height = el.offsetHeight
        const cachedHeight = itemsHeight.value.get(item)

        // 只有当高度发生变化时才更新
        if (!cachedHeight || Math.abs(cachedHeight - height) > 1) {
          itemsHeight.value.set(item, height)
        }
      }
    }
  })

  // 清理引用
  itemRefs.value = []
}

// 滚动到指定项目
const scrollToItem = (index: number, alignment: 'start' | 'center' | 'end' = 'start') => {
  if (!containerRef.value || index < 0 || index >= props.items.length) return

  let offset = 0
  for (let i = 0; i < index; i++) {
    offset += getItemHeight(props.items[i])
  }

  const itemHeight = getItemHeight(props.items[index])
  const containerHeight = visibleHeight.value

  switch (alignment) {
    case 'start':
      break
    case 'center':
      offset -= (containerHeight - itemHeight) / 2
      break
    case 'end':
      offset -= containerHeight - itemHeight
      break
  }

  containerRef.value.scrollTop = Math.max(0, offset)
}

// 滚动到顶部
const scrollToTop = () => {
  if (containerRef.value) {
    containerRef.value.scrollTop = 0
  }
}

// 滚动到底部
const scrollToBottom = () => {
  if (containerRef.value) {
    containerRef.value.scrollTop = totalHeight.value
  }
}

// 获取滚动信息
const getScrollInfo = () => {
  return {
    scrollTop: scrollTop.value,
    scrollHeight: totalHeight.value,
    clientHeight: visibleHeight.value,
    maxScroll: Math.max(0, totalHeight.value - visibleHeight.value)
  }
}

// 监听数据变化
watch(() => props.items.length, () => {
  updateItemHeights()
})

watch(() => visibleItems.value, () => {
  nextTick(() => {
    updateItemHeights()
  })
}, { flush: 'post' })

// 移动端优化：减少渲染频率
let scrollTimer: number | null = null
const throttledHandleScroll = (event: Event) => {
  if (scrollTimer) {
    clearTimeout(scrollTimer)
  }

  scrollTimer = window.setTimeout(() => {
    handleScroll(event)
  }, isMobile.value ? 16 : 8) // 移动端约60fps，桌面端约120fps
}

onMounted(() => {
  if (isMobile.value) {
    containerRef.value?.addEventListener('scroll', throttledHandleScroll, { passive: true })
  }

  updateItemHeights()
})

onUnmounted(() => {
  if (scrollTimer) {
    clearTimeout(scrollTimer)
  }

  if (isMobile.value && containerRef.value) {
    containerRef.value.removeEventListener('scroll', throttledHandleScroll)
  }
})

// 暴露方法给父组件
defineExpose({
  scrollToItem,
  scrollToTop,
  scrollToBottom,
  getScrollInfo,
  updateItemHeights
})
</script>

<style lang="scss" scoped>
@import '@/styles/variables.scss';

.virtual-list-container {
  position: relative;
  overflow-y: auto;
  overflow-x: hidden;
  background-color: var(--bg-primary);
  border: 1px solid var(--border-color);
  border-radius: $radius-lg;

  // 自定义滚动条
  @include custom-scrollbar;

  &::-webkit-scrollbar {
    width: isMobile ? 4px : 8px;
  }

  // 移动端滚动优化
  @include mobile {
    -webkit-overflow-scrolling: touch;
    scroll-behavior: smooth;
  }
}

.virtual-list-phantom {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  z-index: -1;
  pointer-events: none;
}

.virtual-list-content {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
}

.virtual-list-item {
  position: absolute;
  left: 0;
  right: 0;
  display: flex;
  align-items: center;
  border-bottom: 1px solid var(--border-color);
  background-color: var(--card-bg);
  transition: background-color $transition-fast;

  &:hover {
    background-color: var(--hover-bg);
  }

  &:last-child {
    border-bottom: none;
  }
}

.default-item {
  padding: $spacing-md $spacing-lg;
  color: var(--text-primary);
}

.loading-more {
  position: absolute;
  left: 0;
  right: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: $spacing-sm;
  padding: $spacing-lg;
  background-color: var(--bg-secondary);
  border-top: 1px solid var(--border-color);
  color: var(--text-secondary);
  font-size: $font-size-sm;

  .loading-icon {
    animation: rotate 1s linear infinite;
  }
}

.virtual-list-empty {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: $spacing-md;
  color: var(--text-muted);
  font-size: $font-size-base;

  @include mobile {
    font-size: $font-size-sm;
  }
}

// 动画
@keyframes rotate {
  from {
    transform: rotate(0deg);
  }

  to {
    transform: rotate(360deg);
  }
}

// 移动端优化
@include mobile {
  .virtual-list-container {
    .virtual-list-item {
      min-height: 44px; // 最小触摸目标
      padding: $spacing-sm $spacing-md;

      &:active {
        background-color: var(--hover-bg);
      }
    }

    .default-item {
      padding: $spacing-sm $spacing-md;
      font-size: $font-size-sm;
    }
  }
}

// 高对比度模式支持
@media (prefers-contrast: high) {
  .virtual-list-container {
    border-color: var(--text-primary);
  }

  .virtual-list-item {
    border-bottom-color: var(--text-primary);
  }
}

// 减少动画模式支持
@media (prefers-reduced-motion: reduce) {
  .virtual-list-container {
    scroll-behavior: auto;
  }

  .loading-more .loading-icon {
    animation: none;
  }
}
</style>