<template>
  <div
    ref="containerRef"
    class="lazy-image-container"
    :class="{ 'image-loaded': loaded, 'image-error': error }"
    :style="containerStyle"
  >
    <!-- 占位符 -->
    <div
      v-if="!loaded && !error"
      class="image-placeholder"
      :style="placeholderStyle"
    >
      <el-icon v-if="showPlaceholderIcon" :size="placeholderIconSize">
        <Picture />
      </el-icon>
      <span v-if="placeholderText" class="placeholder-text">
        {{ placeholderText }}
      </span>
    </div>

    <!-- 错误状态 -->
    <div v-else-if="error" class="image-error-state">
      <el-icon :size="32">
        <WarningFilled />
      </el-icon>
      <span v-if="errorText" class="error-text">{{ errorText }}</span>
      <el-button v-if="retryable" size="small" @click="retry">
        重试
      </el-button>
    </div>

    <!-- 实际图片 -->
    <img
      v-show="loaded"
      ref="imageRef"
      :src="currentSrc"
      :alt="alt"
      :class="imageClass"
      :style="imageStyle"
      @load="handleImageLoad"
      @error="handleImageError"
      loading="lazy"
    />

    <!-- 加载进度条 -->
    <div
      v-if="showProgress && !loaded && !error"
      class="loading-progress"
      :style="{ width: `${loadProgress}%` }"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from 'vue'
import { Picture, WarningFilled } from '@element-plus/icons-vue'
import { useResponsive } from '@/composables/useResponsive'

interface Props {
  src: string
  alt?: string
  placeholder?: string
  placeholderText?: string
  placeholderIcon?: boolean
  errorText?: string
  retryable?: boolean
  width?: string | number
  height?: string | number
  objectFit?: 'cover' | 'contain' | 'fill' | 'scale-down' | 'none'
  rounded?: boolean
  lazy?: boolean
  showProgress?: boolean
  threshold?: number
  webp?: boolean
  sizes?: string
}

const props = withDefaults(defineProps<Props>(), {
  alt: '',
  placeholder: '',
  placeholderText: '加载中...',
  placeholderIcon: true,
  errorText: '图片加载失败',
  retryable: true,
  objectFit: 'cover',
  rounded: false,
  lazy: true,
  showProgress: true,
  threshold: 0.1,
  webp: true
})

const emit = defineEmits<{
  load: [event: Event]
  error: [event: Event]
  retry: []
}>()

// 响应式引用
const containerRef = ref<HTMLElement>()
const imageRef = ref<HTMLImageElement>()
const loaded = ref(false)
const error = ref(false)
const loadProgress = ref(0)
const retryCount = ref(0)
const maxRetries = 3

const { isMobile } = useResponsive()

// 计算当前图片源
const currentSrc = computed(() => {
  if (!props.src) return ''

  // WebP 支持
  if (props.webp && supportsWebP.value) {
    return convertToWebP(props.src)
  }

  return props.src
})

// WebP 支持检测
const supportsWebP = ref(false)

// 交叉观察器
let observer: IntersectionObserver | null = null
const shouldLoad = ref(!props.lazy)

// 样式计算
const containerStyle = computed(() => {
  const style: Record<string, any> = {}

  if (props.width) {
    style.width = typeof props.width === 'number' ? `${props.width}px` : props.width
  }

  if (props.height) {
    style.height = typeof props.height === 'number' ? `${props.height}px` : props.height
  }

  if (props.rounded) {
    style.borderRadius = 'var(--radius-lg)'
  }

  return style
})

const imageStyle = computed(() => ({
  objectFit: props.objectFit,
  width: '100%',
  height: '100%'
}))

const placeholderStyle = computed(() => ({
  backgroundColor: 'var(--bg-tertiary)',
  backgroundImage: props.placeholder
    ? `url(${props.placeholder})`
    : undefined,
  backgroundSize: 'cover',
  backgroundPosition: 'center',
  backgroundRepeat: 'no-repeat'
}))

const imageClass = computed(() => ({
  'rounded-image': props.rounded
}))

const placeholderIconSize = computed(() => {
  if (isMobile.value) return 24
  return 32
})

// 事件处理
const handleImageLoad = (event: Event) => {
  loaded.value = true
  error.value = false
  loadProgress.value = 100
  emit('load', event)
}

const handleImageError = (event: Event) => {
  error.value = true
  loaded.value = false
  emit('error', event)
}

const retry = () => {
  if (retryCount.value < maxRetries) {
    error.value = false
    loaded.value = false
    retryCount.value++
    loadProgress.value = 0

    // 重置图片源以重新加载
    if (imageRef.value) {
      imageRef.value.src = ''
      // 强制重新加载
      setTimeout(() => {
        if (imageRef.value) {
          imageRef.value.src = currentSrc.value
        }
      }, 100)
    }

    emit('retry')
  }
}

// WebP 转换
const convertToWebP = (src: string): string => {
  // 如果已经是 WebP 格式，直接返回
  if (src.endsWith('.webp')) return src

  // 对于支持的图片格式，尝试转换为 WebP
  const supportedFormats = ['.jpg', '.jpeg', '.png']
  const hasSupportedFormat = supportedFormats.some(format => src.toLowerCase().includes(format))

  if (hasSupportedFormat) {
    // 这里可以集成 CDN 服务来自动转换格式
    // 例如: return src.replace(/\.(jpg|jpeg|png)$/i, '.webp')
    return src // 暂时返回原图
  }

  return src
}

// 检查 WebP 支持
const checkWebPSupport = () => {
  const canvas = document.createElement('canvas')
  const ctx = canvas.getContext('2d')

  if (!ctx) {
    supportsWebP.value = false
    return
  }

  // 创建一个简单的 WebP 图片
  canvas.width = 1
  canvas.height = 1
  const webpData = canvas.toDataURL('image/webp')

  supportsWebP.value = webpData.indexOf('data:image/webp') === 0
}

// 设置交叉观察器
const setupIntersectionObserver = () => {
  if (!props.lazy || !containerRef.value) return

  observer = new IntersectionObserver(
    (entries) => {
      entries.forEach((entry) => {
        if (entry.isIntersecting) {
          shouldLoad.value = true
          observer?.unobserve(entry.target)
        }
      })
    },
    {
      threshold: props.threshold,
      rootMargin: '50px' // 提前 50px 开始加载
    }
  )

  observer.observe(containerRef.value)
}

// 模拟加载进度
const simulateProgress = () => {
  let progress = 0
  const interval = setInterval(() => {
    if (progress < 90 && !loaded.value && !error.value) {
      progress += Math.random() * 30
      loadProgress.value = Math.min(progress, 90)
    } else {
      clearInterval(interval)
    }
  }, 200)

  return () => clearInterval(interval)
}

// 监听图片源变化
watch(() => props.src, (newSrc) => {
  if (newSrc) {
    loaded.value = false
    error.value = false
    loadProgress.value = 0
  }
})

// 监听是否应该加载
watch(shouldLoad, (newValue) => {
  if (newValue && currentSrc.value && imageRef.value) {
    imageRef.value.src = currentSrc.value
    simulateProgress()
  }
})

onMounted(() => {
  checkWebPSupport()

  if (props.lazy) {
    setupIntersectionObserver()
  } else {
    shouldLoad.value = true
  }
})

onUnmounted(() => {
  if (observer) {
    observer.disconnect()
    observer = null
  }
})

// 暴露方法给父组件
defineExpose({
  retry,
  reload: () => {
    loaded.value = false
    error.value = false
    shouldLoad.value = true
  },
  isLoaded: () => loaded.value,
  hasError: () => error.value
})
</script>

<style lang="scss" scoped>
@import '@/styles/variables.scss';

.lazy-image-container {
  position: relative;
  overflow: hidden;
  background-color: var(--bg-tertiary);
  transition: all $transition-base;

  &.image-loaded {
    background-color: transparent;
  }

  &.image-error {
    background-color: var(--danger-bg);
  }

  img {
    display: block;
    transition: opacity $transition-base;
  }

  &.rounded-image {
    border-radius: $radius-lg;
  }
}

.image-placeholder {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  color: var(--text-muted);
  background-color: var(--bg-tertiary);

  .placeholder-text {
    margin-top: $spacing-sm;
    font-size: $font-size-sm;
    text-align: center;
  }
}

.image-error-state {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  color: var(--danger-color);
  background-color: var(--danger-bg);

  .error-text {
    margin: $spacing-sm 0;
    font-size: $font-size-sm;
    text-align: center;
  }
}

.loading-progress {
  position: absolute;
  bottom: 0;
  left: 0;
  height: 3px;
  background-color: var(--accent-color);
  transition: width $transition-base;
  border-radius: 0 3px 3px 0;
}

// 移动端优化
@include mobile {
  .lazy-image-container {
    .image-placeholder {
      .placeholder-text {
        font-size: $font-size-xs;
      }
    }

    .image-error-state {
      .error-text {
        font-size: $font-size-xs;
      }
    }
  }
}

// 淡入动画
@keyframes fadeIn {
  from {
    opacity: 0;
  }

  to {
    opacity: 1;
  }
}

.image-loaded img {
  animation: fadeIn 0.3s ease-out;
}
</style>