<template>
  <el-dialog
    v-model="visible"
    title="Create Strategy Template"
    width="80%"
    max-width="1000px"
    :before-close="handleClose"
    class="create-template-dialog"
  >
    <div class="create-template">
      <!-- Wizard Steps -->
      <div class="wizard-header">
        <el-steps :active="currentStep" finish-status="success">
          <el-step
            v-for="step in steps"
            :key="step.step"
            :title="step.title"
            :description="step.description"
          />
        </el-steps>
      </div>

      <!-- Step Content -->
      <div class="wizard-content">
        <!-- Step 1: Basic Information -->
        <div v-if="currentStep === 0" class="step-content">
          <div class="step-title">
            <h3>Basic Information</h3>
            <p>Provide the basic details about your strategy template</p>
          </div>

          <el-form
            ref="basicFormRef"
            :model="formData.basic"
            :rules="basicRules"
            label-width="120px"
            class="template-form"
          >
            <el-row :gutter="20">
              <el-col :span="12">
                <el-form-item label="Template Name" prop="name">
                  <el-input
                    v-model="formData.basic.name"
                    placeholder="Enter template name"
                  />
                </el-form-item>
              </el-col>
              <el-col :span="12">
                <el-form-item label="Category" prop="categoryId">
                  <el-select
                    v-model="formData.basic.categoryId"
                    placeholder="Select category"
                    style="width: 100%"
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
                </el-form-item>
              </el-col>
            </el-row>

            <el-form-item label="Description" prop="description">
              <el-input
                v-model="formData.basic.description"
                type="textarea"
                :rows="4"
                placeholder="Describe your strategy template"
              />
            </el-form-item>

            <el-row :gutter="20">
              <el-col :span="12">
                <el-form-item label="Strategy Type" prop="type">
                  <el-select
                    v-model="formData.basic.type"
                    placeholder="Select strategy type"
                    style="width: 100%"
                  >
                    <el-option
                      v-for="type in strategyTypes"
                      :key="type.value"
                      :label="type.label"
                      :value="type.value"
                    />
                  </el-select>
                </el-form-item>
              </el-col>
              <el-col :span="12">
                <el-form-item label="Difficulty" prop="difficulty">
                  <el-select
                    v-model="formData.basic.difficulty"
                    placeholder="Select difficulty level"
                    style="width: 100%"
                  >
                    <el-option label="Beginner" value="beginner" />
                    <el-option label="Intermediate" value="intermediate" />
                    <el-option label="Advanced" value="advanced" />
                  </el-select>
                </el-form-item>
              </el-col>
            </el-row>

            <el-row :gutter="20">
              <el-col :span="12">
                <el-form-item label="Author" prop="author">
                  <el-input
                    v-model="formData.basic.author"
                    placeholder="Your name or organization"
                  />
                </el-form-item>
              </el-col>
              <el-col :span="12">
                <el-form-item label="Version" prop="version">
                  <el-input
                    v-model="formData.basic.version"
                    placeholder="1.0.0"
                  />
                </el-form-item>
              </el-col>
            </el-row>

            <el-form-item label="Tags">
              <el-tag
                v-for="tag in formData.basic.tags"
                :key="tag"
                closable
                @close="removeTag(tag)"
                class="tag-item"
              >
                {{ tag }}
              </el-tag>
              <el-input
                v-if="tagInputVisible"
                ref="tagInputRef"
                v-model="tagInputValue"
                size="small"
                class="tag-input"
                @keyup.enter="handleTagInputConfirm"
                @blur="handleTagInputConfirm"
              />
              <el-button
                v-else
                size="small"
                @click="showTagInput"
              >
                + Add Tag
              </el-button>
            </el-form-item>

            <el-form-item>
              <el-checkbox v-model="formData.basic.isPublic">
                Make this template public
              </el-checkbox>
              <el-checkbox v-model="formData.basic.isPremium">
                This is a premium template
              </el-checkbox>
            </el-form-item>
          </el-form>
        </div>

        <!-- Step 2: Strategy Parameters -->
        <div v-if="currentStep === 1" class="step-content">
          <div class="step-title">
            <h3>Strategy Parameters</h3>
            <p>Define the parameters for your strategy template</p>
          </div>

          <el-form
            ref="parametersFormRef"
            :model="formData.parameters"
            label-width="150px"
            class="template-form"
          >
            <!-- Supported Symbols -->
            <el-form-item label="Supported Symbols">
              <div class="supported-symbols">
                <el-tag
                  v-for="symbol in formData.parameters.supportedSymbols"
                  :key="symbol"
                  closable
                  @close="removeSymbol(symbol)"
                  class="symbol-tag"
                >
                  {{ symbol }}
                </el-tag>
                <el-input
                  v-if="symbolInputVisible"
                  ref="symbolInputRef"
                  v-model="symbolInputValue"
                  size="small"
                  placeholder="Symbol (e.g., BTCUSDT)"
                  class="symbol-input"
                  @keyup.enter="handleSymbolInputConfirm"
                  @blur="handleSymbolInputConfirm"
                />
                <el-button
                  v-else
                  size="small"
                  @click="showSymbolInput"
                >
                  + Add Symbol
                </el-button>
              </div>
            </el-form-item>

            <!-- Supported Timeframes -->
            <el-form-item label="Timeframes">
              <el-checkbox-group v-model="formData.parameters.supportedTimeframes">
                <el-checkbox
                  v-for="timeframe in availableTimeframes"
                  :key="timeframe"
                  :label="timeframe"
                >
                  {{ timeframe }}
                </el-checkbox>
              </el-checkbox-group>
            </el-form-item>

            <!-- Default Parameters -->
            <el-form-item label="Default Parameters">
              <div class="parameters-editor">
                <div
                  v-for="(param, index) in formData.parameters.defaultParams"
                  :key="index"
                  class="parameter-row"
                >
                  <el-input
                    v-model="param.name"
                    placeholder="Parameter name"
                    size="small"
                    class="param-name"
                  />
                  <el-input
                    v-model="param.value"
                    placeholder="Default value"
                    size="small"
                    class="param-value"
                  />
                  <el-input
                    v-model="param.description"
                    placeholder="Description"
                    size="small"
                    class="param-description"
                  />
                  <el-button
                    type="danger"
                    size="small"
                    :icon="Delete"
                    @click="removeParameter(index)"
                  />
                </div>
                <el-button
                  type="primary"
                  size="small"
                  :icon="Plus"
                  @click="addParameter"
                >
                  Add Parameter
                </el-button>
              </div>
            </el-form-item>

            <!-- Risk Parameters -->
            <el-form-item label="Risk Parameters">
              <div class="risk-parameters">
                <el-row :gutter="20">
                  <el-col :span="8">
                    <el-form-item label="Risk Level">
                      <el-select v-model="formData.parameters.riskLevel" style="width: 100%">
                        <el-option label="Low" value="low" />
                        <el-option label="Medium" value="medium" />
                        <el-option label="High" value="high" />
                      </el-select>
                    </el-form-item>
                  </el-col>
                  <el-col :span="8">
                    <el-form-item label="Max Leverage">
                      <el-input-number
                        v-model="formData.parameters.maxLeverage"
                        :min="1"
                        :max="100"
                        style="width: 100%"
                      />
                    </el-form-item>
                  </el-col>
                  <el-col :span="8">
                    <el-form-item label="Stop Loss %">
                      <el-input-number
                        v-model="formData.parameters.stopLoss"
                        :min="0"
                        :max="50"
                        :step="0.1"
                        style="width: 100%"
                      />
                    </el-form-item>
                  </el-col>
                </el-row>

                <el-row :gutter="20">
                  <el-col :span="8">
                    <el-form-item label="Take Profit %">
                      <el-input-number
                        v-model="formData.parameters.takeProfit"
                        :min="0"
                        :max="200"
                        :step="0.1"
                        style="width: 100%"
                      />
                    </el-form-item>
                  </el-col>
                  <el-col :span="8">
                    <el-form-item label="Min Capital">
                      <el-input-number
                        v-model="formData.parameters.minCapital"
                        :min="0"
                        :step="100"
                        style="width: 100%"
                      />
                    </el-form-item>
                  </el-col>
                  <el-col :span="8">
                    <el-form-item label="Position Size %">
                      <el-input-number
                        v-model="formData.parameters.positionSize"
                        :min="1"
                        :max="100"
                        style="width: 100%"
                      />
                    </el-form-item>
                  </el-col>
                </el-row>
              </div>
            </el-form-item>
          </el-form>
        </div>

        <!-- Step 3: Performance & Examples -->
        <div v-if="currentStep === 2" class="step-content">
          <div class="step-title">
            <h3>Performance & Examples</h3>
            <p>Add performance expectations and usage examples</p>
          </div>

          <el-form
            ref="performanceFormRef"
            :model="formData.performance"
            label-width="150px"
            class="template-form"
          >
            <!-- Expected Performance -->
            <el-form-item label="Expected Annual Return">
              <el-input-number
                v-model="formData.performance.expectedAnnualReturn"
                :min="-100"
                :max="500"
                :step="0.1"
                :precision="1"
                style="width: 200px"
              />
              <span class="unit">%</span>
            </el-form-item>

            <el-form-item label="Expected Max Drawdown">
              <el-input-number
                v-model="formData.performance.expectedMaxDrawdown"
                :min="0"
                :max="100"
                :step="0.1"
                :precision="1"
                style="width: 200px"
              />
              <span class="unit">%</span>
            </el-form-item>

            <el-form-item label="Expected Sharpe Ratio">
              <el-input-number
                v-model="formData.performance.expectedSharpeRatio"
                :min="-2"
                :max="10"
                :step="0.1"
                :precision="2"
                style="width: 200px"
              />
            </el-form-item>

            <el-form-item label="Expected Win Rate">
              <el-input-number
                v-model="formData.performance.expectedWinRate"
                :min="0"
                :max="100"
                :step="0.1"
                :precision="1"
                style="width: 200px"
              />
              <span class="unit">%</span>
            </el-form-item>

            <!-- Performance Description -->
            <el-form-item label="Performance Description">
              <el-input
                v-model="formData.performance.description"
                type="textarea"
                :rows="3"
                placeholder="Describe the expected performance characteristics..."
              />
            </el-form-item>

            <!-- Examples -->
            <el-form-item label="Usage Examples">
              <div class="examples-editor">
                <div
                  v-for="(example, index) in formData.performance.examples"
                  :key="index"
                  class="example-card"
                >
                  <div class="example-header">
                    <el-input
                      v-model="example.title"
                      placeholder="Example title"
                      size="small"
                    />
                    <el-select
                      v-model="example.marketCondition"
                      placeholder="Market condition"
                      size="small"
                    >
                      <el-option label="Trending" value="trending" />
                      <el-option label="Ranging" value="ranging" />
                      <el-option label="Volatile" value="volatile" />
                      <el-option label="Stable" value="stable" />
                    </el-select>
                    <el-button
                      type="danger"
                      size="small"
                      :icon="Delete"
                      @click="removeExample(index)"
                    />
                  </div>
                  <el-input
                    v-model="example.description"
                    type="textarea"
                    :rows="2"
                    placeholder="Example description"
                    size="small"
                  />
                </div>
                <el-button
                  type="primary"
                  size="small"
                  :icon="Plus"
                  @click="addExample"
                >
                  Add Example
                </el-button>
              </div>
            </el-form-item>

            <!-- Documentation -->
            <el-form-item label="Tutorial URL">
              <el-input
                v-model="formData.performance.tutorialUrl"
                placeholder="https://example.com/tutorial"
              />
            </el-form-item>

            <el-form-item label="Documentation URL">
              <el-input
                v-model="formData.performance.documentationUrl"
                placeholder="https://example.com/docs"
              />
            </el-form-item>
          </el-form>
        </div>

        <!-- Step 4: Review -->
        <div v-if="currentStep === 3" class="step-content">
          <div class="step-title">
            <h3>Review Template</h3>
            <p>Review your template before creating it</p>
          </div>

          <div class="review-content">
            <!-- Basic Information Review -->
            <div class="review-section">
              <h4>Basic Information</h4>
              <div class="review-grid">
                <div class="review-item">
                  <span class="label">Name:</span>
                  <span class="value">{{ formData.basic.name }}</span>
                </div>
                <div class="review-item">
                  <span class="label">Category:</span>
                  <span class="value">{{ getCategoryName(formData.basic.categoryId) }}</span>
                </div>
                <div class="review-item">
                  <span class="label">Type:</span>
                  <span class="value">{{ getStrategyTypeName(formData.basic.type) }}</span>
                </div>
                <div class="review-item">
                  <span class="label">Difficulty:</span>
                  <span class="value">{{ formData.basic.difficulty }}</span>
                </div>
                <div class="review-item">
                  <span class="label">Author:</span>
                  <span class="value">{{ formData.basic.author }}</span>
                </div>
                <div class="review-item">
                  <span class="label">Version:</span>
                  <span class="value">{{ formData.basic.version }}</span>
                </div>
                <div class="review-item full-width">
                  <span class="label">Description:</span>
                  <span class="value">{{ formData.basic.description }}</span>
                </div>
                <div class="review-item full-width">
                  <span class="label">Tags:</span>
                  <span class="value">{{ formData.basic.tags.join(', ') || 'None' }}</span>
                </div>
              </div>
            </div>

            <!-- Parameters Review -->
            <div class="review-section">
              <h4>Parameters</h4>
              <div class="review-grid">
                <div class="review-item full-width">
                  <span class="label">Supported Symbols:</span>
                  <span class="value">{{ formData.parameters.supportedSymbols.join(', ') }}</span>
                </div>
                <div class="review-item full-width">
                  <span class="label">Supported Timeframes:</span>
                  <span class="value">{{ formData.parameters.supportedTimeframes.join(', ') }}</span>
                </div>
                <div class="review-item">
                  <span class="label">Risk Level:</span>
                  <span class="value">{{ formData.parameters.riskLevel }}</span>
                </div>
                <div class="review-item">
                  <span class="label">Max Leverage:</span>
                  <span class="value">{{ formData.parameters.maxLeverage }}x</span>
                </div>
                <div class="review-item">
                  <span class="label">Stop Loss:</span>
                  <span class="value">{{ formData.parameters.stopLoss }}%</span>
                </div>
                <div class="review-item">
                  <span class="label">Take Profit:</span>
                  <span class="value">{{ formData.parameters.takeProfit }}%</span>
                </div>
              </div>
            </div>

            <!-- Performance Review -->
            <div class="review-section">
              <h4>Expected Performance</h4>
              <div class="performance-review">
                <div class="metric-review">
                  <span class="metric-label">Annual Return:</span>
                  <span class="metric-value">{{ formData.performance.expectedAnnualReturn }}%</span>
                </div>
                <div class="metric-review">
                  <span class="metric-label">Max Drawdown:</span>
                  <span class="metric-value">{{ formData.performance.expectedMaxDrawdown }}%</span>
                </div>
                <div class="metric-review">
                  <span class="metric-label">Sharpe Ratio:</span>
                  <span class="metric-value">{{ formData.performance.expectedSharpeRatio }}</span>
                </div>
                <div class="metric-review">
                  <span class="metric-label">Win Rate:</span>
                  <span class="metric-value">{{ formData.performance.expectedWinRate }}%</span>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <template #footer>
      <div class="dialog-footer">
        <el-button @click="handleCancel">Cancel</el-button>
        <el-button v-if="currentStep > 0" @click="handlePrevious">Previous</el-button>
        <el-button
          v-if="currentStep < steps.length - 1"
          type="primary"
          @click="handleNext"
        >
          Next
        </el-button>
        <el-button
          v-if="currentStep === steps.length - 1"
          type="primary"
          :loading="creating"
          @click="handleCreate"
        >
          Create Template
        </el-button>
      </div>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
import { ref, reactive, nextTick, onMounted } from 'vue'
import { ElMessage, ElForm } from 'element-plus'
import { Plus, Delete } from '@element-plus/icons-vue'
import type { StrategyTemplateCategory } from '@/types/strategyTemplate'
import { StrategyType } from '@/types/strategy'
import { strategyTemplateService } from '@/services/strategyTemplateService'

interface Props {
  modelValue: boolean
}

interface Emits {
  (e: 'update:modelValue', value: boolean): void
  (e: 'created', template: any): void
}

const props = defineProps<Props>()
const emit = defineEmits<Emits>()

const visible = ref(false)
const currentStep = ref(0)
const creating = ref(false)
const categories = ref<StrategyTemplateCategory[]>([])

// Form refs
const basicFormRef = ref<InstanceType<typeof ElForm>>()
const parametersFormRef = ref<InstanceType<typeof ElForm>>()
const performanceFormRef = ref<InstanceType<typeof ElForm>>()

// Steps
const steps = [
  { step: 1, title: 'Basic Info', description: 'Template details' },
  { step: 2, title: 'Parameters', description: 'Strategy configuration' },
  { step: 3, title: 'Performance', description: 'Expectations & examples' },
  { step: 4, title: 'Review', description: 'Review & create' }
]

// Form data
const formData = reactive({
  basic: {
    name: '',
    categoryId: '',
    description: '',
    type: '',
    difficulty: 'beginner',
    author: '',
    version: '1.0.0',
    tags: [],
    isPublic: true,
    isPremium: false
  },
  parameters: {
    supportedSymbols: ['BTCUSDT', 'ETHUSDT'],
    supportedTimeframes: ['1h', '4h', '1d'],
    defaultParams: [],
    riskLevel: 'medium',
    maxLeverage: 10,
    stopLoss: 2.0,
    takeProfit: 5.0,
    minCapital: 1000,
    positionSize: 10
  },
  performance: {
    expectedAnnualReturn: 25.0,
    expectedMaxDrawdown: 15.0,
    expectedSharpeRatio: 1.5,
    expectedWinRate: 60.0,
    description: '',
    examples: [],
    tutorialUrl: '',
    documentationUrl: ''
  }
})

// Tag input
const tagInputVisible = ref(false)
const tagInputValue = ref('')
const tagInputRef = ref()

// Symbol input
const symbolInputVisible = ref(false)
const symbolInputValue = ref('')
const symbolInputRef = ref()

// Strategy types
const strategyTypes = [
  { value: 'ma_crossover', label: 'MA Crossover' },
  { value: 'grid_trading', label: 'Grid Trading' },
  { value: 'rsi_mean_reversion', label: 'RSI Mean Reversion' },
  { value: 'bollinger_bands', label: 'Bollinger Bands' },
  { value: 'macomentum', label: 'MACD Momentum' },
  { value: 'arbitrage', label: 'Arbitrage' },
  { value: 'dca', label: 'Dollar Cost Averaging' },
  { value: 'custom', label: 'Custom Strategy' }
]

const availableTimeframes = ['5m', '15m', '30m', '1h', '4h', '12h', '1d', '3d', '1w']

// Validation rules
const basicRules = {
  name: [
    { required: true, message: 'Please enter template name', trigger: 'blur' }
  ],
  categoryId: [
    { required: true, message: 'Please select category', trigger: 'change' }
  ],
  description: [
    { required: true, message: 'Please enter description', trigger: 'blur' },
    { min: 10, message: 'Description must be at least 10 characters', trigger: 'blur' }
  ],
  type: [
    { required: true, message: 'Please select strategy type', trigger: 'change' }
  ],
  difficulty: [
    { required: true, message: 'Please select difficulty level', trigger: 'change' }
  ],
  author: [
    { required: true, message: 'Please enter author name', trigger: 'blur' }
  ],
  version: [
    { required: true, message: 'Please enter version', trigger: 'blur' }
  ]
}

// Computed
const getCategoryName = (categoryId: string): string => {
  const category = categories.value.find(cat => cat.id === categoryId)
  return category?.name || categoryId
}

const getStrategyTypeName = (type: StrategyType): string => {
  const strategyType = strategyTypes.find(st => st.value === type)
  return strategyType?.label || type
}

// Methods
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

const resetForm = () => {
  currentStep.value = 0

  // Reset form data
  Object.assign(formData.basic, {
    name: '',
    categoryId: '',
    description: '',
    type: '',
    difficulty: 'beginner',
    author: '',
    version: '1.0.0',
    tags: [],
    isPublic: true,
    isPremium: false
  })

  Object.assign(formData.parameters, {
    supportedSymbols: ['BTCUSDT', 'ETHUSDT'],
    supportedTimeframes: ['1h', '4h', '1d'],
    defaultParams: [],
    riskLevel: 'medium',
    maxLeverage: 10,
    stopLoss: 2.0,
    takeProfit: 5.0,
    minCapital: 1000,
    positionSize: 10
  })

  Object.assign(formData.performance, {
    expectedAnnualReturn: 25.0,
    expectedMaxDrawdown: 15.0,
    expectedSharpeRatio: 1.5,
    expectedWinRate: 60.0,
    description: '',
    examples: [],
    tutorialUrl: '',
    documentationUrl: ''
  })
}

const validateCurrentStep = (): boolean => {
  if (currentStep.value === 0) {
    return basicFormRef.value?.validate() || false
  }
  if (currentStep.value === 1) {
    return parametersFormRef.value?.validate() || false
  }
  if (currentStep.value === 2) {
    return performanceFormRef.value?.validate() || false
  }
  return true
}

const handleNext = async () => {
  if (await validateCurrentStep()) {
    if (currentStep.value < steps.length - 1) {
      currentStep.value++
    }
  }
}

const handlePrevious = () => {
  if (currentStep.value > 0) {
    currentStep.value--
  }
}

const handleCreate = async () => {
  try {
    creating.value = true

    // Build template data
    const templateData = {
      ...formData.basic,
      category: categories.value.find(cat => cat.id === formData.basic.categoryId),
      defaultType: formData.basic.type as StrategyType,
      supportedSymbols: formData.parameters.supportedSymbols,
      supportedTimeframes: formData.parameters.supportedTimeframes,
      defaultParameters: {},
      parameterRanges: {},
      parameterDescriptions: {},
      defaultRiskParams: {
        riskLevel: formData.parameters.riskLevel,
        maxLeverage: formData.parameters.maxLeverage,
        stopLoss: formData.parameters.stopLoss,
        takeProfit: formData.parameters.takeProfit,
        minCapital: formData.parameters.minCapital,
        positionSize: formData.parameters.positionSize
      },
      riskGuidelines: {
        recommendedLeverage: {
          min: Math.max(1, formData.parameters.maxLeverage / 2),
          max: formData.parameters.maxLeverage,
          description: 'Recommended leverage range'
        },
        recommendedPositionSize: {
          min: Math.max(1, formData.parameters.positionSize / 2),
          max: formData.parameters.positionSize,
          description: 'Recommended position size range'
        },
        riskLevel: formData.parameters.riskLevel as any,
        recommendedExperience: formData.basic.difficulty === 'beginner' ? 'No experience required' : 'Some trading experience recommended',
        minCapital: {
          conservative: formData.parameters.minCapital,
          aggressive: formData.parameters.minCapital / 2
        }
      },
      expectedPerformance: {
        annualReturn: formData.performance.expectedAnnualReturn / 100,
        maxDrawdown: formData.performance.expectedMaxDrawdown / 100,
        sharpeRatio: formData.performance.expectedSharpeRatio,
        winRate: formData.performance.expectedWinRate / 100,
        description: formData.performance.description
      },
      content: {
        overview: formData.basic.description,
        strategy: '',
        setup: '',
        parameters: {},
        risk: '',
        examples: formData.performance.examples
      },
      tutorialUrl: formData.performance.tutorialUrl,
      documentationUrl: formData.performance.documentationUrl,
      requirements: [],
      pros: [],
      cons: []
    }

    // Convert default parameters
    formData.parameters.defaultParams.forEach(param => {
      templateData.defaultParameters[param.name] = param.value
      templateData.parameterDescriptions[param.name] = param.description
      templateData.parameterRanges[param.name] = {
        min: parseFloat(param.value) * 0.5,
        max: parseFloat(param.value) * 2,
        default: parseFloat(param.value),
        step: 0.1,
        type: 'number'
      }
    })

    const response = await strategyTemplateService.createTemplate(templateData)

    if (response.success && response.data) {
      ElMessage.success('Template created successfully')
      emit('created', response.data)
      visible.value = false
    } else {
      ElMessage.error(response.error || 'Failed to create template')
    }
  } catch (error) {
    console.error('Failed to create template:', error)
    ElMessage.error('Failed to create template')
  } finally {
    creating.value = false
  }
}

const handleClose = () => {
  visible.value = false
}

const handleCancel = () => {
  visible.value = false
}

// Tag management
const removeTag = (tag: string) => {
  const index = formData.basic.tags.indexOf(tag)
  if (index > -1) {
    formData.basic.tags.splice(index, 1)
  }
}

const showTagInput = () => {
  tagInputVisible.value = true
  nextTick(() => {
    tagInputRef.value?.focus()
  })
}

const handleTagInputConfirm = () => {
  if (tagInputValue.value && !formData.basic.tags.includes(tagInputValue.value)) {
    formData.basic.tags.push(tagInputValue.value)
  }
  tagInputVisible.value = false
  tagInputValue.value = ''
}

// Symbol management
const removeSymbol = (symbol: string) => {
  const index = formData.parameters.supportedSymbols.indexOf(symbol)
  if (index > -1) {
    formData.parameters.supportedSymbols.splice(index, 1)
  }
}

const showSymbolInput = () => {
  symbolInputVisible.value = true
  nextTick(() => {
    symbolInputRef.value?.focus()
  })
}

const handleSymbolInputConfirm = () => {
  if (symbolInputValue.value && !formData.parameters.supportedSymbols.includes(symbolInputValue.value)) {
    formData.parameters.supportedSymbols.push(symbolInputValue.value.toUpperCase())
  }
  symbolInputVisible.value = false
  symbolInputValue.value = ''
}

// Parameter management
const addParameter = () => {
  formData.parameters.defaultParams.push({
    name: '',
    value: '',
    description: ''
  })
}

const removeParameter = (index: number) => {
  formData.parameters.defaultParams.splice(index, 1)
}

// Example management
const addExample = () => {
  formData.performance.examples.push({
    title: '',
    description: '',
    marketCondition: 'trending'
  })
}

const removeExample = (index: number) => {
  formData.performance.examples.splice(index, 1)
}

// Watch visibility
const handleVisibilityChange = (val: boolean) => {
  if (val) {
    resetForm()
    loadCategories()
  }
}

// Initialize
onMounted(() => {
  loadCategories()
})
</script>

<style lang="scss" scoped>
.create-template-dialog {
  :deep(.el-dialog__body) {
    padding: 0;
    max-height: 80vh;
    overflow-y: auto;
  }
}

.create-template {
  .wizard-header {
    padding: var(--spacing-xl) var(--spacing-lg);
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border-light);
  }

  .wizard-content {
    padding: var(--spacing-xl);
  }

  .step-content {
    .step-title {
      text-align: center;
      margin-bottom: var(--spacing-xl);

      h3 {
        margin: 0 0 var(--spacing-sm) 0;
        font-size: var(--font-size-xl);
        color: var(--text-primary);
      }

      p {
        margin: 0;
        color: var(--text-secondary);
        font-size: var(--font-size-base);
      }
    }

    .template-form {
      max-width: 800px;
      margin: 0 auto;
    }
  }
}

.category-option {
  display: flex;
  align-items: center;
  gap: var(--spacing-xs);

  .category-icon {
    font-size: var(--font-size-sm);
  }
}

.tag-input,
.symbol-input {
  width: 100px;
  margin-left: var(--spacing-xs);
}

.tag-item,
.symbol-tag {
  margin-right: var(--spacing-xs);
  margin-bottom: var(--spacing-xs);
}

.supported-symbols {
  .symbol-tag {
    margin-right: var(--spacing-xs);
    margin-bottom: var(--spacing-xs);
  }

  .symbol-input {
    width: 120px;
  }
}

.parameters-editor {
  .parameter-row {
    display: grid;
    grid-template-columns: 1fr 1fr 2fr auto;
    gap: var(--spacing-sm);
    margin-bottom: var(--spacing-sm);
    align-items: center;

    @include respond-to('mobile') {
      grid-template-columns: 1fr;
    }
  }
}

.risk-parameters {
  background: var(--bg-secondary);
  padding: var(--spacing-lg);
  border-radius: var(--border-radius-md);
}

.unit {
  margin-left: var(--spacing-xs);
  color: var(--text-secondary);
  font-size: var(--font-size-sm);
}

.examples-editor {
  .example-card {
    background: var(--bg-secondary);
    padding: var(--spacing-md);
    border-radius: var(--border-radius-md);
    margin-bottom: var(--spacing-md);

    .example-header {
      display: grid;
      grid-template-columns: 2fr 1fr auto;
      gap: var(--spacing-sm);
      margin-bottom: var(--spacing-sm);
      align-items: center;

      @include respond-to('mobile') {
        grid-template-columns: 1fr;
      }
    }
  }
}

.review-content {
  max-width: 800px;
  margin: 0 auto;

  .review-section {
    margin-bottom: var(--spacing-xl);

    h4 {
      margin: 0 0 var(--spacing-md) 0;
      font-size: var(--font-size-lg);
      color: var(--text-primary);
      font-weight: var(--font-weight-semibold);
      padding-bottom: var(--spacing-xs);
      border-bottom: 1px solid var(--border-light);
    }

    .review-grid {
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
      gap: var(--spacing-md);

      .review-item {
        display: flex;
        flex-direction: column;
        gap: var(--spacing-xs);

        &.full-width {
          grid-column: 1 / -1;
        }

        .label {
          font-size: var(--font-size-sm);
          color: var(--text-secondary);
          font-weight: var(--font-weight-medium);
        }

        .value {
          color: var(--text-primary);
          font-weight: var(--font-weight-medium);
        }
      }
    }
  }

  .performance-review {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
    gap: var(--spacing-md);
    background: var(--bg-secondary);
    padding: var(--spacing-lg);
    border-radius: var(--border-radius-md);

    .metric-review {
      text-align: center;

      .metric-label {
        display: block;
        font-size: var(--font-size-sm);
        color: var(--text-secondary);
        margin-bottom: var(--spacing-xs);
      }

      .metric-value {
        font-size: var(--font-size-lg);
        font-weight: var(--font-weight-semibold);
        color: var(--primary-color);
      }
    }
  }
}

.dialog-footer {
  display: flex;
  justify-content: flex-end;
  gap: var(--spacing-sm);
}
</style>