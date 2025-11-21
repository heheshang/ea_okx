<template>
  <el-dialog
    v-model="visible"
    :title="template?.name"
    width="90%"
    max-width="900px"
    :before-close="handleClose"
    class="template-preview-dialog"
  >
    <div v-if="template" class="template-preview">
      <!-- Header section -->
      <div class="preview-header">
        <div class="template-meta">
          <div class="template-title">
            <h2>{{ template.name }}</h2>
            <div class="template-badges">
              <el-tag :type="getDifficultyType(template.difficulty)">
                {{ template.difficulty }}
              </el-tag>
              <el-tag v-if="template.isPremium" type="warning" :icon="Star">
                Premium
              </el-tag>
              <el-tag v-if="template.isBuiltIn" type="info" :icon="Box">
                Built-in
              </el-tag>
            </div>
          </div>

          <div class="template-category">
            <span class="category-icon" :style="{ color: template.category.color }">
              {{ template.category.icon }}
            </span>
            <span class="category-name">{{ template.category.name }}</span>
          </div>
        </div>

        <div class="template-stats">
          <div class="stat">
            <el-icon><Star /></el-icon>
            <span>{{ template.rating.toFixed(1) }} ({{ template.usage }} reviews)</span>
          </div>
          <div class="stat">
            <el-icon><Download /></el-icon>
            <span>{{ formatNumber(template.downloads) }} downloads</span>
          </div>
          <div class="stat">
            <el-icon><User /></el-icon>
            <span>{{ formatNumber(template.usage) }} active users</span>
          </div>
        </div>
      </div>

      <!-- Tabs -->
      <el-tabs v-model="activeTab" class="preview-tabs">
        <!-- Overview tab -->
        <el-tab-pane label="Overview" name="overview">
          <div class="overview-content">
            <!-- Description -->
            <div class="section">
              <h3>Description</h3>
              <p class="description">{{ template.description }}</p>
            </div>

            <!-- Strategy Details -->
            <div class="section">
              <h3>Strategy Details</h3>
              <div class="strategy-details">
                <div class="detail-item">
                  <span class="label">Strategy Type:</span>
                  <span class="value">{{ getStrategyTypeName(template.defaultType) }}</span>
                </div>
                <div class="detail-item">
                  <span class="label">Supported Symbols:</span>
                  <div class="value">
                    <el-tag
                      v-for="symbol in template.supportedSymbols"
                      :key="symbol"
                      size="small"
                      effect="plain"
                    >
                      {{ symbol }}
                    </el-tag>
                  </div>
                </div>
                <div class="detail-item">
                  <span class="label">Supported Timeframes:</span>
                  <div class="value">
                    <el-tag
                      v-for="timeframe in template.supportedTimeframes"
                      :key="timeframe"
                      size="small"
                      effect="plain"
                    >
                      {{ timeframe }}
                    </el-tag>
                  </div>
                </div>
                <div class="detail-item">
                  <span class="label">Author:</span>
                  <span class="value">{{ template.author }} ({{ template.authorType }})</span>
                </div>
                <div class="detail-item">
                  <span class="label">Version:</span>
                  <span class="value">{{ template.version }}</span>
                </div>
              </div>
            </div>

            <!-- Risk Information -->
            <div class="section">
              <h3>Risk Information</h3>
              <div class="risk-info">
                <div class="risk-level">
                  <span class="label">Risk Level:</span>
                  <el-tag :type="getRiskType(template.riskGuidelines.riskLevel)">
                    {{ template.riskGuidelines.riskLevel }}
                  </el-tag>
                </div>
                <div class="recommended-experience">
                  <span class="label">Recommended Experience:</span>
                  <span class="value">{{ template.riskGuidelines.recommendedExperience }}</span>
                </div>
                <div class="min-capital">
                  <span class="label">Minimum Capital:</span>
                  <div class="value">
                    <span class="capital-amount">
                      Conservative: ${{ formatNumber(template.riskGuidelines.minCapital.conservative) }}
                    </span>
                    <span class="capital-amount">
                      Aggressive: ${{ formatNumber(template.riskGuidelines.minCapital.aggressive) }}
                    </span>
                  </div>
                </div>
                <div class="recommended-leverage">
                  <span class="label">Recommended Leverage:</span>
                  <span class="value">
                    {{ template.riskGuidelines.recommendedLeverage.min }}x - {{ template.riskGuidelines.recommendedLeverage.max }}x
                    <small class="leverage-desc">{{ template.riskGuidelines.recommendedLeverage.description }}</small>
                  </span>
                </div>
              </div>
            </div>

            <!-- Tags -->
            <div v-if="template.tags.length > 0" class="section">
              <h3>Tags</h3>
              <div class="tags">
                <el-tag
                  v-for="tag in template.tags"
                  :key="tag"
                  class="tag"
                  effect="plain"
                >
                  {{ tag }}
                </el-tag>
              </div>
            </div>
          </div>
        </el-tab-pane>

        <!-- Parameters tab -->
        <el-tab-pane label="Parameters" name="parameters">
          <div class="parameters-content">
            <div class="section">
              <h3>Default Parameters</h3>
              <div class="parameters-grid">
                <div
                  v-for="(value, key) in template.defaultParameters"
                  :key="key"
                  class="parameter-item"
                >
                  <div class="parameter-header">
                    <span class="parameter-name">{{ key }}</span>
                    <span class="parameter-value">{{ value }}</span>
                  </div>
                  <div v-if="template.parameterDescriptions[key]" class="parameter-description">
                    {{ template.parameterDescriptions[key] }}
                  </div>
                  <div v-if="template.parameterRanges[key]" class="parameter-range">
                    Range: {{ template.parameterRanges[key].min }} - {{ template.parameterRanges[key].max }}
                    (default: {{ template.parameterRanges[key].default }})
                  </div>
                </div>
              </div>
            </div>

            <div class="section">
              <h3>Risk Parameters</h3>
              <div class="parameters-grid">
                <div
                  v-for="(value, key) in template.defaultRiskParams"
                  :key="key"
                  class="parameter-item"
                >
                  <div class="parameter-header">
                    <span class="parameter-name">{{ key }}</span>
                    <span class="parameter-value">{{ value }}</span>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </el-tab-pane>

        <!-- Performance tab -->
        <el-tab-pane label="Performance" name="performance">
          <div class="performance-content">
            <!-- Expected Performance -->
            <div v-if="template.expectedPerformance" class="section">
              <h3>Expected Performance</h3>
              <div class="performance-metrics">
                <div class="metric-card">
                  <div class="metric-title">Annual Return</div>
                  <div class="metric-value positive">
                    {{ formatPercentage(template.expectedPerformance.annualReturn) }}
                  </div>
                </div>
                <div class="metric-card">
                  <div class="metric-title">Max Drawdown</div>
                  <div class="metric-value negative">
                    {{ formatPercentage(template.expectedPerformance.maxDrawdown) }}
                  </div>
                </div>
                <div class="metric-card">
                  <div class="metric-title">Sharpe Ratio</div>
                  <div class="metric-value">
                    {{ template.expectedPerformance.sharpeRatio.toFixed(2) }}
                  </div>
                </div>
                <div class="metric-card">
                  <div class="metric-title">Win Rate</div>
                  <div class="metric-value">
                    {{ formatPercentage(template.expectedPerformance.winRate) }}
                  </div>
                </div>
              </div>
              <p class="performance-description">
                {{ template.expectedPerformance.description }}
              </p>
            </div>

            <!-- Backtest Results -->
            <div v-if="template.backtestResults" class="section">
              <h3>Backtest Results</h3>
              <div class="backtest-info">
                <div class="backtest-period">
                  <strong>Period:</strong> {{ template.backtestResults.period }}
                </div>
                <div class="backtest-metrics">
                  <div class="metric-card">
                    <div class="metric-title">Return</div>
                    <div class="metric-value" :class="getReturnClass(template.backtestResults.return)">
                      {{ formatPercentage(template.backtestResults.return) }}
                    </div>
                  </div>
                  <div class="metric-card">
                    <div class="metric-title">Sharpe Ratio</div>
                    <div class="metric-value">
                      {{ template.backtestResults.sharpeRatio.toFixed(2) }}
                    </div>
                  </div>
                  <div class="metric-card">
                    <div class="metric-title">Max Drawdown</div>
                    <div class="metric-value negative">
                      {{ formatPercentage(template.backtestResults.maxDrawdown) }}
                    </div>
                  </div>
                  <div class="metric-card">
                    <div class="metric-title">Win Rate</div>
                    <div class="metric-value">
                      {{ formatPercentage(template.backtestResults.winRate) }}
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </el-tab-pane>

        <!-- Examples tab -->
        <el-tab-pane label="Examples" name="examples">
          <div class="examples-content">
            <div class="section">
              <h3>Usage Examples</h3>
              <div class="examples-grid">
                <div
                  v-for="(example, index) in template.content.examples"
                  :key="index"
                  class="example-card"
                >
                  <div class="example-header">
                    <h4>{{ example.title }}</h4>
                    <el-tag :type="getMarketConditionType(example.marketCondition)">
                      {{ example.marketCondition }}
                    </el-tag>
                  </div>
                  <p class="example-description">{{ example.description }}</p>

                  <div v-if="example.parameters" class="example-parameters">
                    <h5>Parameters:</h5>
                    <div class="parameters-list">
                      <div
                        v-for="(value, key) in example.parameters"
                        :key="key"
                        class="parameter-item"
                      >
                        <span class="param-name">{{ key }}:</span>
                        <span class="param-value">{{ value }}</span>
                      </div>
                    </div>
                  </div>

                  <div v-if="example.results" class="example-results">
                    <h5>Results:</h5>
                    <div class="results-grid">
                      <div class="result-item">
                        <span class="result-label">Period:</span>
                        <span class="result-value">{{ example.results.period }}</span>
                      </div>
                      <div class="result-item">
                        <span class="result-label">Return:</span>
                        <span class="result-value" :class="getReturnClass(example.results.return)">
                          {{ formatPercentage(example.results.return) }}
                        </span>
                      </div>
                      <div class="result-item">
                        <span class="result-label">Sharpe:</span>
                        <span class="result-value">{{ example.results.sharpeRatio.toFixed(2) }}</span>
                      </div>
                      <div class="result-item">
                        <span class="result-label">Max DD:</span>
                        <span class="result-value negative">
                          {{ formatPercentage(example.results.maxDrawdown) }}
                        </span>
                      </div>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </el-tab-pane>
      </el-tabs>
    </div>

    <template #footer>
      <div class="dialog-footer">
        <el-button @click="handleClose">Cancel</el-button>
        <el-button @click="handlePreviewPerformance">Preview Performance</el-button>
        <el-button type="primary" @click="handleCreateStrategy">
          Create Strategy
        </el-button>
      </div>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { ElMessage } from 'element-plus'
import { Star, Download, User, Box } from '@element-plus/icons-vue'
import type { StrategyTemplate } from '@/types/strategyTemplate'
import { StrategyType } from '@/types/strategy'

interface Props {
  modelValue: boolean
  template: StrategyTemplate | null
}

interface Emits {
  (e: 'update:modelValue', value: boolean): void
  (e: 'create-strategy', template: StrategyTemplate, customization?: any): void
  (e: 'preview-performance', template: StrategyTemplate): void
}

const props = defineProps<Props>()
const emit = defineEmits<Emits>()

const activeTab = ref('overview')

const visible = computed({
  get: () => props.modelValue,
  set: (value: boolean) => emit('update:modelValue', value)
})

// Watch for template changes to reset to overview tab
watch(() => props.template, () => {
  if (props.template) {
    activeTab.value = 'overview'
  }
})

// Methods
const getDifficultyType = (difficulty: string) => {
  switch (difficulty) {
    case 'beginner': return 'success'
    case 'intermediate': return 'warning'
    case 'advanced': return 'danger'
    default: return 'info'
  }
}

const getRiskType = (riskLevel: string) => {
  switch (riskLevel) {
    case 'low': return 'success'
    case 'medium': return 'warning'
    case 'high': return 'danger'
    default: return 'info'
  }
}

const getMarketConditionType = (condition: string) => {
  switch (condition) {
    case 'trending': return 'primary'
    case 'ranging': return 'success'
    case 'volatile': return 'warning'
    case 'stable': return 'info'
    default: return ''
  }
}

const getReturnClass = (returnValue: number) => {
  return returnValue >= 0 ? 'positive' : 'negative'
}

const getStrategyTypeName = (type: StrategyType): string => {
  const typeNames: Record<StrategyType, string> = {
    ma_crossover: 'MA Crossover',
    grid_trading: 'Grid Trading',
    rsi_mean_reversion: 'RSI Mean Reversion',
    bollinger_bands: 'Bollinger Bands',
    macomentum: 'MACD Momentum',
    arbitrage: 'Arbitrage',
    dca: 'Dollar Cost Averaging',
    custom: 'Custom Strategy'
  }
  return typeNames[type] || type
}

const formatPercentage = (value: number): string => {
  return `${(value * 100).toFixed(1)}%`
}

const formatNumber = (num: number): string => {
  if (num >= 1000) {
    return `${(num / 1000).toFixed(1)}k`
  }
  return num.toString()
}

const handleClose = () => {
  visible.value = false
}

const handleCreateStrategy = () => {
  if (props.template) {
    emit('create-strategy', props.template)
    handleClose()
  }
}

const handlePreviewPerformance = () => {
  if (props.template) {
    emit('preview-performance', props.template)
  }
}
</script>

<style lang="scss" scoped>
@import '../styles/utilities.scss';
.template-preview-dialog {
  :deep(.el-dialog__body) {
    padding: 0;
    max-height: 70vh;
    overflow-y: auto;
  }
}

.template-preview {
  .preview-header {
    padding: var(--spacing-xl) var(--spacing-lg);
    border-bottom: 1px solid var(--border-light);
    background: var(--bg-secondary);

    .template-meta {
      display: flex;
      justify-content: space-between;
      align-items: flex-start;
      margin-bottom: var(--spacing-md);

      @include respond-to('mobile') {
        flex-direction: column;
        gap: var(--spacing-sm);
      }

      .template-title {
        h2 {
          margin: 0 0 var(--spacing-xs) 0;
          font-size: var(--font-size-2xl);
          color: var(--text-primary);
        }

        .template-badges {
          display: flex;
          gap: var(--spacing-xs);
          flex-wrap: wrap;
        }
      }

      .template-category {
        display: flex;
        align-items: center;
        gap: var(--spacing-xs);
        font-size: var(--font-size-lg);
        color: var(--text-secondary);

        .category-name {
          font-weight: var(--font-weight-medium);
        }
      }
    }

    .template-stats {
      display: flex;
      gap: var(--spacing-lg);
      flex-wrap: wrap;

      .stat {
        display: flex;
        align-items: center;
        gap: var(--spacing-xs);
        color: var(--text-secondary);
        font-size: var(--font-size-sm);

        .el-icon {
          font-size: var(--font-size-base);
        }
      }
    }
  }

  .preview-tabs {
    :deep(.el-tabs__header) {
      margin: 0;
      background: var(--bg-primary);
      border-bottom: 1px solid var(--border-light);

      .el-tabs__nav-wrap {
        padding: 0 var(--spacing-lg);
      }
    }

    :deep(.el-tabs__content) {
      padding: var(--spacing-lg);
    }
  }

  .section {
    margin-bottom: var(--spacing-xl);

    &:last-child {
      margin-bottom: 0;
    }

    h3 {
      margin: 0 0 var(--spacing-md) 0;
      font-size: var(--font-size-lg);
      color: var(--text-primary);
      font-weight: var(--font-weight-semibold);
    }

    h4 {
      margin: 0 0 var(--spacing-sm) 0;
      font-size: var(--font-size-base);
      color: var(--text-primary);
      font-weight: var(--font-weight-medium);
    }

    h5 {
      margin: var(--spacing-sm) 0 var(--spacing-xs) 0;
      font-size: var(--font-size-sm);
      color: var(--text-primary);
      font-weight: var(--font-weight-medium);
    }
  }

  .description {
    color: var(--text-secondary);
    line-height: 1.6;
    font-size: var(--font-size-base);
  }

  .strategy-details {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
    gap: var(--spacing-md);

    .detail-item {
      display: flex;
      flex-direction: column;
      gap: var(--spacing-xs);

      .label {
        font-size: var(--font-size-sm);
        color: var(--text-secondary);
        font-weight: var(--font-weight-medium);
      }

      .value {
        color: var(--text-primary);
        display: flex;
        gap: var(--spacing-xs);
        flex-wrap: wrap;
      }
    }
  }

  .risk-info {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
    gap: var(--spacing-md);

    .risk-level,
    .recommended-experience,
    .min-capital,
    .recommended-leverage {
      display: flex;
      flex-direction: column;
      gap: var(--spacing-xs);

      .label {
        font-size: var(--font-size-sm);
        color: var(--text-secondary);
        font-weight: var(--font-weight-medium);
      }

      .value {
        color: var(--text-primary);

        .capital-amount {
          display: block;
          font-size: var(--font-size-sm);
        }

        .leverage-desc {
          display: block;
          font-size: var(--font-size-xs);
          color: var(--text-secondary);
          margin-top: 2px;
        }
      }
    }
  }

  .tags {
    display: flex;
    gap: var(--spacing-xs);
    flex-wrap: wrap;

    .tag {
      font-size: var(--font-size-sm);
    }
  }

  .parameters-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
    gap: var(--spacing-md);

    .parameter-item {
      padding: var(--spacing-md);
      border: 1px solid var(--border-light);
      border-radius: var(--border-radius-md);
      background: var(--bg-secondary);

      .parameter-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-bottom: var(--spacing-xs);

        .parameter-name {
          font-weight: var(--font-weight-medium);
          color: var(--text-primary);
        }

        .parameter-value {
          color: var(--primary-color);
          font-weight: var(--font-weight-medium);
        }
      }

      .parameter-description {
        font-size: var(--font-size-sm);
        color: var(--text-secondary);
        margin-bottom: var(--spacing-xs);
      }

      .parameter-range {
        font-size: var(--font-size-xs);
        color: var(--text-secondary);
      }
    }
  }

  .performance-metrics,
  .backtest-metrics {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
    gap: var(--spacing-md);
    margin-bottom: var(--spacing-lg);

    .metric-card {
      padding: var(--spacing-md);
      border: 1px solid var(--border-light);
      border-radius: var(--border-radius-md);
      text-align: center;

      .metric-title {
        font-size: var(--font-size-sm);
        color: var(--text-secondary);
        margin-bottom: var(--spacing-xs);
      }

      .metric-value {
        font-size: var(--font-size-lg);
        font-weight: var(--font-weight-semibold);

        &.positive {
          color: var(--success-color);
        }

        &.negative {
          color: var(--danger-color);
        }
      }
    }
  }

  .performance-description {
    color: var(--text-secondary);
    font-style: italic;
    margin-top: var(--spacing-md);
  }

  .backtest-info {
    .backtest-period {
      margin-bottom: var(--spacing-md);
      color: var(--text-secondary);
    }
  }

  .examples-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(400px, 1fr));
    gap: var(--spacing-lg);

    .example-card {
      padding: var(--spacing-lg);
      border: 1px solid var(--border-light);
      border-radius: var(--border-radius-md);
      background: var(--bg-secondary);

      .example-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-bottom: var(--spacing-sm);
      }

      .example-description {
        color: var(--text-secondary);
        margin-bottom: var(--spacing-md);
      }

      .example-parameters,
      .example-results {
        .parameters-list,
        .results-grid {
          display: grid;
          grid-template-columns: repeat(auto-fit, minmax(120px, 1fr));
          gap: var(--spacing-xs);
          margin-top: var(--spacing-xs);

          .parameter-item,
          .result-item {
            display: flex;
            justify-content: space-between;
            font-size: var(--font-size-sm);

            .param-name,
            .result-label {
              color: var(--text-secondary);
            }

            .param-value,
            .result-value {
              color: var(--text-primary);
              font-weight: var(--font-weight-medium);

              &.positive {
                color: var(--success-color);
              }

              &.negative {
                color: var(--danger-color);
              }
            }
          }
        }
      }
    }
  }
}

.dialog-footer {
  display: flex;
  justify-content: flex-end;
  gap: var(--spacing-sm);
}

// Mobile responsiveness
@media (max-width: 768px) {
  .template-preview {
    .performance-metrics,
    .backtest-metrics {
      grid-template-columns: repeat(2, 1fr);
    }

    .examples-grid {
      grid-template-columns: 1fr;
    }

    .strategy-details,
    .risk-info {
      grid-template-columns: 1fr;
    }
  }
}
</style>