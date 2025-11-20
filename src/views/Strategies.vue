<template>
  <div class="strategies-page">
    <!-- Enhanced Header Section -->
    <div class="page-header">
      <div class="header-content">
        <div class="header-text">
          <h2>Strategy Management</h2>
          <p class="header-subtitle">Manage and monitor your automated trading strategies</p>
        </div>
        <div class="header-actions">
          <el-button @click="showImportDialog = true" class="import-btn" v-if="!isMobile">
            <el-icon><Upload /></el-icon>
            <span class="btn-text">Import</span>
          </el-button>
          <el-button @click="handleExportSelected" :disabled="selectedStrategies.length === 0" class="export-btn" v-if="!isMobile">
            <el-icon><Download /></el-icon>
            <span class="btn-text">Export</span>
          </el-button>
          <el-button type="primary" @click="showCreateDialog = true" class="create-btn">
            <el-icon><Plus /></el-icon>
            <span class="btn-text">Create Strategy</span>
          </el-button>
        </div>
      </div>
    </div>

    <!-- Enhanced Filters Section -->
    <el-card class="filters-card" shadow="never">
      <div class="filters-content">
        <div class="filters-row">
          <el-input
            v-model="searchText"
            placeholder="Search strategies by name, symbol, or tags..."
            :prefix-icon="Search"
            class="search-input"
            clearable
          />
          <el-select v-model="statusFilter" placeholder="All Statuses" class="filter-select" clearable>
            <el-option
              v-for="status in strategyStatuses"
              :key="status.value"
              :label="status.label"
              :value="status.value"
            >
              <span class="status-option">
                <el-tag :type="status.type" size="small">{{ status.label }}</el-tag>
              </span>
            </el-option>
          </el-select>
          <el-select v-model="typeFilter" placeholder="All Types" class="filter-select" clearable>
            <el-option
              v-for="type in strategyTypes"
              :key="type.value"
              :label="type.label"
              :value="type.value"
            />
          </el-select>
          <el-button @click="showAdvancedFilters = !showAdvancedFilters" :class="{ 'is-active': showAdvancedFilters }">
            <el-icon><Filter /></el-icon>
            <span v-if="!isMobile">Advanced</span>
          </el-button>
        </div>

        <!-- Advanced Filters -->
        <el-collapse-transition>
          <div v-show="showAdvancedFilters" class="advanced-filters">
            <div class="filters-row">
              <el-select v-model="symbolFilter" placeholder="All Symbols" class="filter-select" clearable>
                <el-option
                  v-for="symbol in availableSymbols"
                  :key="symbol"
                  :label="symbol"
                  :value="symbol"
                />
              </el-select>
              <el-select v-model="performanceFilter" placeholder="Performance" class="filter-select" clearable>
                <el-option label="Profitable" value="profitable" />
                <el-option label="Losing" value="losing" />
                <el-option label="High Win Rate (>60%)" value="high_winrate" />
                <el-option label="Low Risk" value="low_risk" />
              </el-select>
              <el-date-picker
                v-model="dateRange"
                type="daterange"
                range-separator="To"
                start-placeholder="Created From"
                end-placeholder="Created To"
                class="date-picker"
                size="default"
              />
            </div>
          </div>
        </el-collapse-transition>
      </div>
    </el-card>

    <!-- Stats Overview -->
    <div class="stats-overview">
      <div class="stat-card">
        <div class="stat-icon total">
          <el-icon><TrendCharts /></el-icon>
        </div>
        <div class="stat-content">
          <div class="stat-value">{{ totalStrategies }}</div>
          <div class="stat-label">Total Strategies</div>
        </div>
      </div>
      <div class="stat-card">
        <div class="stat-icon active">
          <el-icon><VideoPlay /></el-icon>
        </div>
        <div class="stat-content">
          <div class="stat-value">{{ activeStrategies }}</div>
          <div class="stat-label">Active</div>
        </div>
      </div>
      <div class="stat-card">
        <div class="stat-icon profit">
          <el-icon><Money /></el-icon>
        </div>
        <div class="stat-content">
          <div class="stat-value">{{ formatPnL(totalPnL) }}</div>
          <div class="stat-label">Total P&L</div>
        </div>
      </div>
      <div class="stat-card">
        <div class="stat-icon winrate">
          <el-icon><Trophy /></el-icon>
        </div>
        <div class="stat-content">
          <div class="stat-value">{{ (averageWinRate * 100).toFixed(1) }}%</div>
          <div class="stat-label">Avg Win Rate</div>
        </div>
      </div>
    </div>

    <!-- View Toggle and Actions -->
    <div class="toolbar" v-if="!isMobile">
      <div class="view-toggle">
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
      <div class="toolbar-actions">
        <el-button @click="refreshStrategies" :loading="loading" size="small">
          <el-icon><Refresh /></el-icon>
          Refresh
        </el-button>
        <el-dropdown @command="handleBatchAction" v-if="selectedStrategies.length > 0">
          <el-button size="small">
            Batch Actions <el-icon><ArrowDown /></el-icon>
          </el-button>
          <template #dropdown>
            <el-dropdown-menu>
              <el-dropdown-item command="start">Start Selected</el-dropdown-item>
              <el-dropdown-item command="pause">Pause Selected</el-dropdown-item>
              <el-dropdown-item command="stop" divided>Stop Selected</el-dropdown-item>
              <el-dropdown-item command="delete" class="danger-item">Delete Selected</el-dropdown-item>
            </el-dropdown-menu>
          </template>
        </el-dropdown>
      </div>
    </div>

    <!-- Loading State -->
    <div v-if="loading" class="loading-container">
      <el-skeleton :rows="3" animated />
    </div>

    <!-- Empty State -->
    <el-empty
      v-else-if="filteredStrategies.length === 0"
      :description="searchText ? 'No strategies found matching your search criteria.' : 'No strategies created yet.'"
    >
      <template #image>
        <el-icon size="60" color="var(--text-secondary)"><TrendCharts /></el-icon>
      </template>
      <el-button type="primary" @click="showCreateDialog = true">Create Your First Strategy</el-button>
    </el-empty>

    <!-- Grid View with Enhanced Strategy Cards -->
    <div v-else-if="viewMode === 'grid' || isMobile" class="strategies-grid">
      <TransitionGroup name="strategy-card" tag="div">
        <StrategyCard
          v-for="strategy in paginatedStrategies"
          :key="strategy.id"
          :strategy="strategy"
          :selected="selectedStrategies.includes(strategy.id)"
          :compact="isMobile"
          @select="toggleSelection"
          @view="handleView"
          @edit="handleEdit"
          @start="handleStart"
          @pause="handlePause"
          @stop="handleStop"
          @duplicate="handleDuplicate"
          @delete="handleDelete"
          @export="handleExport"
          @toggle-favorite="handleToggleFavorite"
          class="strategy-card-item"
        />
      </TransitionGroup>
    </div>

    <!-- Enhanced List View (Desktop Only) -->
    <el-card v-else shadow="hover" class="table-card">
      <el-table
        :data="paginatedStrategies"
        style="width: 100%"
        @selection-change="handleSelectionChange"
        @sort-change="handleSortChange"
        v-loading="loading"
      >
        <el-table-column type="selection" width="55" />
        <el-table-column prop="name" label="Strategy" width="220" sortable="custom">
          <template #default="{ row }">
            <div class="strategy-name-cell">
              <div class="strategy-info">
                <el-link @click="handleView(row)" type="primary" class="strategy-link">
                  {{ row.name }}
                </el-link>
                <el-tag :type="getStatusType(row.status)" size="small" class="status-tag">
                  {{ row.status }}
                </el-tag>
              </div>
            </div>
          </template>
        </el-table-column>
        <el-table-column prop="type" label="Type" width="150" sortable="custom" />
        <el-table-column prop="symbol" label="Symbol" width="120" sortable="custom">
          <template #default="{ row }">
            <span class="symbol-text">{{ row.symbol }}</span>
          </template>
        </el-table-column>
        <el-table-column prop="performance.totalPnL" label="P&L" align="right" width="120" sortable="custom">
          <template #default="{ row }">
            <span :class="getPnLClass(row.performance.totalPnL)">
              {{ formatPnL(row.performance.totalPnL) }}
            </span>
          </template>
        </el-table-column>
        <el-table-column prop="performance.totalReturn" label="Return" align="right" width="100" sortable="custom">
          <template #default="{ row }">
            <span :class="getReturnClass(row.performance.totalReturn)">
              {{ (row.performance.totalReturn * 100).toFixed(1) }}%
            </span>
          </template>
        </el-table-column>
        <el-table-column prop="performance.winRate" label="Win Rate" align="right" width="100" sortable="custom">
          <template #default="{ row }">
            {{ (row.performance.winRate * 100).toFixed(1) }}%
          </template>
        </el-table-column>
        <el-table-column prop="performance.sharpeRatio" label="Sharpe" align="right" width="90" sortable="custom" />
        <el-table-column prop="performance.maxDrawdown" label="Max DD" align="right" width="100" sortable="custom">
          <template #default="{ row }">
            {{ (row.performance.maxDrawdown * 100).toFixed(1) }}%
          </template>
        </el-table-column>
        <el-table-column prop="performance.totalTrades" label="Trades" align="right" width="80" sortable="custom" />
        <el-table-column prop="createdAt" label="Created" width="180" sortable="custom" />
        <el-table-column label="Actions" width="180" fixed="right">
          <template #default="{ row }">
            <el-space size="small">
              <el-tooltip content="View Details" placement="top">
                <el-button size="small" circle @click="handleView(row)">
                  <el-icon><View /></el-icon>
                </el-button>
              </el-tooltip>
              <el-tooltip content="Edit Strategy" placement="top">
                <el-button size="small" circle @click="handleEdit(row)">
                  <el-icon><Edit /></el-icon>
                </el-button>
              </el-tooltip>
              <el-dropdown @command="(cmd) => handleStrategyAction(cmd, row)">
                <el-button size="small" circle>
                  <el-icon><MoreFilled /></el-icon>
                </el-button>
                <template #dropdown>
                  <el-dropdown-menu>
                    <el-dropdown-item command="duplicate">
                      <el-icon><DocumentCopy /></el-icon>
                      Duplicate
                    </el-dropdown-item>
                    <el-dropdown-item command="export">
                      <el-icon><Download /></el-icon>
                      Export
                    </el-dropdown-item>
                    <el-dropdown-item command="delete" divided class="danger-item">
                      <el-icon><Delete /></el-icon>
                      Delete
                    </el-dropdown-item>
                  </el-dropdown-menu>
                </template>
              </el-dropdown>
            </el-space>
          </template>
        </el-table-column>
      </el-table>

      <!-- Pagination -->
      <div class="pagination-wrapper">
        <el-pagination
          v-model:current-page="currentPage"
          v-model:page-size="pageSize"
          :total="filteredStrategies.length"
          :page-sizes="[12, 24, 48, 96]"
          layout="total, sizes, prev, pager, next, jumper"
          @size-change="handlePageSizeChange"
          @current-change="handleCurrentPageChange"
        />
      </div>
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
import { ref, computed, onMounted, watch } from 'vue'
import { useRouter } from 'vue-router'
import { ElMessage, ElMessageBox, ElNotification } from 'element-plus'
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
  TrendCharts,
  Money,
  Trophy,
  Refresh,
  Upload,
  Download,
  ArrowDown,
  Filter,
  DocumentCopy,
} from '@element-plus/icons-vue'
import { useStrategyStore } from '@/stores/strategy'
import { useResponsive } from '@/composables/useResponsive'
import { strategyService } from '@/services/strategyService'
import type {
  Strategy,
  StrategyStatus,
  StrategyType,
  StrategyFormData,
  StrategyFilter
} from '@/types/strategy'
import StrategyCard from '@/components/StrategyCard.vue'

// Define component name for keep-alive
defineOptions({
  name: 'Strategies'
})

const router = useRouter()
const strategyStore = useStrategyStore()
const { isMobile } = useResponsive()

// Reactive state
const loading = ref(false)
const strategies = ref<Strategy[]>([])
const searchText = ref('')
const statusFilter = ref('')
const typeFilter = ref('')
const symbolFilter = ref('')
const performanceFilter = ref('')
const dateRange = ref<[Date, Date] | null>(null)
const viewMode = ref<'grid' | 'list'>('grid')
const showCreateDialog = ref(false)
const showImportDialog = ref(false)
const showAdvancedFilters = ref(false)
const isEditing = ref(false)
const currentStep = ref(0)
const submitting = ref(false)

// Pagination
const currentPage = ref(1)
const pageSize = ref(12)

// Selection
const selectedStrategies = ref<string[]>([])

// Sorting
const sortField = ref<string>('createdAt')
const sortOrder = ref<'asc' | 'desc'>('desc')

// Enhanced form data with proper typing
const strategyForm = ref<StrategyFormData>({
  name: '',
  description: '',
  symbol: 'BTC-USDT',
  capital: 10000,
  type: 'ma_crossover',
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
    leverage: 1,
  },
  riskParams: {
    stopLoss: 2,
    takeProfit: 5,
    maxPositionSize: 10000,
    dailyLossLimit: 5,
    maxOpenPositions: 3,
    trailingStop: false,
    trailingStopDistance: 1,
    maxDrawdown: 10,
    useLimitOrders: true,
    paperTrading: true,
  },
  config: {
    autoRestart: false,
    notifications: {
      email: true,
      push: true,
      trades: true,
      alerts: true,
      daily: false,
      weekly: false,
    },
    schedule: {
      enabled: false,
      timezone: 'UTC',
    },
    logging: {
      level: 'info',
      saveToFile: true,
      maxFileSize: 10,
    },
  },
  tags: [],
  isPublic: false,
})

// Constants and options
const strategyStatuses = [
  { label: 'Draft', value: 'draft', type: 'info' },
  { label: 'Active', value: 'active', type: 'success' },
  { label: 'Paused', value: 'paused', type: 'warning' },
  { label: 'Stopped', value: 'stopped', type: 'danger' },
  { label: 'Error', value: 'error', type: 'danger' }
]

const strategyTypes = [
  { label: 'MA Crossover', value: 'ma_crossover' },
  { label: 'Grid Trading', value: 'grid_trading' },
  { label: 'RSI Strategy', value: 'rsi_strategy' },
  { label: 'Market Making', value: 'market_making' },
  { label: 'Arbitrage', value: 'arbitrage' },
  { label: 'Custom', value: 'custom' }
]

const availableSymbols = ['BTC-USDT', 'ETH-USDT', 'SOL-USDT', 'BNB-USDT', 'ADA-USDT', 'DOT-USDT', 'AVAX-USDT']

// Computed properties
const filteredStrategies = computed(() => {
  let result = [...strategies.value]

  // Text search
  if (searchText.value) {
    const searchLower = searchText.value.toLowerCase()
    result = result.filter(s =>
      s.name.toLowerCase().includes(searchLower) ||
      s.symbol.toLowerCase().includes(searchLower) ||
      s.type.toLowerCase().includes(searchLower) ||
      s.tags.some(tag => tag.toLowerCase().includes(searchLower))
    )
  }

  // Status filter
  if (statusFilter.value) {
    result = result.filter(s => s.status === statusFilter.value)
  }

  // Type filter
  if (typeFilter.value) {
    result = result.filter(s => s.type === typeFilter.value)
  }

  // Symbol filter
  if (symbolFilter.value) {
    result = result.filter(s => s.symbol === symbolFilter.value)
  }

  // Performance filter
  if (performanceFilter.value) {
    result = result.filter(s => {
      switch (performanceFilter.value) {
        case 'profitable':
          return s.performance.totalPnL > 0
        case 'losing':
          return s.performance.totalPnL < 0
        case 'high_winrate':
          return s.performance.winRate > 0.6
        case 'low_risk':
          return s.performance.maxDrawdown < 0.1
        default:
          return true
      }
    })
  }

  // Date range filter
  if (dateRange.value && dateRange.value.length === 2) {
    const [startDate, endDate] = dateRange.value
    result = result.filter(s => {
      const createdDate = new Date(s.createdAt)
      return createdDate >= startDate && createdDate <= endDate
    })
  }

  // Sorting
  result.sort((a, b) => {
    let aValue: any = getNestedValue(a, sortField.value)
    let bValue: any = getNestedValue(b, sortField.value)

    if (typeof aValue === 'string') {
      aValue = aValue.toLowerCase()
      bValue = bValue.toLowerCase()
    }

    if (sortOrder.value === 'asc') {
      return aValue > bValue ? 1 : -1
    } else {
      return aValue < bValue ? 1 : -1
    }
  })

  return result
})

const paginatedStrategies = computed(() => {
  const start = (currentPage.value - 1) * pageSize.value
  const end = start + pageSize.value
  return filteredStrategies.value.slice(start, end)
})

// Statistics computed properties
const totalStrategies = computed(() => strategies.value.length)
const activeStrategies = computed(() => strategies.value.filter(s => s.status === 'active').length)
const totalPnL = computed(() => strategies.value.reduce((sum, s) => sum + s.performance.totalPnL, 0))
const averageWinRate = computed(() => {
  const strategiesWithTrades = strategies.value.filter(s => s.performance.totalTrades > 0)
  if (strategiesWithTrades.length === 0) return 0
  const totalWinRate = strategiesWithTrades.reduce((sum, s) => sum + s.performance.winRate, 0)
  return totalWinRate / strategiesWithTrades.length
})

// Utility functions
const getNestedValue = (obj: any, path: string): any => {
  return path.split('.').reduce((current, key) => current?.[key], obj)
}

const getStatusType = (status: StrategyStatus): string => {
  const statusConfig = strategyStatuses.find(s => s.value === status)
  return statusConfig?.type || 'info'
}

const formatPnL = (value: number): string => {
  const sign = value >= 0 ? '+' : ''
  return `${sign}$${Math.abs(value).toFixed(2)}`
}

const formatReturn = (value: number): string => {
  const sign = value >= 0 ? '+' : ''
  return `${sign}${(value * 100).toFixed(1)}%`
}

const getPnLClass = (value: number): string => {
  return value >= 0 ? 'positive' : 'negative'
}

const getReturnClass = (value: number): string => {
  return value >= 0 ? 'positive' : 'negative'
}

const getStrategyDescription = (type: StrategyType): string => {
  const descriptions: Record<StrategyType, string> = {
    ma_crossover: 'Trend-following strategy that generates buy/sell signals when short-term MA crosses long-term MA',
    grid_trading: 'Places buy and sell orders at regular intervals to profit from price volatility in a range',
    rsi_strategy: 'Mean reversion strategy using RSI indicator to identify oversold/overbought conditions',
    market_making: 'Provides liquidity by placing buy and sell orders on both sides of the order book',
    arbitrage: 'Exploits price differences between different markets or trading pairs',
    custom: 'Custom strategy with user-defined parameters and logic'
  }
  return descriptions[type] || 'Select a strategy type to see description'
}

// Enhanced strategy management methods
const loadStrategies = async () => {
  loading.value = true
  try {
    const filter: StrategyFilter = {
      sortBy: sortField.value as any,
      sortOrder: sortOrder.value as any
    }
    const response = await strategyService.getStrategies(filter)
    if (response.success && response.data) {
      strategies.value = response.data.strategies
      strategyStore.setStrategies(strategies.value)
    } else {
      throw new Error(response.error || 'Failed to load strategies')
    }
  } catch (error) {
    console.error('Failed to load strategies:', error)
    ElMessage.error('Failed to load strategies')
    // Load mock data for development
    loadMockData()
  } finally {
    loading.value = false
  }
}

const loadMockData = () => {
  strategies.value = [
    {
      id: '1',
      name: 'MA Crossover BTC',
      description: 'Trend-following strategy using 20/50 moving averages',
      type: 'ma_crossover',
      symbol: 'BTC-USDT',
      status: 'active',
      capital: 10000,
      currentEquity: 11250.5,
      allocatedCapital: 10000,
      parameters: {
        shortPeriod: 20,
        longPeriod: 50,
        maType: 'EMA',
        positionSize: 0.5,
        leverage: 1
      },
      riskParams: {
        stopLoss: 2,
        takeProfit: 5,
        maxPositionSize: 10000,
        dailyLossLimit: 5,
        maxOpenPositions: 3,
        trailingStop: false,
        useLimitOrders: true,
        paperTrading: false
      },
      performance: {
        totalPnL: 1250.5,
        totalReturn: 0.125,
        winRate: 0.65,
        profitFactor: 1.8,
        maxDrawdown: 0.08,
        sharpeRatio: 1.4,
        sortinoRatio: 1.9,
        calmarRatio: 1.56,
        totalTrades: 42,
        winningTrades: 27,
        losingTrades: 15,
        averageWin: 85.2,
        averageLoss: -32.1,
        largestWin: 245.8,
        largestLoss: -125.3,
        averageTradeDuration: 4.2,
        expectancy: 28.5,
        monthlyReturns: [],
        dailyReturns: [],
        trades: [],
        equityCurve: []
      },
      createdAt: '2024-01-15T10:30:00Z',
      updatedAt: '2024-01-20T14:22:00Z',
      lastActiveAt: '2024-01-20T14:22:00Z',
      createdBy: 'user1',
      tags: ['trend', 'medium-risk'],
      isFavorite: true,
      isPublic: false,
      version: '1.2.0',
      alerts: [],
      signals: [],
      currentPositions: [],
      config: {
        autoRestart: false,
        notifications: {
          email: true,
          push: true,
          trades: true,
          alerts: true,
          daily: false,
          weekly: false
        },
        schedule: {
          enabled: false,
          timezone: 'UTC'
        },
        logging: {
          level: 'info',
          saveToFile: true,
          maxFileSize: 10
        },
        apiKeys: {
          exchange: 'okx',
          keyId: 'demo-key-1',
          permissions: ['read', 'trade']
        }
      }
    },
    {
      id: '2',
      name: 'Grid Trading ETH',
      description: 'Grid trading strategy for ETH with 10 levels',
      type: 'grid_trading',
      symbol: 'ETH-USDT',
      status: 'active',
      capital: 15000,
      currentEquity: 16200.3,
      allocatedCapital: 15000,
      parameters: {
        gridLevels: 10,
        priceRange: 5,
        orderSize: 100,
        gridType: 'arithmetic',
        positionSize: 0.3,
        leverage: 1
      },
      riskParams: {
        stopLoss: 3,
        takeProfit: 8,
        maxPositionSize: 15000,
        dailyLossLimit: 8,
        maxOpenPositions: 5,
        trailingStop: true,
        trailingStopDistance: 1.5,
        useLimitOrders: true,
        paperTrading: false
      },
      performance: {
        totalPnL: 1200.3,
        totalReturn: 0.08,
        winRate: 0.58,
        profitFactor: 1.6,
        maxDrawdown: 0.12,
        sharpeRatio: 1.1,
        sortinoRatio: 1.5,
        calmarRatio: 0.67,
        totalTrades: 35,
        winningTrades: 20,
        losingTrades: 15,
        averageWin: 95.8,
        averageLoss: -48.2,
        largestWin: 285.4,
        largestLoss: -156.7,
        averageTradeDuration: 2.8,
        expectancy: 18.7,
        monthlyReturns: [],
        dailyReturns: [],
        trades: [],
        equityCurve: []
      },
      createdAt: '2024-02-20T14:20:00Z',
      updatedAt: '2024-02-22T09:15:00Z',
      lastActiveAt: '2024-02-22T09:15:00Z',
      createdBy: 'user1',
      tags: ['grid', 'low-risk'],
      isFavorite: false,
      isPublic: false,
      version: '1.1.0',
      alerts: [],
      signals: [],
      currentPositions: [],
      config: {
        autoRestart: true,
        notifications: {
          email: true,
          push: true,
          trades: true,
          alerts: false,
          daily: true,
          weekly: false
        },
        schedule: {
          enabled: true,
          timezone: 'UTC',
          startTime: '09:00',
          endTime: '17:00',
          daysOfWeek: [1, 2, 3, 4, 5]
        },
        logging: {
          level: 'debug',
          saveToFile: true,
          maxFileSize: 20
        },
        apiKeys: {
          exchange: 'okx',
          keyId: 'demo-key-2',
          permissions: ['read', 'trade']
        }
      }
    },
    {
      id: '3',
      name: 'RSI Strategy SOL',
      description: 'Mean reversion strategy using RSI indicator',
      type: 'rsi_strategy',
      symbol: 'SOL-USDT',
      status: 'paused',
      capital: 8000,
      currentEquity: 7700.0,
      allocatedCapital: 8000,
      parameters: {
        rsiPeriod: 14,
        oversold: 30,
        overbought: 70,
        positionSize: 0.4,
        leverage: 2
      },
      riskParams: {
        stopLoss: 2.5,
        takeProfit: 6,
        maxPositionSize: 8000,
        dailyLossLimit: 6,
        maxOpenPositions: 2,
        trailingStop: false,
        useLimitOrders: true,
        paperTrading: true
      },
      performance: {
        totalPnL: -300.0,
        totalReturn: -0.0375,
        winRate: 0.45,
        profitFactor: 0.8,
        maxDrawdown: 0.15,
        sharpeRatio: -0.3,
        sortinoRatio: -0.4,
        calmarRatio: -0.25,
        totalTrades: 28,
        winningTrades: 13,
        losingTrades: 15,
        averageWin: 45.6,
        averageLoss: -52.8,
        largestWin: 125.9,
        largestLoss: -185.2,
        averageTradeDuration: 1.5,
        expectancy: -12.3,
        monthlyReturns: [],
        dailyReturns: [],
        trades: [],
        equityCurve: []
      },
      createdAt: '2024-03-10T09:15:00Z',
      updatedAt: '2024-03-15T16:30:00Z',
      lastActiveAt: '2024-03-15T16:30:00Z',
      createdBy: 'user1',
      tags: ['rsi', 'high-risk'],
      isFavorite: false,
      isPublic: false,
      version: '1.0.0',
      alerts: [],
      signals: [],
      currentPositions: [],
      config: {
        autoRestart: false,
        notifications: {
          email: false,
          push: true,
          trades: true,
          alerts: true,
          daily: false,
          weekly: false
        },
        schedule: {
          enabled: false,
          timezone: 'UTC'
        },
        logging: {
          level: 'warn',
          saveToFile: false,
          maxFileSize: 5
        },
        apiKeys: {
          exchange: 'okx',
          keyId: 'demo-key-3',
          permissions: ['read', 'trade']
        }
      }
    }
  ]
}

// Enhanced strategy actions
const handleStart = async (strategy: Strategy) => {
  try {
    const response = await strategyService.startStrategy(strategy.id)
    if (response.success) {
      const index = strategies.value.findIndex(s => s.id === strategy.id)
      if (index >= 0) {
        strategies.value[index].status = 'active'
      }
      ElNotification({
        title: 'Strategy Started',
        message: `"${strategy.name}" is now active`,
        type: 'success'
      })
    } else {
      throw new Error(response.error)
    }
  } catch (error) {
    console.error('Failed to start strategy:', error)
    ElMessage.error(`Failed to start strategy: ${error}`)
  }
}

const handlePause = async (strategy: Strategy) => {
  try {
    const response = await strategyService.pauseStrategy(strategy.id)
    if (response.success) {
      const index = strategies.value.findIndex(s => s.id === strategy.id)
      if (index >= 0) {
        strategies.value[index].status = 'paused'
      }
      ElNotification({
        title: 'Strategy Paused',
        message: `"${strategy.name}" has been paused`,
        type: 'warning'
      })
    } else {
      throw new Error(response.error)
    }
  } catch (error) {
    console.error('Failed to pause strategy:', error)
    ElMessage.error(`Failed to pause strategy: ${error}`)
  }
}

const handleStop = async (strategy: Strategy) => {
  try {
    await ElMessageBox.confirm(
      `Are you sure you want to stop "${strategy.name}"? This will close all positions.`,
      'Stop Strategy',
      {
        type: 'warning',
        confirmButtonText: 'Stop Strategy',
        confirmButtonClass: 'el-button--danger'
      }
    )

    const response = await strategyService.stopStrategy(strategy.id, true)
    if (response.success) {
      const index = strategies.value.findIndex(s => s.id === strategy.id)
      if (index >= 0) {
        strategies.value[index].status = 'stopped'
      }
      ElNotification({
        title: 'Strategy Stopped',
        message: `"${strategy.name}" has been stopped and positions closed`,
        type: 'info'
      })
    } else {
      throw new Error(response.error)
    }
  } catch (error) {
    if (error !== 'cancel') {
      console.error('Failed to stop strategy:', error)
      ElMessage.error(`Failed to stop strategy: ${error}`)
    }
  }
}

const handleView = (strategy: Strategy) => {
  router.push(`/strategies/${strategy.id}`)
}

const handleEdit = (strategy: Strategy) => {
  isEditing.value = true
  currentStep.value = 0
  // Load strategy data into form with proper type conversion
  Object.assign(strategyForm.value, {
    ...strategy,
    type: strategy.type as StrategyType
  })
  showCreateDialog.value = true
}

const handleDuplicate = async (strategy: Strategy) => {
  try {
    const response = await strategyService.duplicateStrategy(strategy.id, `${strategy.name} (Copy)`)
    if (response.success && response.data) {
      strategies.value.push(response.data)
      ElNotification({
        title: 'Strategy Duplicated',
        message: `"${strategy.name}" has been duplicated successfully`,
        type: 'success'
      })
    } else {
      throw new Error(response.error)
    }
  } catch (error) {
    console.error('Failed to duplicate strategy:', error)
    ElMessage.error(`Failed to duplicate strategy: ${error}`)
  }
}

const handleDelete = async (strategy: Strategy) => {
  try {
    await ElMessageBox.confirm(
      `Are you sure you want to delete "${strategy.name}"? This action cannot be undone.`,
      'Delete Strategy',
      {
        type: 'warning',
        confirmButtonText: 'Delete',
        confirmButtonClass: 'el-button--danger'
      }
    )

    const response = await strategyService.deleteStrategy(strategy.id)
    if (response.success) {
      strategies.value = strategies.value.filter(s => s.id !== strategy.id)
      strategyStore.removeStrategy(strategy.id)
      ElNotification({
        title: 'Strategy Deleted',
        message: `"${strategy.name}" has been deleted permanently`,
        type: 'info'
      })
    } else {
      throw new Error(response.error)
    }
  } catch (error) {
    if (error !== 'cancel') {
      console.error('Failed to delete strategy:', error)
      ElMessage.error(`Failed to delete strategy: ${error}`)
    }
  }
}

const handleExport = async (strategy: Strategy) => {
  try {
    const response = await strategyService.exportStrategies([strategy.id], false)
    if (response.success && response.data) {
      // Create download link
      const url = URL.createObjectURL(response.data)
      const link = document.createElement('a')
      link.href = url
      link.download = `${strategy.name}_${new Date().toISOString().split('T')[0]}.json`
      document.body.appendChild(link)
      link.click()
      document.body.removeChild(link)
      URL.revokeObjectURL(url)

      ElMessage.success('Strategy exported successfully')
    } else {
      throw new Error(response.error)
    }
  } catch (error) {
    console.error('Failed to export strategy:', error)
    ElMessage.error(`Failed to export strategy: ${error}`)
  }
}

const handleToggleFavorite = async (strategy: Strategy) => {
  const index = strategies.value.findIndex(s => s.id === strategy.id)
  if (index >= 0) {
    strategies.value[index].isFavorite = !strategies.value[index].isFavorite
    // Note: You would typically save this to the backend
    ElMessage.success(strategies.value[index].isFavorite ? 'Added to favorites' : 'Removed from favorites')
  }
}

// Enhanced batch operations
const handleBatchAction = async (action: string) => {
  if (selectedStrategies.value.length === 0) {
    ElMessage.warning('Please select strategies to perform batch actions')
    return
  }

  const actionLabels: Record<string, string> = {
    start: 'Start',
    pause: 'Pause',
    stop: 'Stop',
    delete: 'Delete'
  }

  const label = actionLabels[action] || action

  try {
    if (action === 'delete') {
      await ElMessageBox.confirm(
        `Are you sure you want to delete ${selectedStrategies.value.length} selected strategies? This action cannot be undone.`,
        `Delete ${selectedStrategies.value.length} Strategies`,
        {
          type: 'warning',
          confirmButtonText: 'Delete All',
          confirmButtonClass: 'el-button--danger'
        }
      )
    }

    let successCount = 0
    let failureCount = 0

    for (const strategyId of selectedStrategies.value) {
      try {
        const strategy = strategies.value.find(s => s.id === strategyId)
        if (!strategy) continue

        switch (action) {
          case 'start':
            await strategyService.startStrategy(strategyId)
            strategy.status = 'active'
            break
          case 'pause':
            await strategyService.pauseStrategy(strategyId)
            strategy.status = 'paused'
            break
          case 'stop':
            await strategyService.stopStrategy(strategyId, true)
            strategy.status = 'stopped'
            break
          case 'delete':
            await strategyService.deleteStrategy(strategyId)
            strategies.value = strategies.value.filter(s => s.id !== strategyId)
            break
        }
        successCount++
      } catch (error) {
        failureCount++
        console.error(`Failed to ${action} strategy ${strategyId}:`, error)
      }
    }

    selectedStrategies.value = []

    ElNotification({
      title: 'Batch Operation Complete',
      message: `${action} completed: ${successCount} successful, ${failureCount} failed`,
      type: failureCount === 0 ? 'success' : 'warning'
    })
  } catch (error) {
    if (error !== 'cancel') {
      console.error(`Batch ${action} failed:`, error)
      ElMessage.error(`Batch ${action} failed: ${error}`)
    }
  }
}

const handleExportSelected = async () => {
  if (selectedStrategies.value.length === 0) {
    ElMessage.warning('Please select strategies to export')
    return
  }

  try {
    const response = await strategyService.exportStrategies(selectedStrategies.value, false)
    if (response.success && response.data) {
      const url = URL.createObjectURL(response.data)
      const link = document.createElement('a')
      link.href = url
      link.download = `strategies_export_${new Date().toISOString().split('T')[0]}.json`
      document.body.appendChild(link)
      link.click()
      document.body.removeChild(link)
      URL.revokeObjectURL(url)

      ElMessage.success(`${selectedStrategies.value.length} strategies exported successfully`)
      selectedStrategies.value = []
    } else {
      throw new Error(response.error)
    }
  } catch (error) {
    console.error('Failed to export strategies:', error)
    ElMessage.error(`Failed to export strategies: ${error}`)
  }
}

// Event handlers
const handleSelectionChange = (selection: Strategy[]) => {
  selectedStrategies.value = selection.map(s => s.id)
}

const toggleSelection = (strategyId: string) => {
  const index = selectedStrategies.value.indexOf(strategyId)
  if (index >= 0) {
    selectedStrategies.value.splice(index, 1)
  } else {
    selectedStrategies.value.push(strategyId)
  }
}

const handleStrategyAction = (command: string, strategy: Strategy) => {
  switch (command) {
    case 'duplicate':
      handleDuplicate(strategy)
      break
    case 'export':
      handleExport(strategy)
      break
    case 'delete':
      handleDelete(strategy)
      break
  }
}

const handleSortChange = ({ prop, order }: { prop: string; order: string | null }) => {
  sortField.value = prop || 'createdAt'
  sortOrder.value = order === 'ascending' ? 'asc' : 'desc'
}

const handlePageSizeChange = (size: number) => {
  pageSize.value = size
  currentPage.value = 1
}

const handleCurrentPageChange = (page: number) => {
  currentPage.value = page
}

const refreshStrategies = async () => {
  await loadStrategies()
  ElMessage.success('Strategies refreshed successfully')
}

// Reset form with proper typing
const resetForm = () => {
  isEditing.value = false
  currentStep.value = 0
  strategyForm.value = {
    name: '',
    description: '',
    symbol: 'BTC-USDT',
    capital: 10000,
    type: 'ma_crossover',
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
      leverage: 1,
    },
    riskParams: {
      stopLoss: 2,
      takeProfit: 5,
      maxPositionSize: 10000,
      dailyLossLimit: 5,
      maxOpenPositions: 3,
      trailingStop: false,
      trailingStopDistance: 1,
      maxDrawdown: 10,
      useLimitOrders: true,
      paperTrading: true,
    },
    config: {
      autoRestart: false,
      notifications: {
        email: true,
        push: true,
        trades: true,
        alerts: true,
        daily: false,
        weekly: false,
      },
      schedule: {
        enabled: false,
        timezone: 'UTC',
      },
      logging: {
        level: 'info',
        saveToFile: true,
        maxFileSize: 10,
      },
    },
    tags: [],
    isPublic: false,
  }
}

// Watch for changes and load data
watch([sortField, sortOrder], () => {
  currentPage.value = 1
})

onMounted(() => {
  loadStrategies()
})
</script>

<style lang="scss" scoped>
@import '@/styles/variables.scss';
@import '@/styles/utilities.scss';
@import '@/styles/animations.scss';

.strategies-page {
  // Enhanced Header Styles
  .page-header {
    margin-bottom: $spacing-xl;

    .header-content {
      display: flex;
      justify-content: space-between;
      align-items: flex-start;
      gap: $spacing-lg;

      @include mobile {
        flex-direction: column;
        gap: $spacing-md;
      }

      .header-text {
        h2 {
          margin: 0 0 $spacing-xs 0;
          font-size: $font-size-3xl;
          font-weight: $font-weight-bold;
          color: var(--text-primary);

          @include mobile {
            font-size: $font-size-2xl;
          }
        }

        .header-subtitle {
          margin: 0;
          font-size: $font-size-base;
          color: var(--text-secondary);
          font-weight: $font-weight-regular;
        }
      }

      .header-actions {
        display: flex;
        gap: $spacing-md;
        align-items: center;

        @include mobile {
          flex-direction: column;
          width: 100%;
        }

        .btn-text {
          @include mobile {
            display: none;
          }
        }

        .import-btn,
        .export-btn {
          @include mobile {
            width: 100%;
          }
        }
      }
    }
  }

  // Enhanced Filters Card
  .filters-card {
    margin-bottom: $spacing-lg;
    background-color: var(--bg-secondary);
    border: 1px solid var(--border-color);

    .filters-content {
      .filters-row {
        display: flex;
        gap: $spacing-md;
        align-items: center;
        flex-wrap: wrap;

        .search-input {
          flex: 1;
          min-width: 280px;

          @include mobile {
            min-width: 200px;
          }
        }

        .filter-select {
          width: 160px;

          @include mobile {
            width: 140px;
          }
        }

        .el-button.is-active {
          background-color: var(--accent-color);
          border-color: var(--accent-color);
          color: white;
        }
      }

      .advanced-filters {
        margin-top: $spacing-lg;
        padding-top: $spacing-lg;
        border-top: 1px solid var(--border-color);

        .filters-row {
          display: flex;
          gap: $spacing-md;
          align-items: center;
          flex-wrap: wrap;
        }

        .date-picker {
          width: 240px;

          @include mobile {
            width: 100%;
          }
        }
      }
    }
  }

  // Stats Overview Cards
  .stats-overview {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
    gap: $spacing-lg;
    margin-bottom: $spacing-xl;

    @include mobile {
      grid-template-columns: 1fr;
      gap: $spacing-md;
    }

    .stat-card {
      display: flex;
      align-items: center;
      gap: $spacing-md;
      padding: $spacing-lg;
      background-color: var(--bg-secondary);
      border: 1px solid var(--border-color);
      border-radius: $radius-lg;
      transition: all $transition-base;

      &:hover {
        transform: translateY(-2px);
        box-shadow: 0 8px 16px var(--shadow-color);
      }

      .stat-icon {
        display: flex;
        align-items: center;
        justify-content: center;
        width: 48px;
        height: 48px;
        border-radius: $radius-lg;
        font-size: 20px;

        &.total {
          background-color: rgba(88, 102, 126, 0.1);
          color: var(--info-color);
        }

        &.active {
          background-color: rgba(52, 211, 153, 0.1);
          color: var(--success-color);
        }

        &.profit {
          background-color: rgba(34, 197, 94, 0.1);
          color: var(--success-color);
        }

        &.winrate {
          background-color: rgba(251, 191, 36, 0.1);
          color: var(--warning-color);
        }
      }

      .stat-content {
        flex: 1;

        .stat-value {
          font-size: $font-size-xl;
          font-weight: $font-weight-bold;
          color: var(--text-primary);
          line-height: 1.2;
        }

        .stat-label {
          font-size: $font-size-sm;
          color: var(--text-secondary);
          font-weight: $font-weight-medium;
          margin-top: 2px;
        }
      }
    }
  }

  // Toolbar with View Toggle and Actions
  .toolbar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: $spacing-lg;

    @include mobile {
      flex-direction: column;
      gap: $spacing-md;
      align-items: stretch;
    }

    .view-toggle {
      // View toggle styles handled by Element Plus
    }

    .toolbar-actions {
      display: flex;
      gap: $spacing-sm;
      align-items: center;

      @include mobile {
        justify-content: center;
      }
    }
  }

  .loading-container {
    margin-top: $spacing-3xl;
  }

  // Enhanced Grid View Styles
  .strategies-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(360px, 1fr));
    gap: $spacing-xl;

    @include mobile {
      grid-template-columns: 1fr;
      gap: $spacing-lg;
    }
  }

  // Strategy card transitions
  .strategy-card-item {
    transition: all $transition-base;
  }

  .strategy-card-enter-active,
  .strategy-card-leave-active {
    transition: all 0.3s ease;
  }

  .strategy-card-enter-from {
    opacity: 0;
    transform: translateY(20px);
  }

  .strategy-card-leave-to {
    opacity: 0;
    transform: scale(0.9);
  }

  .strategy-card-move {
    transition: transform 0.3s ease;
  }

  // Enhanced Table View Styles
  .table-card {
    margin-top: $spacing-lg;
    background-color: var(--bg-secondary);
    border: 1px solid var(--border-color);

    .strategy-name-cell {
      .strategy-info {
        .strategy-link {
          font-weight: $font-weight-semibold;
          margin-right: $spacing-sm;
        }

        .status-tag {
          font-size: $font-size-xs;
        }
      }
    }

    .symbol-text {
      font-family: 'SF Mono', 'Monaco', 'Inconsolata', 'Roboto Mono', monospace;
      color: var(--accent-color);
      font-weight: $font-weight-medium;
    }

    .pagination-wrapper {
      display: flex;
      justify-content: center;
      margin-top: $spacing-xl;
      padding-top: $spacing-lg;
      border-top: 1px solid var(--border-color);
    }
  }

  // Status option styling
  .status-option {
    display: flex;
    align-items: center;
    gap: $spacing-sm;
  }

  // Enhanced color classes
  .positive {
    color: var(--success-color) !important;
  }

  .negative {
    color: var(--danger-color) !important;
  }

  // Danger item styling
  .danger-item {
    color: var(--danger-color);

    &:hover {
      background-color: rgba(239, 68, 68, 0.1) !important;
    }
  }

  // Element Plus component deep styling
  :deep(.el-card) {
    background-color: var(--bg-secondary);
    border: 1px solid var(--border-color);
    color: var(--text-primary);
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);

    &:hover {
      box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
    }
  }

  :deep(.el-table) {
    background-color: var(--bg-secondary);
    color: var(--text-primary);
    border-radius: $radius-lg;
    overflow: hidden;

    th {
      background-color: var(--bg-tertiary);
      color: var(--text-secondary);
      font-weight: $font-weight-semibold;
      border-bottom: 1px solid var(--border-color);
    }

    td {
      border-bottom: 1px solid var(--border-color);
    }

    tr {
      background-color: var(--bg-secondary);
      transition: background-color $transition-fast;

      &:hover > td {
        background-color: var(--bg-tertiary);
      }
    }

    .el-table__header-wrapper {
      background-color: var(--bg-tertiary);
    }
  }

  :deep(.el-dialog) {
    background-color: var(--bg-secondary);
    border-radius: $radius-lg;

    .el-dialog__header {
      border-bottom: 1px solid var(--border-color);
      padding: $spacing-lg $spacing-xl;
    }

    .el-dialog__body {
      padding: $spacing-xl;
    }

    .el-dialog__footer {
      border-top: 1px solid var(--border-color);
      padding: $spacing-lg $spacing-xl;
    }
  }

  :deep(.el-form-item__label) {
    color: var(--text-secondary);
    font-weight: $font-weight-medium;
  }

  :deep(.el-step__title) {
    color: var(--text-primary);
    font-weight: $font-weight-medium;

    &.is-process,
    &.is-finish {
      color: var(--primary-color);
    }
  }

  :deep(.el-divider__text) {
    background-color: var(--bg-secondary);
    color: var(--text-secondary);
    font-weight: $font-weight-medium;
  }

  :deep(.el-dropdown-menu) {
    background-color: var(--bg-secondary);
    border: 1px solid var(--border-color);
    border-radius: $radius-md;

    .el-dropdown-menu__item {
      color: var(--text-primary);

      &:hover {
        background-color: var(--hover-bg);
        color: var(--accent-color);
      }

      &.is-disabled {
        color: var(--text-disabled);
      }
    }

    .el-dropdown-menu__item--divided {
      border-top: 1px solid var(--border-color);
    }
  }

  :deep(.el-collapse-transition) {
    transition: all $transition-base ease;
  }

  :deep(.el-tag) {
    border: 1px solid;

    &.el-tag--success {
      background-color: rgba(52, 211, 153, 0.1);
      border-color: rgba(52, 211, 153, 0.3);
      color: var(--success-color);
    }

    &.el-tag--warning {
      background-color: rgba(251, 191, 36, 0.1);
      border-color: rgba(251, 191, 36, 0.3);
      color: var(--warning-color);
    }

    &.el-tag--danger {
      background-color: rgba(239, 68, 68, 0.1);
      border-color: rgba(239, 68, 68, 0.3);
      color: var(--danger-color);
    }

    &.el-tag--info {
      background-color: rgba(88, 102, 126, 0.1);
      border-color: rgba(88, 102, 126, 0.3);
      color: var(--info-color);
    }
  }

  // Button enhancements
  :deep(.el-button) {
    transition: all $transition-fast;

    &:hover {
      transform: translateY(-1px);
    }

    &.el-button--primary {
      background: linear-gradient(135deg, var(--primary-color), #4f46e5);
      border: none;

      &:hover {
        background: linear-gradient(135deg, #4f46e5, #3730a3);
      }
    }

    &.el-button--success {
      background: linear-gradient(135deg, var(--success-color), #059669);
      border: none;

      &:hover {
        background: linear-gradient(135deg, #059669, #047857);
      }
    }

    &.el-button--warning {
      background: linear-gradient(135deg, var(--warning-color), #d97706);
      border: none;

      &:hover {
        background: linear-gradient(135deg, #d97706, #b45309);
      }
    }

    &.el-button--danger {
      background: linear-gradient(135deg, var(--danger-color), #dc2626);
      border: none;

      &:hover {
        background: linear-gradient(135deg, #dc2626, #b91c1c);
      }
    }
  }

  // Empty state styling
  :deep(.el-empty) {
    padding: $spacing-4xl;

    .el-empty__description {
      color: var(--text-secondary);
      font-size: $font-size-base;
      margin-top: $spacing-lg;
    }
  }
}
</style>
