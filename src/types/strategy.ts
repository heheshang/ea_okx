/**
 * Strategy Management System Types
 * Comprehensive type definitions for trading strategies
 */

export type StrategyStatus = 'draft' | 'active' | 'paused' | 'stopped' | 'error'
export type StrategyType = 'ma_crossover' | 'grid_trading' | 'rsi_strategy' | 'market_making' | 'arbitrage' | 'custom'
export type Timeframe = '1m' | '5m' | '15m' | '30m' | '1h' | '4h' | '1d' | '1w'
export type OrderType = 'market' | 'limit' | 'stop' | 'stop_limit'
export type PositionSide = 'long' | 'short' | 'both'

export interface StrategyParameters {
  // MA Crossover Parameters
  shortPeriod?: number
  longPeriod?: number
  maType?: 'SMA' | 'EMA' | 'WMA'

  // Grid Trading Parameters
  gridLevels?: number
  priceRange?: number
  orderSize?: number
  gridType?: 'arithmetic' | 'geometric'

  // RSI Strategy Parameters
  rsiPeriod?: number
  oversold?: number
  overbought?: number

  // Common Parameters
  positionSize?: number
  leverage?: number
  timeframe?: Timeframe
  [key: string]: any
}

export interface RiskParameters {
  stopLoss?: number
  takeProfit?: number
  maxPositionSize?: number
  dailyLossLimit?: number
  maxOpenPositions?: number
  trailingStop?: boolean
  trailingStopDistance?: number
  maxDrawdown?: number
  useLimitOrders?: boolean
  paperTrading?: boolean
}

export interface StrategyPerformance {
  totalPnL: number
  totalReturn: number
  winRate: number
  profitFactor: number
  maxDrawdown: number
  sharpeRatio: number
  sortinoRatio: number
  calmarRatio: number
  totalTrades: number
  winningTrades: number
  losingTrades: number
  averageWin: number
  averageLoss: number
  largestWin: number
  largestLoss: number
  averageTradeDuration: number
  expectancy: number
  monthlyReturns: MonthlyReturn[]
  dailyReturns: DailyReturn[]
  trades: Trade[]
  equityCurve: EquityPoint[]
}

export interface MonthlyReturn {
  month: string
  return: number
  pnl: number
  trades: number
}

export interface DailyReturn {
  date: string
  return: number
  equity: number
  trades: number
}

export interface Trade {
  id: string
  strategyId: string
  symbol: string
  side: 'buy' | 'sell'
  type: OrderType
  quantity: number
  price: number
  timestamp: string
  pnl?: number
  commission: number
  status: 'open' | 'closed' | 'cancelled'
  exitPrice?: number
  exitTimestamp?: string
  fees: number
  notes?: string
}

export interface EquityPoint {
  timestamp: string
  equity: number
  pnl: number
  openPositions: number
}

export interface StrategyTemplate {
  id: string
  name: string
  description: string
  type: StrategyType
  category: string
  difficulty: 'beginner' | 'intermediate' | 'advanced'
  minCapital: number
  expectedReturn?: {
    annual: number
    maxDrawdown: number
    sharpeRatio: number
  }
  riskLevel: 'low' | 'medium' | 'high'
  parameters: StrategyParameters
  riskParams: RiskParameters
  tags: string[]
  isBuiltIn: boolean
  author?: string
  usage: number
  rating: number
}

export interface StrategySignal {
  id: string
  strategyId: string
  symbol: string
  type: 'buy' | 'sell' | 'close'
  price: number
  quantity: number
  timestamp: string
  confidence: number
  reason: string
  indicators?: Record<string, any>
  status: 'pending' | 'executed' | 'cancelled' | 'failed'
  executedAt?: string
  executedPrice?: number
}

export interface StrategyAlert {
  id: string
  strategyId: string
  type: 'warning' | 'error' | 'info' | 'success'
  title: string
  message: string
  timestamp: string
  severity: 'low' | 'medium' | 'high' | 'critical'
  isRead: boolean
  metadata?: Record<string, any>
}

export interface Strategy {
  id: string
  name: string
  description: string
  type: StrategyType
  symbol: string
  status: StrategyStatus
  capital: number
  currentEquity: number
  allocatedCapital: number
  parameters: StrategyParameters
  riskParams: RiskParameters
  performance: StrategyPerformance
  createdAt: string
  updatedAt: string
  lastActiveAt?: string
  createdBy: string
  tags: string[]
  isFavorite: boolean
  isPublic: boolean
  templateId?: string
  parentStrategyId?: string
  version: string
  backtestResults?: BacktestResult
  liveResults?: StrategyPerformance
  alerts: StrategyAlert[]
  signals: StrategySignal[]
  currentPositions: Position[]
  config: StrategyConfig
}

export interface StrategyConfig {
  autoRestart: boolean
  notifications: {
    email: boolean
    push: boolean
    trades: boolean
    alerts: boolean
    daily: boolean
    weekly: boolean
  }
  schedule: {
    enabled: boolean
    timezone: string
    startTime?: string
    endTime?: string
    daysOfWeek?: number[]
  }
  apiKeys: {
    exchange: string
    keyId: string
    permissions: string[]
  }
  logging: {
    level: 'debug' | 'info' | 'warn' | 'error'
    saveToFile: boolean
    maxFileSize: number
  }
}

export interface Position {
  id: string
  strategyId: string
  symbol: string
  side: PositionSide
  quantity: number
  entryPrice: number
  currentPrice: number
  pnl: number
  pnlPercentage: number
  margin: number
  marginPercentage: number
  leverage: number
  timestamp: string
  fees: number
  status: 'open' | 'closed' | 'liquidated'
  stopLoss?: number
  takeProfit?: number
  trailingStop?: number
}

export interface BacktestResult {
  id: string
  strategyId: string
  name: string
  startDate: string
  endDate: string
  initialCapital: number
  finalCapital: number
  totalReturn: number
  annualizedReturn: number
  maxDrawdown: number
  sharpeRatio: number
  sortinoRatio: number
  calmarRatio: number
  winRate: number
  profitFactor: number
  totalTrades: number
  winningTrades: number
  losingTrades: number
  averageWin: number
  averageLoss: number
  largestWin: number
  largestLoss: number
  averageTradeDuration: number
  monthlyReturns: MonthlyReturn[]
  equityCurve: EquityPoint[]
  trades: Trade[]
  parameters: StrategyParameters
  riskParams: RiskParameters
  createdAt: string
  status: 'running' | 'completed' | 'failed' | 'cancelled'
  progress?: number
  errorMessage?: string
}

export interface StrategyFilter {
  search?: string
  status?: StrategyStatus[]
  type?: StrategyType[]
  symbol?: string[]
  tags?: string[]
  dateRange?: {
    start: string
    end: string
  }
  performanceRange?: {
    minReturn?: number
    maxReturn?: number
    minWinRate?: number
    maxWinRate?: number
  }
  capitalRange?: {
    min: number
    max: number
  }
  createdBy?: string[]
  isFavorite?: boolean
  isPublic?: boolean
  dateField?: 'createdAt' | 'updatedAt' | 'lastActiveAt'
  sortBy?: 'name' | 'createdAt' | 'updatedAt' | 'performance' | 'winRate' | 'pnl'
  sortOrder?: 'asc' | 'desc'
}

export interface StrategyStats {
  total: number
  byStatus: Record<StrategyStatus, number>
  byType: Record<StrategyType, number>
  activeStrategies: number
  totalCapital: number
  totalPnL: number
  averageReturn: number
  bestPerformer: Strategy
  worstPerformer: Strategy
  topSymbols: Array<{
    symbol: string
    count: number
    totalPnL: number
  }>
  recentActivity: Array<{
    strategyId: string
    strategyName: string
    action: string
    timestamp: string
  }>
}

// Strategy creation/editing form types
export interface StrategyFormData {
  // Basic Info
  name: string
  description: string
  symbol: string
  capital: number
  type: StrategyType
  timeframe: Timeframe

  // Parameters
  parameters: StrategyParameters

  // Risk Management
  riskParams: RiskParameters

  // Configuration
  config: Partial<StrategyConfig>

  // Metadata
  tags: string[]
  isPublic: boolean
  templateId?: string
}

// Strategy validation types
export interface StrategyValidationRule {
  field: string
  rule: 'required' | 'min' | 'max' | 'pattern' | 'custom'
  value?: any
  message: string
  validator?: (value: any) => boolean
}

export interface StrategyValidationError {
  field: string
  message: string
  value: any
}

// Strategy comparison types
export interface StrategyComparison {
  strategies: Strategy[]
  metrics: Array<{
    key: keyof StrategyPerformance
    label: string
    format: 'currency' | 'percentage' | 'number' | 'ratio'
  }>
  period: '1d' | '1w' | '1m' | '3m' | '6m' | '1y' | 'all'
  benchmark?: string
}

// Strategy export/import types
export interface StrategyExport {
  version: string
  strategies: Strategy[]
  templates?: StrategyTemplate[]
  exportedAt: string
  exportedBy: string
  includePrivateData: boolean
}

export interface StrategyImportResult {
  success: boolean
  imported: number
  skipped: number
  errors: string[]
  warnings: string[]
  duplicates: string[]
}

// Market data types for strategy execution
export interface MarketData {
  symbol: string
  price: number
  volume: number
  high24h: number
  low24h: number
  change24h: number
  changePercent24h: number
  timestamp: string
  indicators?: Record<string, number>
}

// Strategy execution context
export interface ExecutionContext {
  strategy: Strategy
  marketData: MarketData
  positions: Position[]
  orders: any[]
  balance: number
  timestamp: string
  isBacktest: boolean
}

// Strategy action types
export type StrategyAction =
  | 'create'
  | 'update'
  | 'delete'
  | 'start'
  | 'stop'
  | 'pause'
  | 'resume'
  | 'restart'
  | 'duplicate'
  | 'export'
  | 'import'
  | 'backtest'
  | 'optimize'

// API response types
export interface StrategyApiResponse<T = any> {
  success: boolean
  data?: T
  error?: string
  message?: string
  timestamp: string
}

export interface StrategyListResponse extends StrategyApiResponse {
  data?: {
    strategies: Strategy[]
    total: number
    page: number
    pageSize: number
    stats: StrategyStats
  }
}