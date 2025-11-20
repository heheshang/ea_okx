<template>
  <div class="backtest">
    <!-- Backtest Configuration -->
    <el-row :gutter="20">
      <el-col :xs="24" :sm="24" :md="8">
        <el-card shadow="hover">
          <template #header>
            <div class="card-header">
              <el-icon><TrendCharts /></el-icon>
              <span>Backtest Configuration</span>
            </div>
          </template>
          <el-form :model="backtestConfig" :label-width="isMobile ? '110px' : '130px'" label-position="left">
            <el-form-item label="Strategy">
              <el-select v-model="backtestConfig.strategyId" placeholder="Select strategy">
                <el-option
                  v-for="strategy in strategies"
                  :key="strategy.id"
                  :label="strategy.name"
                  :value="strategy.id"
                />
              </el-select>
            </el-form-item>
            <el-form-item label="Symbol">
              <el-select v-model="backtestConfig.symbol" placeholder="Select symbol">
                <el-option
                  v-for="symbol in symbols"
                  :key="symbol"
                  :label="symbol"
                  :value="symbol"
                />
              </el-select>
            </el-form-item>
            <el-form-item label="Date Range">
              <el-date-picker
                v-model="backtestConfig.dateRange"
                type="daterange"
                range-separator="To"
                start-placeholder="Start date"
                end-placeholder="End date"
                value-format="YYYY-MM-DD"
              />
            </el-form-item>
            <el-form-item label="Initial Capital">
              <el-input-number
                v-model="backtestConfig.initialCapital"
                :min="1000"
                :step="1000"
                controls-position="right"
              />
            </el-form-item>
            <el-form-item label="Commission">
              <el-input-number
                v-model="backtestConfig.commission"
                :min="0"
                :max="1"
                :step="0.0001"
                :precision="4"
                controls-position="right"
              />
            </el-form-item>
            <el-form-item label="Slippage">
              <el-input-number
                v-model="backtestConfig.slippage"
                :min="0"
                :max="1"
                :step="0.0001"
                :precision="4"
                controls-position="right"
              />
            </el-form-item>
            <el-form-item>
              <el-button
                type="primary"
                :loading="isRunning"
                @click="runBacktest"
                class="run-backtest-btn"
              >
                <el-icon v-if="!isRunning"><TrendCharts /></el-icon>
                <span class="btn-text">{{ isRunning ? 'Running...' : 'Run Backtest' }}</span>
              </el-button>
            </el-form-item>
          </el-form>
        </el-card>
      </el-col>

      <el-col :xs="24" :sm="24" :md="16" class="mt-mobile">
        <el-card shadow="hover">
          <template #header>
            <div class="card-header">
              <span>Performance Metrics</span>
              <el-button size="small" @click="exportResults" class="export-btn">
                <el-icon><Download /></el-icon>
                <span class="btn-text">Export</span>
              </el-button>
            </div>
          </template>
          <el-row :gutter="15">
            <el-col :xs="12" :sm="8" :md="8" v-for="metric in performanceMetrics" :key="metric.label">
              <div class="metric-box">
                <div class="metric-label">{{ metric.label }}</div>
                <div
                  class="metric-value"
                  :class="metric.value >= 0 ? 'text-success' : 'text-danger'"
                >
                  {{ formatMetricValue(metric.value, metric.format) }}
                </div>
              </div>
            </el-col>
          </el-row>
        </el-card>
      </el-col>
    </el-row>

    <!-- Equity Curve Chart -->
    <el-row :gutter="20" class="mt-20">
      <el-col :span="24">
        <el-card shadow="hover">
          <template #header>
            <div class="card-header">
              <span>Equity Curve</span>
              <el-radio-group v-model="chartType" size="small">
                <el-radio-button label="equity">Equity</el-radio-button>
                <el-radio-button label="drawdown">Drawdown</el-radio-button>
                <el-radio-button label="both">Both</el-radio-button>
              </el-radio-group>
            </div>
          </template>
          <div class="chart-container" ref="equityChartRef"></div>
        </el-card>
      </el-col>
    </el-row>

    <!-- Trade Analysis -->
    <el-row :gutter="20" class="mt-20">
      <el-col :xs="24" :sm="24" :md="12">
        <el-card shadow="hover">
          <template #header>
            <span>Trade Distribution</span>
          </template>
          <div class="chart-container" ref="tradeDistChartRef"></div>
        </el-card>
      </el-col>
      <el-col :xs="24" :sm="24" :md="12" class="mt-mobile">
        <el-card shadow="hover">
          <template #header>
            <span>Returns Distribution</span>
          </template>
          <div class="chart-container" ref="returnsDistChartRef"></div>
        </el-card>
      </el-col>
    </el-row>

    <!-- Trade List -->
    <el-row :gutter="20" class="mt-20">
      <el-col :span="24">
        <el-card shadow="hover">
          <template #header>
            <div class="card-header">
              <span>Trade History ({{ tradeList.length }} trades)</span>
              <el-space>
                <el-select v-model="tradeFilter" size="small" placeholder="Filter">
                  <el-option label="All Trades" value="all" />
                  <el-option label="Winning Trades" value="win" />
                  <el-option label="Losing Trades" value="loss" />
                </el-select>
              </el-space>
            </div>
          </template>
          <el-table :data="filteredTrades" style="width: 100%" max-height="400">
            <el-table-column prop="id" label="#" width="60" />
            <el-table-column prop="timestamp" label="Time" width="180" />
            <el-table-column prop="symbol" label="Symbol" width="120" />
            <el-table-column prop="side" label="Side" width="80">
              <template #default="{ row }">
                <el-tag :type="row.side === 'Buy' ? 'success' : 'danger'" size="small">
                  {{ row.side }}
                </el-tag>
              </template>
            </el-table-column>
            <el-table-column prop="quantity" label="Quantity" align="right" width="100" />
            <el-table-column prop="entryPrice" label="Entry" align="right" width="120">
              <template #default="{ row }">
                ${{ row.entryPrice.toLocaleString() }}
              </template>
            </el-table-column>
            <el-table-column prop="exitPrice" label="Exit" align="right" width="120">
              <template #default="{ row }">
                ${{ row.exitPrice.toLocaleString() }}
              </template>
            </el-table-column>
            <el-table-column prop="pnl" label="P&L" align="right" width="120">
              <template #default="{ row }">
                <span :class="row.pnl >= 0 ? 'text-success' : 'text-danger'">
                  {{ formatPnL(row.pnl) }}
                </span>
              </template>
            </el-table-column>
            <el-table-column prop="return" label="Return %" align="right" width="100">
              <template #default="{ row }">
                <span :class="row.return >= 0 ? 'text-success' : 'text-danger'">
                  {{ row.return >= 0 ? '+' : '' }}{{ row.return.toFixed(2) }}%
                </span>
              </template>
            </el-table-column>
            <el-table-column prop="holdingPeriod" label="Holding Period" />
          </el-table>
        </el-card>
      </el-col>
    </el-row>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { ElMessage } from 'element-plus'
import { Download, TrendCharts } from '@element-plus/icons-vue'
import * as echarts from 'echarts'
import { useResponsive } from '@/composables/useResponsive'

const { isMobile } = useResponsive()

// Define component name for keep-alive
defineOptions({
  name: 'Backtest'
})

interface BacktestConfig {
  strategyId: string
  symbol: string
  dateRange: [string, string] | null
  initialCapital: number
  commission: number
  slippage: number
}

interface Trade {
  id: number
  timestamp: string
  symbol: string
  side: string
  quantity: number
  entryPrice: number
  exitPrice: number
  pnl: number
  return: number
  holdingPeriod: string
}

const strategies = ref([
  { id: '1', name: 'MA Crossover' },
  { id: '2', name: 'Grid Trading' },
  { id: '3', name: 'RSI Strategy' },
])

const symbols = ref(['BTC-USDT', 'ETH-USDT', 'SOL-USDT', 'BNB-USDT'])

const backtestConfig = ref<BacktestConfig>({
  strategyId: '1',
  symbol: 'BTC-USDT',
  dateRange: null,
  initialCapital: 10000,
  commission: 0.001,
  slippage: 0.0005,
})

const isRunning = ref(false)
const chartType = ref('equity')
const tradeFilter = ref('all')

const performanceMetrics = ref([
  { label: 'Total Return', value: 24.5, format: 'percent' },
  { label: 'Sharpe Ratio', value: 1.85, format: 'number' },
  { label: 'Max Drawdown', value: -12.3, format: 'percent' },
  { label: 'Win Rate', value: 65.2, format: 'percent' },
  { label: 'Profit Factor', value: 2.15, format: 'number' },
  { label: 'Total Trades', value: 142, format: 'integer' },
])

const tradeList = ref<Trade[]>([
  {
    id: 1,
    timestamp: '2024-01-15 10:30:00',
    symbol: 'BTC-USDT',
    side: 'Buy',
    quantity: 0.5,
    entryPrice: 43500,
    exitPrice: 44200,
    pnl: 350,
    return: 1.61,
    holdingPeriod: '2h 15m',
  },
  {
    id: 2,
    timestamp: '2024-01-16 14:20:00',
    symbol: 'BTC-USDT',
    side: 'Sell',
    quantity: 0.3,
    entryPrice: 44500,
    exitPrice: 43800,
    pnl: 210,
    return: 1.57,
    holdingPeriod: '1h 45m',
  },
  {
    id: 3,
    timestamp: '2024-01-17 09:45:00',
    symbol: 'BTC-USDT',
    side: 'Buy',
    quantity: 0.8,
    entryPrice: 43200,
    exitPrice: 42900,
    pnl: -240,
    return: -0.69,
    holdingPeriod: '3h 20m',
  },
])

const equityChartRef = ref<HTMLDivElement>()
const tradeDistChartRef = ref<HTMLDivElement>()
const returnsDistChartRef = ref<HTMLDivElement>()

const filteredTrades = computed(() => {
  if (tradeFilter.value === 'win') {
    return tradeList.value.filter((t) => t.pnl > 0)
  } else if (tradeFilter.value === 'loss') {
    return tradeList.value.filter((t) => t.pnl < 0)
  }
  return tradeList.value
})

const formatMetricValue = (value: number, format: string) => {
  if (format === 'percent') {
    return `${value >= 0 ? '+' : ''}${value.toFixed(2)}%`
  } else if (format === 'integer') {
    return value.toString()
  }
  return value.toFixed(2)
}

const formatPnL = (value: number) => {
  const sign = value >= 0 ? '+' : ''
  return `${sign}$${value.toFixed(2)}`
}

const runBacktest = async () => {
  if (!backtestConfig.value.dateRange) {
    ElMessage.warning('Please select date range')
    return
  }

  isRunning.value = true
  try {
    const result = await invoke('run_backtest', {
      strategyId: backtestConfig.value.strategyId,
      symbol: backtestConfig.value.symbol,
      startDate: backtestConfig.value.dateRange[0],
      endDate: backtestConfig.value.dateRange[1],
      initialCapital: backtestConfig.value.initialCapital,
      commission: backtestConfig.value.commission,
      slippage: backtestConfig.value.slippage,
    })
    console.log('Backtest result:', result)
    ElMessage.success('Backtest completed successfully')
    // Update metrics and charts with result
  } catch (error) {
    console.error('Backtest failed:', error)
    ElMessage.error('Backtest failed')
  } finally {
    isRunning.value = false
  }
}

const exportResults = () => {
  ElMessage.info('Exporting results...')
  // Implement export functionality
}

const initCharts = () => {
  // Equity Curve Chart
  if (equityChartRef.value) {
    const chart = echarts.init(equityChartRef.value, 'dark')
    const dates = Array.from({ length: 30 }, (_, i) => `Day ${i + 1}`)
    const equity = Array.from({ length: 30 }, (_, i) => 10000 + Math.random() * 2000 + i * 50)

    chart.setOption({
      tooltip: { trigger: 'axis' },
      grid: { left: '3%', right: '4%', bottom: '3%', containLabel: true },
      xAxis: { type: 'category', data: dates },
      yAxis: { type: 'value', name: 'Equity ($)' },
      series: [
        {
          name: 'Equity',
          type: 'line',
          data: equity,
          smooth: true,
          itemStyle: { color: '#539bf5' },
          areaStyle: { color: 'rgba(83, 155, 245, 0.2)' },
        },
      ],
    })
  }

  // Trade Distribution Chart
  if (tradeDistChartRef.value) {
    const chart = echarts.init(tradeDistChartRef.value, 'dark')
    chart.setOption({
      tooltip: { trigger: 'item' },
      series: [
        {
          name: 'Trades',
          type: 'pie',
          radius: ['40%', '70%'],
          data: [
            { value: 65, name: 'Winning Trades', itemStyle: { color: '#57ab5a' } },
            { value: 35, name: 'Losing Trades', itemStyle: { color: '#e5534b' } },
          ],
          label: { formatter: '{b}: {c} ({d}%)' },
        },
      ],
    })
  }

  // Returns Distribution Chart
  if (returnsDistChartRef.value) {
    const chart = echarts.init(returnsDistChartRef.value, 'dark')
    chart.setOption({
      tooltip: { trigger: 'axis' },
      grid: { left: '3%', right: '4%', bottom: '3%', containLabel: true },
      xAxis: {
        type: 'category',
        data: ['-5%', '-3%', '-1%', '0%', '1%', '3%', '5%', '7%'],
      },
      yAxis: { type: 'value', name: 'Frequency' },
      series: [
        {
          type: 'bar',
          data: [5, 12, 18, 25, 35, 28, 15, 8],
          itemStyle: {
            color: (params: any) => {
              return params.dataIndex < 3 ? '#e5534b' : '#57ab5a'
            },
          },
        },
      ],
    })
  }
}

onMounted(() => {
  initCharts()
})
</script>

<style lang="scss" scoped>
@import '@/styles/variables.scss';
@import '@/styles/utilities.scss';

.backtest {
  .mt-20 {
    margin-top: $spacing-xl;
  }
  
  .mt-mobile {
    @include mobile {
      margin-top: $spacing-xl;
    }
  }

  .card-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    flex-wrap: wrap;
    gap: $spacing-md;
    
    .el-icon {
      margin-right: $spacing-xs;
    }
    
    span {
      @include mobile {
        font-size: $font-size-sm;
      }
    }
  }
  
  .run-backtest-btn {
    width: 100%;
    min-height: $touch-target-min;
    
    @include mobile {
      min-height: $touch-target-comfortable;
    }
  }
  
  .export-btn {
    @media (max-width: 480px) {
      .btn-text {
        display: none;
      }
    }
  }

  .metric-box {
    padding: $spacing-lg;
    background-color: var(--bg-tertiary);
    border-radius: $radius-md;
    text-align: center;
    margin-bottom: $spacing-md;
    transition: all $transition-base;
    
    &:hover {
      transform: translateY(-2px);
      box-shadow: 0 4px 12px var(--shadow-color);
    }
    
    @include mobile {
      padding: $spacing-md;
    }

    .metric-label {
      font-size: $font-size-xs;
      color: var(--text-secondary);
      margin-bottom: $spacing-sm;
      font-weight: $font-weight-medium;
      
      @include mobile {
        font-size: 11px;
      }
    }

    .metric-value {
      font-size: $font-size-xl;
      font-weight: $font-weight-bold;
      
      @include mobile {
        font-size: $font-size-base;
      }
    }
  }

  .chart-container {
    height: 350px;
    
    @include mobile {
      height: 250px;
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

  :deep(.el-form-item__label) {
    color: var(--text-secondary);
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
