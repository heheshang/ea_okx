/**
 * Strategy Validation Utilities
 * Comprehensive validation rules for trading strategies
 */

import type {
  StrategyFormData,
  StrategyValidationRule,
  StrategyValidationError,
  StrategyParameters,
  RiskParameters,
  StrategyType
} from '@/types/strategy'

export class StrategyValidator {
  private rules: Map<string, StrategyValidationRule[]> = new Map()
  private errors: StrategyValidationError[] = []

  constructor() {
    this.initializeRules()
  }

  private initializeRules(): void {
    // Basic Info Rules
    this.addRule('name', {
      field: 'name',
      rule: 'required',
      message: 'Strategy name is required'
    })

    this.addRule('name', {
      field: 'name',
      rule: 'min',
      value: 3,
      message: 'Strategy name must be at least 3 characters'
    })

    this.addRule('name', {
      field: 'name',
      rule: 'max',
      value: 50,
      message: 'Strategy name cannot exceed 50 characters'
    })

    this.addRule('name', {
      field: 'name',
      rule: 'pattern',
      value: /^[a-zA-Z0-9\s\-_]+$/,
      message: 'Strategy name can only contain letters, numbers, spaces, hyphens, and underscores'
    })

    this.addRule('description', {
      field: 'description',
      rule: 'max',
      value: 500,
      message: 'Description cannot exceed 500 characters'
    })

    this.addRule('symbol', {
      field: 'symbol',
      rule: 'required',
      message: 'Trading symbol is required'
    })

    this.addRule('capital', {
      field: 'capital',
      rule: 'required',
      message: 'Capital allocation is required'
    })

    this.addRule('capital', {
      field: 'capital',
      rule: 'min',
      value: 100,
      message: 'Minimum capital allocation is $100'
    })

    this.addRule('capital', {
      field: 'capital',
      rule: 'max',
      value: 10000000,
      message: 'Maximum capital allocation is $10,000,000'
    })

    // Type-specific parameter rules
    this.initializeMA_CrossoverRules()
    this.initializeGridTradingRules()
    this.initializeRSIStrategyRules()

    // Risk parameter rules
    this.initializeRiskParameterRules()
  }

  private initializeMA_CrossoverRules(): void {
    this.addRule('parameters.shortPeriod', {
      field: 'parameters.shortPeriod',
      rule: 'required',
      message: 'Short MA period is required for MA Crossover strategy'
    })

    this.addRule('parameters.shortPeriod', {
      field: 'parameters.shortPeriod',
      rule: 'min',
      value: 5,
      message: 'Short MA period must be at least 5'
    })

    this.addRule('parameters.shortPeriod', {
      field: 'parameters.shortPeriod',
      rule: 'max',
      value: 100,
      message: 'Short MA period cannot exceed 100'
    })

    this.addRule('parameters.longPeriod', {
      field: 'parameters.longPeriod',
      rule: 'required',
      message: 'Long MA period is required for MA Crossover strategy'
    })

    this.addRule('parameters.longPeriod', {
      field: 'parameters.longPeriod',
      rule: 'min',
      value: 10,
      message: 'Long MA period must be at least 10'
    })

    this.addRule('parameters.longPeriod', {
      field: 'parameters.longPeriod',
      rule: 'max',
      value: 500,
      message: 'Long MA period cannot exceed 500'
    })

    this.addRule('parameters.maType', {
      field: 'parameters.maType',
      rule: 'required',
      message: 'MA type is required'
    })

    // Custom rule: short period must be less than long period
    this.addRule('parameters.longPeriod', {
      field: 'parameters.longPeriod',
      rule: 'custom',
      validator: (value, formData) => {
        return !formData?.parameters?.shortPeriod || value > formData.parameters.shortPeriod
      },
      message: 'Long MA period must be greater than short MA period'
    })
  }

  private initializeGridTradingRules(): void {
    this.addRule('parameters.gridLevels', {
      field: 'parameters.gridLevels',
      rule: 'required',
      message: 'Grid levels are required for Grid Trading strategy'
    })

    this.addRule('parameters.gridLevels', {
      field: 'parameters.gridLevels',
      rule: 'min',
      value: 3,
      message: 'Minimum 3 grid levels are required'
    })

    this.addRule('parameters.gridLevels', {
      field: 'parameters.gridLevels',
      rule: 'max',
      value: 100,
      message: 'Maximum 100 grid levels allowed'
    })

    this.addRule('parameters.priceRange', {
      field: 'parameters.priceRange',
      rule: 'required',
      message: 'Price range is required for Grid Trading strategy'
    })

    this.addRule('parameters.priceRange', {
      field: 'parameters.priceRange',
      rule: 'min',
      value: 0.1,
      message: 'Price range must be at least 0.1%'
    })

    this.addRule('parameters.priceRange', {
      field: 'parameters.priceRange',
      rule: 'max',
      value: 50,
      message: 'Price range cannot exceed 50%'
    })

    this.addRule('parameters.orderSize', {
      field: 'parameters.orderSize',
      rule: 'required',
      message: 'Order size per grid is required'
    })

    this.addRule('parameters.orderSize', {
      field: 'parameters.orderSize',
      rule: 'min',
      value: 10,
      message: 'Minimum order size is 10'
    })
  }

  private initializeRSIStrategyRules(): void {
    this.addRule('parameters.rsiPeriod', {
      field: 'parameters.rsiPeriod',
      rule: 'required',
      message: 'RSI period is required for RSI Strategy'
    })

    this.addRule('parameters.rsiPeriod', {
      field: 'parameters.rsiPeriod',
      rule: 'min',
      value: 2,
      message: 'RSI period must be at least 2'
    })

    this.addRule('parameters.rsiPeriod', {
      field: 'parameters.rsiPeriod',
      rule: 'max',
      value: 100,
      message: 'RSI period cannot exceed 100'
    })

    this.addRule('parameters.oversold', {
      field: 'parameters.oversold',
      rule: 'required',
      message: 'Oversold threshold is required'
    })

    this.addRule('parameters.oversold', {
      field: 'parameters.oversold',
      rule: 'min',
      value: 1,
      message: 'Oversold threshold must be at least 1'
    })

    this.addRule('parameters.oversold', {
      field: 'parameters.oversold',
      rule: 'max',
      value: 40,
      message: 'Oversold threshold cannot exceed 40'
    })

    this.addRule('parameters.overbought', {
      field: 'parameters.overbought',
      rule: 'required',
      message: 'Overbought threshold is required'
    })

    this.addRule('parameters.overbought', {
      field: 'parameters.overbought',
      rule: 'min',
      value: 60,
      message: 'Overbought threshold must be at least 60'
    })

    this.addRule('parameters.overbought', {
      field: 'parameters.overbought',
      rule: 'max',
      value: 99,
      message: 'Overbought threshold cannot exceed 99'
    })

    // Custom rule: overbought must be greater than oversold
    this.addRule('parameters.overbought', {
      field: 'parameters.overbought',
      rule: 'custom',
      validator: (value, formData) => {
        return !formData?.parameters?.oversold || value > formData.parameters.oversold
      },
      message: 'Overbought threshold must be greater than oversold threshold'
    })
  }

  private initializeRiskParameterRules(): void {
    this.addRule('riskParams.stopLoss', {
      field: 'riskParams.stopLoss',
      rule: 'min',
      value: 0.1,
      message: 'Stop loss must be at least 0.1%'
    })

    this.addRule('riskParams.stopLoss', {
      field: 'riskParams.stopLoss',
      rule: 'max',
      value: 100,
      message: 'Stop loss cannot exceed 100%'
    })

    this.addRule('riskParams.takeProfit', {
      field: 'riskParams.takeProfit',
      rule: 'min',
      value: 0.1,
      message: 'Take profit must be at least 0.1%'
    })

    this.addRule('riskParams.takeProfit', {
      field: 'riskParams.takeProfit',
      rule: 'max',
      value: 100,
      message: 'Take profit cannot exceed 100%'
    })

    this.addRule('riskParams.maxPositionSize', {
      field: 'riskParams.maxPositionSize',
      rule: 'min',
      value: 100,
      message: 'Maximum position size must be at least $100'
    })

    this.addRule('riskParams.dailyLossLimit', {
      field: 'riskParams.dailyLossLimit',
      rule: 'min',
      value: 1,
      message: 'Daily loss limit must be at least 1%'
    })

    this.addRule('riskParams.dailyLossLimit', {
      field: 'riskParams.dailyLossLimit',
      rule: 'max',
      value: 100,
      message: 'Daily loss limit cannot exceed 100%'
    })

    this.addRule('riskParams.maxOpenPositions', {
      field: 'riskParams.maxOpenPositions',
      rule: 'min',
      value: 1,
      message: 'Maximum open positions must be at least 1'
    })

    this.addRule('riskParams.maxOpenPositions', {
      field: 'riskParams.maxOpenPositions',
      rule: 'max',
      value: 50,
      message: 'Maximum open positions cannot exceed 50'
    })

    // Position size validation
    this.addRule('parameters.positionSize', {
      field: 'parameters.positionSize',
      rule: 'min',
      value: 0.01,
      message: 'Position size must be at least 1%'
    })

    this.addRule('parameters.positionSize', {
      field: 'parameters.positionSize',
      rule: 'max',
      value: 1,
      message: 'Position size cannot exceed 100%'
    })

    // Custom rule: max position size should not exceed allocated capital
    this.addRule('riskParams.maxPositionSize', {
      field: 'riskParams.maxPositionSize',
      rule: 'custom',
      validator: (value, formData) => {
        return value <= (formData?.capital || 0)
      },
      message: 'Maximum position size cannot exceed allocated capital'
    })

    // Risk-reward ratio validation
    this.addRule('riskParams.takeProfit', {
      field: 'riskParams.takeProfit',
      rule: 'custom',
      validator: (value, formData) => {
        const stopLoss = formData?.riskParams?.stopLoss
        return !stopLoss || value > stopLoss * 1.5 // Minimum 1.5:1 risk-reward ratio
      },
      message: 'Take profit should be at least 1.5x stop loss for better risk-reward ratio'
    })
  }

  private addRule(field: string, rule: StrategyValidationRule): void {
    if (!this.rules.has(field)) {
      this.rules.set(field, [])
    }
    this.rules.get(field)!.push(rule)
  }

  public validate(formData: Partial<StrategyFormData>): StrategyValidationError[] {
    this.errors = []

    // Validate all rules
    for (const [field, fieldRules] of this.rules) {
      for (const rule of fieldRules) {
        const value = this.getFieldValue(formData, field)

        if (!this.validateRule(rule, value, formData)) {
          this.errors.push({
            field,
            message: rule.message,
            value
          })
        }
      }
    }

    // Type-specific validation
    if (formData.type) {
      this.validateTypeSpecificParameters(formData.type, formData.parameters || {})
    }

    // Cross-field validation
    this.validateCrossFields(formData)

    return this.errors
  }

  private validateRule(rule: StrategyValidationRule, value: any, formData: Partial<StrategyFormData>): boolean {
    switch (rule.rule) {
      case 'required':
        return value !== undefined && value !== null && value !== ''

      case 'min':
        return typeof value === 'number' && value >= rule.value

      case 'max':
        return typeof value === 'number' && value <= rule.value

      case 'pattern':
        return typeof value === 'string' && new RegExp(rule.value).test(value)

      case 'custom':
        return rule.validator ? rule.validator(value, formData) : true

      default:
        return true
    }
  }

  private getFieldValue(formData: Partial<StrategyFormData>, field: string): any {
    const parts = field.split('.')
    let value: any = formData

    for (const part of parts) {
      if (value && typeof value === 'object' && part in value) {
        value = value[part]
      } else {
        return undefined
      }
    }

    return value
  }

  private validateTypeSpecificParameters(type: StrategyType, parameters: StrategyParameters): void {
    switch (type) {
      case 'ma_crossover':
        // MA crossover specific validations
        if (parameters.shortPeriod && parameters.longPeriod) {
          if (parameters.shortPeriod >= parameters.longPeriod) {
            this.errors.push({
              field: 'parameters.shortPeriod',
              message: 'Short MA period must be less than long MA period',
              value: parameters.shortPeriod
            })
          }

          // Recommended period ratio
          const ratio = parameters.longPeriod / parameters.shortPeriod
          if (ratio > 10) {
            this.errors.push({
              field: 'parameters.longPeriod',
              message: 'Consider using a smaller ratio between long and short periods (recommended: 2-4x)',
              value: parameters.longPeriod
            })
          }
        }
        break

      case 'grid_trading':
        // Grid trading specific validations
        if (parameters.gridLevels && parameters.orderSize) {
          const totalExposure = parameters.gridLevels * parameters.orderSize
          if (totalExposure > 100000) {
            this.errors.push({
              field: 'parameters.gridLevels',
              message: 'Total grid exposure is very high. Consider reducing grid levels or order size',
              value: totalExposure
            })
          }
        }
        break

      case 'rsi_strategy':
        // RSI strategy specific validations
        if (parameters.oversold && parameters.overbought) {
          const spread = parameters.overbought - parameters.oversold
          if (spread < 20) {
            this.errors.push({
              field: 'parameters.oversold',
              message: 'RSI spread is too narrow. Consider using a wider spread (recommended: 30+)',
              value: parameters.oversold
            })
          }
        }
        break
    }
  }

  private validateCrossFields(formData: Partial<StrategyFormData>): void {
    // Capital vs position size validation
    if (formData.capital && formData.parameters?.positionSize) {
      const positionValue = formData.capital * formData.parameters.positionSize
      const maxPositionSize = formData.riskParams?.maxPositionSize || formData.capital

      if (positionValue > maxPositionSize) {
        this.errors.push({
          field: 'parameters.positionSize',
          message: 'Position size exceeds maximum position size limit',
          value: formData.parameters.positionSize
        })
      }
    }

    // Risk management validation
    if (formData.riskParams?.stopLoss && formData.riskParams?.takeProfit) {
      const riskRewardRatio = formData.riskParams.takeProfit / formData.riskParams.stopLoss

      if (riskRewardRatio < 1) {
        this.errors.push({
          field: 'riskParams.takeProfit',
          message: 'Risk-reward ratio should be at least 1:1 for profitable trading',
          value: formData.riskParams.takeProfit
        })
      }
    }

    // Leverage validation
    if (formData.parameters?.leverage && formData.riskParams?.maxDrawdown) {
      const maxRecommendedLeverage = 100 / formData.riskParams.maxDrawdown

      if (formData.parameters.leverage > maxRecommendedLeverage) {
        this.errors.push({
          field: 'parameters.leverage',
          message: `Leverage is too high for the specified max drawdown. Recommended: ${maxRecommendedLeverage.toFixed(1)}x or lower`,
          value: formData.parameters.leverage
        })
      }
    }
  }

  public hasErrors(): boolean {
    return this.errors.length > 0
  }

  public getErrors(): StrategyValidationError[] {
    return this.errors
  }

  public getFieldErrors(field: string): StrategyValidationError[] {
    return this.errors.filter(error => error.field === field)
  }

  public getFirstError(): StrategyValidationError | null {
    return this.errors.length > 0 ? this.errors[0] : null
  }

  public clearErrors(): void {
    this.errors = []
  }

  public isValidField(field: string, formData: Partial<StrategyFormData>): boolean {
    const fieldRules = this.rules.get(field)
    if (!fieldRules) return true

    for (const rule of fieldRules) {
      const value = this.getFieldValue(formData, field)
      if (!this.validateRule(rule, value, formData)) {
        return false
      }
    }

    return true
  }

  public getFieldWarning(field: string, formData: Partial<StrategyFormData>): string | null {
    // Return warnings (not errors) for suboptimal configurations
    switch (field) {
      case 'riskParams.stopLoss':
        const stopLoss = formData.riskParams?.stopLoss
        if (stopLoss && stopLoss > 10) {
          return 'Wide stop loss may result in larger losses. Consider tighter risk management.'
        }
        break

      case 'parameters.leverage':
        const leverage = formData.parameters?.leverage
        if (leverage && leverage > 10) {
          return 'High leverage increases risk significantly. Use with caution.'
        }
        break

      case 'riskParams.dailyLossLimit':
        const dailyLossLimit = formData.riskParams?.dailyLossLimit
        if (dailyLossLimit && dailyLossLimit > 20) {
          return 'High daily loss limit. Consider lower limits for better risk management.'
        }
        break
    }

    return null
  }
}

// Export singleton instance
export const strategyValidator = new StrategyValidator()

// Utility functions
export const validateStrategyForm = (formData: Partial<StrategyFormData>): StrategyValidationError[] => {
  return strategyValidator.validate(formData)
}

export const isValidStrategyForm = (formData: Partial<StrategyFormData>): boolean => {
  const errors = validateStrategyForm(formData)
  return errors.length === 0
}

export const getFieldValidationMessage = (field: string, formData: Partial<StrategyFormData>): string | null => {
  if (!strategyValidator.isValidField(field, formData)) {
    const errors = strategyValidator.getFieldErrors(field)
    return errors.length > 0 ? errors[0].message : null
  }
  return null
}

export const getFieldWarningMessage = (field: string, formData: Partial<StrategyFormData>): string | null => {
  return strategyValidator.getFieldWarning(field, formData)
}