<template>
  <el-card
    class="strategy-card"
    :class="cardClasses"
    shadow="hover"
    @click="handleCardClick"
  >
    <!-- Card Header -->
    <template #header>
      <div class="card-header">
        <div class="strategy-info">
          <div class="strategy-title">
            <h3>{{ strategy.name }}</h3>
            <div class="strategy-meta">
              <el-tag :type="getStatusType(strategy.status)" size="small" class="status-tag">
                {{ getStatusText(strategy.status) }}
              </el-tag>
              <el-tag :type="getTypeColor(strategy.type)" size="small" class="type-tag">
                {{ getTypeText(strategy.type) }}
              </el-tag>
              <el-tag v-if="strategy.isFavorite" type="warning" size="small" class="favorite-tag">
                <el-icon><Star /></el-icon>
                Favorite
              </el-tag>
            </div>
          </div>
          <div class="strategy-description">
            {{ strategy.description || 'No description available' }}
          </div>
        </div>

        <!-- Action Menu -->
        <el-dropdown trigger="click" @click.stop>
          <el-button class="more-actions" :icon="MoreFilled" circle size="small" />
          <template #dropdown>
            <el-dropdown-menu>
              <el-dropdown-item @click="handleView">
                <el-icon><View /></el-icon>
                View Details
              </el-dropdown-item>
              <el-dropdown-item @click="handleEdit">
                <el-icon><Edit /></el-icon>
                Edit
              </el-dropdown-item>
              <el-dropdown-item @click="handleDuplicate">
                <el-icon><CopyDocument /></el-icon>
                Duplicate
              </el-dropdown-item>
              <el-dropdown-item @click="toggleFavorite">
                <el-icon>
                  <Star v-if="!strategy.isFavorite" />
                  <StarFilled v-else />
                </el-icon>
                {{ strategy.isFavorite ? 'Remove Favorite' : 'Add Favorite' }}
              </el-dropdown-item>
              <el-dropdown-item divided @click="handleExport">
                <el-icon><Download /></el-icon>
                Export
              </el-dropdown-item>
              <el-dropdown-item @click="handleDelete" class="danger-item">
                <el-icon><Delete /></el-icon>
                Delete
              </el-dropdown-item>
            </el-dropdown-menu>
          </template>
        </el-dropdown>
      </div>
    </template>

    <!-- Card Content -->
    <div class="card-content">
      <!-- Strategy Info Grid -->
      <div class="info-grid">
        <div class="info-item">
          <span class="info-label">Symbol</span>
          <span class="info-value symbol">{{ strategy.symbol }}</span>
        </div>
        <div class="info-item">
          <span class="info-label">Capital</span>
          <span class="info-value">${{ formatCurrency(strategy.capital) }}</span>
        </div>
        <div class="info-item">
          <span class="info-label">Trades</span>
          <span class="info-value">{{ strategy.performance.totalTrades }}</span>
        </div>
        <div class="info-item">
          <span class="info-label">Created</span>
          <span class="info-value">{{ formatDate(strategy.createdAt) }}</span>
        </div>
      </div>

      <!-- Performance Stats -->
      <div class="performance-stats">
        <div class="stat-item">
          <div class="stat-header">
            <span class="stat-label">P&L</span>
            <el-tooltip :content="getPnLTooltip()" placement="top">
              <el-icon class="info-icon"><InfoFilled /></el-icon>
            </el-tooltip>
          </div>
          <div class="stat-value" :class="getPnLClass(strategy.performance.totalPnL)">
            {{ formatPnL(strategy.performance.totalPnL) }}
          </div>
          <div class="stat-change" :class="getReturnClass(strategy.performance.totalReturn)">
            ({{ formatReturn(strategy.performance.totalReturn) }})
          </div>
        </div>

        <div class="stat-divider"></div>

        <div class="stat-item">
          <div class="stat-header">
            <span class="stat-label">Win Rate</span>
            <el-tooltip :content="getWinRateTooltip()" placement="top">
              <el-icon class="info-icon"><InfoFilled /></el-icon>
            </el-tooltip>
          </div>
          <div class="stat-value">
            {{ formatPercentage(strategy.performance.winRate) }}
          </div>
          <div class="stat-details">
            {{ strategy.performance.winningTrades }}W / {{ strategy.performance.losingTrades }}L
          </div>
        </div>

        <div class="stat-divider"></div>

        <div class="stat-item">
          <div class="stat-header">
            <span class="stat-label">Sharpe</span>
            <el-tooltip :content="getSharpeTooltip()" placement="top">
              <el-icon class="info-icon"><InfoFilled /></el-icon>
            </el-tooltip>
          </div>
          <div class="stat-value" :class="getSharpeClass(strategy.performance.sharpeRatio)">
            {{ formatNumber(strategy.performance.sharpeRatio) }}
          </div>
          <div class="stat-details">
            Risk-adjusted returns
          </div>
        </div>

        <div class="stat-divider"></div>

        <div class="stat-item">
          <div class="stat-header">
            <span class="stat-label">Max DD</span>
            <el-tooltip :content="getDrawdownTooltip()" placement="top">
              <el-icon class="info-icon"><InfoFilled /></el-icon>
            </el-tooltip>
          </div>
          <div class="stat-value negative">
            {{ formatPercentage(strategy.performance.maxDrawdown) }}
          </div>
          <div class="stat-details">
            Max drawdown
          </div>
        </div>
      </div>

      <!-- Mini Chart (if available) -->
      <div v-if="showMiniChart" class="mini-chart-container">
        <div ref="chartRef" class="mini-chart"></div>
      </div>

      <!-- Tags -->
      <div v-if="strategy.tags.length > 0" class="strategy-tags">
        <el-tag
          v-for="tag in strategy.tags.slice(0, 3)"
          :key="tag"
          size="small"
          type="info"
          class="tag"
        >
          {{ tag }}
        </el-tag>
        <el-tag
          v-if="strategy.tags.length > 3"
          size="small"
          type="info"
          class="tag"
        >
          +{{ strategy.tags.length - 3 }}
        </el-tag>
      </div>
    </div>

    <!-- Card Footer -->
    <template #footer>
      <div class="card-footer">
        <!-- Status Actions -->
        <div class="status-actions">
          <el-button
            v-if="canStart"
            type="success"
            size="small"
            @click="handleStart"
            @click.stop
            :loading="actionLoading"
          >
            <el-icon><VideoPlay /></el-icon>
            Start
          </el-button>

          <el-button
            v-if="canPause"
            type="warning"
            size="small"
            @click="handlePause"
            @click.stop
            :loading="actionLoading"
          >
            <el-icon><VideoPause /></el-icon>
            Pause
          </el-button>

          <el-button
            v-if="canStop"
            type="danger"
            size="small"
            @click="handleStop"
            @click.stop
            :loading="actionLoading"
          >
            <el-icon><CircleClose /></el-icon>
            Stop
          </el-button>

          <el-button
            v-if="canResume"
            type="primary"
            size="small"
            @click="handleResume"
            @click.stop
            :loading="actionLoading"
          >
            <el-icon><VideoPlay /></el-icon>
            Resume
          </el-button>
        </div>

        <!-- Quick Stats -->
        <div class="quick-stats">
          <div class="quick-stat">
            <el-icon><TrendCharts /></el-icon>
            <span>{{ formatCurrency(strategy.currentEquity) }}</span>
          </div>
          <div class="quick-stat">
            <el-icon><Clock /></el-icon>
            <span>{{ formatLastActive(strategy.lastActiveAt) }}</span>
          </div>
        </div>
      </div>
    </template>
  </el-card>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { useRouter } from 'vue-router'
import { ElMessage, ElMessageBox } from 'element-plus'
import dayjs from 'dayjs'
import relativeTime from 'dayjs/plugin/relativeTime'
import {
  MoreFilled,
  View,
  Edit,
  Delete,
  Star,
  StarFilled,
  CopyDocument,
  Download,
  VideoPlay,
  VideoPause,
  CircleClose,
  InfoFilled,
  TrendCharts,
  Clock
} from '@element-plus/icons-vue'

import type { Strategy, StrategyStatus, StrategyType } from '@/types/strategy'
import { strategyService } from '@/services/strategyService'
import { useResponsive } from '@/composables/useResponsive'
import { useChartResponsive } from '@/composables/useChartResponsive'

dayjs.extend(relativeTime)

interface Props {
  strategy: Strategy
  showMiniChart?: boolean
  compact?: boolean
}

interface Emits {
  'view': [strategy: Strategy]
  'edit': [strategy: Strategy]
  'delete': [strategy: Strategy]
  'duplicate': [strategy: Strategy]
  'favorite': [strategy: Strategy]
  'export': [strategy: Strategy]
  'start': [strategy: Strategy]
  'stop': [strategy: Strategy]
  'pause': [strategy: Strategy]
  'resume': [strategy: Strategy]
}

const props = withDefaults(defineProps<Props>(), {
  showMiniChart: true,
  compact: false
})

const emit = defineEmits<Emits>()

const router = useRouter()
const { isMobile } = useResponsive()
const chartRef = ref<HTMLElement>()

const actionLoading = ref(false)

// Computed properties
const cardClasses = computed(() => ({
  'strategy-card': true,
  'is-mobile': isMobile.value,
  'is-compact': props.compact,
  'status-active': props.strategy.status === 'active',
  'status-paused': props.strategy.status === 'paused',
  'status-draft': props.strategy.status === 'draft',
  'status-stopped': props.strategy.status === 'stopped',
  'status-error': props.strategy.status === 'error'
}))

const canStart = computed(() =>
  ['draft', 'paused', 'stopped'].includes(props.strategy.status)
)

const canPause = computed(() =>
  props.strategy.status === 'active'
)

const canStop = computed(() =>
  ['active', 'paused'].includes(props.strategy.status)
)

const canResume = computed(() =>
  props.strategy.status === 'paused'
)

// Mini chart setup
const { chartHeight, responsiveChartOption, registerChartInstance } = useChartResponsive(
  {
    grid: { top: 0, left: 0, right: 0, bottom: 0 },
    xAxis: { show: false },
    yAxis: { show: false },
    series: [{
      type: 'line',
      data: props.strategy.performance.equityCurve?.map(point => [point.timestamp, point.equity]) || [],
      smooth: true,
      symbol: 'none',
      lineStyle: {
        width: 2,
        color: props.strategy.performance.totalPnL >= 0 ? '#57ab5a' : '#e5534b'
      },
      areaStyle: {
        opacity: 0.1,
        color: props.strategy.performance.totalPnL >= 0 ? '#57ab5a' : '#e5534b'
      }
    }]
  },
  {
    mobile: {
      height: 60
    },
    desktop: {
      height: 80
    }
  }
)

// Initialize mini chart
onMounted(() => {
  if (props.showMiniChart && chartRef.value && props.strategy.performance.equityCurve?.length > 0) {
    import('echarts').then(({ init }) => {
      const chart = init(chartRef.value)
      chart.setOption(responsiveChartOption.value)
      registerChartInstance(chart)
    })
  }
})

// Methods
const handleCardClick = () => {
  emit('view', props.strategy)
}

const handleView = () => {
  emit('view', props.strategy)
}

const handleEdit = () => {
  emit('edit', props.strategy)
}

const handleDelete = async () => {
  try {
    await ElMessageBox.confirm(
      `Are you sure you want to delete strategy "${props.strategy.name}"? This action cannot be undone.`,
      'Delete Strategy',
      {
        type: 'warning',
        confirmButtonText: 'Delete',
        confirmButtonClass: 'el-button--danger',
        cancelButtonText: 'Cancel'
      }
    )

    const response = await strategyService.deleteStrategy(props.strategy.id)
    if (response.success) {
      ElMessage.success('Strategy deleted successfully')
      emit('delete', props.strategy)
    } else {
      ElMessage.error(response.error || 'Failed to delete strategy')
    }
  } catch (error) {
    if (error !== 'cancel') {
      ElMessage.error('Failed to delete strategy')
    }
  }
}

const handleDuplicate = async () => {
  try {
    const response = await strategyService.duplicateStrategy(props.strategy.id)
    if (response.success) {
      ElMessage.success('Strategy duplicated successfully')
      emit('duplicate', props.strategy)
    } else {
      ElMessage.error(response.error || 'Failed to duplicate strategy')
    }
  } catch (error) {
    ElMessage.error('Failed to duplicate strategy')
  }
}

const toggleFavorite = async () => {
  try {
    const response = await strategyService.updateStrategy(props.strategy.id, {
      ...props.strategy,
      isFavorite: !props.strategy.isFavorite
    })
    if (response.success) {
      ElMessage.success(props.strategy.isFavorite ? 'Removed from favorites' : 'Added to favorites')
      emit('favorite', response.data!)
    } else {
      ElMessage.error(response.error || 'Failed to update favorite status')
    }
  } catch (error) {
    ElMessage.error('Failed to update favorite status')
  }
}

const handleExport = async () => {
  try {
    const response = await strategyService.exportStrategies([props.strategy.id])
    if (response.success) {
      // Download file
      const url = URL.createObjectURL(response.data!)
      const a = document.createElement('a')
      a.href = url
      a.download = `strategy-${props.strategy.name}-${Date.now()}.json`
      a.click()
      URL.revokeObjectURL(url)

      ElMessage.success('Strategy exported successfully')
      emit('export', props.strategy)
    } else {
      ElMessage.error(response.error || 'Failed to export strategy')
    }
  } catch (error) {
    ElMessage.error('Failed to export strategy')
  }
}

const handleStart = async () => {
  actionLoading.value = true
  try {
    const response = await strategyService.startStrategy(props.strategy.id)
    if (response.success) {
      ElMessage.success('Strategy started successfully')
      emit('start', { ...props.strategy, status: 'active' })
    } else {
      ElMessage.error(response.error || 'Failed to start strategy')
    }
  } catch (error) {
    ElMessage.error('Failed to start strategy')
  } finally {
    actionLoading.value = false
  }
}

const handlePause = async () => {
  actionLoading.value = true
  try {
    const response = await strategyService.pauseStrategy(props.strategy.id)
    if (response.success) {
      ElMessage.success('Strategy paused successfully')
      emit('pause', { ...props.strategy, status: 'paused' })
    } else {
      ElMessage.error(response.error || 'Failed to pause strategy')
    }
  } catch (error) {
    ElMessage.error('Failed to pause strategy')
  } finally {
    actionLoading.value = false
  }
}

const handleStop = async () => {
  actionLoading.value = true
  try {
    const response = await strategyService.stopStrategy(props.strategy.id)
    if (response.success) {
      ElMessage.success('Strategy stopped successfully')
      emit('stop', { ...props.strategy, status: 'stopped' })
    } else {
      ElMessage.error(response.error || 'Failed to stop strategy')
    }
  } catch (error) {
    ElMessage.error('Failed to stop strategy')
  } finally {
    actionLoading.value = false
  }
}

const handleResume = async () => {
  actionLoading.value = true
  try {
    const response = await strategyService.resumeStrategy(props.strategy.id)
    if (response.success) {
      ElMessage.success('Strategy resumed successfully')
      emit('resume', { ...props.strategy, status: 'active' })
    } else {
      ElMessage.error(response.error || 'Failed to resume strategy')
    }
  } catch (error) {
    ElMessage.error('Failed to resume strategy')
  } finally {
    actionLoading.value = false
  }
}

// Formatting helpers
const getStatusType = (status: StrategyStatus): 'primary' | 'success' | 'warning' | 'info' | 'danger' => {
  const types: Record<StrategyStatus, 'primary' | 'success' | 'warning' | 'info' | 'danger'> = {
    active: 'success',
    draft: 'info',
    paused: 'warning',
    stopped: 'danger',
    error: 'danger'
  }
  return types[status] || 'info'
}

const getStatusText = (status: StrategyStatus) => {
  const texts: Record<StrategyStatus, string> = {
    active: 'Active',
    draft: 'Draft',
    paused: 'Paused',
    stopped: 'Stopped',
    error: 'Error'
  }
  return texts[status] || status
}

const getTypeColor = (type: StrategyType): 'primary' | 'success' | 'warning' | 'info' | 'danger' => {
  const colors: Record<StrategyType, 'primary' | 'success' | 'warning' | 'info' | 'danger'> = {
    ma_crossover: 'primary',
    grid_trading: 'success',
    rsi_strategy: 'warning',
    market_making: 'info',
    arbitrage: 'danger',
    custom: 'primary' // Default to primary for custom strategies
  }
  return colors[type] || 'primary'
}

const getTypeText = (type: StrategyType) => {
  const texts: Record<StrategyType, string> = {
    ma_crossover: 'MA Crossover',
    grid_trading: 'Grid Trading',
    rsi_strategy: 'RSI Strategy',
    market_making: 'Market Making',
    arbitrage: 'Arbitrage',
    custom: 'Custom'
  }
  return texts[type] || type
}

const formatCurrency = (value: number) => {
  return new Intl.NumberFormat('en-US', {
    style: 'currency',
    currency: 'USD',
    minimumFractionDigits: 0,
    maximumFractionDigits: 0
  }).format(value)
}

const formatPnL = (value: number) => {
  const sign = value >= 0 ? '+' : ''
  return `${sign}${formatCurrency(Math.abs(value))}`
}

const formatReturn = (value: number) => {
  const sign = value >= 0 ? '+' : ''
  return `${sign}${(value * 100).toFixed(2)}%`
}

const formatPercentage = (value: number) => {
  return `${(value * 100).toFixed(1)}%`
}

const formatNumber = (value: number) => {
  return value.toFixed(2)
}

const formatDate = (dateString: string) => {
  return dayjs(dateString).format('MMM DD, YYYY')
}

const formatLastActive = (dateString?: string) => {
  if (!dateString) return 'Never'
  return dayjs(dateString).fromNow()
}

const getPnLClass = (pnl: number) => ({
  positive: pnl > 0,
  negative: pnl < 0,
  neutral: pnl === 0
})

const getReturnClass = (returnValue: number) => ({
  positive: returnValue > 0,
  negative: returnValue < 0,
  neutral: returnValue === 0
})

const getSharpeClass = (sharpe: number) => {
  if (sharpe > 2) return 'excellent'
  if (sharpe > 1) return 'good'
  if (sharpe > 0) return 'neutral'
  return 'poor'
}

// Tooltip content
const getPnLTooltip = () => {
  const { totalPnL, totalReturn, totalTrades } = props.strategy.performance
  return `Total P&L: ${formatPnL(totalPnL)}\nTotal Return: ${formatReturn(totalReturn)}\nTotal Trades: ${totalTrades}`
}

const getWinRateTooltip = () => {
  const { winRate, winningTrades, losingTrades } = props.strategy.performance
  return `Win Rate: ${formatPercentage(winRate)}\nWinning Trades: ${winningTrades}\nLosing Trades: ${losingTrades}`
}

const getSharpeTooltip = () => {
  return 'Sharpe ratio measures risk-adjusted returns. Higher is better.\n> 2: Excellent\n1-2: Good\n0-1: Acceptable\n< 0: Poor'
}

const getDrawdownTooltip = () => {
  return 'Maximum drawdown is the largest peak-to-trough decline in portfolio value.'
}
</script>

<style lang="scss" scoped>
@import '@/styles/variables.scss';
@import '@/styles/utilities.scss';

.strategy-card {
  transition: all $transition-base;
  cursor: pointer;
  border: 2px solid transparent;

  &:hover {
    transform: translateY(-2px);
    box-shadow: $shadow-xl;
  }

  &.status-active {
    border-color: var(--success-color);
  }

  &.status-paused {
    border-color: var(--warning-color);
  }

  &.status-draft {
    border-color: var(--info-color);
  }

  &.status-stopped {
    border-color: var(--danger-color);
  }

  &.status-error {
    border-color: var(--danger-color);
  }

  &.is-mobile {
    .card-header {
      flex-direction: column;
      align-items: flex-start;
      gap: $spacing-sm;
    }

    .info-grid {
      grid-template-columns: 1fr 1fr;
    }

    .performance-stats {
      grid-template-columns: 1fr;
      gap: $spacing-sm;

      .stat-divider {
        display: none;
      }
    }

    .mini-chart-container {
      height: 60px;
    }
  }

  &.is-compact {
    .card-content {
      padding: $spacing-lg;
    }

    .info-grid {
      grid-template-columns: repeat(2, 1fr);
    }

    .performance-stats {
      grid-template-columns: repeat(2, 1fr);
    }
  }
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: $spacing-md;

  .strategy-info {
    flex: 1;
    min-width: 0;
  }

  .strategy-title {
    h3 {
      margin: 0 0 $spacing-xs 0;
      font-size: $font-size-lg;
      font-weight: $font-weight-semibold;
      color: var(--text-primary);
      line-height: 1.2;
    }

    .strategy-meta {
      display: flex;
      flex-wrap: wrap;
      gap: $spacing-xs;
      margin-top: $spacing-xs;

      .tag {
        font-size: $font-size-xs;
      }
    }
  }

  .strategy-description {
    color: var(--text-secondary);
    font-size: $font-size-sm;
    line-height: 1.4;
    margin-top: $spacing-xs;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }

  .more-actions {
    border: none;
    background: var(--bg-tertiary);
    color: var(--text-secondary);

    &:hover {
      background: var(--hover-bg);
      color: var(--text-primary);
    }
  }
}

.card-content {
  padding: $spacing-xl;

  @include mobile {
    padding: $spacing-lg;
  }
}

.info-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: $spacing-md;
  margin-bottom: $spacing-lg;

  .info-item {
    display: flex;
    flex-direction: column;
    gap: $spacing-xs;

    .info-label {
      font-size: $font-size-xs;
      color: var(--text-secondary);
      font-weight: $font-weight-medium;
      text-transform: uppercase;
    }

    .info-value {
      font-size: $font-size-sm;
      font-weight: $font-weight-medium;
      color: var(--text-primary);

      &.symbol {
        font-family: 'Courier New', monospace;
        color: var(--accent-color);
        font-weight: $font-weight-bold;
      }
    }
  }
}

.performance-stats {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: $spacing-lg;
  padding: $spacing-lg;
  background-color: var(--bg-tertiary);
  border-radius: $radius-md;
  margin-bottom: $spacing-lg;

  .stat-item {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: $spacing-xs;

    .stat-header {
      display: flex;
      align-items: center;
      gap: $spacing-xs;

      .stat-label {
        font-size: $font-size-xs;
        font-weight: $font-weight-medium;
        color: var(--text-secondary);
        text-transform: uppercase;
      }

      .info-icon {
        font-size: 12px;
        color: var(--text-muted);
        cursor: help;
      }
    }

    .stat-value {
      font-size: $font-size-xl;
      font-weight: $font-weight-bold;
      color: var(--text-primary);

      &.positive {
        color: var(--success-color);
      }

      &.negative {
        color: var(--danger-color);
      }

      &.excellent {
        color: var(--success-color);
      }

      &.good {
        color: var(--warning-color);
      }

      &.poor {
        color: var(--danger-color);
      }
    }

    .stat-change,
    .stat-details {
      font-size: $font-size-xs;
      color: var(--text-secondary);
      text-align: center;

      &.positive {
        color: var(--success-color);
      }

      &.negative {
        color: var(--danger-color);
      }
    }
  }

  .stat-divider {
    width: 1px;
    background-color: var(--border-color);
    align-self: stretch;
  }
}

.mini-chart-container {
  margin-bottom: $spacing-lg;
  border-radius: $radius-md;
  overflow: hidden;
  background-color: var(--bg-primary);

  .mini-chart {
    width: 100%;
    height: 80px;

    @include mobile {
      height: 60px;
    }
  }
}

.strategy-tags {
  display: flex;
  flex-wrap: wrap;
  gap: $spacing-xs;

  .tag {
    font-size: $font-size-xs;
  }
}

.card-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: $spacing-md;

  @include mobile {
    flex-direction: column;
    gap: $spacing-md;
    align-items: stretch;
  }

  .status-actions {
    display: flex;
    gap: $spacing-sm;

    @include mobile {
      justify-content: center;
    }

    .el-button {
      flex: 1;
      min-width: 0;
    }
  }

  .quick-stats {
    display: flex;
    gap: $spacing-md;

    @include mobile {
      justify-content: space-between;
    }

    .quick-stat {
      display: flex;
      align-items: center;
      gap: $spacing-xs;
      font-size: $font-size-sm;
      color: var(--text-secondary);

      .el-icon {
        font-size: 14px;
      }
    }
  }
}

.danger-item {
  color: var(--danger-color) !important;
}

:deep(.el-card__header) {
  padding: $spacing-lg $spacing-xl;
  background-color: var(--bg-secondary);
  border-bottom: 1px solid var(--border-color);
}

:deep(.el-card__body) {
  background-color: var(--bg-secondary);
}

:deep(.el-card__footer) {
  background-color: var(--bg-secondary);
  border-top: 1px solid var(--border-color);
}

// Loading states
.loading-overlay {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: rgba(255, 255, 255, 0.8);
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: $radius-lg;
  z-index: 1;
}
</style>