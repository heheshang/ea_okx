import { computed, ref, watch } from 'vue'
import { useResponsive } from './useResponsive'

export interface ChartResponsiveConfig {
  mobile?: {
    height?: string | number
    grid?: {
      left?: string
      right?: string
      top?: string
      bottom?: string
    }
    fontSize?: number
    symbolSize?: number
    lineStyle?: {
      width?: number
    }
    showDataZoom?: boolean
    showLegend?: boolean
    legendPosition?: 'top' | 'bottom' | 'left' | 'right'
  }
  tablet?: {
    height?: string | number
    grid?: {
      left?: string
      right?: string
      top?: string
      bottom?: string
    }
    fontSize?: number
    symbolSize?: number
  }
  desktop?: {
    height?: string | number
    grid?: {
      left?: string
      right?: string
      top?: string
      bottom?: string
    }
    fontSize?: number
    symbolSize?: number
  }
}

export interface ChartSeriesConfig {
  type: string
  smooth?: boolean
  symbol?: string | 'none' | 'circle' | 'rect' | 'triangle' | 'diamond'
  symbolSize?: number
  lineStyle?: {
    width?: number
    type?: string
  }
  areaStyle?: {
    opacity?: number
  }
  itemStyle?: {
    borderWidth?: number
    borderColor?: string
  }
}

export function useChartResponsive(
  baseOption: any = {},
  responsiveConfig: ChartResponsiveConfig = {}
) {
  const { isMobile, isTablet, currentBreakpoint } = useResponsive()
  const chartInstance = ref<any>(null)

  // 默认响应式配置
  const defaultResponsiveConfig: ChartResponsiveConfig = {
    mobile: {
      height: 300,
      grid: {
        left: '8%',
        right: '5%',
        top: '15%',
        bottom: '20%'
      },
      fontSize: 10,
      symbolSize: 4,
      lineStyle: {
        width: 2
      },
      showDataZoom: true,
      showLegend: false,
      legendPosition: 'bottom'
    },
    tablet: {
      height: 350,
      grid: {
        left: '6%',
        right: '4%',
        top: '12%',
        bottom: '15%'
      },
      fontSize: 11,
      symbolSize: 5
    },
    desktop: {
      height: 400,
      grid: {
        left: '3%',
        right: '3%',
        top: '8%',
        bottom: '10%'
      },
      fontSize: 12,
      symbolSize: 6
    }
  }

  // 合并配置
  const config = {
    mobile: { ...defaultResponsiveConfig.mobile, ...responsiveConfig.mobile },
    tablet: { ...defaultResponsiveConfig.tablet, ...responsiveConfig.tablet },
    desktop: { ...defaultResponsiveConfig.desktop, ...responsiveConfig.desktop }
  }

  // 获取当前断点配置
  const currentConfig = computed(() => {
    if (isMobile.value) return config.mobile
    if (isTablet.value) return config.tablet
    return config.desktop
  })

  // 响应式图表高度
  const chartHeight = computed(() => {
    const height = currentConfig.value?.height || 400
    return typeof height === 'number' ? `${height}px` : height
  })

  // 响应式网格配置
  const responsiveGrid = computed(() => {
    return {
      ...baseOption.grid,
      ...currentConfig.value?.grid
    }
  })

  // 响应式字体大小
  const responsiveText = computed(() => ({
    textStyle: {
      fontSize: currentConfig.value?.fontSize || 12,
      color: 'var(--text-secondary)'
    }
  }))

  // 响应式系列配置
  const getResponsiveSeries = (series: ChartSeriesConfig[]): ChartSeriesConfig[] => {
    return series.map(item => ({
      ...item,
      symbol: isMobile.value && item.symbol !== 'none' ? 'none' : item.symbol,
      symbolSize: isMobile.value
        ? (currentConfig.value?.symbolSize || 4)
        : (item.symbolSize || 6),
      lineStyle: {
        ...item.lineStyle,
        width: isMobile.value
          ? (currentConfig.value?.lineStyle?.width || 2)
          : (item.lineStyle?.width || 3)
      }
    }))
  }

  // 响应式提示框配置
  const responsiveTooltip = computed(() => ({
    trigger: 'axis',
    backgroundColor: 'var(--bg-secondary)',
    borderColor: 'var(--border-color)',
    textStyle: {
      color: 'var(--text-primary)',
      fontSize: currentConfig.value?.fontSize || 12
    },
    extraCssText: 'border-radius: 8px; box-shadow: 0 4px 12px rgba(0,0,0,0.1);',
    ...baseOption.tooltip,
    // 移动端优化
    ...(isMobile.value && {
      confine: true,
      appendToBody: true,
      hideDelay: 100,
      enterable: false
    })
  }))

  // 响应式图例配置
  const responsiveLegend = computed(() => {
    const showLegend = currentConfig.value?.showLegend !== false
    const legendPosition = currentConfig.value?.legendPosition || 'top'

    return {
      show: showLegend,
      type: isMobile.value ? 'scroll' : 'plain',
      orient: legendPosition === 'left' || legendPosition === 'right' ? 'vertical' : 'horizontal',
      ...((legendPosition === 'top' || legendPosition === 'bottom') ? {
        top: legendPosition === 'top' ? 0 : 'auto',
        bottom: legendPosition === 'bottom' ? 0 : 'auto'
      } : {
        left: legendPosition === 'left' ? 0 : 'auto',
        right: legendPosition === 'right' ? 0 : 'auto'
      }),
      textStyle: {
        fontSize: currentConfig.value?.fontSize || 12,
        color: 'var(--text-secondary)'
      },
      ...baseOption.legend
    }
  })

  // 响应式数据缩放配置
  const responsiveDataZoom = computed(() => {
    const showDataZoom = currentConfig.value?.showDataZoom || false

    if (!showDataZoom || isMobile.value) {
      return undefined
    }

    return [
      {
        type: 'inside',
        xAxisIndex: 0,
        filterMode: 'none',
        throttle: 50
      },
      {
        type: 'slider',
        xAxisIndex: 0,
        filterMode: 'none',
        height: 20,
        bottom: 10,
        handleStyle: {
          color: 'var(--accent-color)'
        },
        textStyle: {
          color: 'var(--text-secondary)'
        }
      }
    ]
  })

  // 移动端触摸优化配置
  const mobileTouchConfig = computed(() => {
    if (!isMobile.value) return {}

    return {
      brush: {
        toolbox: ['lineX', 'clear'],
        xAxisIndex: 0
      },
      dataZoom: [
        {
          type: 'inside',
          xAxisIndex: 0,
          filterMode: 'none',
          minSpan: 10,
          maxSpan: 100
        }
      ]
    }
  })

  // 完整的响应式配置
  const responsiveChartOption = computed(() => {
    return {
      ...baseOption,
      grid: responsiveGrid.value,
      textStyle: responsiveText.value.textStyle,
      tooltip: responsiveTooltip.value,
      legend: responsiveLegend.value,
      dataZoom: responsiveDataZoom.value,
      series: baseOption.series ? getResponsiveSeries(baseOption.series) : [],
      ...mobileTouchConfig.value,
      // 添加主题色彩
      color: [
        'var(--accent-color)',
        'var(--success-color)',
        'var(--warning-color)',
        'var(--danger-color)',
        'var(--info-color)'
      ]
    }
  })

  // 监听断点变化，重新渲染图表
  watch(currentBreakpoint, () => {
    if (chartInstance.value) {
      chartInstance.value.setOption(responsiveChartOption.value, true)
    }
  }, { flush: 'post' })

  // 图表实例注册
  const registerChartInstance = (instance: any) => {
    chartInstance.value = instance
    if (instance) {
      instance.setOption(responsiveChartOption.value)
    }
  }

  // 性能优化：防抖重绘
  const debounceResize = (() => {
    let timeoutId: number | null = null
    return () => {
      if (timeoutId) clearTimeout(timeoutId)
      timeoutId = window.setTimeout(() => {
        if (chartInstance.value) {
          chartInstance.value.resize()
        }
      }, 150)
    }
  })()

  return {
    chartHeight,
    responsiveChartOption,
    registerChartInstance,
    resizeChart: debounceResize,
    currentConfig,
    isMobile,
    isTablet
  }
}

/**
 * 交易图表专用配置
 */
export function useTradingChartConfig() {
  const baseOption = {
    animation: false, // 交易图表禁用动画以提升性能
    backgroundColor: 'transparent',
    xAxis: {
      type: 'category',
      boundaryGap: false,
      axisLine: {
        lineStyle: {
          color: 'var(--border-color)'
        }
      },
      axisTick: {
        show: false
      },
      axisLabel: {
        color: 'var(--text-muted)'
      }
    },
    yAxis: {
      type: 'value',
      axisLine: {
        show: false
      },
      axisTick: {
        show: false
      },
      axisLabel: {
        color: 'var(--text-muted)'
      },
      splitLine: {
        lineStyle: {
          color: 'var(--border-color)',
          type: 'dashed'
        }
      }
    }
  }

  return useChartResponsive(baseOption, {
    mobile: {
      height: 250,
      showDataZoom: true,
      fontSize: 9,
      symbolSize: 3,
      lineStyle: { width: 1.5 }
    },
    desktop: {
      height: 450,
      showDataZoom: true,
      symbolSize: 5,
      lineStyle: { width: 2 }
    }
  })
}

/**
 * 仪表板图表配置
 */
export function useDashboardChartConfig() {
  return useChartResponsive({
    animation: true,
    animationDuration: 300,
    backgroundColor: 'transparent'
  }, {
    mobile: {
      height: 280,
      showLegend: false,
      fontSize: 10
    },
    desktop: {
      height: 350,
      showLegend: true,
      legendPosition: 'top'
    }
  })
}