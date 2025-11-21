<template>
  <el-card
    class="template-card"
    :class="cardClasses"
    shadow="hover"
    @click="handleCardClick"
  >
    <!-- Card header -->
    <template #header>
      <div class="card-header">
        <div class="template-info">
          <div class="template-title">
            <h3>{{ template.name }}</h3>
            <div class="template-badges">
              <el-tag
                :type="getDifficultyType(template.difficulty)"
                size="small"
              >
                {{ template.difficulty }}
              </el-tag>
              <el-tag
                v-if="template.isPremium"
                type="warning"
                size="small"
                :icon="Star"
              >
                Premium
              </el-tag>
              <el-tag
                v-if="template.isBuiltIn"
                type="info"
                size="small"
                :icon="Box"
              >
                Built-in
              </el-tag>
            </div>
          </div>

          <div class="category-info">
            <span class="category-icon" :style="{ color: template.category.color }">
              {{ template.category.icon }}
            </span>
            <span class="category-name">{{ template.category.name }}</span>
          </div>
        </div>

        <div class="template-actions" @click.stop>
          <el-dropdown trigger="click">
            <el-button type="text" :icon="MoreFilled" class="action-btn" />
            <template #dropdown>
              <el-dropdown-menu>
                <el-dropdown-item @click="handlePreview">
                  <el-icon><View /></el-icon>
                  Preview
                </el-dropdown-item>
                <el-dropdown-item @click="handleCreateStrategy">
                  <el-icon><Plus /></el-icon>
                  Create Strategy
                </el-dropdown-item>
                <el-dropdown-item @click="handleDuplicate">
                  <el-icon><CopyDocument /></el-icon>
                  Duplicate
                </el-dropdown-item>
                <el-dropdown-item @click="handleExport">
                  <el-icon><Download /></el-icon>
                  Export
                </el-dropdown-item>
                <el-dropdown-item
                  v-if="!template.isBuiltIn"
                  @click="handleDelete"
                  divided
                >
                  <el-icon><Delete /></el-icon>
                  Delete
                </el-dropdown-item>
              </el-dropdown-menu>
            </template>
          </el-dropdown>
        </div>
      </div>
    </template>

    <!-- Card content -->
    <div class="card-content">
      <!-- Description -->
      <div class="template-description">
        <p>{{ template.description }}</p>
      </div>

      <!-- Strategy type -->
      <div class="strategy-type">
        <span class="type-label">Strategy Type:</span>
        <span class="type-value">{{ getStrategyTypeName(template.defaultType) }}</span>
      </div>

      <!-- Supported symbols and timeframes -->
      <div class="supported-info">
        <div class="supported-symbols">
          <span class="info-label">Symbols:</span>
          <div class="symbol-tags">
            <el-tag
              v-for="symbol in template.supportedSymbols.slice(0, 3)"
              :key="symbol"
              size="small"
              type="info"
              effect="plain"
            >
              {{ symbol }}
            </el-tag>
            <el-tag
              v-if="template.supportedSymbols.length > 3"
              size="small"
              type="info"
              effect="plain"
            >
              +{{ template.supportedSymbols.length - 3 }}
            </el-tag>
          </div>
        </div>

        <div class="supported-timeframes">
          <span class="info-label">Timeframes:</span>
          <div class="timeframe-tags">
            <el-tag
              v-for="timeframe in template.supportedTimeframes.slice(0, 2)"
              :key="timeframe"
              size="small"
              effect="plain"
            >
              {{ timeframe }}
            </el-tag>
            <el-tag
              v-if="template.supportedTimeframes.length > 2"
              size="small"
              effect="plain"
            >
              +{{ template.supportedTimeframes.length - 2 }}
            </el-tag>
          </div>
        </div>
      </div>

      <!-- Performance preview -->
      <div v-if="template.backtestResults" class="performance-preview">
        <div class="performance-title">Backtest Results</div>
        <div class="performance-stats">
          <div class="stat-item">
            <span class="stat-label">Return</span>
            <span class="stat-value" :class="getReturnClass(template.backtestResults.return)">
              {{ formatPercentage(template.backtestResults.return) }}
            </span>
          </div>
          <div class="stat-item">
            <span class="stat-label">Sharpe</span>
            <span class="stat-value">{{ template.backtestResults.sharpeRatio.toFixed(2) }}</span>
          </div>
          <div class="stat-item">
            <span class="stat-label">Max DD</span>
            <span class="stat-value negative">
              {{ formatPercentage(template.backtestResults.maxDrawdown) }}
            </span>
          </div>
          <div class="stat-item">
            <span class="stat-label">Win Rate</span>
            <span class="stat-value">{{ formatPercentage(template.backtestResults.winRate) }}</span>
          </div>
        </div>
      </div>

      <!-- Risk indicators -->
      <div class="risk-indicators">
        <div class="risk-level">
          <span class="risk-label">Risk Level:</span>
          <el-tag
            :type="getRiskType(template.riskGuidelines.riskLevel)"
            size="small"
          >
            {{ template.riskGuidelines.riskLevel }}
          </el-tag>
        </div>

        <div class="min-capital">
          <span class="capital-label">Min Capital:</span>
          <span class="capital-value">
            ${{ formatNumber(template.riskGuidelines.minCapital.conservative) }}
          </span>
        </div>
      </div>

      <!-- Tags -->
      <div v-if="template.tags.length > 0" class="template-tags">
        <el-tag
          v-for="tag in template.tags.slice(0, 4)"
          :key="tag"
          size="small"
          effect="plain"
          class="tag"
        >
          {{ tag }}
        </el-tag>
        <span
          v-if="template.tags.length > 4"
          class="more-tags"
        >
          +{{ template.tags.length - 4 }}
        </span>
      </div>
    </div>

    <!-- Card footer -->
    <template #footer>
      <div class="card-footer">
        <div class="template-stats">
          <div class="stat">
            <el-icon><Star /></el-icon>
            <span>{{ template.rating.toFixed(1) }}</span>
          </div>
          <div class="stat">
            <el-icon><Download /></el-icon>
            <span>{{ formatNumber(template.downloads) }}</span>
          </div>
          <div class="stat">
            <el-icon><User /></el-icon>
            <span>{{ formatNumber(template.usage) }}</span>
          </div>
        </div>

        <div class="template-meta">
          <span class="author">by {{ template.author }}</span>
          <span class="version">v{{ template.version }}</span>
        </div>
      </div>
    </template>
  </el-card>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import {
  MoreFilled,
  View,
  Plus,
  CopyDocument,
  Download,
  Delete,
  Star,
  Box,
  User
} from '@element-plus/icons-vue'
import type { StrategyTemplate } from '@/types/strategyTemplate'
import { StrategyType } from '@/types/strategy'

interface Props {
  template: StrategyTemplate
}

interface Emits {
  (e: 'select', template: StrategyTemplate): void
  (e: 'preview', template: StrategyTemplate): void
  (e: 'create-strategy', template: StrategyTemplate): void
  (e: 'delete', template: StrategyTemplate): void
}

const props = defineProps<Props>()
const emit = defineEmits<Emits>()

// Computed
const cardClasses = computed(() => ({
  'premium': props.template.isPremium,
  'built-in': props.template.isBuiltIn,
  'high-difficulty': props.template.difficulty === 'advanced',
  'beginner-difficulty': props.template.difficulty === 'beginner'
}))

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

const handleCardClick = () => {
  emit('select', props.template)
}

const handlePreview = () => {
  emit('preview', props.template)
}

const handleCreateStrategy = () => {
  emit('create-strategy', props.template)
}

const handleDuplicate = async () => {
  try {
    // This would call the template service to duplicate
    ElMessage.success('Template duplicated successfully')
  } catch (error) {
    ElMessage.error('Failed to duplicate template')
  }
}

const handleExport = async () => {
  try {
    // This would call the template service to export
    ElMessage.success('Template exported successfully')
  } catch (error) {
    ElMessage.error('Failed to export template')
  }
}

const handleDelete = async () => {
  try {
    await ElMessageBox.confirm(
      `Are you sure you want to delete "${props.template.name}"?`,
      'Delete Template',
      {
        type: 'warning',
        confirmButtonText: 'Delete',
        cancelButtonText: 'Cancel'
      }
    )

    emit('delete', props.template)
  } catch {
    // User cancelled
  }
}
</script>

<style lang="scss" scoped>
.template-card {
  cursor: pointer;
  transition: all 0.3s ease;
  height: 100%;

  &:hover {
    transform: translateY(-2px);
    box-shadow: 0 8px 25px rgba(0, 0, 0, 0.15);
  }

  &.premium {
    border-left: 4px solid var(--warning-color);
  }

  &.built-in {
    border-left: 4px solid var(--info-color);
  }

  &.high-difficulty {
    border-left: 4px solid var(--danger-color);
  }

  &.beginner-difficulty {
    border-left: 4px solid var(--success-color);
  }

  :deep(.el-card__header) {
    padding: var(--spacing-md) var(--spacing-lg);
    border-bottom: 1px solid var(--border-light);
  }

  :deep(.el-card__body) {
    padding: var(--spacing-lg);
  }

  :deep(.el-card__footer) {
    padding: var(--spacing-md) var(--spacing-lg);
    border-top: 1px solid var(--border-light);
  }

  .card-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    gap: var(--spacing-sm);

    .template-info {
      flex: 1;
      min-width: 0;

      .template-title {
        margin-bottom: var(--spacing-xs);

        h3 {
          margin: 0;
          font-size: var(--font-size-lg);
          font-weight: var(--font-weight-semibold);
          color: var(--text-primary);
          line-height: 1.4;
          display: -webkit-box;
          -webkit-line-clamp: 2;
          -webkit-box-orient: vertical;
          overflow: hidden;
        }

        .template-badges {
          display: flex;
          gap: var(--spacing-xs);
          margin-top: var(--spacing-xs);
          flex-wrap: wrap;
        }
      }

      .category-info {
        display: flex;
        align-items: center;
        gap: var(--spacing-xs);
        font-size: var(--font-size-sm);
        color: var(--text-secondary);

        .category-icon {
          font-size: var(--font-size-base);
        }

        .category-name {
          font-weight: var(--font-weight-medium);
        }
      }
    }

    .template-actions {
      .action-btn {
        color: var(--text-secondary);
        padding: var(--spacing-xs);

        &:hover {
          color: var(--primary-color);
        }
      }
    }
  }

  .card-content {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-md);

    .template-description {
      p {
        margin: 0;
        color: var(--text-secondary);
        font-size: var(--font-size-sm);
        line-height: 1.5;
        display: -webkit-box;
        -webkit-line-clamp: 3;
        -webkit-box-orient: vertical;
        overflow: hidden;
      }
    }

    .strategy-type {
      display: flex;
      align-items: center;
      gap: var(--spacing-xs);
      font-size: var(--font-size-sm);

      .type-label {
        color: var(--text-secondary);
      }

      .type-value {
        color: var(--text-primary);
        font-weight: var(--font-weight-medium);
      }
    }

    .supported-info {
      display: flex;
      flex-direction: column;
      gap: var(--spacing-xs);

      .supported-symbols,
      .supported-timeframes {
        display: flex;
        align-items: center;
        gap: var(--spacing-xs);
        font-size: var(--font-size-sm);

        .info-label {
          color: var(--text-secondary);
          min-width: 60px;
        }

        .symbol-tags,
        .timeframe-tags {
          display: flex;
          gap: var(--spacing-xs);
          flex-wrap: wrap;
        }
      }
    }

    .performance-preview {
      .performance-title {
        font-size: var(--font-size-sm);
        font-weight: var(--font-weight-medium);
        color: var(--text-primary);
        margin-bottom: var(--spacing-xs);
      }

      .performance-stats {
        display: grid;
        grid-template-columns: 1fr 1fr;
        gap: var(--spacing-sm);

        .stat-item {
          display: flex;
          flex-direction: column;
          align-items: center;
          padding: var(--spacing-xs);
          background: var(--bg-secondary);
          border-radius: var(--border-radius-sm);

          .stat-label {
            font-size: var(--font-size-xs);
            color: var(--text-secondary);
            margin-bottom: 2px;
          }

          .stat-value {
            font-size: var(--font-size-sm);
            font-weight: var(--font-weight-medium);
            color: var(--text-primary);

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

    .risk-indicators {
      display: flex;
      justify-content: space-between;
      align-items: center;
      font-size: var(--font-size-sm);

      .risk-level,
      .min-capital {
        display: flex;
        align-items: center;
        gap: var(--spacing-xs);

        .risk-label,
        .capital-label {
          color: var(--text-secondary);
        }

        .capital-value {
          color: var(--text-primary);
          font-weight: var(--font-weight-medium);
        }
      }
    }

    .template-tags {
      display: flex;
      gap: var(--spacing-xs);
      align-items: center;
      flex-wrap: wrap;

      .tag {
        font-size: var(--font-size-xs);
      }

      .more-tags {
        font-size: var(--font-size-xs);
        color: var(--text-secondary);
      }
    }
  }

  .card-footer {
    display: flex;
    justify-content: space-between;
    align-items: center;

    .template-stats {
      display: flex;
      gap: var(--spacing-md);

      .stat {
        display: flex;
        align-items: center;
        gap: var(--spacing-xs);
        font-size: var(--font-size-sm);
        color: var(--text-secondary);

        .el-icon {
          font-size: var(--font-size-xs);
        }
      }
    }

    .template-meta {
      display: flex;
      gap: var(--spacing-sm);
      font-size: var(--font-size-xs);
      color: var(--text-secondary);

      .author {
        font-weight: var(--font-weight-medium);
      }

      .version {
        opacity: 0.7;
      }
    }
  }
}

// Mobile responsiveness
@media (max-width: 768px) {
  .template-card {
    .card-header {
      flex-direction: column;
      align-items: stretch;
      gap: var(--spacing-sm);
    }

    .card-content {
      .performance-preview .performance-stats {
        grid-template-columns: repeat(4, 1fr);
        gap: var(--spacing-xs);

        .stat-item {
          padding: 2px;
        }
      }

      .risk-indicators {
        flex-direction: column;
        align-items: flex-start;
        gap: var(--spacing-xs);
      }
    }

    .card-footer {
      flex-direction: column;
      align-items: flex-start;
      gap: var(--spacing-sm);
    }
  }
}
</style>