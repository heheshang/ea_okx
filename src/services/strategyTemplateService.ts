/**
 * Strategy Template Service
 * Handles all operations for strategy templates including CRUD, validation, and recommendations
 */

import { invoke } from '@tauri-apps/api/core'
import type {
  StrategyTemplate,
  StrategyTemplateFilter,
  StrategyTemplateListResponse,
  StrategyTemplateCategory,
  TemplateCustomization,
  CreateFromTemplateRequest,
  TemplateValidationResult,
  TemplateCreationData,
  TemplateReview,
  TemplateAnalytics,
  TemplateRecommendation,
  TemplateExport,
  TemplateImportResult
} from '@/types/strategyTemplate'
import type { Strategy, StrategyFormData, StrategyType } from '@/types/strategy'

export class StrategyTemplateService {
  private baseUrl = '/api/strategy-templates'

  /**
   * Get all strategy templates with filtering and pagination
   */
  async getTemplates(filter?: StrategyTemplateFilter): Promise<StrategyTemplateListResponse> {
    try {
      const response = await invoke<StrategyTemplateListResponse>('get_strategy_templates', { filter })
      return {
        ...response,
        success: true
      }
    } catch (error) {
      console.error('Failed to fetch strategy templates:', error)
      return {
        success: false,
        error: error as string,
        timestamp: new Date().toISOString(),
        templates: [],
        total: 0,
        page: 0,
        pageSize: 0,
        filters: {} as any,
        categories: []
      }
    }
  }

  /**
   * Get a single strategy template by ID
   */
  async getTemplate(id: string): Promise<{ success: boolean; data?: StrategyTemplate; error?: string; timestamp: string }> {
    try {
      const template = await invoke<StrategyTemplate>('get_strategy_template', { id })
      return {
        success: true,
        data: template,
        timestamp: new Date().toISOString()
      }
    } catch (error) {
      console.error(`Failed to fetch strategy template ${id}:`, error)
      return {
        success: false,
        error: error as string,
        timestamp: new Date().toISOString()
      }
    }
  }

  /**
   * Get template categories
   */
  async getCategories(): Promise<{ success: boolean; data?: StrategyTemplateCategory[]; error?: string; timestamp: string }> {
    try {
      const categories = await invoke<StrategyTemplateCategory[]>('get_template_categories')
      return {
        success: true,
        data: categories,
        timestamp: new Date().toISOString()
      }
    } catch (error) {
      console.error('Failed to fetch template categories:', error)
      return {
        success: false,
        error: error as string,
        timestamp: new Date().toISOString()
      }
    }
  }

  /**
   * Create a new strategy from template
   */
  async createFromTemplate(request: CreateFromTemplateRequest): Promise<{ success: boolean; data?: Strategy; error?: string; timestamp: string }> {
    try {
      const strategy = await invoke<Strategy>('create_from_template', request)
      return {
        success: true,
        data: strategy,
        message: 'Strategy created from template successfully',
        timestamp: new Date().toISOString()
      }
    } catch (error) {
      console.error('Failed to create strategy from template:', error)
      return {
        success: false,
        error: error as string,
        timestamp: new Date().toISOString()
      }
    }
  }

  /**
   * Create a custom strategy template
   */
  async createTemplate(templateData: Partial<StrategyTemplate>): Promise<{ success: boolean; data?: StrategyTemplate; error?: string; timestamp: string }> {
    try {
      const template = await invoke<StrategyTemplate>('create_strategy_template', { templateData })
      return {
        success: true,
        data: template,
        message: 'Strategy template created successfully',
        timestamp: new Date().toISOString()
      }
    } catch (error) {
      console.error('Failed to create strategy template:', error)
      return {
        success: false,
        error: error as string,
        timestamp: new Date().toISOString()
      }
    }
  }

  /**
   * Update an existing strategy template
   */
  async updateTemplate(id: string, templateData: Partial<StrategyTemplate>): Promise<{ success: boolean; data?: StrategyTemplate; error?: string; timestamp: string }> {
    try {
      const template = await invoke<StrategyTemplate>('update_strategy_template', { id, templateData })
      return {
        success: true,
        data: template,
        message: 'Strategy template updated successfully',
        timestamp: new Date().toISOString()
      }
    } catch (error) {
      console.error(`Failed to update strategy template ${id}:`, error)
      return {
        success: false,
        error: error as string,
        timestamp: new Date().toISOString()
      }
    }
  }

  /**
   * Delete a strategy template
   */
  async deleteTemplate(id: string): Promise<{ success: boolean; message?: string; error?: string; timestamp: string }> {
    try {
      await invoke('delete_strategy_template', { id })
      return {
        success: true,
        message: 'Strategy template deleted successfully',
        timestamp: new Date().toISOString()
      }
    } catch (error) {
      console.error(`Failed to delete strategy template ${id}:`, error)
      return {
        success: false,
        error: error as string,
        timestamp: new Date().toISOString()
      }
    }
  }

  /**
   * Validate template parameters
   */
  async validateTemplate(templateData: Partial<StrategyTemplate>): Promise<TemplateValidationResult> {
    try {
      const validation = await invoke<TemplateValidationResult>('validate_template', { templateData })
      return validation
    } catch (error) {
      console.error('Failed to validate template:', error)
      return {
        isValid: false,
        errors: [{ field: 'general', message: error as string, severity: 'error' }],
        warnings: [],
        recommendations: []
      }
    }
  }

  /**
   * Get template analytics
   */
  async getTemplateAnalytics(id: string): Promise<{ success: boolean; data?: TemplateAnalytics; error?: string; timestamp: string }> {
    try {
      const analytics = await invoke<TemplateAnalytics>('get_template_analytics', { id })
      return {
        success: true,
        data: analytics,
        timestamp: new Date().toISOString()
      }
    } catch (error) {
      console.error(`Failed to fetch template analytics ${id}:`, error)
      return {
        success: false,
        error: error as string,
        timestamp: new Date().toISOString()
      }
    }
  }

  /**
   * Get recommended templates based on user behavior
   */
  async getRecommendedTemplates(userId?: string, limit = 5): Promise<{ success: boolean; data?: TemplateRecommendation[]; error?: string; timestamp: string }> {
    try {
      const recommendations = await invoke<TemplateRecommendation[]>('get_recommended_templates', { userId, limit })
      return {
        success: true,
        data: recommendations,
        timestamp: new Date().toISOString()
      }
    } catch (error) {
      console.error('Failed to get recommended templates:', error)
      return {
        success: false,
        error: error as string,
        timestamp: new Date().toISOString()
      }
    }
  }

  /**
   * Rate a template
   */
  async rateTemplate(id: string, rating: number, review?: string): Promise<{ success: boolean; message?: string; error?: string; timestamp: string }> {
    try {
      await invoke('rate_template', { id, rating, review })
      return {
        success: true,
        message: 'Template rated successfully',
        timestamp: new Date().toISOString()
      }
    } catch (error) {
      console.error(`Failed to rate template ${id}:`, error)
      return {
        success: false,
        error: error as string,
        timestamp: new Date().toISOString()
      }
    }
  }

  /**
   * Get template reviews
   */
  async getTemplateReviews(id: string, limit = 10, offset = 0): Promise<{ success: boolean; data?: TemplateReview[]; error?: string; timestamp: string }> {
    try {
      const reviews = await invoke<TemplateReview[]>('get_template_reviews', { id, limit, offset })
      return {
        success: true,
        data: reviews,
        timestamp: new Date().toISOString()
      }
    } catch (error) {
      console.error(`Failed to fetch template reviews ${id}:`, error)
      return {
        success: false,
        error: error as string,
        timestamp: new Date().toISOString()
      }
    }
  }

  /**
   * Export a template
   */
  async exportTemplate(id: string, format: 'json' | 'yaml' = 'json', includeCustomData = false): Promise<{ success: boolean; data?: TemplateExport; error?: string; timestamp: string }> {
    try {
      const exportData = await invoke<string>('export_template', { id, format, includeCustomData })

      let blob: Blob
      if (format === 'json') {
        blob = new Blob([exportData], { type: 'application/json' })
      } else {
        blob = new Blob([exportData], { type: 'application/x-yaml' })
      }

      const exportDataObject: TemplateExport = {
        template: {} as StrategyTemplate, // This would be populated by the backend
        exportedAt: new Date().toISOString(),
        exportedBy: 'current-user',
        includeCustomData,
        format
      }

      return {
        success: true,
        data: exportDataObject,
        timestamp: new Date().toISOString()
      }
    } catch (error) {
      console.error(`Failed to export template ${id}:`, error)
      return {
        success: false,
        error: error as string,
        timestamp: new Date().toISOString()
      }
    }
  }

  /**
   * Import a template from file
   */
  async importTemplate(file: File): Promise<TemplateImportResult> {
    try {
      const fileContent = await this.readFileAsText(file)
      const result = await invoke('import_template', {
        fileContent,
        filename: file.name
      })

      return {
        success: true,
        template: result.template,
        errors: [],
        warnings: result.warnings || [],
        isValid: result.isValid,
        conflictingIds: result.conflictingIds
      }
    } catch (error) {
      console.error('Failed to import template:', error)
      return {
        success: false,
        errors: [error as string],
        warnings: [],
        isValid: false
      }
    }
  }

  /**
   * Download a template file
   */
  async downloadTemplate(id: string, format: 'json' | 'yaml' = 'json'): Promise<{ success: boolean; error?: string; timestamp: string }> {
    try {
      const exportResult = await this.exportTemplate(id, format)
      if (exportResult.success && exportResult.data) {
        // Create download link
        const url = URL.createObjectURL(new Blob([JSON.stringify(exportResult.data)], { type: 'application/json' }))
        const link = document.createElement('a')
        link.href = url
        link.download = `template_${exportResult.data.template.name}_${new Date().toISOString().split('T')[0]}.${format}`
        document.body.appendChild(link)
        link.click()
        document.body.removeChild(link)
        URL.revokeObjectURL(url)
      }

      return {
        success: exportResult.success,
        error: exportResult.error,
        timestamp: exportResult.timestamp
      }
    } catch (error) {
      console.error(`Failed to download template ${id}:`, error)
      return {
        success: false,
        error: error as string,
        timestamp: new Date().toISOString()
      }
    }
  }

  /**
   * Search templates
   */
  async searchTemplates(query: string, filters?: StrategyTemplateFilter): Promise<StrategyTemplateListResponse> {
    try {
      const searchFilter = {
        ...filters,
        search: query
      }

      const response = await invoke<StrategyTemplateListResponse>('search_strategy_templates', { filter: searchFilter })
      return {
        ...response,
        success: true
      }
    } catch (error) {
      console.error('Failed to search templates:', error)
      return {
        success: false,
        error: error as string,
        timestamp: new Date().toISOString(),
        templates: [],
        total: 0,
        page: 0,
        pageSize: 0,
        filters: {} as any,
        categories: []
      }
    }
  }

  /**
   * Get popular templates
   */
  async getPopularTemplates(limit = 10): Promise<{ success: boolean; data?: StrategyTemplate[]; error?: string; timestamp: string }> {
    try {
      const templates = await invoke<StrategyTemplate[]>('get_popular_templates', { limit })
      return {
        success: true,
        data: templates,
        timestamp: new Date().toISOString()
      }
    } catch (error) {
      console.error('Failed to get popular templates:', error)
      return {
        success: false,
        error: error as string,
        timestamp: new Date().toISOString()
      }
    }
  }

  /**
   * Get new templates
   */
  async getNewTemplates(limit = 10): Promise<{ success: boolean; data?: StrategyTemplate[]; error?: string; timestamp: string }> {
    try {
      const templates = await invoke<StrategyTemplate[]>('get_new_templates', { limit })
      return {
        success: true,
        data: templates,
        timestamp: new Date().toISOString()
      }
    } catch (error) {
      console.error('Failed to get new templates:', error)
      return {
        success: false,
        error: error as string,
        timestamp: new Date().toISOString()
      }
    }
  }

  /**
   * Preview template performance
   */
  async previewTemplatePerformance(templateId: string, parameters: Record<string, any>, symbol: string, timeframe: string): Promise<{ success: boolean; data?: any; error?: string; timestamp: string }> {
    try {
      const result = await invoke('preview_template_performance', { templateId, parameters, symbol, timeframe })
      return {
        success: true,
        data: result,
        timestamp: new Date().toISOString()
      }
    } catch (error) {
      console.error(`Failed to preview template performance ${templateId}:`, error)
      return {
        success: false,
        error: error as string,
        timestamp: new Date().toISOString()
      }
    }
  }

  // Helper methods
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
export const strategyTemplateService = new StrategyTemplateService()

// Export convenience functions
export const {
  getTemplates,
  getTemplate,
  getCategories,
  createFromTemplate,
  createTemplate,
  updateTemplate,
  deleteTemplate,
  validateTemplate,
  getTemplateAnalytics,
  getRecommendedTemplates,
  rateTemplate,
  getTemplateReviews,
  exportTemplate,
  importTemplate,
  downloadTemplate,
  searchTemplates,
  getPopularTemplates,
  getNewTemplates,
  previewTemplatePerformance
} = strategyTemplateService