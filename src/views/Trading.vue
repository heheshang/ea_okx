<template>
  <div class="trading-monitor">
    <!-- Real-time Market Data -->
    <el-row :gutter="20">
      <el-col :span="24">
        <el-card shadow="hover">
          <template #header>
            <div class="card-header">
              <span>Real-time Market Data</span>
              <el-space>
                <el-select v-model="selectedSymbol" placeholder="Select Symbol" size="small" style="width: 150px">
                  <el-option
                    v-for="symbol in symbols"
                    :key="symbol"
                    :label="symbol"
                    :value="symbol"
                  />
                </el-select>
                <el-button :icon="Refresh" size="small" @click="refreshData">Refresh</el-button>
              </el-space>
            </div>
          </template>
          <el-row :gutter="15">
            <el-col :span="6" v-for="price in marketPrices" :key="price.symbol">
              <div class="price-card">
                <div class="symbol">{{ price.symbol }}</div>
                <div class="price" :class="price.change >= 0 ? 'text-success' : 'text-danger'">
                  ${{ price.price.toLocaleString() }}
                </div>
                <div class="change" :class="price.change >= 0 ? 'text-success' : 'text-danger'">
                  {{ price.change >= 0 ? '+' : '' }}{{ price.change.toFixed(2) }}%
                </div>
              </div>
            </el-col>
          </el-row>
        </el-card>
      </el-col>
    </el-row>

    <!-- Active Orders & Positions -->
    <el-row :gutter="20" class="mt-20">
      <el-col :span="12">
        <el-card shadow="hover">
          <template #header>
            <div class="card-header">
              <span>Active Orders</span>
              <el-button size="small" type="primary" @click="showPlaceOrderDialog = true">
                <el-icon><Plus /></el-icon>
                Place Order
              </el-button>
            </div>
          </template>
          <el-table :data="activeOrders" style="width: 100%" max-height="400">
            <el-table-column prop="orderId" label="Order ID" width="120">
              <template #default="{ row }">
                <span class="mono">{{ row.orderId.substring(0, 8) }}</span>
              </template>
            </el-table-column>
            <el-table-column prop="symbol" label="Symbol" width="120" />
            <el-table-column prop="side" label="Side" width="80">
              <template #default="{ row }">
                <el-tag :type="row.side === 'Buy' ? 'success' : 'danger'" size="small">
                  {{ row.side }}
                </el-tag>
              </template>
            </el-table-column>
            <el-table-column prop="type" label="Type" width="100" />
            <el-table-column prop="quantity" label="Quantity" align="right" />
            <el-table-column prop="price" label="Price" align="right">
              <template #default="{ row }">
                {{ row.price ? '$' + row.price.toLocaleString() : 'Market' }}
              </template>
            </el-table-column>
            <el-table-column prop="status" label="Status" width="100">
              <template #default="{ row }">
                <el-tag :type="getOrderStatusType(row.status)" size="small">
                  {{ row.status }}
                </el-tag>
              </template>
            </el-table-column>
            <el-table-column label="Actions" width="100" fixed="right">
              <template #default="{ row }">
                <el-button
                  size="small"
                  type="danger"
                  text
                  @click="cancelOrder(row.orderId)"
                  :disabled="!canCancelOrder(row.status)"
                >
                  Cancel
                </el-button>
              </template>
            </el-table-column>
          </el-table>
        </el-card>
      </el-col>

      <el-col :span="12">
        <el-card shadow="hover">
          <template #header>
            <div class="card-header">
              <span>Current Positions</span>
              <el-button size="small" type="danger" @click="closeAllPositions">
                Close All
              </el-button>
            </div>
          </template>
          <el-table :data="currentPositions" style="width: 100%" max-height="400">
            <el-table-column prop="symbol" label="Symbol" width="120" />
            <el-table-column prop="side" label="Side" width="80">
              <template #default="{ row }">
                <el-tag :type="row.side === 'Long' ? 'success' : 'danger'" size="small">
                  {{ row.side }}
                </el-tag>
              </template>
            </el-table-column>
            <el-table-column prop="quantity" label="Quantity" align="right" />
            <el-table-column prop="entryPrice" label="Entry Price" align="right">
              <template #default="{ row }">
                ${{ row.entryPrice.toLocaleString() }}
              </template>
            </el-table-column>
            <el-table-column prop="currentPrice" label="Current Price" align="right">
              <template #default="{ row }">
                ${{ row.currentPrice.toLocaleString() }}
              </template>
            </el-table-column>
            <el-table-column prop="pnl" label="Unrealized P&L" align="right">
              <template #default="{ row }">
                <span :class="row.pnl >= 0 ? 'text-success' : 'text-danger'">
                  {{ formatPnL(row.pnl) }}
                </span>
              </template>
            </el-table-column>
            <el-table-column label="Actions" width="100" fixed="right">
              <template #default="{ row }">
                <el-button size="small" type="warning" text @click="closePosition(row.symbol)">
                  Close
                </el-button>
              </template>
            </el-table-column>
          </el-table>
        </el-card>
      </el-col>
    </el-row>

    <!-- Trade History -->
    <el-row :gutter="20" class="mt-20">
      <el-col :span="24">
        <el-card shadow="hover">
          <template #header>
            <div class="card-header">
              <span>Trade History</span>
              <el-space>
                <el-date-picker
                  v-model="dateRange"
                  type="daterange"
                  range-separator="To"
                  start-placeholder="Start date"
                  end-placeholder="End date"
                  size="small"
                />
                <el-button size="small" :icon="Download">Export</el-button>
              </el-space>
            </div>
          </template>
          <el-table :data="tradeHistory" style="width: 100%" max-height="400">
            <el-table-column prop="timestamp" label="Time" width="180" />
            <el-table-column prop="orderId" label="Order ID" width="120">
              <template #default="{ row }">
                <span class="mono">{{ row.orderId.substring(0, 8) }}</span>
              </template>
            </el-table-column>
            <el-table-column prop="symbol" label="Symbol" width="120" />
            <el-table-column prop="side" label="Side" width="80">
              <template #default="{ row }">
                <el-tag :type="row.side === 'Buy' ? 'success' : 'danger'" size="small">
                  {{ row.side }}
                </el-tag>
              </template>
            </el-table-column>
            <el-table-column prop="quantity" label="Quantity" align="right" />
            <el-table-column prop="price" label="Price" align="right">
              <template #default="{ row }">
                ${{ row.price.toLocaleString() }}
              </template>
            </el-table-column>
            <el-table-column prop="fee" label="Fee" align="right">
              <template #default="{ row }">
                ${{ row.fee.toFixed(2) }}
              </template>
            </el-table-column>
            <el-table-column prop="pnl" label="P&L" align="right">
              <template #default="{ row }">
                <span :class="row.pnl >= 0 ? 'text-success' : 'text-danger'">
                  {{ formatPnL(row.pnl) }}
                </span>
              </template>
            </el-table-column>
            <el-table-column prop="strategy" label="Strategy" width="150" />
          </el-table>
        </el-card>
      </el-col>
    </el-row>

    <!-- Place Order Dialog -->
    <el-dialog v-model="showPlaceOrderDialog" title="Place Order" width="500px">
      <el-form :model="orderForm" label-width="120px">
        <el-form-item label="Symbol">
          <el-select v-model="orderForm.symbol" placeholder="Select symbol">
            <el-option
              v-for="symbol in symbols"
              :key="symbol"
              :label="symbol"
              :value="symbol"
            />
          </el-select>
        </el-form-item>
        <el-form-item label="Side">
          <el-radio-group v-model="orderForm.side">
            <el-radio label="Buy">Buy</el-radio>
            <el-radio label="Sell">Sell</el-radio>
          </el-radio-group>
        </el-form-item>
        <el-form-item label="Order Type">
          <el-select v-model="orderForm.type" placeholder="Select type">
            <el-option label="Market" value="Market" />
            <el-option label="Limit" value="Limit" />
            <el-option label="Stop Loss" value="StopLoss" />
          </el-select>
        </el-form-item>
        <el-form-item label="Quantity">
          <el-input-number v-model="orderForm.quantity" :min="0.01" :step="0.01" />
        </el-form-item>
        <el-form-item label="Price" v-if="orderForm.type === 'Limit'">
          <el-input-number v-model="orderForm.price" :min="0" :step="0.01" />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="showPlaceOrderDialog = false">Cancel</el-button>
        <el-button type="primary" @click="placeOrder">Place Order</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { ElMessage, ElMessageBox } from 'element-plus'
import { Refresh, Plus, Download } from '@element-plus/icons-vue'

// Define component name for keep-alive
defineOptions({
  name: 'Trading'
})

interface MarketPrice {
  symbol: string
  price: number
  change: number
}

interface Order {
  orderId: string
  symbol: string
  side: string
  type: string
  quantity: number
  price: number | null
  status: string
}

interface Position {
  symbol: string
  side: string
  quantity: number
  entryPrice: number
  currentPrice: number
  pnl: number
}

interface Trade {
  timestamp: string
  orderId: string
  symbol: string
  side: string
  quantity: number
  price: number
  fee: number
  pnl: number
  strategy: string
}

const selectedSymbol = ref('BTC-USDT')
const symbols = ref(['BTC-USDT', 'ETH-USDT', 'SOL-USDT', 'BNB-USDT'])
const dateRange = ref<[Date, Date]>()
const showPlaceOrderDialog = ref(false)

const marketPrices = ref<MarketPrice[]>([
  { symbol: 'BTC-USDT', price: 45300, change: 2.5 },
  { symbol: 'ETH-USDT', price: 3200, change: -1.2 },
  { symbol: 'SOL-USDT', price: 120, change: 5.3 },
  { symbol: 'BNB-USDT', price: 580, change: 0.8 },
])

const activeOrders = ref<Order[]>([
  { orderId: 'a1b2c3d4-e5f6-7890', symbol: 'BTC-USDT', side: 'Buy', type: 'Limit', quantity: 0.5, price: 45000, status: 'Submitted' },
  { orderId: 'b2c3d4e5-f6g7-8901', symbol: 'ETH-USDT', side: 'Sell', type: 'Market', quantity: 2.0, price: null, status: 'Partial' },
])

const currentPositions = ref<Position[]>([
  { symbol: 'BTC-USDT', side: 'Long', quantity: 1.5, entryPrice: 44500, currentPrice: 45300, pnl: 1200 },
  { symbol: 'ETH-USDT', side: 'Short', quantity: 5.0, entryPrice: 3250, currentPrice: 3200, pnl: 250 },
])

const tradeHistory = ref<Trade[]>([
  { timestamp: '2024-11-20 10:30:15', orderId: 'c3d4e5f6-g7h8-9012', symbol: 'BTC-USDT', side: 'Buy', quantity: 0.5, price: 44800, fee: 22.40, pnl: 250, strategy: 'MA Crossover' },
  { timestamp: '2024-11-20 09:15:30', orderId: 'd4e5f6g7-h8i9-0123', symbol: 'ETH-USDT', side: 'Sell', quantity: 2.0, price: 3220, fee: 6.44, pnl: -40, strategy: 'Grid Trading' },
  { timestamp: '2024-11-20 08:45:00', orderId: 'e5f6g7h8-i9j0-1234', symbol: 'SOL-USDT', side: 'Buy', quantity: 10.0, price: 115, fee: 1.15, pnl: 50, strategy: 'RSI Strategy' },
])

const orderForm = ref({
  symbol: 'BTC-USDT',
  side: 'Buy',
  type: 'Limit',
  quantity: 0.1,
  price: 45000,
})

let unlistenMarketData: (() => void) | null = null
let unlistenOrderUpdate: (() => void) | null = null

const getOrderStatusType = (status: string) => {
  const types: Record<string, any> = {
    'Submitted': 'info',
    'Partial': 'warning',
    'Filled': 'success',
    'Cancelled': 'info',
    'Rejected': 'danger',
  }
  return types[status] || 'info'
}

const canCancelOrder = (status: string) => {
  return ['Submitted', 'Partial'].includes(status)
}

const formatPnL = (value: number) => {
  const sign = value >= 0 ? '+' : ''
  return `${sign}$${value.toFixed(2)}`
}

const refreshData = async () => {
  try {
    const data = await invoke('get_market_data', { symbol: selectedSymbol.value })
    console.log('Market data:', data)
    ElMessage.success('Data refreshed')
  } catch (error) {
    console.error('Failed to refresh data:', error)
    ElMessage.error('Failed to refresh data')
  }
}

const placeOrder = async () => {
  try {
    await invoke('place_order', {
      symbol: orderForm.value.symbol,
      side: orderForm.value.side,
      orderType: orderForm.value.type,
      quantity: orderForm.value.quantity,
      price: orderForm.value.price,
    })
    ElMessage.success('Order placed successfully')
    showPlaceOrderDialog.value = false
    // Refresh orders
  } catch (error) {
    console.error('Failed to place order:', error)
    ElMessage.error('Failed to place order')
  }
}

const cancelOrder = async (orderId: string) => {
  try {
    await ElMessageBox.confirm('Are you sure to cancel this order?', 'Confirm', {
      type: 'warning',
    })
    await invoke('cancel_order', { orderId })
    ElMessage.success('Order cancelled')
    // Remove from active orders
    activeOrders.value = activeOrders.value.filter(o => o.orderId !== orderId)
  } catch (error) {
    if (error !== 'cancel') {
      console.error('Failed to cancel order:', error)
      ElMessage.error('Failed to cancel order')
    }
  }
}

const closePosition = async (symbol: string) => {
  try {
    await ElMessageBox.confirm(`Close position for ${symbol}?`, 'Confirm', {
      type: 'warning',
    })
    await invoke('close_position', { symbol })
    ElMessage.success('Position closed')
    // Remove from positions
    currentPositions.value = currentPositions.value.filter(p => p.symbol !== symbol)
  } catch (error) {
    if (error !== 'cancel') {
      console.error('Failed to close position:', error)
      ElMessage.error('Failed to close position')
    }
  }
}

const closeAllPositions = async () => {
  try {
    await ElMessageBox.confirm('Are you sure to close all positions?', 'Warning', {
      type: 'warning',
      confirmButtonText: 'Close All',
    })
    await invoke('close_all_positions')
    ElMessage.success('All positions closed')
    currentPositions.value = []
  } catch (error) {
    if (error !== 'cancel') {
      console.error('Failed to close all positions:', error)
      ElMessage.error('Failed to close all positions')
    }
  }
}

onMounted(async () => {
  // Listen to real-time market data updates
  unlistenMarketData = await listen('market-data-update', (event: any) => {
    const data = event.payload
    const index = marketPrices.value.findIndex(p => p.symbol === data.symbol)
    if (index !== -1) {
      marketPrices.value[index] = {
        symbol: data.symbol,
        price: data.price,
        change: data.change,
      }
    }
  })

  // Listen to order status updates
  unlistenOrderUpdate = await listen('order-status-change', (event: any) => {
    const data = event.payload
    const index = activeOrders.value.findIndex(o => o.orderId === data.order_id)
    if (index !== -1) {
      activeOrders.value[index].status = data.status
      if (data.status === 'Filled' || data.status === 'Cancelled') {
        // Remove from active orders after delay
        setTimeout(() => {
          activeOrders.value = activeOrders.value.filter(o => o.orderId !== data.order_id)
        }, 3000)
      }
    }
  })
})

onUnmounted(() => {
  if (unlistenMarketData) unlistenMarketData()
  if (unlistenOrderUpdate) unlistenOrderUpdate()
})
</script>

<style lang="scss" scoped>
.trading-monitor {
  .mt-20 {
    margin-top: 20px;
  }

  .card-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .price-card {
    padding: 15px;
    background-color: var(--bg-tertiary);
    border-radius: 8px;
    text-align: center;

    .symbol {
      font-size: 14px;
      color: var(--text-secondary);
      margin-bottom: 8px;
    }

    .price {
      font-size: 24px;
      font-weight: bold;
      margin-bottom: 5px;
    }

    .change {
      font-size: 14px;
    }
  }

  .mono {
    font-family: 'Courier New', monospace;
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
}
</style>
