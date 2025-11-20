/**
 * Strategy Management Service
 * Handles all CRUD operations and business logic for trading strategies
 */

import { invoke } from '@tauri-apps/api/core'
import type {
  Strategy,
  StrategyFormData,
  StrategyFilter,
  StrategyListResponse,
  StrategyApiResponse,
  StrategyStats,
  BacktestResult,
  StrategyTemplate,
  StrategyPerformance,
  Trade,
  Position,
  StrategySignal
} from '@/types/strategy'

export class StrategyService {
  private baseUrl = '/api/strategies'

  /**
   * Get all strategies with filtering and pagination
   */
  async getStrategies(filter?: StrategyFilter): Promise<StrategyListResponse> {
    try {
      const response = await invoke<StrategyListResponse>('get_strategies', { filter })
      return {
        ...response,
        success: true
      }
    } catch (error) {
      console.error('Failed to fetch strategies:', error)
      return {
        success: false,
        error: error as string,
        timestamp: new Date().toISOString()
      }
    }
  }

  /**
   * Get a single strategy by ID
   */
  async getStrategy(id: string): Promise<StrategyApiResponse<Strategy>> {
    try {
      const strategy = await invoke<Strategy>('get_strategy', { id })
      return {
        success: true,
        data: strategy,
        timestamp: new Date().toISOString()
      }
    } catch (error) {
      console.error(`Failed to fetch strategy ${id}:`, error)
      return {
        success: false,
        error: error as string,
        timestamp: new Date().toISOString()
      }
    }
  }

  /**
   * Create a new strategy
   */
  async createStrategy(formData: StrategyFormData): Promise<StrategyApiResponse<Strategy>> {
    try {
      // Validate form data before sending
      const validationResponse = await this.validateStrategyData(formData)
      if (!validationResponse.success) {
        return validationResponse as StrategyApiResponse<Strategy>
      }

      const strategy = await invoke<Strategy>('create_strategy', {
        strategyData: this.formatStrategyData(formData)
      })

      return {
        success: true,
        data: strategy,
        message: 'Strategy created successfully',
        timestamp: new Date().toISOString()
      }
    } catch (error) {
      console.error('Failed to create strategy:', error)
      return {
        success: false,
        error: error as string,
        timestamp: new Date().toISOString()
      }
    }
  }

  /**
   * Update an existing strategy
   */
  async updateStrategy(id: string, formData: StrategyFormData): Promise<StrategyApiResponse<Strategy>> {
    try {
      // Validate form data before sending
      const validationResponse = await this.validateStrategyData(formData)
      if (!validationResponse.success) {
        return validationResponse as StrategyApiResponse<Strategy>
      }

      const strategy = await invoke<Strategy>('update_strategy', {
        id,
        strategyData: this.formatStrategyData(formData)
      })

      return {
        success: true,
        data: strategy,
        message: 'Strategy updated successfully',
        timestamp: new Date().toISOString()
      }
    } catch (error) {
      console.error(`Failed to update strategy ${id}:`, error)
      return {
        success: false,
        error: error as string,
        timestamp: new Date().toISOString()
      }
    }
  }

  /**
   * Delete a strategy
   */
  async deleteStrategy(id: string): Promise<StrategyApiResponse> {
    try {
      await invoke('delete_strategy', { id })
      return {
        success: true,
        message: 'Strategy deleted successfully',
        timestamp: new Date().toISOString()
      }
    } catch (error) {
      console.error(`Failed to delete strategy ${id}:`, error)
      return {
        success: false,
        error: error as string,
        timestamp: new Date().toISOString()
      }
    }
  }

  /**
   * Duplicate a strategy
   */
  async duplicateStrategy(id: string, newName?: string): Promise<StrategyApiResponse<Strategy>> {
    try {
      const strategy = await invoke<Strategy>('duplicate_strategy', {
        id,
        newName
      })

      return {
        success: true,
        data: strategy,
        message: 'Strategy duplicated successfully',
        timestamp: new Date().toISOString()
      }
    } catch (error) {
      console.error(`Failed to duplicate strategy ${id}:`, error)
      return {
        success: false,
        error: error as string,
        timestamp: new Date().toISOString()
      }
    }
  }

  /**
   * Start a strategy
   */
  async startStrategy(id: string): Promise<StrategyApiResponse> {
    try {
      await invoke('start_strategy', { id })
      return {
        success: true,
        message: 'Strategy started successfully',
        timestamp: new Date().toISOString()
      }
    } catch (error) {
      console.error(`Failed to start strategy ${id}:`, error)
      return {
        success: false,
        error: error as string,
        timestamp: new Date().toISOString()
      }
    }
  }

  /**
   * Stop a strategy
   */
  async stopStrategy(id: string, closePositions = true): Promise<StrategyApiResponse> {
    try {
      await invoke('stop_strategy', { id, closePositions })
      return {
        success: true,
        message: 'Strategy stopped successfully',
        timestamp: new Date().toISOString()
      }
    } catch (error) {
      console.error(`Failed to stop strategy ${id}:`, error)
      return {
        success: false,
        error: error as string,
        timestamp: new Date().toISOString()
      }
    }
  }

  /**
   * Pause a strategy
   */
  async pauseStrategy(id: string): Promise<StrategyApiResponse> {
    try {
      await invoke('pause_strategy', { id })
      return {
        success: true,
        message: 'Strategy paused successfully',
        timestamp: new Date().toISOString()
      }
    } catch (error) {
      console.error(`Failed to pause strategy ${id}:`, error)
      return {
        success: false,
        error: error as string,
        timestamp: new Date().toISOString()
      }
    }
  }

  /**
   * Resume a paused strategy
   */
  async resumeStrategy(id: string): Promise<StrategyApiResponse> {
    try {
      await invoke('resume_strategy', { id })
      return {
        success: true,
        message: 'Strategy resumed successfully',
        timestamp: new Date().toISOString()
      }
    } catch (error) {
      console.error(`Failed to resume strategy ${id}:`, error)
      return {
        success: false,
        error: error as string,
        timestamp: new Date().toISOString()
      }
    }
  }

  /**
   * Get strategy statistics
   */
  async getStrategyStats(): Promise<StrategyApiResponse<StrategyStats>> {
    try {
      const stats = await invoke<StrategyStats>('get_strategy_stats')
      return {
        success: true,
        data: stats,
        timestamp: new Date().toISOString()
      }
    } catch (error) {
      console.error('Failed to fetch strategy statistics:', error)
      return {
        success: false,
        error: error as string,
        timestamp: new Date().toISOString()
      }
    }
  }

  /**
   * Get strategy performance data
   */
  async getStrategyPerformance(id: string, period?: string): Promise<StrategyApiResponse<StrategyPerformance>> {
    try {
      const performance = await invoke<StrategyPerformance>('get_strategy_performance', {
        id,
        period
      })
      return {
        success: true,
        data: performance,
        timestamp: new Date().toISOString()
      }
    } catch (error) {
      console.error(`Failed to fetch performance for strategy ${id}:`, error)
      return {
        success: false,
        error: error as string,
        timestamp: new Date().toISOString()
      }
    }
  }

  /**
   * Get strategy trades
   */
  async getStrategyTrades(id: string, limit = 100, offset = 0): Promise<StrategyApiResponse<Trade[]>> {
    try {
      const trades = await invoke<Trade[]>('get_strategy_trades', {
        id,
        limit,
        offset
      })
      return {
        success: true,
        data: trades,
        timestamp: new Date().toISOString()
      }
    } catch (error) {
      console.error(`Failed to fetch trades for strategy ${id}:`, error)
      return {
        success: false,
        error: error as string,
        timestamp: new Date().toISOString()
      }
    }
  }

  /**
   * Get strategy positions
   */
  async getStrategyPositions(id: string): Promise<StrategyApiResponse<Position[]>> {
    try {
      const positions = await invoke<Position[]>('get_strategy_positions', { id })
      return {
        success: true,
        data: positions,
        timestamp: new Date().toISOString()
      }
    } catch (error) {
      console.error(`Failed to fetch positions for strategy ${id}:`, error)
      return {
        success: false,
        error: error as string,
        timestamp: new Date().toISOString()
      }
    }
  }

  /**
   * Get strategy signals
   */
  async getStrategySignals(id: string, limit = 50): Promise<StrategyApiResponse<StrategySignal[]>> {
    try {
      const signals = await invoke<StrategySignal[]>('get_strategy_signals', {
        id,
        limit
      })
      return {
        success: true,
        data: signals,
        timestamp: new Date().toISOString()
      }
    } catch (error) {
      console.error(`Failed to fetch signals for strategy ${id}:`, error)
      return {
        success: false,
        error: error as string,
        timestamp: new Date().toISOString()
      }
    }
  }

  /**
   * Run backtest for a strategy
   */
  async runBacktest(strategyId: string, backtestConfig: any): Promise<StrategyApiResponse<BacktestResult>> {
    try {
      const result = await invoke<BacktestResult>('run_backtest', {
        strategyId,
        config: backtestConfig
      })
      return {
        success: true,
        data: result,
        message: 'Backtest completed successfully',
        timestamp: new Date().toISOString()
      }
    } catch (error) {
      console.error(`Failed to run backtest for strategy ${strategyId}:`, error)
      return {
        success: false,
        error: error as string,
        timestamp: new Date().toISOString()
      }
    }
  }

  /**
   * Get backtest results
   */
  async getBacktestResult(resultId: string): Promise<StrategyApiResponse<BacktestResult>> {
    try {
      const result = await invoke<BacktestResult>('get_backtest_result', { resultId })
      return {
        success: true,
        data: result,
        timestamp: new Date().toISOString()
      }
    } catch (error) {
      console.error(`Failed to fetch backtest result ${resultId}:`, error)
      return {
        success: false,
        error: error as string,
        timestamp: new Date().toISOString()
      }
    }
  }

  /**
   * Get strategy templates
   */
  async getStrategyTemplates(category?: string): Promise<StrategyApiResponse<StrategyTemplate[]>> {
    try {
      const templates = await invoke<StrategyTemplate[]>('get_strategy_templates', { category })
      return {
        success: true,
        data: templates,
        timestamp: new Date().toISOString()
      }
    } catch (error) {
      console.error('Failed to fetch strategy templates:', error)
      return {
        success: false,
        error: error as string,
        timestamp: new Date().toISOString()
      }
    }
  }

  /**
   * Create strategy from template
   */
  async createFromTemplate(templateId: string, customization: Partial<StrategyFormData>): Promise<StrategyApiResponse<Strategy>> {
    try {
      const strategy = await invoke<Strategy>('create_from_template', {
        templateId,
        customization
      })
      return {
        success: true,
        data: strategy,
        message: 'Strategy created from template successfully',
        timestamp: new Date().toISOString()
      }
    } catch (error) {
      console.error(`Failed to create strategy from template ${templateId}:`, error)
      return {
        success: false,
        error: error as string,
        timestamp: new Date().toISOString()
      }
    }
  }

  /**
   * Export strategies
   */
  async exportStrategies(strategyIds: string[], includePrivateData = false): Promise<StrategyApiResponse<Blob>> {
    try {
      const exportData = await invoke<string>('export_strategies', {
        strategyIds,
        includePrivateData
      })

      // Convert string to Blob
      const blob = new Blob([exportData], { type: 'application/json' })
      return {
        success: true,
        data: blob,
        message: 'Strategies exported successfully',
        timestamp: new Date().toISOString()
      }
    } catch (error) {
      console.error('Failed to export strategies:', error)
      return {
        success: false,
        error: error as string,
        timestamp: new Date().toISOString()
      }
    }
  }

  /**
   * Import strategies
   */
  async importStrategies(file: File): Promise<StrategyApiResponse<any>> {
    try {
      const fileContent = await this.readFileAsText(file)
      const result = await invoke('import_strategies', {
        fileContent: fileContent,
        filename: file.name
      })
      return {
        success: true,
        data: result,
        message: 'Strategies imported successfully',
        timestamp: new Date().toISOString()
      }
    } catch (error) {
      console.error('Failed to import strategies:', error)
      return {
        success: false,
        error: error as string,
        timestamp: new Date().toISOString()
      }
    }
  }

  /**
   * Optimize strategy parameters
   */
  async optimizeStrategy(strategyId: string, optimizationConfig: any): Promise<StrategyApiResponse<any>> {
    try {
      const result = await invoke('optimize_strategy', {
        strategyId,
        config: optimizationConfig
      })
      return {
        success: true,
        data: result,
        message: 'Strategy optimization completed',
        timestamp: new Date().toISOString()
      }
    } catch (error) {
      console.error(`Failed to optimize strategy ${strategyId}:`, error)
      return {
        success: false,
        error: error as string,
        timestamp: new Date().toISOString()
      }
    }
  }

  // Private helper methods

  private async validateStrategyData(formData: StrategyFormData): Promise<StrategyApiResponse> {
    try {
      // Client-side validation
      const { validateStrategyForm } = await import('@/utils/strategyValidation')
      const errors = validateStrategyForm(formData)

      if (errors.length > 0) {
        return {
          success: false,
          error: 'Validation failed',
          message: errors.map(e => e.message).join(', '),
          timestamp: new Date().toISOString()
        }
      }

      return { success: true, timestamp: new Date().toISOString() }
    } catch (error) {
      return {
        success: false,
        error: 'Validation error',
        timestamp: new Date().toISOString()
      }
    }
  }

  private formatStrategyData(formData: StrategyFormData): any {
    return {
      name: formData.name,
      description: formData.description,
      symbol: formData.symbol,
      capital: formData.capital,
      type: formData.type,
      timeframe: formData.timeframe,
      parameters: formData.parameters,
      riskParams: formData.riskParams,
      config: formData.config,
      tags: formData.tags,
      isPublic: formData.isPublic,
      templateId: formData.templateId,
      // Add timestamps
      createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString(),
      // Set initial status
      status: 'draft',
      // Initialize performance metrics
      performance: {
        totalPnL: 0,
        totalReturn: 0,
        winRate: 0,
        profitFactor: 0,
        maxDrawdown: 0,
        sharpeRatio: 0,
        sortinoRatio: 0,
        calmarRatio: 0,
        totalTrades: 0,
        winningTrades: 0,
        losingTrades: 0,
        averageWin: 0,
        averageLoss: 0,
        largestWin: 0,
        largestLoss: 0,
        averageTradeDuration: 0,
        expectancy: 0,
        monthlyReturns: [],
        dailyReturns: [],
        trades: [],
        equityCurve: []
      }
    }
  }

  private async readFileAsText(file: File): Promise<string> {
    return new Promise((resolve, reject) => {
      const reader = new FileReader()
      reader.onload = (e) => resolve(e.target?.result as string)
      reader.onerror = (e) => reject(new Error('Failed to read file'))
      reader.readAsText(file)
    })
  }
}

// Export singleton instance
export const strategyService = new StrategyService()

// Export convenience functions
export const {
  getStrategies,
  getStrategy,
  createStrategy,
  updateStrategy,
  deleteStrategy,
  duplicateStrategy,
  startStrategy,
  stopStrategy,
  pauseStrategy,
  resumeStrategy,
  getStrategyStats,
  getStrategyPerformance,
  getStrategyTrades,
  getStrategyPositions,
  getStrategySignals,
  runBacktest,
  getBacktestResult,
  getStrategyTemplates,
  createFromTemplate,
  exportStrategies,
  importStrategies,
  optimizeStrategy
} = strategyService