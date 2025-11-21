<template>
  <div class="templates-view">
    <div class="templates-container">
      <!-- Page Header -->
      <div class="page-header">
        <div class="header-content">
          <div class="header-left">
            <h1 class="page-title">
              <span class="title-icon">📋</span>
              Strategy Templates
            </h1>
            <p class="page-subtitle">
              Browse, create, and import trading strategy templates
            </p>
          </div>

          <div class="header-actions">
            <el-button
              type="primary"
              :icon="Plus"
              @click="showCreateDialog = true"
            >
              Create Template
            </el-button>
            <el-button
              :icon="Upload"
              @click="showImportDialog = true"
            >
              Import Template
            </el-button>
          </div>
        </div>

        <!-- Quick Stats -->
        <div class="quick-stats">
          <div class="stat-card">
            <div class="stat-icon">📚</div>
            <div class="stat-content">
              <div class="stat-value">{{ stats.totalTemplates }}</div>
              <div class="stat-label">Total Templates</div>
            </div>
          </div>

          <div class="stat-card">
            <div class="stat-icon">⭐</div>
            <div class="stat-content">
              <div class="stat-value">{{ stats.premiumTemplates }}</div>
              <div class="stat-label">Premium</div>
            </div>
          </div>

          <div class="stat-card">
            <div class="stat-icon">👥</div>
            <div class="stat-content">
              <div class="stat-value">{{ stats.communityTemplates }}</div>
              <div class="stat-label">Community</div>
            </div>
          </div>

          <div class="stat-card">
            <div class="stat-icon">🏗️</div>
            <div class="stat-content">
              <div class="stat-value">{{ stats.builtInTemplates }}</div>
              <div class="stat-label">Built-in</div>
            </div>
          </div>
        </div>
      </div>

      <!-- Main Content -->
      <div class="main-content">
        <!-- Featured Templates -->
        <div v-if="!loading && featuredTemplates.length > 0" class="featured-section">
          <div class="section-header">
            <h2>Featured Templates</h2>
            <el-button type="text" @click="viewAllFeatured">
              View All
              <el-icon><ArrowRight /></el-icon>
            </el-button>
          </div>

          <div class="featured-grid">
            <div
              v-for="template in featuredTemplates"
              :key="template.id"
              class="featured-template"
            >
              <TemplateCard
                :template="template"
                @select="handleTemplateSelect"
                @preview="handleTemplatePreview"
                @create-strategy="handleCreateStrategy"
              />
            </div>
          </div>
        </div>

        <!-- Template Browser -->
        <div class="browser-section">
          <div class="section-header">
            <h2>Browse Templates</h2>
            <div class="section-actions">
              <el-button-group>
                <el-button
                  :type="viewMode === 'grid' ? 'primary' : 'default'"
                  @click="viewMode = 'grid'"
                  :icon="Grid"
                >
                  Grid
                </el-button>
                <el-button
                  :type="viewMode === 'list' ? 'primary' : 'default'"
                  @click="viewMode = 'list'"
                  :icon="List"
                >
                  List
                </el-button>
              </el-button-group>
            </div>
          </div>

          <TemplateBrowser
            :view-mode="viewMode"
            @template-select="handleTemplateSelect"
            @template-preview="handleTemplatePreview"
            @create-strategy="handleCreateStrategy"
            @template-created="handleTemplateCreated"
            @template-imported="handleTemplateImported"
          />
        </div>
      </div>

      <!-- Recommended Templates -->
      <div v-if="recommendedTemplates.length > 0" class="recommended-section">
        <div class="section-header">
          <h2>Recommended for You</h2>
          <el-button type="text" @click="refreshRecommendations">
            <el-icon><Refresh /></el-icon>
            Refresh
          </el-button>
        </div>

        <div class="recommended-grid">
          <div
            v-for="item in recommendedTemplates"
            :key="item.template.id"
            class="recommended-item"
          >
            <div class="recommendation-card">
              <div class="recommendation-header">
                <h4>{{ item.template.name }}</h4>
                <el-tag size="small" type="success">
                  {{ Math.round(item.confidence * 100) }}% match
                </el-tag>
              </div>

              <p class="recommendation-reason">{{ item.reason }}</p>

              <div class="recommendation-actions">
                <el-button
                  size="small"
                  @click="handleTemplatePreview(item.template)"
                >
                  Preview
                </el-button>
                <el-button
                  type="primary"
                  size="small"
                  @click="handleCreateStrategy(item.template)"
                >
                  Use Template
                </el-button>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Dialogs -->
      <TemplatePreviewDialog
        v-model="showPreviewDialog"
        :template="selectedTemplate"
        @create-strategy="handleCreateStrategy"
        @preview-performance="handlePreviewPerformance"
      />

      <CreateTemplateDialog
        v-model="showCreateDialog"
        @created="handleTemplateCreated"
      />

      <ImportTemplateDialog
        v-model="showImportDialog"
        @imported="handleTemplateImported"
      />
    </div>
  </div>
</template> 

<script setup lang="ts">
import { ref, reactive, onMounted, computed } from 'vue'
import { useRouter } from 'vue-router'
import { ElMessage } from 'element-plus'
import {
  Plus,
  Upload,
  ArrowRight,
  Grid,
  List,
  Refresh
} from '@element-plus/icons-vue'
import MainLayout from '@/layouts/MainLayout.vue'
import TemplateBrowser from '@/components/TemplateBrowser.vue'
import TemplateCard from '@/components/TemplateCard.vue'
import TemplatePreviewDialog from '@/components/TemplatePreviewDialog.vue'
import CreateTemplateDialog from '@/components/CreateTemplateDialog.vue'
import ImportTemplateDialog from '@/components/ImportTemplateDialog.vue'
import type { StrategyTemplate, TemplateRecommendation, StrategyTemplateListResponse } from '@/types/strategyTemplate'
import { strategyTemplateService } from '@/services/strategyTemplateService'

const router = useRouter()

// State
const loading = ref(true)
const viewMode = ref<'grid' | 'list'>('grid')
const templates = ref<StrategyTemplate[]>([])
const featuredTemplates = ref<StrategyTemplate[]>([])
const recommendedTemplates = ref<TemplateRecommendation[]>([])

// Dialogs
const showPreviewDialog = ref(false)
const showCreateDialog = ref(false)
const showImportDialog = ref(false)
const selectedTemplate = ref<StrategyTemplate | null>(null)

// Stats
const stats = reactive({
  totalTemplates: 0,
  premiumTemplates: 0,
  communityTemplates: 0,
  builtInTemplates: 0
})

// Methods
const loadTemplates = async () => {
  try {
    loading.value = true

    const response = await strategyTemplateService.getTemplates({
      limit: 50,
      sortBy: 'popularity',
      sortOrder: 'desc'
    }) as StrategyTemplateListResponse & { success?: boolean; error?: string }

    if (response.success) {
      templates.value = response.templates || []
      updateStats()
    } else if (response.error) {
      ElMessage.error(response.error || 'Failed to load templates')
    } else {
      // Direct response without success/error wrapper
      templates.value = response.templates || []
      updateStats()
    }
  } catch (error) {
    console.error('Failed to load templates:', error)
    ElMessage.error('Failed to load templates')
  } finally {
    loading.value = false
  }
}

const loadFeaturedTemplates = async () => {
  try {
    const response = await strategyTemplateService.getPopularTemplates(6)

    if (response.success && response.data) {
      featuredTemplates.value = response.data
    }
  } catch (error) {
    console.error('Failed to load featured templates:', error)
  }
}

const loadRecommendedTemplates = async () => {
  try {
    const response = await strategyTemplateService.getRecommendedTemplates(undefined, 4)

    if (response.success && response.data) {
      recommendedTemplates.value = response.data
    }
  } catch (error) {
    console.error('Failed to load recommended templates:', error)
  }
}

const updateStats = () => {
  const allTemplates = templates.value

  stats.totalTemplates = allTemplates.length
  stats.premiumTemplates = allTemplates.filter(t => t.isPremium).length
  stats.communityTemplates = allTemplates.filter(t => t.authorType === 'community').length
  stats.builtInTemplates = allTemplates.filter(t => t.isBuiltIn).length
}

const handleTemplateSelect = (template: StrategyTemplate) => {
  selectedTemplate.value = template
  showPreviewDialog.value = true
}

const handleTemplatePreview = (template: StrategyTemplate) => {
  selectedTemplate.value = template
  showPreviewDialog.value = true
}

const handleCreateStrategy = async (template: StrategyTemplate, customization?: any) => {
  try {
    const request = {
      templateId: template.id,
      customization: customization || {}
    }

    const response = await strategyTemplateService.createFromTemplate(request)

    if (response.success && response.data) {
      ElMessage.success('Strategy created successfully from template')
      router.push(`/strategies/${response.data.id}`)
    } else {
      ElMessage.error(response.error || 'Failed to create strategy from template')
    }
  } catch (error) {
    console.error('Failed to create strategy from template:', error)
    ElMessage.error('Failed to create strategy from template')
  }
}

const handlePreviewPerformance = (template: StrategyTemplate) => {
  // This would open a performance preview dialog or navigate to performance page
  ElMessage.info('Performance preview feature coming soon')
}

const handleTemplateCreated = (template: any) => {
  ElMessage.success('Template created successfully')
  loadTemplates()
  loadFeaturedTemplates()
}

const handleTemplateImported = (result: any) => {
  ElMessage.success('Template imported successfully')
  loadTemplates()
  loadFeaturedTemplates()
}

const refreshRecommendations = async () => {
  await loadRecommendedTemplates()
  ElMessage.success('Recommendations refreshed')
}

const viewAllFeatured = () => {
  // This would navigate to a featured templates page or filter the browser
  ElMessage.info('View all featured templates feature coming soon')
}

// Lifecycle
onMounted(async () => {
  await Promise.all([
    loadTemplates(),
    loadFeaturedTemplates(),
    loadRecommendedTemplates()
  ])
})
</script>

<style lang="scss" scoped>
@import '../styles/utilities.scss';
.templates-view {
  min-height: 100vh;
  background: var(--bg-primary);
}

.templates-container {
  padding: var(--spacing-lg);
  max-width: 1400px;
  margin: 0 auto;

  .page-header {
    margin-bottom: var(--spacing-2xl);

    .header-content {
      display: flex;
      justify-content: space-between;
      align-items: flex-start;
      margin-bottom: var(--spacing-xl);

      @include respond-to('mobile') {
        flex-direction: column;
        gap: var(--spacing-md);
      }

      .header-left {
        .page-title {
          display: flex;
          align-items: center;
          gap: var(--spacing-sm);
          margin: 0 0 var(--spacing-xs) 0;
          font-size: var(--font-size-3xl);
          font-weight: var(--font-weight-bold);
          color: var(--text-primary);

          .title-icon {
            font-size: var(--font-size-2xl);
          }
        }

        .page-subtitle {
          margin: 0;
          color: var(--text-secondary);
          font-size: var(--font-size-lg);
        }
      }

      .header-actions {
        display: flex;
        gap: var(--spacing-sm);

        @include respond-to('mobile') {
          flex-direction: column;
          width: 100%;

          .el-button {
            width: 100%;
          }
        }
      }
    }

    .quick-stats {
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
      gap: var(--spacing-md);

      .stat-card {
        display: flex;
        align-items: center;
        gap: var(--spacing-md);
        padding: var(--spacing-lg);
        background: var(--bg-secondary);
        border-radius: var(--border-radius-md);
        border: 1px solid var(--border-light);
        transition: all 0.3s ease;

        &:hover {
          transform: translateY(-2px);
          box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
        }

        .stat-icon {
          font-size: 2rem;
          width: 50px;
          height: 50px;
          display: flex;
          align-items: center;
          justify-content: center;
          background: var(--primary-color-light);
          border-radius: var(--border-radius-md);
        }

        .stat-content {
          .stat-value {
            font-size: var(--font-size-xl);
            font-weight: var(--font-weight-bold);
            color: var(--text-primary);
            line-height: 1.2;
          }

          .stat-label {
            font-size: var(--font-size-sm);
            color: var(--text-secondary);
          }
        }
      }
    }
  }

  .main-content {
    .featured-section {
      margin-bottom: var(--spacing-2xl);

      .section-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-bottom: var(--spacing-lg);

        h2 {
          margin: 0;
          font-size: var(--font-size-xl);
          color: var(--text-primary);
          font-weight: var(--font-weight-semibold);
        }
      }

      .featured-grid {
        display: grid;
        grid-template-columns: repeat(auto-fit, minmax(320px, 1fr));
        gap: var(--spacing-lg);

        @include respond-to('mobile') {
          grid-template-columns: 1fr;
        }
      }
    }

    .browser-section {
      .section-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-bottom: var(--spacing-lg);

        h2 {
          margin: 0;
          font-size: var(--font-size-xl);
          color: var(--text-primary);
          font-weight: var(--font-weight-semibold);
        }

        .section-actions {
          .el-button-group {
            @include respond-to('mobile') {
              display: none;
            }
          }
        }
      }
    }
  }

  .recommended-section {
    margin-top: var(--spacing-2xl);

    .section-header {
      display: flex;
      justify-content: space-between;
      align-items: center;
      margin-bottom: var(--spacing-lg);

      h2 {
        margin: 0;
        font-size: var(--font-size-xl);
        color: var(--text-primary);
        font-weight: var(--font-weight-semibold);
      }
    }

    .recommended-grid {
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
      gap: var(--spacing-md);

      .recommended-item {
        .recommendation-card {
          padding: var(--spacing-lg);
          background: var(--bg-secondary);
          border-radius: var(--border-radius-md);
          border: 1px solid var(--border-light);

          .recommendation-header {
            display: flex;
            justify-content: space-between;
            align-items: flex-start;
            margin-bottom: var(--spacing-sm);

            h4 {
              margin: 0;
              font-size: var(--font-size-base);
              color: var(--text-primary);
              line-height: 1.4;
            }
          }

          .recommendation-reason {
            margin: 0 0 var(--spacing-md) 0;
            font-size: var(--font-size-sm);
            color: var(--text-secondary);
            line-height: 1.5;
          }

          .recommendation-actions {
            display: flex;
            gap: var(--spacing-sm);

            .el-button {
              flex: 1;
            }
          }
        }
      }
    }
  }
}
</style>