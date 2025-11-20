<template>
  <div class="strategies-page">
    <!-- Header Section -->
    <div class="page-header">
      <h2>Strategy Management</h2>
      <el-button type="primary" @click="showCreateDialog = true" class="create-btn">
        <el-icon><Plus /></el-icon>
        <span class="btn-text">Create Strategy</span>
      </el-button>
    </div>

    <!-- Filters Section -->
    <div class="filters-section">
      <el-input
        v-model="searchText"
        placeholder="Search strategies..."
        :prefix-icon="Search"
        class="search-input"
        clearable
      />
      <el-select v-model="statusFilter" placeholder="Filter by status" class="status-filter" clearable>
        <el-option label="All" value="" />
        <el-option label="Active" value="Active" />
        <el-option label="Draft" value="Draft" />
        <el-option label="Paused" value="Paused" />
        <el-option label="Stopped" value="Stopped" />
      </el-select>
      <el-select v-model="typeFilter" placeholder="Filter by type" class="type-filter" clearable v-if="!isMobile">
        <el-option label="All Types" value="" />
        <el-option label="MA Crossover" value="MA Crossover" />
        <el-option label="Grid Trading" value="Grid Trading" />
        <el-option label="RSI Strategy" value="RSI Strategy" />
      </el-select>
    </div>

    <!-- View Toggle -->
    <div class="view-toggle" v-if="!isMobile">
      <el-radio-group v-model="viewMode" size="small">
        <el-radio-button value="grid">
          <el-icon><Grid /></el-icon>
          Grid
        </el-radio-button>
        <el-radio-button value="list">
          <el-icon><List /></el-icon>
          List
        </el-radio-button>
      </el-radio-group>
    </div>

    <!-- Loading State -->
    <div v-if="loading" class="loading-container">
      <el-skeleton :rows="3" animated />
    </div>

    <!-- Grid View -->
    <div v-else-if="viewMode === 'grid' || isMobile" class="strategies-grid">
      <div
        v-for="strategy in filteredStrategies"
        :key="strategy.id"
        class="strategy-card"
        @click="handleView(strategy)"
      >
        <div class="card-header">
          <div class="strategy-info">
            <h3 class="strategy-name">{{ strategy.name }}</h3>
            <el-tag :type="getStatusType(strategy.status)" size="small" class="status-tag">
              {{ strategy.status }}
            </el-tag>
          </div>
          <el-dropdown trigger="click" @click.stop>
            <el-icon class="more-icon"><MoreFilled /></el-icon>
            <template #dropdown>
              <el-dropdown-menu>
                <el-dropdown-item @click="handleView(strategy)">
                  <el-icon><View /></el-icon>
                  View Details
                </el-dropdown-item>
                <el-dropdown-item @click="handleEdit(strategy)">
                  <el-icon><Edit /></el-icon>
                  Edit
                </el-dropdown-item>
                <el-dropdown-item divided @click="handleDelete(strategy)" class="danger-item">
                  <el-icon><Delete /></el-icon>
                  Delete
                </el-dropdown-item>
              </el-dropdown-menu>
            </template>
          </el-dropdown>
        </div>

        <div class="card-body">
          <div class="info-row">
            <span class="label">Type:</span>
            <span class="value">{{ strategy.type }}</span>
          </div>
          <div class="info-row">
            <span class="label">Symbol:</span>
            <span class="value symbol">{{ strategy.symbol }}</span>
          </div>
          <div class="info-row">
            <span class="label">Trades:</span>
            <span class="value">{{ strategy.trades }}</span>
          </div>
        </div>

        <div class="card-stats">
          <div class="stat-item">
            <span class="stat-label">P&L</span>
            <span class="stat-value" :class="strategy.pnl >= 0 ? 'positive' : 'negative'">
              {{ formatPnL(strategy.pnl) }}
            </span>
          </div>
          <div class="stat-divider"></div>
          <div class="stat-item">
            <span class="stat-label">Win Rate</span>
            <span class="stat-value">{{ (strategy.winRate * 100).toFixed(1) }}%</span>
          </div>
        </div>

        <div class="card-footer" @click.stop>
          <el-button
            v-if="strategy.status === 'Draft' || strategy.status === 'Paused'"
            type="success"
            size="small"
            @click="handleStart(strategy)"
            class="action-btn"
          >
            <el-icon><VideoPlay /></el-icon>
            Start
          </el-button>
          <el-button
            v-if="strategy.status === 'Active'"
            type="warning"
            size="small"
            @click="handlePause(strategy)"
            class="action-btn"
          >
            <el-icon><VideoPause /></el-icon>
            Pause
          </el-button>
          <el-button
            v-if="strategy.status !== 'Stopped'"
            type="danger"
            size="small"
            @click="handleStop(strategy)"
            class="action-btn"
          >
            <el-icon><CircleClose /></el-icon>
            Stop
          </el-button>
        </div>
      </div>
    </div>

    <!-- List View (Desktop Only) -->
    <el-card v-else shadow="hover" class="table-card">
      <el-table :data="filteredStrategies" style="width: 100%">
        <el-table-column prop="name" label="Name" width="200">
          <template #default="{ row }">
            <el-link @click="handleView(row)" type="primary">{{ row.name }}</el-link>
          </template>
        </el-table-column>
        <el-table-column prop="type" label="Type" width="150" />
        <el-table-column prop="symbol" label="Symbol" width="120" />
        <el-table-column prop="status" label="Status" width="120">
          <template #default="{ row }">
            <el-tag :type="getStatusType(row.status)">{{ row.status }}</el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="pnl" label="P&L" align="right" width="150">
          <template #default="{ row }">
            <span :class="row.pnl >= 0 ? 'text-success' : 'text-danger'">
              {{ formatPnL(row.pnl) }}
            </span>
          </template>
        </el-table-column>
        <el-table-column prop="winRate" label="Win Rate" align="right" width="100">
          <template #default="{ row }">
            {{ (row.winRate * 100).toFixed(1) }}%
          </template>
        </el-table-column>
        <el-table-column prop="trades" label="Trades" align="right" width="100" />
        <el-table-column prop="createdAt" label="Created" width="180" />
        <el-table-column label="Actions" width="280" fixed="right">
          <template #default="{ row }">
            <el-space>
              <el-button
                v-if="row.status === 'Draft' || row.status === 'Paused'"
                size="small"
                type="success"
                @click="handleStart(row)"
              >
                <el-icon><VideoPlay /></el-icon>
                Start
              </el-button>
              <el-button
                v-if="row.status === 'Active'"
                size="small"
                type="warning"
                @click="handlePause(row)"
              >
                <el-icon><VideoPause /></el-icon>
                Pause
              </el-button>
              <el-button
                v-if="row.status !== 'Stopped'"
                size="small"
                type="danger"
                @click="handleStop(row)"
              >
                <el-icon><CircleClose /></el-icon>
                Stop
              </el-button>
              <el-button size="small" @click="handleView(row)">
                <el-icon><View /></el-icon>
                View
              </el-button>
              <el-button size="small" @click="handleEdit(row)">
                <el-icon><Edit /></el-icon>
              </el-button>
              <el-button size="small" type="danger" text @click="handleDelete(row)">
                <el-icon><Delete /></el-icon>
              </el-button>
            </el-space>
          </template>
        </el-table-column>
      </el-table>
    </el-card>

    <!-- Create/Edit Strategy Dialog -->
    <el-dialog
      v-model="showCreateDialog"
      :title="isEditing ? 'Edit Strategy' : 'Create New Strategy'"
      width="800px"
      :close-on-click-modal="false"
    >
      <el-steps :active="currentStep" finish-status="success" align-center style="margin-bottom: 30px">
        <el-step title="Basic Info" />
        <el-step title="Strategy Type" />
        <el-step title="Parameters" />
        <el-step title="Risk Control" />
      </el-steps>

      <!-- Step 1: Basic Info -->
      <div v-show="currentStep === 0">
        <el-form :model="strategyForm" label-width="150px" label-position="left">
          <el-form-item label="Strategy Name" required>
            <el-input
              v-model="strategyForm.name"
              placeholder="Enter strategy name"
              maxlength="50"
              show-word-limit
            />
          </el-form-item>
          <el-form-item label="Description">
            <el-input
              v-model="strategyForm.description"
              type="textarea"
              :rows="3"
              placeholder="Describe your strategy..."
              maxlength="200"
              show-word-limit
            />
          </el-form-item>
          <el-form-item label="Trading Symbol" required>
            <el-select v-model="strategyForm.symbol" placeholder="Select symbol">
              <el-option label="BTC-USDT" value="BTC-USDT" />
              <el-option label="ETH-USDT" value="ETH-USDT" />
              <el-option label="SOL-USDT" value="SOL-USDT" />
              <el-option label="BNB-USDT" value="BNB-USDT" />
            </el-select>
          </el-form-item>
          <el-form-item label="Capital Allocation" required>
            <el-input-number
              v-model="strategyForm.capital"
              :min="1000"
              :max="1000000"
              :step="1000"
              controls-position="right"
              style="width: 200px"
            />
            <span style="margin-left: 10px; color: var(--text-secondary)">USDT</span>
          </el-form-item>
        </el-form>
      </div>

      <!-- Step 2: Strategy Type -->
      <div v-show="currentStep === 1">
        <el-form :model="strategyForm" label-width="150px" label-position="left">
          <el-form-item label="Strategy Type" required>
            <el-radio-group v-model="strategyForm.type">
              <el-radio label="MA Crossover">Moving Average Crossover</el-radio>
              <el-radio label="Grid Trading">Grid Trading</el-radio>
              <el-radio label="RSI Strategy">RSI Mean Reversion</el-radio>
              <el-radio label="Market Making">Market Making</el-radio>
              <el-radio label="Arbitrage">Arbitrage</el-radio>
            </el-radio-group>
          </el-form-item>

          <el-divider />

          <el-alert
            :title="getStrategyDescription(strategyForm.type)"
            type="info"
            :closable="false"
            style="margin-bottom: 20px"
          />

          <el-form-item label="Timeframe">
            <el-select v-model="strategyForm.timeframe" placeholder="Select timeframe">
              <el-option label="1 minute" value="1m" />
              <el-option label="5 minutes" value="5m" />
              <el-option label="15 minutes" value="15m" />
              <el-option label="1 hour" value="1h" />
              <el-option label="4 hours" value="4h" />
              <el-option label="1 day" value="1d" />
            </el-select>
          </el-form-item>
        </el-form>
      </div>

      <!-- Step 3: Parameters -->
      <div v-show="currentStep === 2">
        <el-form :model="strategyForm" label-width="180px" label-position="left">
          <!-- MA Crossover Parameters -->
          <template v-if="strategyForm.type === 'MA Crossover'">
            <el-form-item label="Short Period">
              <el-input-number v-model="strategyForm.parameters.shortPeriod" :min="5" :max="100" />
            </el-form-item>
            <el-form-item label="Long Period">
              <el-input-number v-model="strategyForm.parameters.longPeriod" :min="20" :max="200" />
            </el-form-item>
            <el-form-item label="MA Type">
              <el-select v-model="strategyForm.parameters.maType">
                <el-option label="Simple (SMA)" value="SMA" />
                <el-option label="Exponential (EMA)" value="EMA" />
              </el-select>
            </el-form-item>
          </template>

          <!-- Grid Trading Parameters -->
          <template v-if="strategyForm.type === 'Grid Trading'">
            <el-form-item label="Grid Levels">
              <el-input-number v-model="strategyForm.parameters.gridLevels" :min="3" :max="50" />
            </el-form-item>
            <el-form-item label="Price Range (%)">
              <el-input-number
                v-model="strategyForm.parameters.priceRange"
                :min="1"
                :max="50"
                :step="0.5"
                :precision="1"
              />
            </el-form-item>
            <el-form-item label="Order Size per Grid">
              <el-input-number
                v-model="strategyForm.parameters.orderSize"
                :min="10"
                :max="10000"
                :step="10"
              />
            </el-form-item>
          </template>

          <!-- RSI Strategy Parameters -->
          <template v-if="strategyForm.type === 'RSI Strategy'">
            <el-form-item label="RSI Period">
              <el-input-number v-model="strategyForm.parameters.rsiPeriod" :min="5" :max="50" />
            </el-form-item>
            <el-form-item label="Oversold Threshold">
              <el-input-number v-model="strategyForm.parameters.oversold" :min="10" :max="40" />
            </el-form-item>
            <el-form-item label="Overbought Threshold">
              <el-input-number v-model="strategyForm.parameters.overbought" :min="60" :max="90" />
            </el-form-item>
          </template>

          <!-- Common Parameters -->
          <el-divider content-position="left">Common Settings</el-divider>
          <el-form-item label="Position Size (%)">
            <el-input-number
              v-model="strategyForm.parameters.positionSize"
              :min="0.01"
              :max="1"
              :step="0.05"
              :precision="2"
            />
            <span style="margin-left: 10px; color: var(--text-secondary)">
              {{ (strategyForm.parameters.positionSize * 100).toFixed(0) }}% of capital
            </span>
          </el-form-item>
        </el-form>
      </div>

      <!-- Step 4: Risk Control -->
      <div v-show="currentStep === 3">
        <el-form :model="strategyForm" label-width="180px" label-position="left">
          <el-form-item label="Stop Loss (%)">
            <el-input-number
              v-model="strategyForm.stopLoss"
              :min="0.1"
              :max="50"
              :step="0.5"
              :precision="1"
            />
          </el-form-item>
          <el-form-item label="Take Profit (%)">
            <el-input-number
              v-model="strategyForm.takeProfit"
              :min="0.5"
              :max="100"
              :step="0.5"
              :precision="1"
            />
          </el-form-item>
          <el-form-item label="Max Position Size">
            <el-input-number
              v-model="strategyForm.maxPositionSize"
              :min="100"
              :max="100000"
              :step="100"
            />
            <span style="margin-left: 10px; color: var(--text-secondary)">USDT</span>
          </el-form-item>
          <el-form-item label="Daily Loss Limit (%)">
            <el-input-number
              v-model="strategyForm.dailyLossLimit"
              :min="1"
              :max="50"
              :step="1"
            />
          </el-form-item>
          <el-form-item label="Max Open Positions">
            <el-input-number v-model="strategyForm.maxOpenPositions" :min="1" :max="10" />
          </el-form-item>

          <el-divider content-position="left">Advanced Options</el-divider>
          <el-form-item label="Enable Trailing Stop">
            <el-switch v-model="strategyForm.trailingStop" />
          </el-form-item>
          <el-form-item label="Use Limit Orders">
            <el-switch v-model="strategyForm.useLimitOrders" />
          </el-form-item>
          <el-form-item label="Paper Trading Mode">
            <el-switch v-model="strategyForm.paperTrading" />
            <span style="margin-left: 10px; color: var(--text-secondary)">
              Test strategy without real money
            </span>
          </el-form-item>
        </el-form>
      </div>

      <template #footer>
        <div style="display: flex; justify-content: space-between">
          <el-button v-if="currentStep > 0" @click="currentStep--">Previous</el-button>
          <div style="flex: 1"></div>
          <el-button @click="showCreateDialog = false">Cancel</el-button>
          <el-button v-if="currentStep < 3" type="primary" @click="currentStep++">Next</el-button>
          <el-button v-else type="primary" @click="handleSubmit" :loading="submitting">
            {{ isEditing ? 'Update' : 'Create' }} Strategy
          </el-button>
        </div>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useRouter } from 'vue-router'
import { ElMessage, ElMessageBox } from 'element-plus'
import {
  Plus,
  Search,
  VideoPlay,
  VideoPause,
  CircleClose,
  View,
  Edit,
  Delete,
  MoreFilled,
  Grid,
  List,
} from '@element-plus/icons-vue'
import { useStrategyStore } from '@/stores/strategy'
import { useResponsive } from '@/composables/useResponsive'

// Define component name for keep-alive
defineOptions({
  name: 'Strategies'
})

const router = useRouter()
const strategyStore = useStrategyStore()
const { isMobile } = useResponsive()

const loading = ref(false)
const searchText = ref('')
const statusFilter = ref('')
const typeFilter = ref('')
const viewMode = ref<'grid' | 'list'>('grid')
const showCreateDialog = ref(false)
const isEditing = ref(false)
const currentStep = ref(0)
const submitting = ref(false)

const strategies = ref([
  {
    id: '1',
    name: 'MA Crossover BTC',
    type: 'MA Crossover',
    symbol: 'BTC-USDT',
    status: 'Active',
    pnl: 2500.5,
    winRate: 0.65,
    trades: 42,
    createdAt: '2024-01-15 10:30:00',
  },
  {
    id: '2',
    name: 'Grid Trading ETH',
    type: 'Grid Trading',
    symbol: 'ETH-USDT',
    status: 'Active',
    pnl: 1200.3,
    winRate: 0.58,
    trades: 35,
    createdAt: '2024-02-20 14:20:00',
  },
  {
    id: '3',
    name: 'RSI Strategy SOL',
    type: 'RSI Strategy',
    symbol: 'SOL-USDT',
    status: 'Paused',
    pnl: -300.0,
    winRate: 0.45,
    trades: 28,
    createdAt: '2024-03-10 09:15:00',
  },
])

const strategyForm = ref({
  name: '',
  description: '',
  symbol: 'BTC-USDT',
  capital: 10000,
  type: 'MA Crossover',
  timeframe: '1h',
  parameters: {
    shortPeriod: 20,
    longPeriod: 50,
    maType: 'SMA',
    gridLevels: 10,
    priceRange: 5,
    orderSize: 100,
    rsiPeriod: 14,
    oversold: 30,
    overbought: 70,
    positionSize: 0.2,
  },
  stopLoss: 2,
  takeProfit: 5,
  maxPositionSize: 10000,
  dailyLossLimit: 5,
  maxOpenPositions: 3,
  trailingStop: false,
  useLimitOrders: true,
  paperTrading: true,
})

const filteredStrategies = computed(() => {
  let result = strategies.value

  if (searchText.value) {
    result = result.filter(
      (s) =>
        s.name.toLowerCase().includes(searchText.value.toLowerCase()) ||
        s.type.toLowerCase().includes(searchText.value.toLowerCase())
    )
  }

  if (statusFilter.value) {
    result = result.filter((s) => s.status === statusFilter.value)
  }
  
  if (typeFilter.value) {
    result = result.filter((s) => s.type === typeFilter.value)
  }

  return result
})

const getStatusType = (status: string) => {
  const types: Record<string, any> = {
    Active: 'success',
    Draft: 'info',
    Paused: 'warning',
    Stopped: 'danger',
  }
  return types[status] || 'info'
}

const formatPnL = (value: number) => {
  const sign = value >= 0 ? '+' : ''
  return `${sign}$${value.toFixed(2)}`
}

const getStrategyDescription = (type: string) => {
  const descriptions: Record<string, string> = {
    'MA Crossover':
      'Trend-following strategy that generates buy/sell signals when short-term MA crosses long-term MA',
    'Grid Trading':
      'Places buy and sell orders at regular intervals to profit from price volatility in a range',
    'RSI Strategy':
      'Mean reversion strategy using RSI indicator to identify oversold/overbought conditions',
    'Market Making': 'Provides liquidity by placing buy and sell orders on both sides of the order book',
    Arbitrage: 'Exploits price differences between different markets or trading pairs',
  }
  return descriptions[type] || 'Select a strategy type to see description'
}

const loadStrategies = async () => {
  loading.value = true
  try {
    const data = await invoke('get_strategies')
    strategies.value = data as any
    strategyStore.setStrategies(strategies.value)
  } catch (error) {
    console.error('Failed to load strategies:', error)
    ElMessage.error('Failed to load strategies')
  } finally {
    loading.value = false
  }
}

const handleStart = async (row: any) => {
  try {
    await invoke('start_strategy', { strategyId: row.id })
    row.status = 'Active'
    ElMessage.success(`Strategy "${row.name}" started`)
  } catch (error) {
    console.error('Failed to start strategy:', error)
    ElMessage.error('Failed to start strategy')
  }
}

const handlePause = async (row: any) => {
  try {
    await invoke('pause_strategy', { strategyId: row.id })
    row.status = 'Paused'
    ElMessage.success(`Strategy "${row.name}" paused`)
  } catch (error) {
    console.error('Failed to pause strategy:', error)
    ElMessage.error('Failed to pause strategy')
  }
}

const handleStop = async (row: any) => {
  try {
    await ElMessageBox.confirm(
      `Are you sure to stop strategy "${row.name}"? All open positions will be closed.`,
      'Warning',
      { type: 'warning' }
    )
    await invoke('stop_strategy', { strategyId: row.id })
    row.status = 'Stopped'
    ElMessage.success(`Strategy "${row.name}" stopped`)
  } catch (error) {
    if (error !== 'cancel') {
      console.error('Failed to stop strategy:', error)
      ElMessage.error('Failed to stop strategy')
    }
  }
}

const handleView = (row: any) => {
  router.push(`/strategies/${row.id}`)
}

const handleEdit = (row: any) => {
  isEditing.value = true
  currentStep.value = 0
  // Load strategy data into form
  Object.assign(strategyForm.value, row)
  showCreateDialog.value = true
}

const handleDelete = async (row: any) => {
  try {
    await ElMessageBox.confirm(
      `Are you sure to delete strategy "${row.name}"? This action cannot be undone.`,
      'Warning',
      {
        type: 'warning',
        confirmButtonText: 'Delete',
        confirmButtonClass: 'el-button--danger',
      }
    )
    await invoke('delete_strategy', { strategyId: row.id })
    strategies.value = strategies.value.filter((s) => s.id !== row.id)
    strategyStore.removeStrategy(row.id)
    ElMessage.success(`Strategy "${row.name}" deleted`)
  } catch (error) {
    if (error !== 'cancel') {
      console.error('Failed to delete strategy:', error)
      ElMessage.error('Failed to delete strategy')
    }
  }
}

const handleSubmit = async () => {
  // Validation
  if (!strategyForm.value.name) {
    ElMessage.warning('Please enter strategy name')
    return
  }

  submitting.value = true
  try {
    const strategyData: any = {
      ...strategyForm.value,
      status: 'Draft',
      pnl: 0,
      winRate: 0,
      trades: 0,
      createdAt: new Date().toLocaleString(),
    }

    if (isEditing.value) {
      await invoke('update_strategy', strategyData)
      const index = strategies.value.findIndex((s) => s.id === strategyData.id)
      if (index >= 0) {
        strategies.value[index] = strategyData
      }
      ElMessage.success('Strategy updated successfully')
    } else {
      const result = await invoke('create_strategy', strategyData)
      strategyData.id = (result as any).id || String(Date.now())
      strategies.value.push(strategyData)
      strategyStore.addStrategy(strategyData)
      ElMessage.success('Strategy created successfully')
    }

    showCreateDialog.value = false
    resetForm()
  } catch (error) {
    console.error('Failed to create/update strategy:', error)
    ElMessage.error(`Failed to ${isEditing.value ? 'update' : 'create'} strategy`)
  } finally {
    submitting.value = false
  }
}

const resetForm = () => {
  isEditing.value = false
  currentStep.value = 0
  strategyForm.value = {
    name: '',
    description: '',
    symbol: 'BTC-USDT',
    capital: 10000,
    type: 'MA Crossover',
    timeframe: '1h',
    parameters: {
      shortPeriod: 20,
      longPeriod: 50,
      maType: 'SMA',
      gridLevels: 10,
      priceRange: 5,
      orderSize: 100,
      rsiPeriod: 14,
      oversold: 30,
      overbought: 70,
      positionSize: 0.2,
    },
    stopLoss: 2,
    takeProfit: 5,
    maxPositionSize: 10000,
    dailyLossLimit: 5,
    maxOpenPositions: 3,
    trailingStop: false,
    useLimitOrders: true,
    paperTrading: true,
  }
}

onMounted(() => {
  loadStrategies()
})
</script>

<style lang="scss" scoped>
@import '@/styles/variables.scss';
@import '@/styles/utilities.scss';

.strategies-page {
  .page-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: $spacing-xl;

    h2 {
      margin: 0;
      font-size: $font-size-3xl;
      font-weight: $font-weight-bold;
      color: var(--text-primary);
      
      @include mobile {
        font-size: $font-size-2xl;
      }
    }
    
    .create-btn {
      @include mobile {
        .btn-text {
          display: none;
        }
      }
    }
  }
  
  .filters-section {
    display: flex;
    gap: $spacing-md;
    margin-bottom: $spacing-lg;
    flex-wrap: wrap;
    
    .search-input {
      flex: 1;
      min-width: 200px;
      
      @include mobile {
        min-width: 150px;
      }
    }
    
    .status-filter,
    .type-filter {
      width: 150px;
      
      @include mobile {
        width: 130px;
      }
    }
  }
  
  .view-toggle {
    margin-bottom: $spacing-lg;
    display: flex;
    justify-content: flex-end;
  }
  
  .loading-container {
    margin-top: $spacing-3xl;
  }
  
  // Grid View Styles
  .strategies-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
    gap: $spacing-xl;
    
    @include mobile {
      grid-template-columns: 1fr;
      gap: $spacing-lg;
    }
  }
  
  .strategy-card {
    background-color: var(--bg-secondary);
    border: 1px solid var(--border-color);
    border-radius: $radius-lg;
    padding: $spacing-xl;
    transition: all $transition-base;
    cursor: pointer;
    
    @include mobile {
      padding: $spacing-lg;
    }
    
    &:hover {
      transform: translateY(-4px);
      box-shadow: 0 12px 24px var(--shadow-color);
      border-color: var(--accent-color);
    }
    
    .card-header {
      display: flex;
      justify-content: space-between;
      align-items: flex-start;
      margin-bottom: $spacing-lg;
      
      .strategy-info {
        flex: 1;
        min-width: 0;
        
        .strategy-name {
          font-size: $font-size-lg;
          font-weight: $font-weight-semibold;
          color: var(--text-primary);
          margin: 0 0 $spacing-sm 0;
          
          @include mobile {
            font-size: $font-size-base;
          }
        }
        
        .status-tag {
          font-size: $font-size-xs;
        }
      }
      
      .more-icon {
        font-size: 20px;
        color: var(--text-secondary);
        cursor: pointer;
        padding: $spacing-xs;
        border-radius: $radius-sm;
        transition: all $transition-fast;
        
        &:hover {
          background-color: var(--hover-bg);
          color: var(--accent-color);
        }
      }
    }
    
    .card-body {
      margin-bottom: $spacing-lg;
      
      .info-row {
        display: flex;
        justify-content: space-between;
        align-items: center;
        padding: $spacing-sm 0;
        border-bottom: 1px solid var(--border-color);
        
        &:last-child {
          border-bottom: none;
        }
        
        .label {
          font-size: $font-size-sm;
          color: var(--text-secondary);
          font-weight: $font-weight-medium;
        }
        
        .value {
          font-size: $font-size-sm;
          color: var(--text-primary);
          font-weight: $font-weight-medium;
          
          &.symbol {
            font-family: 'Courier New', monospace;
            color: var(--accent-color);
          }
        }
      }
    }
    
    .card-stats {
      display: flex;
      gap: $spacing-lg;
      padding: $spacing-lg;
      background-color: var(--bg-tertiary);
      border-radius: $radius-md;
      margin-bottom: $spacing-lg;
      
      .stat-item {
        flex: 1;
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: $spacing-xs;
        
        .stat-label {
          font-size: $font-size-xs;
          color: var(--text-secondary);
          font-weight: $font-weight-medium;
          text-transform: uppercase;
        }
        
        .stat-value {
          font-size: $font-size-xl;
          font-weight: $font-weight-bold;
          color: var(--text-primary);
          
          @include mobile {
            font-size: $font-size-lg;
          }
          
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
      }
    }
    
    .card-footer {
      display: flex;
      gap: $spacing-sm;
      flex-wrap: wrap;
      
      .action-btn {
        flex: 1;
        min-width: 80px;
        
        @include mobile {
          font-size: $font-size-xs;
        }
      }
    }
  }
  
  // Table View Styles
  .table-card {
    margin-top: $spacing-lg;
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

  :deep(.el-dialog) {
    background-color: var(--bg-secondary);
  }

  :deep(.el-form-item__label) {
    color: var(--text-secondary);
  }

  :deep(.el-step__title) {
    color: var(--text-primary);

    &.is-process,
    &.is-finish {
      color: var(--primary-color);
    }
  }

  :deep(.el-divider__text) {
    background-color: var(--bg-secondary);
    color: var(--text-secondary);
  }
}
</style>
