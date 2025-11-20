<template>
  <div
    ref="chartContainerRef"
    class="responsive-chart-container"
    :class="chartContainerClass"
    :style="{ height: chartHeight }"
  >
    <!-- 加载状态 -->
    <div v-if="loading" class="chart-loading">
      <el-icon class="loading-icon" :size="32">
        <Loading />
      </el-icon>
      <span class="loading-text">{{ loadingText }}</span>
    </div>

    <!-- 错误状态 -->
    <div v-else-if="error" class="chart-error">
      <el-icon class="error-icon" :size="32">
        <Warning />
      </el-icon>
      <span class="error-text">{{ errorMessage }}</span>
      <el-button size="small" type="primary" @click="handleRetry">
        重试
      </el-button>
    </div>

    <!-- 空数据状态 -->
    <div v-else-if="!hasData" class="chart-empty">
      <el-icon class="empty-icon" :size="48">
        <TrendCharts />
      </el-icon>
      <span class="empty-text">{{ emptyText }}</span>
    </div>

    <!-- 图表内容 -->
    <div
      v-else
      ref="chartRef"
      class="chart-content"
      :class="{ 'chart-rendered': isRendered }"
    />

    <!-- 移动端触摸指示器 -->
    <div
      v-if="isMobile && showTouchIndicator"
      class="touch-indicator"
      :class="{ active: isTouching }"
    >
      <el-icon><ArrowRight /></el-icon>
      <span>滑动查看详情</span>
    </div>

    <!-- 图表工具栏 -->
    <div v-if="showToolbar" class="chart-toolbar">
      <el-button-group size="small">
        <el-button
          v-for="tool in toolbarTools"
          :key="tool.key"
          :icon="tool.icon"
          :type="activeTool === tool.key ? 'primary' : 'default'"
          @click="handleToolClick(tool.key)"
        >
          <span v-if="!isMobile">{{ tool.label }}</span>
        </el-button>
      </el-button-group>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted, nextTick } from 'vue'
import { Loading, Warning, TrendCharts, ArrowRight, Download, Refresh, ZoomIn, ZoomOut } from '@element-plus/icons-vue'
import { useChartResponsive, useTradingChartConfig } from '@/composables/useChartResponsive'
import { useResponsive } from '@/composables/useResponsive'

interface ChartData {
  xAxis: string[]
  series: Array<{
    name: string
    data: number[]
    type?: string
    color?: string
  }>
}

interface Props {
  data?: ChartData
  type?: 'line' | 'bar' | 'candlestick' | 'area'
  loading?: boolean
  error?: boolean
  errorMessage?: string
  emptyText?: string
  loadingText?: string
  height?: string | number
  showToolbar?: boolean
  autoResize?: boolean
  theme?: 'trading' | 'dashboard' | 'default'
}

const props = withDefaults(defineProps<Props>(), {
  type: 'line',
  loading: false,
  error: false,
  errorMessage: '图表加载失败',
  emptyText: '暂无数据',
  loadingText: '加载中...',
  showToolbar: true,
  autoResize: true,
  theme: 'default'
})

const emit = defineEmits<{
  retry: []
  toolChange: [tool: string]
  export: []
}>()

// 响应式和引用
const { isMobile, isTablet } = useResponsive()
const chartRef = ref<HTMLElement>()
const chartInstance = ref<any>(null)
const isRendered = ref(false)
const isTouching = ref(false)
const activeTool = ref('pan')

// 图表配置
const chartConfig = computed(() => {
  const data = props.data

  if (!data || !data.xAxis || !data.series) {
    return {}
  }

  const baseConfig = {
    xAxis: {
      type: 'category',
      data: data.xAxis,
      axisLabel: {
        fontSize: isMobile.value ? 10 : 12,
        color: 'var(--text-secondary)',
        rotate: isMobile.value ? 45 : 0
      }
    },
    yAxis: {
      type: 'value',
      axisLabel: {
        fontSize: isMobile.value ? 10 : 12,
        color: 'var(--text-secondary)'
      }
    },
    series: data.series.map((item, index) => ({
      name: item.name,
      type: item.type || props.type,
      data: item.data,
      smooth: true,
      symbol: isMobile.value ? 'none' : 'circle',
      symbolSize: isMobile.value ? 3 : 5,
      lineStyle: {
        width: isMobile.value ? 2 : 3,
        color: item.color || undefined
      },
      itemStyle: {
        color: item.color || undefined
      },
      areaStyle: props.type === 'area' ? {
        opacity: 0.3
      } : undefined
    }))
  }

  return baseConfig
})

// 使用响应式图表配置
const {
  chartHeight,
  responsiveChartOption,
  registerChartInstance,
  resizeChart
} = useChartResponsive(chartConfig.value, {
  mobile: {
    height: props.height || 280,
    showLegend: false,
    showDataZoom: true,
    fontSize: 10
  },
  tablet: {
    height: props.height || 320,
    showLegend: true,
    legendPosition: 'bottom'
  },
  desktop: {
    height: props.height || 400,
    showLegend: true,
    legendPosition: 'top'
  }
})

// 计算属性
const hasData = computed(() => {
  return props.data && props.data.series && props.data.series.length > 0
})

const chartContainerClass = computed(() => ({
  'is-mobile': isMobile.value,
  'is-tablet': isTablet.value,
  'is-loading': props.loading,
  'has-error': props.error,
  'is-empty': !hasData.value,
  'touch-enabled': isMobile.value
}))

const showTouchIndicator = computed(() => {
  return isMobile.value && hasData.value && !props.loading && !props.error
})

// 工具栏配置
const toolbarTools = computed(() => {
  const tools = [
    { key: 'pan', label: '平移', icon: ArrowRight },
    { key: 'refresh', label: '刷新', icon: Refresh }
  ]

  if (!isMobile.value) {
    tools.push(
      { key: 'zoom-in', label: '放大', icon: ZoomIn },
      { key: 'zoom-out', label: '缩小', icon: ZoomOut },
      { key: 'export', label: '导出', icon: Download }
    )
  }

  return tools
})

// 事件处理
const handleRetry = () => {
  emit('retry')
}

const handleToolClick = (tool: string) => {
  activeTool.value = tool

  switch (tool) {
    case 'refresh':
      emit('retry')
      break
    case 'export':
      emit('export')
      break
    case 'zoom-in':
      if (chartInstance.value) {
        chartInstance.value.dispatchAction({
          type: 'dataZoom',
          start: 20,
          end: 80
        })
      }
      break
    case 'zoom-out':
      if (chartInstance.value) {
        chartInstance.value.dispatchAction({
          type: 'dataZoom',
          start: 0,
          end: 100
        })
      }
      break
  }

  emit('toolChange', tool)
}

// 触摸事件处理
const handleTouchStart = () => {
  isTouching.value = true
}

const handleTouchEnd = () => {
  setTimeout(() => {
    isTouching.value = false
  }, 1000)
}

// 初始化图表
const initChart = async () => {
  if (!chartRef.value || !hasData.value) return

  try {
    // 动态导入 ECharts
    const { init } = await import('echarts')

    // 创建图表实例
    const instance = init(chartRef.value)

    // 设置配置
    instance.setOption(responsiveChartOption.value)

    // 注册实例
    registerChartInstance(instance)

    chartInstance.value = instance
    isRendered.value = true

    // 添加事件监听
    if (isMobile.value) {
      chartRef.value.addEventListener('touchstart', handleTouchStart)
      chartRef.value.addEventListener('touchend', handleTouchEnd)
    }

  } catch (error) {
    console.error('图表初始化失败:', error)
  }
}

// 监听数据变化
watch(() => props.data, async (newData) => {
  if (newData && hasData.value) {
    await nextTick()
    if (!chartInstance.value) {
      await initChart()
    } else {
      chartInstance.value.setOption(responsiveChartOption.value, true)
    }
  }
}, { deep: true, immediate: true })

// 窗口大小变化处理
const handleResize = () => {
  if (chartInstance.value && props.autoResize) {
    resizeChart()
  }
}

onMounted(async () => {
  if (hasData.value && !props.loading) {
    await nextTick()
    await initChart()
  }

  window.addEventListener('resize', handleResize)
})

onUnmounted(() => {
  if (chartInstance.value) {
    chartInstance.value.dispose()
    chartInstance.value = null
  }

  if (chartRef.value) {
    chartRef.value.removeEventListener('touchstart', handleTouchStart)
    chartRef.value.removeEventListener('touchend', handleTouchEnd)
  }

  window.removeEventListener('resize', handleResize)
})

// 暴露方法给父组件
defineExpose({
  getChartInstance: () => chartInstance.value,
  resizeChart,
  exportChart: () => {
    if (chartInstance.value) {
      const url = chartInstance.value.getDataURL({
        type: 'png',
        backgroundColor: 'var(--bg-primary)'
      })
      const a = document.createElement('a')
      a.href = url
      a.download = `chart-${Date.now()}.png`
      a.click()
    }
  }
})
</script>

<style lang="scss" scoped>
@import '@/styles/variables.scss';

.responsive-chart-container {
  position: relative;
  width: 100%;
  background-color: var(--card-bg);
  border-radius: $radius-lg;
  border: 1px solid var(--border-color);
  overflow: hidden;

  &.is-mobile {
    border-radius: $radius-md;
    touch-action: pan-x pan-y;
  }

  .chart-content {
    width: 100%;
    height: 100%;
    opacity: 0;
    transition: opacity $transition-base;

    &.chart-rendered {
      opacity: 1;
    }
  }
}

// 状态显示
.chart-loading,
.chart-error,
.chart-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  padding: $spacing-2xl;
  text-align: center;
  color: var(--text-secondary);

  .loading-icon {
    animation: rotate 2s linear infinite;
    color: var(--accent-color);
  }

  .error-icon {
    color: var(--danger-color);
  }

  .empty-icon {
    color: var(--text-muted);
  }

  .loading-text,
  .error-text,
  .empty-text {
    margin-top: $spacing-md;
    font-size: $font-size-sm;
    color: var(--text-secondary);
  }
}

// 触摸指示器
.touch-indicator {
  position: absolute;
  bottom: $spacing-lg;
  left: 50%;
  transform: translateX(-50%);
  display: flex;
  align-items: center;
  gap: $spacing-sm;
  padding: $spacing-sm $spacing-md;
  background-color: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: $radius-full;
  color: var(--text-secondary);
  font-size: $font-size-xs;
  opacity: 0;
  transition: opacity $transition-base;
  pointer-events: none;

  &.active {
    opacity: 1;
    animation: touch-pulse 2s infinite;
  }
}

@keyframes touch-pulse {
  0%, 100% {
    opacity: 0.8;
  }

  50% {
    opacity: 1;
  }
}

// 图表工具栏
.chart-toolbar {
  position: absolute;
  top: $spacing-md;
  right: $spacing-md;
  z-index: $z-sticky;

  @include mobile {
    top: $spacing-sm;
    right: $spacing-sm;
  }
}

// 工具栏按钮优化
:deep(.el-button-group) {
  .el-button {
    background-color: var(--bg-secondary);
    border-color: var(--border-color);
    color: var(--text-secondary);

    &:hover {
      background-color: var(--hover-bg);
      color: var(--text-primary);
    }

    &.el-button--primary {
      background-color: var(--accent-color);
      border-color: var(--accent-color);
      color: white;
    }
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
  .responsive-chart-container {
    .chart-toolbar {
      .el-button-group {
        .el-button {
          padding: $spacing-xs $spacing-sm;
          font-size: $font-size-xs;

          .btn-text-mobile-hide {
            display: none;
          }
        }
      }
    }
  }
}

// 平板端优化
@include tablet {
  .responsive-chart-container {
    .chart-toolbar {
      .el-button-group {
        .el-button {
          padding: $spacing-sm $spacing-md;
        }
      }
    }
  }
}
</style>