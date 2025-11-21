<template>
  <div class="template-browser">
    <!-- Header with search and filters -->
    <div class="browser-header">
      <div class="header-content">
        <div class="header-title">
          <h2>Strategy Templates</h2>
          <p class="subtitle">Choose from pre-built strategies or create your own</p>
        </div>

        <div class="header-actions">
          <el-button type="primary" @click="showCreateDialog = true" :icon="Plus">
            Create Template
          </el-button>
          <el-button @click="showImportDialog = true" :icon="Upload">
            Import Template
          </el-button>
        </div>
      </div>

      <!-- Search and filters -->
      <div class="search-filters">
        <div class="search-section">
          <el-input
            v-model="searchQuery"
            placeholder="Search templates..."
            :prefix-icon="Search"
            clearable
            @input="handleSearch"
            class="search-input"
          />
        </div>

        <div class="filter-section">
          <el-select
            v-model="selectedCategories"
            multiple
            placeholder="Categories"
            collapse-tags
            collapse-tags-tooltip
            @change="handleFilterChange"
            class="filter-select"
          >
            <el-option
              v-for="category in categories"
              :key="category.id"
              :label="category.name"
              :value="category.id"
            >
              <div class="category-option">
                <span class="category-icon">{{ category.icon }}</span>
                <span>{{ category.name }}</span>
              </div>
            </el-option>
          </el-select>

          <el-select
            v-model="selectedDifficulty"
            placeholder="Difficulty"
            @change="handleFilterChange"
            class="filter-select"
          >
            <el-option label="All Levels" value="" />
            <el-option label="Beginner" value="beginner" />
            <el-option label="Intermediate" value="intermediate" />
            <el-option label="Advanced" value="advanced" />
          </el-select>

          <el-select
            v-model="sortBy"
            placeholder="Sort by"
            @change="handleSortChange"
            class="filter-select"
          >
            <el-option label="Most Popular" value="popularity" />
            <el-option label="Highest Rated" value="rating" />
            <el-option label="Newest" value="newest" />
            <el-option label="Most Downloads" value="downloads" />
          </el-select>
        </div>
      </div>
    </div>

    <!-- Quick access categories -->
    <div class="quick-categories" v-if="!searchQuery">
      <div class="category-chips">
        <el-button
          v-for="category in categories"
          :key="category.id"
          :class="['category-chip', { active: selectedCategories.includes(category.id) }]"
          @click="toggleCategory(category.id)"
          :color="category.color"
          variant="plain"
          size="small"
        >
          <span class="chip-icon">{{ category.icon }}</span>
          {{ category.name }}
        </el-button>
      </div>
    </div>

    <!-- Loading state -->
    <div v-if="loading" class="loading-container">
      <el-skeleton :rows="3" animated />
      <el-skeleton :rows="3" animated />
      <el-skeleton :rows="3" animated />
    </div>

    <!-- Templates grid -->
    <div v-else-if="templates.length > 0" class="templates-grid">
      <div
        v-for="template in templates"
        :key="template.id"
        class="template-card-wrapper"
      >
        <TemplateCard
          :template="template"
          @select="handleTemplateSelect"
          @preview="handleTemplatePreview"
          @create-strategy="handleCreateStrategy"
        />
      </div>
    </div>

    <!-- Empty state -->
    <div v-else class="empty-state">
      <div class="empty-content">
        <div class="empty-icon">📋</div>
        <h3>No templates found</h3>
        <p>Try adjusting your search criteria or create a custom template</p>
        <el-button type="primary" @click="showCreateDialog = true">
          Create Your First Template
        </el-button>
      </div>
    </div>

    <!-- Pagination -->
    <div v-if="templates.length > 0 && totalPages > 1" class="pagination-container">
      <el-pagination
        v-model:current-page="currentPage"
        v-model:page-size="pageSize"
        :page-sizes="[12, 24, 48]"
        :total="totalTemplates"
        layout="total, sizes, prev, pager, next, jumper"
        @size-change="handleSizeChange"
        @current-change="handlePageChange"
      />
    </div>

    <!-- Template Preview Dialog -->
    <TemplatePreviewDialog
      v-model="showPreviewDialog"
      :template="selectedTemplate"
      @create-strategy="handleCreateStrategy"
    />

    <!-- Create Template Dialog -->
    <CreateTemplateDialog
      v-model="showCreateDialog"
      @created="handleTemplateCreated"
    />

    <!-- Import Template Dialog -->
    <ImportTemplateDialog
      v-model="showImportDialog"
      @imported="handleTemplateImported"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted, computed, watch } from 'vue'
import { useRouter } from 'vue-router'
import { ElMessage } from 'element-plus'
import { Search, Plus, Upload } from '@element-plus/icons-vue'
import { debounce } from 'lodash-es'
import type { StrategyTemplate, StrategyTemplateCategory, StrategyTemplateFilter } from '@/types/strategyTemplate'
import { strategyTemplateService } from '@/services/strategyTemplateService'
import TemplateCard from './TemplateCard.vue'
import TemplatePreviewDialog from './TemplatePreviewDialog.vue'
import CreateTemplateDialog from './CreateTemplateDialog.vue'
import ImportTemplateDialog from './ImportTemplateDialog.vue'

const router = useRouter()

// State
const loading = ref(false)
const templates = ref<StrategyTemplate[]>([])
const categories = ref<StrategyTemplateCategory[]>([])
const totalTemplates = ref(0)
const currentPage = ref(1)
const pageSize = ref(24)
const totalPages = ref(0)

// Search and filters
const searchQuery = ref('')
const selectedCategories = ref<string[]>([])
const selectedDifficulty = ref('')
const sortBy = ref('popularity')

// Dialogs
const showPreviewDialog = ref(false)
const showCreateDialog = ref(false)
const showImportDialog = ref(false)
const selectedTemplate = ref<StrategyTemplate | null>(null)

// Filter state
const filters = reactive<StrategyTemplateFilter>({
  sortBy: 'popularity',
  sortOrder: 'desc',
  limit: 24,
  offset: 0
})

// Computed
const filteredTemplates = computed(() => {
  let result = templates.value

  if (searchQuery.value) {
    const query = searchQuery.value.toLowerCase()
    result = result.filter(template =>
      template.name.toLowerCase().includes(query) ||
      template.description.toLowerCase().includes(query) ||
      template.tags.some(tag => tag.toLowerCase().includes(query))
    )
  }

  if (selectedCategories.value.length > 0) {
    result = result.filter(template =>
      selectedCategories.value.includes(template.category.id)
    )
  }

  if (selectedDifficulty.value) {
    result = result.filter(template =>
      template.difficulty === selectedDifficulty.value
    )
  }

  return result
})

// Methods
const loadTemplates = async () => {
  try {
    loading.value = true

    const updatedFilters = {
      ...filters,
      search: searchQuery.value || undefined,
      category: selectedCategories.value.length > 0 ? selectedCategories.value : undefined,
      difficulty: selectedDifficulty.value ? [selectedDifficulty.value] : undefined,
      offset: (currentPage.value - 1) * pageSize.value,
      limit: pageSize.value
    }

    const response = await strategyTemplateService.getTemplates(updatedFilters)

    if (response.success) {
      templates.value = response.templates || []
      totalTemplates.value = response.total || 0
      totalPages.value = Math.ceil(totalTemplates.value / pageSize.value)
    } else {
      ElMessage.error(response.error || 'Failed to load templates')
    }
  } catch (error) {
    console.error('Failed to load templates:', error)
    ElMessage.error('Failed to load templates')
  } finally {
    loading.value = false
  }
}

const loadCategories = async () => {
  try {
    const response = await strategyTemplateService.getCategories()
    if (response.success && response.data) {
      categories.value = response.data
    }
  } catch (error) {
    console.error('Failed to load categories:', error)
  }
}

const handleSearch = debounce(() => {
  currentPage.value = 1
  loadTemplates()
}, 300)

const handleFilterChange = () => {
  currentPage.value = 1
  loadTemplates()
}

const handleSortChange = () => {
  filters.sortBy = sortBy.value as any
  loadTemplates()
}

const handlePageChange = (page: number) => {
  currentPage.value = page
  loadTemplates()
}

const handleSizeChange = (size: number) => {
  pageSize.value = size
  currentPage.value = 1
  loadTemplates()
}

const toggleCategory = (categoryId: string) => {
  const index = selectedCategories.value.indexOf(categoryId)
  if (index > -1) {
    selectedCategories.value.splice(index, 1)
  } else {
    selectedCategories.value.push(categoryId)
  }
  handleFilterChange()
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

const handleTemplateCreated = (template: StrategyTemplate) => {
  ElMessage.success('Template created successfully')
  loadTemplates()
}

const handleTemplateImported = () => {
  ElMessage.success('Template imported successfully')
  loadTemplates()
}

// Watchers
watch([searchQuery, selectedCategories, selectedDifficulty, sortBy], () => {
  currentPage.value = 1
  loadTemplates()
})

// Lifecycle
onMounted(() => {
  loadCategories()
  loadTemplates()
})
</script>

<style lang="scss" scoped>
@import '../styles/utilities.scss';
.template-browser {
  padding: var(--spacing-lg);
  max-width: 1400px;
  margin: 0 auto;

  .browser-header {
    margin-bottom: var(--spacing-xl);

    .header-content {
      display: flex;
      justify-content: space-between;
      align-items: flex-start;
      margin-bottom: var(--spacing-lg);

      @include respond-to('mobile') {
        flex-direction: column;
        gap: var(--spacing-md);
      }

      .header-title {
        h2 {
          margin: 0 0 var(--spacing-xs) 0;
          color: var(--text-primary);
          font-size: var(--font-size-2xl);
          font-weight: var(--font-weight-semibold);
        }

        .subtitle {
          margin: 0;
          color: var(--text-secondary);
          font-size: var(--font-size-base);
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

    .search-filters {
      display: flex;
      flex-direction: column;
      gap: var(--spacing-md);

      @include respond-to('tablet') {
        flex-direction: row;
        align-items: center;
      }

      .search-section {
        flex: 1;

        .search-input {
          max-width: 400px;

          @include respond-to('mobile') {
            max-width: none;
          }
        }
      }

      .filter-section {
        display: flex;
        gap: var(--spacing-sm);
        flex-wrap: wrap;

        .filter-select {
          min-width: 150px;
        }
      }
    }
  }

  .quick-categories {
    margin-bottom: var(--spacing-xl);

    .category-chips {
      display: flex;
      gap: var(--spacing-sm);
      flex-wrap: wrap;

      .category-chip {
        display: flex;
        align-items: center;
        gap: var(--spacing-xs);
        border: 1px solid var(--border-light);
        background: var(--bg-primary);
        color: var(--text-secondary);

        &:hover {
          border-color: var(--primary-color);
          color: var(--primary-color);
        }

        &.active {
          background: var(--primary-color);
          color: white;
          border-color: var(--primary-color);
        }

        .chip-icon {
          font-size: var(--font-size-sm);
        }
      }
    }
  }

  .loading-container {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
    gap: var(--spacing-lg);
    margin-bottom: var(--spacing-xl);
  }

  .templates-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
    gap: var(--spacing-lg);
    margin-bottom: var(--spacing-xl);

    @include respond-to('mobile') {
      grid-template-columns: 1fr;
    }

    .template-card-wrapper {
      height: fit-content;
    }
  }

  .empty-state {
    text-align: center;
    padding: var(--spacing-3xl) var(--spacing-lg);

    .empty-content {
      max-width: 400px;
      margin: 0 auto;

      .empty-icon {
        font-size: 4rem;
        margin-bottom: var(--spacing-lg);
        opacity: 0.5;
      }

      h3 {
        margin: 0 0 var(--spacing-sm) 0;
        color: var(--text-primary);
        font-size: var(--font-size-xl);
      }

      p {
        margin: 0 0 var(--spacing-lg) 0;
        color: var(--text-secondary);
      }
    }
  }

  .pagination-container {
    display: flex;
    justify-content: center;
    margin-top: var(--spacing-xl);
  }
}

// Category option styling
:deep(.category-option) {
  display: flex;
  align-items: center;
  gap: var(--spacing-xs);

  .category-icon {
    font-size: var(--font-size-sm);
  }
}
</style>