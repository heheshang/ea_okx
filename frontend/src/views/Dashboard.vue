<template>
  <div class="dashboard">
    <el-row :gutter="20">
      <!-- System Metrics Cards -->
      <el-col :span="6" v-for="metric in metrics" :key="metric.title">
        <el-card class="metric-card" shadow="hover">
          <div class="metric-content">
            <div class="metric-icon" :style="{ backgroundColor: metric.color }">
              <el-icon :size="24"><component :is="metric.icon" /></el-icon>
            </div>
            <div class="metric-info">
              <div class="metric-title">{{ metric.title }}</div>
              <div class="metric-value">{{ metric.value }}</div>
              <div class="metric-change" :class="metric.trend">
                {{ metric.change }}
              </div>
            </div>
          </div>
        </el-card>
      </el-col>
    </el-row>

    <!-- Charts Row -->
    <el-row :gutter="20" class="mt-20">
      <el-col :span="12">
        <el-card shadow="hover">
          <template #header>
            <div class="card-header">
              <span>Strategy Performance</span>
              <el-button size="small" type="primary" text>View All</el-button>
            </div>
          </template>
          <div class="chart-container" ref="equityChartRef"></div>
        </el-card>
      </el-col>
      <el-col :span="12">
        <el-card shadow="hover">
          <template #header>
            <div class="card-header">
              <span>Real-time Market Data</span>
              <el-button size="small" type="primary" text>Refresh</el-button>
            </div>
          </template>
          <div class="chart-container" ref="marketChartRef"></div>
        </el-card>
      </el-col>
    </el-row>

    <!-- Active Strategies & Recent Alerts -->
    <el-row :gutter="20" class="mt-20">
      <el-col :span="12">
        <el-card shadow="hover">
          <template #header>
            <div class="card-header">
              <span>Active Strategies</span>
              <el-button size="small" type="primary" text>Manage</el-button>
            </div>
          </template>
          <el-table :data="activeStrategies" style="width: 100%" height="300">
            <el-table-column prop="name" label="Name" />
            <el-table-column prop="status" label="Status">
              <template #default="{ row }">
                <el-tag :type="getStatusType(row.status)">{{ row.status }}</el-tag>
              </template>
            </el-table-column>
            <el-table-column prop="pnl" label="P&L">
              <template #default="{ row }">
                <span :class="row.pnl >= 0 ? 'text-success' : 'text-danger'">
                  {{ formatPnL(row.pnl) }}
                </span>
              </template>
            </el-table-column>
            <el-table-column prop="winRate" label="Win Rate">
              <template #default="{ row }">
                {{ (row.winRate * 100).toFixed(1) }}%
              </template>
            </el-table-column>
          </el-table>
        </el-card>
      </el-col>
      <el-col :span="12">
        <el-card shadow="hover">
          <template #header>
            <div class="card-header">
              <span>Recent Alerts</span>
              <el-button size="small" type="primary" text>View All</el-button>
            </div>
          </template>
          <el-timeline class="alert-timeline">
            <el-timeline-item
              v-for="alert in recentAlerts"
              :key="alert.id"
              :timestamp="alert.timestamp"
              :type="getAlertType(alert.level)"
            >
              {{ alert.message }}
            </el-timeline-item>
          </el-timeline>
        </el-card>
      </el-col>
    </el-row>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import * as echarts from 'echarts'
import { useConfigStore } from '@/stores/config'

// Define component name for keep-alive
defineOptions({
  name: 'Dashboard'
})

const configStore = useConfigStore()
const isDark = computed(() => configStore.theme === 'dark')

let equityChart: echarts.ECharts | null = null
let marketChart: echarts.ECharts | null = null

// System metrics
const metrics = ref([
  { title: 'Total P&L', value: '$12,450', change: '+8.5%', trend: 'up', color: '#57ab5a', icon: 'Money' },
  { title: 'Active Strategies', value: '5', change: '+2', trend: 'up', color: '#539bf5', icon: 'TrendCharts' },
  { title: 'Win Rate', value: '65%', change: '+3.2%', trend: 'up', color: '#539bf5', icon: 'PieChart' },
  { title: 'System Health', value: '98%', change: '-1%', trend: 'down', color: '#57ab5a', icon: 'Check' }
])

// Active strategies
const activeStrategies = ref([
  { name: 'MA Crossover', status: 'Active', pnl: 2500.50, winRate: 0.65 },
  { name: 'Grid Trading', status: 'Active', pnl: 1200.30, winRate: 0.58 },
  { name: 'RSI Strategy', status: 'Paused', pnl: -300.00, winRate: 0.45 },
])

// Recent alerts
const recentAlerts = ref([
  { id: 1, level: 'INFO', message: 'Strategy "MA Crossover" generated buy signal for BTC-USDT', timestamp: '2024-11-20 10:30' },
  { id: 2, level: 'WARNING', message: 'Daily loss limit approaching 80%', timestamp: '2024-11-20 10:15' },
  { id: 3, level: 'INFO', message: 'Successfully placed order #12345', timestamp: '2024-11-20 10:00' },
])

const equityChartRef = ref<HTMLDivElement>()
const marketChartRef = ref<HTMLDivElement>()

// Helper functions
const getStatusType = (status: string) => {
  return status === 'Active' ? 'success' : status === 'Paused' ? 'warning' : 'info'
}

const getAlertType = (level: string) => {
  const types: Record<string, any> = {
    'INFO': 'primary',
    'WARNING': 'warning',
    'ERROR': 'danger',
    'CRITICAL': 'danger'
  }
  return types[level] || 'info'
}

const formatPnL = (value: number) => {
  const sign = value >= 0 ? '+' : ''
  return `${sign}$${value.toFixed(2)}`
}

// Initialize charts - optimized version
const initCharts = () => {
  const chartTheme = isDark.value ? 'dark' : undefined
  const axisLineColor = isDark.value ? '#444c56' : '#d0d7de'
  const axisLabelColor = isDark.value ? '#adbac7' : '#59636e'
  const splitLineColor = isDark.value ? '#2d333b' : '#eaeef2'
  
  if (equityChartRef.value) {
    // Dispose old chart if exists
    if (equityChart) {
      equityChart.dispose()
    }
    equityChart = echarts.init(equityChartRef.value, chartTheme)
    equityChart.setOption({
      backgroundColor: 'transparent',
      tooltip: { trigger: 'axis' },
      xAxis: { 
        type: 'category', 
        data: ['Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat', 'Sun'],
        axisLine: { lineStyle: { color: axisLineColor } },
        axisLabel: { color: axisLabelColor }
      },
      yAxis: { 
        type: 'value',
        axisLine: { lineStyle: { color: axisLineColor } },
        axisLabel: { color: axisLabelColor },
        splitLine: { lineStyle: { color: splitLineColor } }
      },
      series: [{
        data: [10000, 10500, 10300, 11200, 11800, 12100, 12450],
        type: 'line',
        smooth: true,
        itemStyle: { color: '#539bf5' },
        areaStyle: { color: 'rgba(83, 155, 245, 0.2)' }
      }]
    })
  }

  if (marketChartRef.value) {
    // Dispose old chart if exists
    if (marketChart) {
      marketChart.dispose()
    }
    marketChart = echarts.init(marketChartRef.value, chartTheme)
    marketChart.setOption({
      backgroundColor: 'transparent',
      tooltip: { trigger: 'axis' },
      xAxis: { 
        type: 'category', 
        data: ['10:00', '10:30', '11:00', '11:30', '12:00'],
        axisLine: { lineStyle: { color: axisLineColor } },
        axisLabel: { color: axisLabelColor }
      },
      yAxis: { 
        type: 'value',
        axisLine: { lineStyle: { color: axisLineColor } },
        axisLabel: { color: axisLabelColor },
        splitLine: { lineStyle: { color: splitLineColor } }
      },
      series: [{
        data: [45000, 45200, 44800, 45500, 45300],
        type: 'line',
        itemStyle: { color: '#57ab5a' }
      }]
    })
  }
}

// Load system metrics from Tauri
const loadSystemMetrics = async () => {
  try {
    const data = await invoke('get_system_metrics')
    console.log('System metrics:', data)
    // Update metrics with real data
  } catch (error) {
    console.error('Failed to load system metrics:', error)
  }
}

onMounted(() => {
  initCharts()
  loadSystemMetrics()
  
  // Listen for theme changes and update charts
  window.addEventListener('theme-changed', () => {
    initCharts()
  })
})
</script>

<style lang="scss" scoped>
.dashboard {
  .mt-20 {
    margin-top: 20px;
  }

  .metric-card {
    background-color: var(--bg-secondary);
    border: 1px solid var(--border-color);

    :deep(.el-card__body) {
      padding: 20px;
    }

    .metric-content {
      display: flex;
      align-items: center;
      gap: 15px;

      .metric-icon {
        width: 50px;
        height: 50px;
        border-radius: 12px;
        display: flex;
        align-items: center;
        justify-content: center;
        color: white;
      }

      .metric-info {
        flex: 1;

        .metric-title {
          font-size: 14px;
          color: var(--text-secondary);
          margin-bottom: 5px;
        }

        .metric-value {
          font-size: 24px;
          font-weight: bold;
          color: var(--text-primary);
          margin-bottom: 5px;
        }

        .metric-change {
          font-size: 12px;
          
          &.up {
            color: var(--success-color);
          }

          &.down {
            color: var(--danger-color);
          }
        }
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

  .text-success {
    color: var(--success-color);
  }

  .text-danger {
    color: var(--danger-color);
  }

  .alert-timeline {
    padding: 10px 0;
    max-height: 260px;
    overflow-y: auto;
  }

  :deep(.el-card) {
    background-color: var(--bg-secondary);
    border: 1px solid var(--border-color);
    color: var(--text-primary);
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
