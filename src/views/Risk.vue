<template>
  <div class="risk-center">
    <!-- Risk Summary Cards -->
    <el-row :gutter="20">
      <el-col :xs="12" :sm="12" :md="6" v-for="metric in riskMetrics" :key="metric.label">
        <el-card class="risk-card" shadow="hover">
          <div class="risk-content">
            <div class="risk-icon" :class="metric.status">
              <el-icon :size="28"><component :is="metric.icon" /></el-icon>
            </div>
            <div class="risk-info">
              <div class="risk-label">{{ metric.label }}</div>
              <div class="risk-value">{{ metric.value }}</div>
              <div class="risk-status" :class="metric.status">{{ metric.statusText }}</div>
            </div>
          </div>
        </el-card>
      </el-col>
    </el-row>

    <!-- VaR Charts -->
    <el-row :gutter="20" class="mt-20">
      <el-col :xs="24" :sm="24" :md="12">
        <el-card shadow="hover">
          <template #header>
            <div class="card-header">
              <span>Value at Risk (VaR)</span>
              <el-space wrap :size="10">
                <el-select v-model="varMethod" size="small" style="width: 150px">
                  <el-option label="Historical" value="historical" />
                  <el-option label="Parametric" value="parametric" />
                  <el-option label="Monte Carlo" value="monte_carlo" />
                </el-select>
                <el-select v-model="varConfidence" size="small" style="width: 100px">
                  <el-option label="95%" :value="0.95" />
                  <el-option label="99%" :value="0.99" />
                </el-select>
              </el-space>
            </div>
          </template>
          <div class="chart-container" ref="varChartRef"></div>
        </el-card>
      </el-col>
      <el-col :xs="24" :sm="24" :md="12" class="mt-mobile">
        <el-card shadow="hover">
          <template #header>
            <div class="card-header">
              <span>Drawdown Analysis</span>
              <el-radio-group v-model="drawdownPeriod" size="small">
                <el-radio-button label="7D">7D</el-radio-button>
                <el-radio-button label="30D">30D</el-radio-button>
                <el-radio-button label="90D">90D</el-radio-button>
              </el-radio-group>
            </div>
          </template>
          <div class="chart-container" ref="drawdownChartRef"></div>
        </el-card>
      </el-col>
    </el-row>

    <!-- Risk Limits Configuration -->
    <el-row :gutter="20" class="mt-20">
      <el-col :xs="24" :sm="24" :md="12">
        <el-card shadow="hover" class="risk-limits-card">
          <template #header>
            <div class="card-header">
              <div class="header-left">
                <el-icon class="header-icon"><Lock /></el-icon>
                <span>Risk Limits</span>
              </div>
              <el-button size="small" type="primary" @click="saveRiskLimits" class="save-btn">
                <el-icon><Select /></el-icon>
                <span class="btn-text">Save</span>
              </el-button>
            </div>
          </template>
          <el-form :model="riskLimits" :label-width="isMobile ? '140px' : '200px'" label-position="left" class="risk-form">
            <el-divider content-position="left">
              <el-icon><TrendCharts /></el-icon>
              <span>Position Limits</span>
            </el-divider>
            <el-form-item label="Max Position Size" class="form-item-enhanced">
              <div class="input-group">
                <el-input-number
                  v-model="riskLimits.maxPositionSize"
                  :min="0"
                  :step="1000"
                  controls-position="right"
                  class="limit-input"
                />
                <span class="unit">USDT</span>
              </div>
            </el-form-item>
            <el-form-item label="Max Leverage" class="form-item-enhanced">
              <div class="input-group">
                <el-slider
                  v-model="riskLimits.maxLeverage"
                  :min="1"
                  :max="10"
                  :step="0.5"
                  :show-tooltip="true"
                  :marks="{ 1: '1x', 5: '5x', 10: '10x' }"
                  class="leverage-slider"
                />
                <span class="slider-value">{{ riskLimits.maxLeverage.toFixed(1) }}x</span>
              </div>
            </el-form-item>
            <el-form-item label="Max Concentration" class="form-item-enhanced">
              <div class="input-group">
                <el-slider
                  v-model="riskLimits.maxConcentration"
                  :min="0"
                  :max="100"
                  :step="5"
                  :show-tooltip="true"
                  class="concentration-slider"
                />
                <span class="slider-value">{{ riskLimits.maxConcentration }}%</span>
              </div>
            </el-form-item>
            <el-form-item label="Max Open Positions" class="form-item-enhanced">
              <div class="input-group">
                <el-input-number
                  v-model="riskLimits.maxOpenPositions"
                  :min="1"
                  :max="50"
                  controls-position="right"
                  class="limit-input"
                />
              </div>
            </el-form-item>

            <el-divider content-position="left">
              <el-icon><Warning /></el-icon>
              <span>Loss Limits</span>
            </el-divider>
            <el-form-item label="Daily Loss Limit" class="form-item-enhanced">
              <div class="input-group">
                <el-slider
                  v-model="riskLimits.dailyLossLimit"
                  :min="0"
                  :max="100"
                  :step="1"
                  :show-tooltip="true"
                  class="loss-slider"
                />
                <span class="slider-value">{{ riskLimits.dailyLossLimit }}%</span>
              </div>
            </el-form-item>
            <el-form-item label="Max Drawdown Limit" class="form-item-enhanced">
              <div class="input-group">
                <el-slider
                  v-model="riskLimits.maxDrawdownLimit"
                  :min="0"
                  :max="100"
                  :step="5"
                  :show-tooltip="true"
                  class="loss-slider"
                />
                <span class="slider-value">{{ riskLimits.maxDrawdownLimit }}%</span>
              </div>
            </el-form-item>

            <el-divider content-position="left">
              <el-icon><Select /></el-icon>
              <span>Stop Loss/Take Profit</span>
            </el-divider>
            <el-form-item label="Default Stop Loss" class="form-item-enhanced">
              <div class="input-group">
                <el-input-number
                  v-model="riskLimits.stopLossPercent"
                  :min="0"
                  :max="50"
                  :step="0.5"
                  :precision="1"
                  controls-position="right"
                  class="limit-input"
                />
                <span class="unit">%</span>
              </div>
            </el-form-item>
            <el-form-item label="Default Take Profit" class="form-item-enhanced">
              <div class="input-group">
                <el-input-number
                  v-model="riskLimits.takeProfitPercent"
                  :min="0"
                  :max="100"
                  :step="1"
                  controls-position="right"
                  class="limit-input"
                />
                <span class="unit">%</span>
              </div>
            </el-form-item>
          </el-form>
        </el-card>
      </el-col>

      <el-col :xs="24" :sm="24" :md="12" class="mt-mobile">
        <el-card shadow="hover" class="heatmap-card">
          <template #header>
            <div class="card-header">
              <div class="header-left">
                <el-icon class="header-icon"><TrendCharts /></el-icon>
                <span>Risk Heat Map</span>
              </div>
              <el-tooltip content="Higher values indicate higher risk" placement="top">
                <el-icon class="info-icon"><InfoFilled /></el-icon>
              </el-tooltip>
            </div>
          </template>
          <div class="chart-container heatmap-container" ref="heatmapChartRef"></div>
          <div class="heatmap-legend">
            <div class="legend-item">
              <span class="legend-color" style="background: #57ab5a"></span>
              <span class="legend-text">Low Risk</span>
            </div>
            <div class="legend-item">
              <span class="legend-color" style="background: #c69026"></span>
              <span class="legend-text">Medium Risk</span>
            </div>
            <div class="legend-item">
              <span class="legend-color" style="background: #e5534b"></span>
              <span class="legend-text">High Risk</span>
            </div>
          </div>
        </el-card>

        <el-card shadow="hover" class="mt-20">
          <template #header>
            <div class="card-header">
              <span>Risk Alerts</span>
              <el-button size="small" text @click="clearAlerts">Clear All</el-button>
            </div>
          </template>
          <el-timeline class="alert-timeline">
            <el-timeline-item
              v-for="alert in riskAlerts"
              :key="alert.id"
              :timestamp="alert.timestamp"
              :type="getAlertType(alert.level)"
            >
              <div class="alert-content">
                <strong>{{ alert.title }}</strong>
                <p>{{ alert.message }}</p>
              </div>
            </el-timeline-item>
          </el-timeline>
          <el-empty v-if="riskAlerts.length === 0" description="No risk alerts" />
        </el-card>
      </el-col>
    </el-row>

    <!-- Position Risk Analysis -->
    <el-row :gutter="20" class="mt-20">
      <el-col :span="24">
        <el-card shadow="hover">
          <template #header>
            <span>Position Risk Analysis</span>
          </template>
          <el-table :data="positionRisks" style="width: 100%">
            <el-table-column prop="symbol" label="Symbol" width="120" />
            <el-table-column prop="size" label="Position Size" align="right" width="150">
              <template #default="{ row }">
                ${{ row.size.toLocaleString() }}
              </template>
            </el-table-column>
            <el-table-column prop="leverage" label="Leverage" align="right" width="100">
              <template #default="{ row }">
                {{ row.leverage.toFixed(2) }}x
              </template>
            </el-table-column>
            <el-table-column prop="var95" label="VaR (95%)" align="right" width="120">
              <template #default="{ row }">
                <span class="text-danger">-${{ row.var95.toLocaleString() }}</span>
              </template>
            </el-table-column>
            <el-table-column prop="cvar95" label="CVaR (95%)" align="right" width="120">
              <template #default="{ row }">
                <span class="text-danger">-${{ row.cvar95.toLocaleString() }}</span>
              </template>
            </el-table-column>
            <el-table-column prop="sharpe" label="Sharpe Ratio" align="right" width="120">
              <template #default="{ row }">
                {{ row.sharpe.toFixed(2) }}
              </template>
            </el-table-column>
            <el-table-column prop="volatility" label="Volatility" align="right" width="120">
              <template #default="{ row }">
                {{ (row.volatility * 100).toFixed(2) }}%
              </template>
            </el-table-column>
            <el-table-column prop="riskScore" label="Risk Score" align="right" width="120">
              <template #default="{ row }">
                <el-tag :type="getRiskScoreType(row.riskScore)" size="small">
                  {{ row.riskScore }}
                </el-tag>
              </template>
            </el-table-column>
          </el-table>
        </el-card>
      </el-col>
    </el-row>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { ElMessage } from 'element-plus'
import { Select, Warning, TrendCharts, Lock, InfoFilled } from '@element-plus/icons-vue'
import * as echarts from 'echarts'
import { useResponsive } from '@/composables/useResponsive'

// Define component name for keep-alive
defineOptions({
  name: 'Risk'
})

const { isMobile } = useResponsive()

interface RiskLimits {
  maxPositionSize: number
  maxLeverage: number
  maxConcentration: number
  maxOpenPositions: number
  dailyLossLimit: number
  maxDrawdownLimit: number
  stopLossPercent: number
  takeProfitPercent: number
}

interface RiskAlert {
  id: number
  level: string
  title: string
  message: string
  timestamp: string
}

interface PositionRisk {
  symbol: string
  size: number
  leverage: number
  var95: number
  cvar95: number
  sharpe: number
  volatility: number
  riskScore: string
}

const varMethod = ref('historical')
const varConfidence = ref(0.95)
const drawdownPeriod = ref('30D')

const riskMetrics = ref([
  {
    label: 'Portfolio VaR',
    value: '$2,450',
    statusText: 'Normal',
    status: 'success',
    icon: TrendCharts,
  },
  {
    label: 'Max Drawdown',
    value: '8.5%',
    statusText: 'Warning',
    status: 'warning',
    icon: Warning,
  },
  {
    label: 'Leverage',
    value: '2.3x',
    statusText: 'Normal',
    status: 'success',
    icon: Lock,
  },
  {
    label: 'Risk Score',
    value: '7.2/10',
    statusText: 'Moderate',
    status: 'warning',
    icon: TrendCharts,
  },
])

const riskLimits = ref<RiskLimits>({
  maxPositionSize: 10000,
  maxLeverage: 3,
  maxConcentration: 25,
  maxOpenPositions: 10,
  dailyLossLimit: 5,
  maxDrawdownLimit: 20,
  stopLossPercent: 2,
  takeProfitPercent: 5,
})

const riskAlerts = ref<RiskAlert[]>([
  {
    id: 1,
    level: 'WARNING',
    title: 'High Leverage Detected',
    message: 'BTC-USDT position leverage exceeds 3x threshold',
    timestamp: '2024-11-20 10:30',
  },
  {
    id: 2,
    level: 'INFO',
    title: 'Drawdown Approaching',
    message: 'Current drawdown is at 7.5%, approaching 10% warning level',
    timestamp: '2024-11-20 10:15',
  },
])

const positionRisks = ref<PositionRisk[]>([
  {
    symbol: 'BTC-USDT',
    size: 68000,
    leverage: 2.5,
    var95: 3200,
    cvar95: 4100,
    sharpe: 1.85,
    volatility: 0.15,
    riskScore: 'Medium',
  },
  {
    symbol: 'ETH-USDT',
    size: 16000,
    leverage: 1.8,
    var95: 950,
    cvar95: 1250,
    sharpe: 1.65,
    volatility: 0.18,
    riskScore: 'Low',
  },
])

const varChartRef = ref<HTMLDivElement>()
const drawdownChartRef = ref<HTMLDivElement>()
const heatmapChartRef = ref<HTMLDivElement>()

const getAlertType = (level: string) => {
  const types: Record<string, any> = {
    INFO: 'primary',
    WARNING: 'warning',
    ERROR: 'danger',
    CRITICAL: 'danger',
  }
  return types[level] || 'info'
}

const getRiskScoreType = (score: string) => {
  const types: Record<string, any> = {
    Low: 'success',
    Medium: 'warning',
    High: 'danger',
  }
  return types[score] || 'info'
}

const saveRiskLimits = async () => {
  try {
    await invoke('update_risk_limits', { limits: riskLimits.value })
    ElMessage.success('Risk limits updated')
  } catch (error) {
    console.error('Failed to update risk limits:', error)
    ElMessage.error('Failed to update risk limits')
  }
}

const clearAlerts = () => {
  riskAlerts.value = []
  ElMessage.success('Alerts cleared')
}

const initCharts = () => {
  // VaR Chart
  if (varChartRef.value) {
    const chart = echarts.init(varChartRef.value, 'dark')
    const dates = Array.from({ length: 30 }, (_, i) => `Day ${i + 1}`)
    const var95 = Array.from({ length: 30 }, () => 2000 + Math.random() * 1000)
    const var99 = Array.from({ length: 30 }, () => 2500 + Math.random() * 1200)

    chart.setOption({
      tooltip: { trigger: 'axis' },
      legend: { data: ['VaR 95%', 'VaR 99%'], bottom: 0 },
      grid: { left: '3%', right: '4%', bottom: '10%', containLabel: true },
      xAxis: { type: 'category', data: dates },
      yAxis: { type: 'value', name: 'VaR ($)' },
      series: [
        {
          name: 'VaR 95%',
          type: 'line',
          data: var95,
          smooth: true,
          itemStyle: { color: '#e5534b' },
        },
        {
          name: 'VaR 99%',
          type: 'line',
          data: var99,
          smooth: true,
          itemStyle: { color: '#8b3434' },
        },
      ],
    })
  }

  // Drawdown Chart
  if (drawdownChartRef.value) {
    const chart = echarts.init(drawdownChartRef.value, 'dark')
    const dates = Array.from({ length: 30 }, (_, i) => `Day ${i + 1}`)
    const drawdown = Array.from({ length: 30 }, () => -Math.random() * 10)

    chart.setOption({
      tooltip: { trigger: 'axis' },
      grid: { left: '3%', right: '4%', bottom: '3%', containLabel: true },
      xAxis: { type: 'category', data: dates },
      yAxis: { type: 'value', name: 'Drawdown (%)' },
      series: [
        {
          name: 'Drawdown',
          type: 'line',
          data: drawdown,
          smooth: true,
          itemStyle: { color: '#e5534b' },
          areaStyle: { color: 'rgba(229, 83, 75, 0.2)' },
        },
      ],
    })
  }

  // Risk Heatmap
  if (heatmapChartRef.value) {
    const chart = echarts.init(heatmapChartRef.value, 'dark')
    const symbols = ['BTC-USDT', 'ETH-USDT', 'SOL-USDT', 'BNB-USDT']
    const metrics = ['VaR', 'Volatility', 'Leverage', 'Concentration']
    const data: any[] = []

    symbols.forEach((symbol, i) => {
      metrics.forEach((metric, j) => {
        data.push([j, i, Math.random() * 100])
      })
    })

    chart.setOption({
      tooltip: { position: 'top' },
      grid: { height: '65%', top: '10%' },
      xAxis: { type: 'category', data: metrics, splitArea: { show: true } },
      yAxis: { type: 'category', data: symbols, splitArea: { show: true } },
      visualMap: {
        min: 0,
        max: 100,
        calculable: true,
        orient: 'horizontal',
        left: 'center',
        bottom: '5%',
        inRange: {
          color: ['#57ab5a', '#e5534b'],
        },
      },
      series: [
        {
          name: 'Risk',
          type: 'heatmap',
          data: data,
          label: { show: true },
          emphasis: { itemStyle: { shadowBlur: 10, shadowColor: 'rgba(0, 0, 0, 0.5)' } },
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

.risk-center {
  .mt-20 {
    margin-top: $spacing-xl;
  }
  
  .mt-mobile {
    @include mobile {
      margin-top: $spacing-xl;
    }
  }

  .risk-card {
    background-color: var(--bg-secondary);
    border: 1px solid var(--border-color);
    margin-bottom: 15px;

    :deep(.el-card__body) {
      padding: 20px;
      
      @media (max-width: 768px) {
        padding: 15px;
      }
    }

    .risk-content {
      display: flex;
      align-items: center;
      gap: 15px;
      
      @media (max-width: 768px) {
        gap: 10px;
      }

      .risk-icon {
        width: 50px;
        height: 50px;
        border-radius: 12px;
        display: flex;
        align-items: center;
        justify-content: center;
        color: white;
        flex-shrink: 0;
        
        @media (max-width: 768px) {
          width: 40px;
          height: 40px;
          
          :deep(.el-icon) {
            font-size: 22px !important;
          }
        }

        &.success {
          background-color: #57ab5a;
        }

        &.warning {
          background-color: #e5534b;
        }

        &.danger {
          background-color: #8b3434;
        }
      }

      .risk-info {
        flex: 1;
        min-width: 0;

        .risk-label {
          font-size: 14px;
          color: var(--text-secondary);
          margin-bottom: 5px;
          
          @media (max-width: 768px) {
            font-size: 12px;
          }
        }

        .risk-value {
          font-size: 24px;
          font-weight: bold;
          color: var(--text-primary);
          margin-bottom: 5px;
          
          @media (max-width: 768px) {
            font-size: 18px;
          }
        }

        .risk-status {
          font-size: 12px;
          
          @media (max-width: 768px) {
            font-size: 11px;
          }

          &.success {
            color: #57ab5a;
          }

          &.warning {
            color: #e5534b;
          }

          &.danger {
            color: #8b3434;
          }
        }
      }
    }
  }

  .card-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    flex-wrap: wrap;
    gap: $spacing-md;
    
    .header-left {
      display: flex;
      align-items: center;
      gap: $spacing-sm;
      
      .header-icon {
        font-size: 20px;
        color: var(--accent-color);
      }
    }
    
    .info-icon {
      font-size: 18px;
      color: var(--text-secondary);
      cursor: help;
      transition: color $transition-fast;
      
      &:hover {
        color: var(--accent-color);
      }
    }
    
    span {
      font-weight: $font-weight-medium;
      
      @include mobile {
        font-size: $font-size-sm;
      }
    }
    
    .save-btn {
      min-height: 32px;
      
      @include mobile {
        min-height: 36px;
        
        .btn-text {
          @media (max-width: 480px) {
            display: none;
          }
        }
      }
    }
  }

  .chart-container {
    height: 300px;
    
    @include mobile {
      height: 250px;
    }
  }
  
  // Risk Limits Card
  .risk-limits-card {
    .risk-form {
      :deep(.el-divider) {
        margin: $spacing-2xl 0 $spacing-lg;
        
        .el-divider__text {
          display: flex;
          align-items: center;
          gap: $spacing-sm;
          font-weight: $font-weight-semibold;
          color: var(--text-primary);
          
          .el-icon {
            font-size: 16px;
            color: var(--accent-color);
          }
        }
      }
      
      .form-item-enhanced {
        margin-bottom: $spacing-xl;
        
        .input-group {
          display: flex;
          align-items: center;
          gap: $spacing-md;
          width: 100%;
          
          .limit-input {
            flex: 1;
            
            @include mobile {
              width: 100%;
            }
          }
          
          .leverage-slider,
          .concentration-slider,
          .loss-slider {
            flex: 1;
            margin-right: $spacing-lg;
            
            @include mobile {
              margin-right: $spacing-md;
            }
          }
          
          .slider-value {
            min-width: 50px;
            text-align: right;
            font-weight: $font-weight-semibold;
            color: var(--accent-color);
            font-size: $font-size-base;
          }
          
          .unit {
            min-width: 50px;
            color: var(--text-secondary);
            font-size: $font-size-sm;
          }
        }
      }
    }
  }
  
  // Heat Map Card
  .heatmap-card {
    .heatmap-container {
      margin-bottom: $spacing-lg;
    }
    
    .heatmap-legend {
      display: flex;
      justify-content: center;
      gap: $spacing-2xl;
      padding: $spacing-lg 0;
      border-top: 1px solid var(--border-color);
      
      @include mobile {
        flex-direction: column;
        gap: $spacing-md;
      }
      
      .legend-item {
        display: flex;
        align-items: center;
        gap: $spacing-sm;
        
        .legend-color {
          width: 20px;
          height: 20px;
          border-radius: $radius-sm;
          
          @include mobile {
            width: 16px;
            height: 16px;
          }
        }
        
        .legend-text {
          font-size: $font-size-sm;
          color: var(--text-secondary);
          font-weight: $font-weight-medium;
          
          @include mobile {
            font-size: $font-size-xs;
          }
        }
      }
    }
  }

  .alert-timeline {
    padding: 10px 0;
    max-height: 240px;
    overflow-y: auto;

    .alert-content {
      strong {
        display: block;
        margin-bottom: 5px;
      }

      p {
        margin: 0;
        font-size: 13px;
        color: var(--text-secondary);
      }
    }
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

  :deep(.el-divider__text) {
    background-color: var(--bg-secondary);
    color: var(--text-secondary);
  }
}
</style>
