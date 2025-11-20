/**
 * Strategy Template System Types
 * Comprehensive type definitions for strategy templates and quick creation
 */

import type { StrategyType, Timeframe, StrategyParameters, RiskParameters, StrategyConfig } from './strategy'

export interface StrategyTemplate {
  id: string
  name: string
  description: string
  category: StrategyTemplateCategory
  difficulty: 'beginner' | 'intermediate' | 'advanced'
  tags: string[]

  // Template metadata
  author: string
  authorType: 'system' | 'community' | 'premium'
  rating: number
  usage: number
  downloads: number
  isBuiltIn: boolean
  isPremium: boolean
  version: string
  createdAt: string
  updatedAt: string

  // Strategy configuration
  defaultType: StrategyType
  supportedSymbols: string[]
  supportedTimeframes: Timeframe[]

  // Template parameters
  defaultParameters: StrategyParameters
  parameterRanges: TemplateParameterRanges
  parameterDescriptions: Record<string, string>

  // Risk configuration
  defaultRiskParams: RiskParameters
  riskGuidelines: TemplateRiskGuidelines

  // Performance expectations
  expectedPerformance?: {
    annualReturn: number
    maxDrawdown: number
    sharpeRatio: number
    winRate: number
    description: string
  }

  // Preview data
  previewChart?: string
  backtestResults?: {
    period: string
    return: number
    sharpeRatio: number
    maxDrawdown: number
    winRate: number
  }

  // Template content
  content: TemplateContent

  // Additional metadata
  tutorialUrl?: string
  documentationUrl?: string
  youtubeUrl?: string
  requirements?: string[]
  pros: string[]
  cons: string[]
}

export interface StrategyTemplateCategory {
  id: string
  name: string
  description: string
  icon: string
  color: string
  order: number
}

export interface TemplateParameterRanges {
  [key: string]: {
    min: number
    max: number
    default: number
    step: number
    type: 'number' | 'string' | 'boolean' | 'select'
    options?: string[]
  }
}

export interface TemplateRiskGuidelines {
  recommendedLeverage: {
    min: number
    max: number
    description: string
  }
  recommendedPositionSize: {
    min: number
    max: number
    description: string
  }
  riskLevel: 'low' | 'medium' | 'high'
  recommendedExperience: string
  minCapital: {
    conservative: number
    aggressive: number
  }
}

export interface TemplateContent {
  overview: string
  strategy: string
  setup: string
  parameters: Record<string, TemplateParameterInfo>
  risk: string
  examples: TemplateExample[]
}

export interface TemplateParameterInfo {
  description: string
  importance: 'low' | 'medium' | 'high' | 'critical'
  impact: string
  range: string
  tips: string[]
}

export interface TemplateExample {
  title: string
  description: string
  marketCondition: 'trending' | 'ranging' | 'volatile' | 'stable'
  parameters: Partial<StrategyParameters>
  riskParams: Partial<RiskParameters>
  results?: {
    period: string
    return: number
    sharpeRatio: number
    maxDrawdown: number
  }
}

export interface CreateFromTemplateRequest {
  templateId: string
  customization: TemplateCustomization
}

export interface TemplateCustomization {
  name?: string
  description?: string
  symbol?: string
  timeframe?: Timeframe
  parameters?: Partial<StrategyParameters>
  riskParams?: Partial<RiskParameters>
  config?: Partial<StrategyConfig>
  tags?: string[]
}

export interface StrategyTemplateFilter {
  search?: string
  category?: string[]
  difficulty?: ('beginner' | 'intermediate' | 'advanced')[]
  type?: StrategyType[]
  symbol?: string[]
  tags?: string[]
  authorType?: ('system' | 'community' | 'premium')[]
  rating?: {
    min?: number
    max?: number
  }
  isBuiltIn?: boolean
  isPremium?: boolean
  sortBy?: 'popularity' | 'rating' | 'newest' | 'downloads' | 'usage'
  sortOrder?: 'asc' | 'desc'
  limit?: number
  offset?: number
}

export interface StrategyTemplateListResponse {
  templates: StrategyTemplate[]
  total: number
  page: number
  pageSize: number
  filters: StrategyTemplateFilter
  categories: StrategyTemplateCategory[]
}

export interface TemplateCreationSteps {
  step: number
  totalSteps: number
  title: string
  description: string
  isCompleted: boolean
  isCurrent: boolean
}

export interface TemplateValidationResult {
  isValid: boolean
  errors: TemplateValidationError[]
  warnings: TemplateValidationWarning[]
  recommendations: string[]
}

export interface TemplateValidationError {
  field: string
  message: string
  severity: 'error'
  value?: any
}

export interface TemplateValidationWarning {
  field: string
  message: string
  severity: 'warning'
  value?: any
}

// Template creation wizard types
export interface TemplateCreationData {
  templateId: string
  strategyData: {
    name: string
    description: string
    symbol: string
    type: StrategyType
    timeframe: Timeframe
    parameters: StrategyParameters
    riskParams: RiskParameters
    config: Partial<StrategyConfig>
    tags: string[]
    isPublic: boolean
  }
  customization: {
    parameters?: Record<string, any>
    riskParams?: Record<string, any>
    config?: Record<string, any>
  }
}

export interface TemplateWizardStep {
  component: string
  title: string
  description: string
  validation?: (data: TemplateCreationData) => TemplateValidationResult
  canSkip?: boolean
  dependsOn?: string[]
}

// Template comparison
export interface TemplateComparison {
  templates: StrategyTemplate[]
  criteria: string[]
  results: ComparisonResult[]
}

export interface ComparisonResult {
  templateId: string
  score: number
  strengths: string[]
  weaknesses: string[]
  recommendation: string
}

// Template sharing and community features
export interface TemplateReview {
  id: string
  templateId: string
  userId: string
  userName: string
  rating: number
  review: string
  pros: string[]
  cons: string[]
  helpfulVotes: number
  createdAt: string
  updatedAt: string
  isVerified: boolean
}

export interface TemplateAnalytics {
  templateId: string
  views: number
  downloads: number
  usageCount: number
  successRate: number
  averageReturn: number
  averageSharpe: number
  riskAdjustedScore: number
  userSatisfaction: number
  performanceHistory: TemplatePerformanceData[]
}

export interface TemplatePerformanceData {
  date: string
  users: number
  avgReturn: number
  medianReturn: number
  avgSharpe: number
  avgMaxDrawdown: number
  successRate: number
}

// Export/Import types
export interface TemplateExport {
  template: StrategyTemplate
  exportedAt: string
  exportedBy: string
  includeCustomData: boolean
  format: 'json' | 'yaml'
}

export interface TemplateImportResult {
  success: boolean
  template?: StrategyTemplate
  errors: string[]
  warnings: string[]
  isValid: boolean
  conflictingIds?: string[]
}

// Recommended templates
export interface TemplateRecommendation {
  template: StrategyTemplate
  reason: string
  confidence: number
  score: number
  matchCriteria: string[]
}