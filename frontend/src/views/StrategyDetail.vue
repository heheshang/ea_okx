<template>
  <div class="strategy-detail">
    <!-- Header with Controls -->
    <el-card shadow="hover" class="header-card">
      <el-row justify="space-between" align="middle">
        <el-col :span="12">
          <el-space alignment="center">
            <el-button :icon="Back" @click="$router.back()">Back</el-button>
            <div class="strategy-title">
              <h2>{{ strategyData.name }}</h2>
              <el-tag :type="getStatusType(strategyData.status)" size="large">
                {{ strategyData.status }}
              </el-tag>
            </div>
          </el-space>
        </el-col>
        <el-col :span="12" style="text-align: right">
          <el-space>
            <el-button
              v-if="strategyData.status === 'Active'"
              type="warning"
              @click="pauseStrategy"
            >
              <el-icon><VideoPause /></el-icon>
              Pause
            </el-button>
            <el-button
              v-if="strategyData.status === 'Paused'"
              type="success"
              @click="resumeStrategy"
            >
              <el-icon><VideoPlay /></el-icon>
              Resume
            </el-button>
            <el-button
              v-if="['Draft', 'Paused'].includes(strategyData.status)"
              type="primary"
              @click="startStrategy"
            >
              <el-icon><VideoPlay /></el-icon>
              Start
            </el-button>
            <el-button type="danger" @click="stopStrategy">
              <el-icon><CircleClose /></el-icon>
              Stop
            </el-button>
            <el-button :icon="Edit" @click="editStrategy">Edit</el-button>
          </el-space>
        </el-col>
      </el-row>
    </el-card>

    <!-- Performance Metrics -->
    <el-row :gutter="20" class="mt-20">
      <el-col :span="6" v-for="metric in performanceMetrics" :key="metric.label">
        <el-card shadow="hover" class="metric-card">
          <div class="metric-label">{{ metric.label }}</div>
          <div class="metric-value" :class="metric.value >= 0 ? 'text-success' : 'text-danger'">
            {{ formatMetric(metric.value, metric.format) }}
          </div>
          <div class="metric-change" :class="metric.trend">
            {{ metric.change }}
          </div>
        </el-card>
      </el-col>
    </el-row>

    <!-- Performance Charts -->
    <el-row :gutter="20" class="mt-20">
      <el-col :span="16">
        <el-card shadow="hover">
          <template #header>
            <div class="card-header">
              <span>Equity Curve</span>
              <el-radio-group v-model="timeframe" size="small">
                <el-radio-button label="1D">1D</el-radio-button>
                <el-radio-button label="1W">1W</el-radio-button>
                <el-radio-button label="1M">1M</el-radio-button>
                <el-radio-button label="All">All</el-radio-button>
              </el-radio-group>
            </div>
          </template>
          <div class="chart-container" ref="equityChartRef"></div>
        </el-card>
      </el-col>
      <el-col :span="8">
        <el-card shadow="hover">
          <template #header>
            <span>Win/Loss Distribution</span>
          </template>
          <div class="chart-container" ref="winLossChartRef"></div>
        </el-card>
      </el-col>
    </el-row>

    <!-- Strategy Configuration & Recent Trades -->
    <el-row :gutter="20" class="mt-20">
      <el-col :span="12">
        <el-card shadow="hover">
          <template #header>
            <span>Strategy Configuration</span>
          </template>
          <el-descriptions :column="1" border>
            <el-descriptions-item label="Strategy ID">
              <span class="mono">{{ $route.params.id }}</span>
            </el-descriptions-item>
            <el-descriptions-item label="Symbol">{{ strategyData.symbol }}</el-descriptions-item>
            <el-descriptions-item label="Type">{{ strategyData.type }}</el-descriptions-item>
            <el-descriptions-item label="Capital Allocation">
              ${{ strategyData.capital.toLocaleString() }}
            </el-descriptions-item>
            <el-descriptions-item label="Max Position Size">
              ${{ strategyData.maxPositionSize.toLocaleString() }}
            </el-descriptions-item>
            <el-descriptions-item label="Stop Loss">
              {{ strategyData.stopLoss }}%
            </el-descriptions-item>
            <el-descriptions-item label="Take Profit">
              {{ strategyData.takeProfit }}%
            </el-descriptions-item>
            <el-descriptions-item label="Created At">
              {{ strategyData.createdAt }}
            </el-descriptions-item>
          </el-descriptions>

          <el-divider content-position="left">Parameters</el-divider>
          <el-descriptions :column="1" border>
            <el-descriptions-item
              v-for="(value, key) in strategyData.parameters"
              :key="key"
              :label="formatParamLabel(key)"
            >
              {{ value }}
            </el-descriptions-item>
          </el-descriptions>
        </el-card>
      </el-col>

      <el-col :span="12">
        <el-card shadow="hover">
          <template #header>
            <div class="card-header">
              <span>Recent Trades</span>
              <el-button size="small" text @click="viewAllTrades">View All</el-button>
            </div>
          </template>
          <el-table :data="recentTrades" style="width: 100%" max-height="400">
            <el-table-column prop="timestamp" label="Time" width="150" />
            <el-table-column prop="side" label="Side" width="80">
              <template #default="{ row }">
                <el-tag :type="row.side === 'Buy' ? 'success' : 'danger'" size="small">
                  {{ row.side }}
                </el-tag>
              </template>
            </el-table-column>
            <el-table-column prop="quantity" label="Qty" align="right" />
            <el-table-column prop="price" label="Price" align="right">
              <template #default="{ row }">
                ${{ row.price.toLocaleString() }}
              </template>
            </el-table-column>
            <el-table-column prop="pnl" label="P&L" align="right">
              <template #default="{ row }">
                <span :class="row.pnl >= 0 ? 'text-success' : 'text-danger'">
                  {{ formatPnL(row.pnl) }}
                </span>
              </template>
            </el-table-column>
          </el-table>
        </el-card>

        <el-card shadow="hover" class="mt-20">
          <template #header>
            <span>Strategy Signals</span>
          </template>
          <el-timeline class="signal-timeline">
            <el-timeline-item
              v-for="signal in signals"
              :key="signal.id"
              :timestamp="signal.timestamp"
              :type="getSignalType(signal.type)"
            >
              <div class="signal-content">
                <strong>{{ signal.type }} Signal</strong>
                <p>{{ signal.reason }}</p>
                <div class="signal-meta">
                  <span>Confidence: {{ (signal.confidence * 100).toFixed(0) }}%</span>
                  <span v-if="signal.targetPrice">
                    Target: ${{ signal.targetPrice.toLocaleString() }}
                  </span>
                </div>
              </div>
            </el-timeline-item>
          </el-timeline>
        </el-card>
      </el-col>
    </el-row>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { invoke } from '@tauri-apps/api/core'
import { ElMessage, ElMessageBox } from 'element-plus'
import {
  Back,
  Edit,
  VideoPlay,
  VideoPause,
  CircleClose,
} from '@element-plus/icons-vue'
import * as echarts from 'echarts'

const route = useRoute()
const router = useRouter()

const timeframe = ref('1M')

const strategyData = ref({
  name: 'MA Crossover Strategy',
  status: 'Active',
  symbol: 'BTC-USDT',
  type: 'Trend Following',
  capital: 50000,
  maxPositionSize: 10000,
  stopLoss: 2,
  takeProfit: 5,
  createdAt: '2024-01-15 10:30:00',
  parameters: {
    shortPeriod: 20,
    longPeriod: 50,
    positionSize: 0.2,
    riskPerTrade: 0.02,
  },
})

const performanceMetrics = ref([
  { label: 'Total P&L', value: 12450, format: 'currency', change: '+8.5%', trend: 'up' },
  { label: 'Win Rate', value: 65.2, format: 'percent', change: '+3.2%', trend: 'up' },
  { label: 'Sharpe Ratio', value: 1.85, format: 'number', change: '+0.15', trend: 'up' },
  { label: 'Max Drawdown', value: -8.5, format: 'percent', change: '-1.2%', trend: 'down' },
])

const recentTrades = ref([
  { timestamp: '2024-11-20 10:30', side: 'Buy', quantity: 0.5, price: 45000, pnl: 250 },
  { timestamp: '2024-11-20 09:15', side: 'Sell', quantity: 0.3, price: 44800, pnl: 150 },
  { timestamp: '2024-11-20 08:45', side: 'Buy', quantity: 0.8, price: 44500, pnl: -100 },
])

const signals = ref([
  {
    id: 1,
    type: 'Buy',
    reason: 'Short MA crossed above Long MA',
    confidence: 0.85,
    targetPrice: 46000,
    timestamp: '2024-11-20 10:30',
  },
  {
    id: 2,
    type: 'Hold',
    reason: 'No clear trend signal',
    confidence: 0.60,
    targetPrice: null,
    timestamp: '2024-11-20 09:45',
  },
])

const equityChartRef = ref<HTMLDivElement>()
const winLossChartRef = ref<HTMLDivElement>()

const getStatusType = (status: string) => {
  const types: Record<string, any> = {
    Active: 'success',
    Paused: 'warning',
    Draft: 'info',
    Stopped: 'danger',
  }
  return types[status] || 'info'
}

const getSignalType = (type: string) => {
  const types: Record<string, any> = {
    Buy: 'success',
    Sell: 'danger',
    Hold: 'info',
  }
  return types[type] || 'info'
}

const formatMetric = (value: number, format: string) => {
  if (format === 'currency') {
    return `$${value.toLocaleString()}`
  } else if (format === 'percent') {
    return `${value.toFixed(1)}%`
  }
  return value.toFixed(2)
}

const formatPnL = (value: number) => {
  const sign = value >= 0 ? '+' : ''
  return `${sign}$${value.toFixed(2)}`
}

const formatParamLabel = (key: string) => {
  return key
    .replace(/([A-Z])/g, ' $1')
    .replace(/^./, (str) => str.toUpperCase())
    .trim()
}

const startStrategy = async () => {
  try {
    await invoke('start_strategy', { strategyId: route.params.id })
    strategyData.value.status = 'Active'
    ElMessage.success('Strategy started')
  } catch (error) {
    console.error('Failed to start strategy:', error)
    ElMessage.error('Failed to start strategy')
  }
}

const pauseStrategy = async () => {
  try {
    await invoke('pause_strategy', { strategyId: route.params.id })
    strategyData.value.status = 'Paused'
    ElMessage.success('Strategy paused')
  } catch (error) {
    console.error('Failed to pause strategy:', error)
    ElMessage.error('Failed to pause strategy')
  }
}

const resumeStrategy = async () => {
  try {
    await invoke('resume_strategy', { strategyId: route.params.id })
    strategyData.value.status = 'Active'
    ElMessage.success('Strategy resumed')
  } catch (error) {
    console.error('Failed to resume strategy:', error)
    ElMessage.error('Failed to resume strategy')
  }
}

const stopStrategy = async () => {
  try {
    await ElMessageBox.confirm(
      'Are you sure to stop this strategy? All open positions will be closed.',
      'Warning',
      { type: 'warning' }
    )
    await invoke('stop_strategy', { strategyId: route.params.id })
    strategyData.value.status = 'Stopped'
    ElMessage.success('Strategy stopped')
  } catch (error) {
    if (error !== 'cancel') {
      console.error('Failed to stop strategy:', error)
      ElMessage.error('Failed to stop strategy')
    }
  }
}

const editStrategy = () => {
  router.push(`/strategies/edit/${route.params.id}`)
}

const viewAllTrades = () => {
  router.push(`/trading?strategy=${route.params.id}`)
}

const initCharts = () => {
  // Equity Curve
  if (equityChartRef.value) {
    const chart = echarts.init(equityChartRef.value, 'dark')
    const dates = Array.from({ length: 30 }, (_, i) => `Day ${i + 1}`)
    const equity = Array.from({ length: 30 }, (_, i) => 50000 + Math.random() * 5000 + i * 400)

    chart.setOption({
      tooltip: { trigger: 'axis' },
      grid: { left: '3%', right: '4%', bottom: '3%', containLabel: true },
      xAxis: { type: 'category', data: dates },
      yAxis: { type: 'value', name: 'Equity ($)' },
      series: [
        {
          data: equity,
          type: 'line',
          smooth: true,
          itemStyle: { color: '#539bf5' },
          areaStyle: { color: 'rgba(83, 155, 245, 0.2)' },
        },
      ],
    })
  }

  // Win/Loss Chart
  if (winLossChartRef.value) {
    const chart = echarts.init(winLossChartRef.value, 'dark')
    chart.setOption({
      tooltip: { trigger: 'item' },
      series: [
        {
          type: 'pie',
          radius: ['40%', '70%'],
          data: [
            { value: 65, name: 'Wins', itemStyle: { color: '#57ab5a' } },
            { value: 35, name: 'Losses', itemStyle: { color: '#e5534b' } },
          ],
          label: { formatter: '{b}: {c} ({d}%)' },
        },
      ],
    })
  }
}

onMounted(() => {
  initCharts()
  // Load strategy data from backend
  // await invoke('get_strategy', { strategyId: route.params.id })
})
</script>

<style lang="scss" scoped>
.strategy-detail {
  .mt-20 {
    margin-top: 20px;
  }

  .header-card {
    margin-bottom: 20px;

    .strategy-title {
      display: flex;
      align-items: center;
      gap: 15px;

      h2 {
        margin: 0;
      }
    }
  }

  .metric-card {
    text-align: center;

    .metric-label {
      font-size: 14px;
      color: var(--text-secondary);
      margin-bottom: 10px;
    }

    .metric-value {
      font-size: 28px;
      font-weight: bold;
      margin-bottom: 5px;
    }

    .metric-change {
      font-size: 13px;

      &.up {
        color: var(--success-color);
      }

      &.down {
        color: var(--danger-color);
      }
    }
  }

  .card-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .chart-container {
    height: 300px;
  }

  .mono {
    font-family: 'Courier New', monospace;
  }

  .signal-timeline {
    padding: 10px 0;
    max-height: 350px;
    overflow-y: auto;

    .signal-content {
      strong {
        display: block;
        margin-bottom: 5px;
      }

      p {
        margin: 5px 0;
        font-size: 13px;
        color: var(--text-secondary);
      }

      .signal-meta {
        display: flex;
        gap: 15px;
        margin-top: 5px;
        font-size: 12px;
        color: var(--text-secondary);
      }
    }
  }

  .text-success {
    color: var(--success-color);
  }

  .text-danger {
    color: var(--danger-color);
  }

  :deep(.el-card) {
    background-color: var(--bg-secondary);
    border: 1px solid var(--border-color);
    color: var(--text-primary);
  }

  :deep(.el-descriptions) {
    --el-descriptions-item-bordered-label-background: var(--bg-tertiary);
  }

  :deep(.el-table) {
    background-color: var(--bg-secondary);
    color: var(--text-primary);

    th {
      background-color: var(--bg-tertiary);
      color: var(--text-secondary);
    }

    tr {
      background-color: var(--bg-secondary);

      &:hover > td {
        background-color: var(--bg-tertiary);
      }
    }
  }
}
</style>
