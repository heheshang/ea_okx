<template>
  <el-dialog
    v-model="visible"
    title="Import Strategy Template"
    width="600px"
    :before-close="handleClose"
    class="import-template-dialog"
  >
    <div class="import-template">
      <!-- Import Method Selection -->
      <div class="import-methods">
        <el-radio-group v-model="importMethod" class="method-group">
          <el-radio value="file" class="method-option">
            <div class="method-content">
              <div class="method-icon">📁</div>
              <div class="method-info">
                <div class="method-title">Import from File</div>
                <div class="method-desc">Upload a template file from your computer</div>
              </div>
            </div>
          </el-radio>

          <el-radio value="url" class="method-option">
            <div class="method-content">
              <div class="method-icon">🔗</div>
              <div class="method-info">
                <div class="method-title">Import from URL</div>
                <div class="method-desc">Import a template from a web URL</div>
              </div>
            </div>
          </el-radio>

          <el-radio value="text" class="method-option">
            <div class="method-content">
              <div class="method-icon">📝</div>
              <div class="method-info">
                <div class="method-title">Import from Text</div>
                <div class="method-desc">Paste template JSON/YAML content directly</div>
              </div>
            </div>
          </el-radio>
        </el-radio-group>
      </div>

      <!-- File Upload -->
      <div v-if="importMethod === 'file'" class="import-section">
        <div class="section-title">
          <h4>Select Template File</h4>
          <p>Upload a JSON or YAML template file</p>
        </div>

        <el-upload
          ref="uploadRef"
          class="upload-area"
          drag
          :auto-upload="false"
          :show-file-list="true"
          :limit="1"
          accept=".json,.yaml,.yml"
          @change="handleFileChange"
          @exceed="handleFileExceed"
        >
          <div class="upload-content">
            <el-icon class="upload-icon"><UploadFilled /></el-icon>
            <div class="upload-text">
              <p>Drop template file here or <em>click to browse</em></p>
              <p class="upload-hint">Supports JSON and YAML formats</p>
            </div>
          </div>
        </el-upload>

        <div v-if="selectedFile" class="file-info">
          <div class="file-details">
            <span class="file-name">{{ selectedFile.name }}</span>
            <span class="file-size">({{ formatFileSize(selectedFile.size) }})</span>
          </div>
          <el-button type="danger" size="small" @click="clearFile">
            Remove
          </el-button>
        </div>
      </div>

      <!-- URL Import -->
      <div v-if="importMethod === 'url'" class="import-section">
        <div class="section-title">
          <h4>Template URL</h4>
          <p>Enter the URL of the template file</p>
        </div>

        <el-form ref="urlFormRef" :model="urlForm" :rules="urlRules">
          <el-form-item prop="url">
            <el-input
              v-model="urlForm.url"
              placeholder="https://example.com/template.json"
              size="large"
              clearable
            >
              <template #prepend>
                <el-icon><Link /></el-icon>
              </template>
            </el-input>
          </el-form-item>

          <el-form-item>
            <el-checkbox v-model="urlForm.validateSsl">
              Validate SSL certificate
            </el-checkbox>
          </el-form-item>
        </el-form>

        <div v-if="urlPreview" class="url-preview">
          <h5>Template Preview:</h5>
          <div class="preview-info">
            <p><strong>Name:</strong> {{ urlPreview.name }}</p>
            <p><strong>Description:</strong> {{ urlPreview.description }}</p>
            <p><strong>Author:</strong> {{ urlPreview.author }}</p>
            <p><strong>Version:</strong> {{ urlPreview.version }}</p>
          </div>
        </div>
      </div>

      <!-- Text Import -->
      <div v-if="importMethod === 'text'" class="import-section">
        <div class="section-title">
          <h4>Template Content</h4>
          <p>Paste the template JSON/YAML content below</p>
        </div>

        <el-form ref="textFormRef" :model="textForm" :rules="textRules">
          <el-form-item prop="content">
            <el-input
              v-model="textForm.content"
              type="textarea"
              :rows="12"
              placeholder="Paste template JSON or YAML content here..."
              class="text-input"
            />
          </el-form-item>

          <div class="text-toolbar">
            <el-button @click="formatJSON" :disabled="!textForm.content">
              Format JSON
            </el-button>
            <el-button @click="validateJSON" :disabled="!textForm.content">
              Validate
            </el-button>
            <el-button @click="clearText">
              Clear
            </el-button>
          </div>
        </el-form>

        <div v-if="textValidation" class="validation-result">
          <div class="validation-header">
            <h5>Validation Result:</h5>
            <el-tag :type="textValidation.isValid ? 'success' : 'danger'">
              {{ textValidation.isValid ? 'Valid' : 'Invalid' }}
            </el-tag>
          </div>

          <div v-if="textValidation.isValid && textValidation.template" class="preview-info">
            <p><strong>Name:</strong> {{ textValidation.template.name }}</p>
            <p><strong>Description:</strong> {{ textValidation.template.description }}</p>
            <p><strong>Author:</strong> {{ textValidation.template.author }}</p>
          </div>

          <div v-else-if="!textValidation.isValid" class="validation-errors">
            <p><strong>Errors:</strong></p>
            <ul>
              <li v-for="error in textValidation.errors" :key="error">
                {{ error }}
              </li>
            </ul>
          </div>
        </div>
      </div>

      <!-- Import Options -->
      <div class="import-options">
        <div class="section-title">
          <h4>Import Options</h4>
        </div>

        <el-form :model="importOptions" label-width="150px">
          <el-form-item label="Handle Conflicts">
            <el-radio-group v-model="importOptions.conflictResolution">
              <el-radio value="skip">Skip existing templates</el-radio>
              <el-radio value="overwrite">Overwrite existing templates</el-radio>
              <el-radio value="rename">Create with new name</el-radio>
            </el-radio-group>
          </el-form-item>

          <el-form-item label="Import Mode">
            <el-radio-group v-model="importOptions.importMode">
              <el-radio value="validate">Validate only (dry run)</el-radio>
              <el-radio value="import">Import templates</el-radio>
            </el-radio-group>
          </el-form-item>

          <el-form-item>
            <el-checkbox v-model="importOptions.preserveId">
              Preserve original template IDs
            </el-checkbox>
          </el-form-item>

          <el-form-item>
            <el-checkbox v-model="importOptions.importPrivateData">
              Import private/sensitive data
            </el-checkbox>
          </el-form-item>
        </el-form>
      </div>
    </div>

    <template #footer>
      <div class="dialog-footer">
        <el-button @click="handleClose">Cancel</el-button>
        <el-button
          v-if="importMethod === 'url' && !urlPreview"
          @click="previewUrl"
          :loading="previewing"
        >
          Preview
        </el-button>
        <el-button
          type="primary"
          @click="handleImport"
          :loading="importing"
          :disabled="!canImport"
        >
          {{ importOptions.importMode === 'validate' ? 'Validate' : 'Import' }}
        </el-button>
      </div>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
import { ref, reactive, computed, watch } from 'vue'
import { ElMessage, ElForm } from 'element-plus'
import { UploadFilled, Link } from '@element-plus/icons-vue'
import { strategyTemplateService } from '@/services/strategyTemplateService'

interface Props {
  modelValue: boolean
}

interface Emits {
  (e: 'update:modelValue', value: boolean): void
  (e: 'imported', result: any): void
}

const props = defineProps<Props>()
const emit = defineEmits<Emits>()

const visible = computed({
  get: () => props.modelValue,
  set: (value: boolean) => emit('update:modelValue', value)
})

// State
const importMethod = ref('file')
const importing = ref(false)
const previewing = ref(false)
const selectedFile = ref<File | null>(null)
const urlPreview = ref<any>(null)
const textValidation = ref<any>(null)

// Form refs
const uploadRef = ref()
const urlFormRef = ref<InstanceType<typeof ElForm>>()
const textFormRef = ref<InstanceType<typeof ElForm>>()

// Form data
const urlForm = reactive({
  url: '',
  validateSsl: true
})

const textForm = reactive({
  content: ''
})

const importOptions = reactive({
  conflictResolution: 'skip',
  importMode: 'import',
  preserveId: false,
  importPrivateData: false
})

// Validation rules
const urlRules = {
  url: [
    { required: true, message: 'Please enter a URL', trigger: 'blur' },
    { type: 'url', message: 'Please enter a valid URL', trigger: 'blur' }
  ]
}

const textRules = {
  content: [
    { required: true, message: 'Please enter template content', trigger: 'blur' },
    { min: 10, message: 'Content must be at least 10 characters', trigger: 'blur' }
  ]
}

// Computed
const canImport = computed(() => {
  switch (importMethod.value) {
    case 'file':
      return selectedFile.value !== null
    case 'url':
      return urlForm.url && urlPreview.value
    case 'text':
      return textForm.content && textValidation.value?.isValid
    default:
      return false
  }
})

// Methods
const formatFileSize = (bytes: number): string => {
  if (bytes === 0) return '0 Bytes'
  const k = 1024
  const sizes = ['Bytes', 'KB', 'MB', 'GB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i]
}

const handleFileChange = (file: any) => {
  selectedFile.value = file.raw
}

const handleFileExceed = () => {
  ElMessage.warning('Only one file can be uploaded at a time')
}

const clearFile = () => {
  selectedFile.value = null
  uploadRef.value?.clearFiles()
}

const previewUrl = async () => {
  try {
    previewing.value = true

    const response = await fetch(urlForm.url, {
      method: 'HEAD',
      mode: 'cors'
    })

    if (!response.ok) {
      throw new Error(`HTTP ${response.status}: ${response.statusText}`)
    }

    // For a real implementation, we would fetch the actual content
    // For now, just simulate a preview
    urlPreview.value = {
      name: 'Template from URL',
      description: 'Template description would be extracted from the file',
      author: 'Unknown',
      version: '1.0.0'
    }

    ElMessage.success('Template preview loaded')
  } catch (error) {
    console.error('Failed to preview URL:', error)
    ElMessage.error('Failed to load template from URL')
  } finally {
    previewing.value = false
  }
}

const formatJSON = () => {
  try {
    const parsed = JSON.parse(textForm.content)
    textForm.content = JSON.stringify(parsed, null, 2)
    ElMessage.success('JSON formatted successfully')
  } catch (error) {
    ElMessage.error('Invalid JSON format')
  }
}

const validateJSON = () => {
  try {
    const parsed = JSON.parse(textForm.content)

    // Basic structure validation
    if (!parsed.name || !parsed.description) {
      throw new Error('Template must have name and description')
    }

    textValidation.value = {
      isValid: true,
      template: parsed,
      errors: []
    }

    ElMessage.success('Template content is valid')
  } catch (error) {
    textValidation.value = {
      isValid: false,
      template: null,
      errors: [error instanceof Error ? error.message : 'Unknown validation error']
    }

    ElMessage.error('Template content is invalid')
  }
}

const clearText = () => {
  textForm.content = ''
  textValidation.value = null
}

const handleImport = async () => {
  try {
    importing.value = true

    let result

    switch (importMethod.value) {
      case 'file':
        if (selectedFile.value) {
          result = await strategyTemplateService.importTemplate(selectedFile.value)
        }
        break

      case 'url':
        // For URL import, we would first download the file then import
        ElMessage.info('URL import not yet implemented')
        return

      case 'text':
        // For text import, create a blob from the content
        if (textForm.content) {
          const blob = new Blob([textForm.content], { type: 'application/json' })
          const file = new File([blob], 'template.json', { type: 'application/json' })
          result = await strategyTemplateService.importTemplate(file)
        }
        break

      default:
        throw new Error('Unknown import method')
    }

    if (result) {
      if (result.success) {
        if (importOptions.importMode === 'validate') {
          ElMessage.success('Template validation completed successfully')
        } else {
          ElMessage.success(`Template imported successfully${result.template ? ': ' + result.template.name : ''}`)
          emit('imported', result)
        }
        visible.value = false
      } else {
        const errorMsg = Array.isArray(result.errors)
          ? result.errors.join(', ')
          : result.error || 'Import failed'

        ElMessage.error(errorMsg)

        // Show validation warnings if any
        if (result.warnings && result.warnings.length > 0) {
          ElMessage.warning(`Warnings: ${result.warnings.join(', ')}`)
        }
      }
    }
  } catch (error) {
    console.error('Import failed:', error)
    ElMessage.error('Import failed: ' + (error instanceof Error ? error.message : 'Unknown error'))
  } finally {
    importing.value = false
  }
}

const handleClose = () => {
  visible.value = false
}

const resetForm = () => {
  importMethod.value = 'file'
  selectedFile.value = null
  urlForm.url = ''
  urlForm.validateSsl = true
  urlPreview.value = null
  textForm.content = ''
  textValidation.value = null

  Object.assign(importOptions, {
    conflictResolution: 'skip',
    importMode: 'import',
    preserveId: false,
    importPrivateData: false
  })

  // Clear file upload
  uploadRef.value?.clearFiles()
}

// Watch for dialog close
watch(visible, (val) => {
  if (!val) {
    resetForm()
  }
})
</script>

<style lang="scss" scoped>
@import '../styles/utilities.scss';
.import-template-dialog {
  :deep(.el-dialog__body) {
    padding: var(--spacing-lg);
    max-height: 80vh;
    overflow-y: auto;
  }
}

.import-template {
  .import-methods {
    margin-bottom: var(--spacing-xl);

    .method-group {
      display: flex;
      flex-direction: column;
      gap: var(--spacing-md);

      @include respond-to('tablet') {
        flex-direction: row;
      }

      .method-option {
        flex: 1;
        margin: 0;

        :deep(.el-radio__input) {
          display: none;
        }

        :deep(.el-radio__label) {
          width: 100%;
          padding: 0;
        }

        .method-content {
          display: flex;
          align-items: center;
          gap: var(--spacing-md);
          padding: var(--spacing-lg);
          border: 2px solid var(--border-light);
          border-radius: var(--border-radius-md);
          cursor: pointer;
          transition: all 0.3s ease;

          &:hover {
            border-color: var(--primary-color);
            background: var(--bg-secondary);
          }

          .method-icon {
            font-size: 2rem;
            width: 60px;
            height: 60px;
            display: flex;
            align-items: center;
            justify-content: center;
            background: var(--bg-secondary);
            border-radius: var(--border-radius-md);
          }

          .method-info {
            flex: 1;

            .method-title {
              font-weight: var(--font-weight-semibold);
              color: var(--text-primary);
              margin-bottom: var(--spacing-xs);
            }

            .method-desc {
              font-size: var(--font-size-sm);
              color: var(--text-secondary);
            }
          }
        }

        &.is-checked {
          .method-content {
            border-color: var(--primary-color);
            background: var(--primary-color-light);
            color: var(--primary-color);

            .method-icon {
              background: var(--primary-color);
            }
          }
        }
      }
    }
  }

  .import-section {
    margin-bottom: var(--spacing-xl);

    .section-title {
      margin-bottom: var(--spacing-lg);

      h4 {
        margin: 0 0 var(--spacing-xs) 0;
        font-size: var(--font-size-lg);
        color: var(--text-primary);
      }

      p {
        margin: 0;
        color: var(--text-secondary);
        font-size: var(--font-size-sm);
      }
    }
  }

  .upload-area {
    width: 100%;

    :deep(.el-upload-dragger) {
      width: 100%;
      padding: var(--spacing-2xl);
      border: 2px dashed var(--border-light);
      border-radius: var(--border-radius-md);
      background: var(--bg-secondary);
      transition: all 0.3s ease;

      &:hover {
        border-color: var(--primary-color);
        background: var(--primary-color-light);
      }

      .upload-content {
        text-align: center;

        .upload-icon {
          font-size: 3rem;
          color: var(--text-secondary);
          margin-bottom: var(--spacing-md);
        }

        .upload-text {
          p {
            margin: var(--spacing-xs) 0;

            &.upload-hint {
              font-size: var(--font-size-sm);
              color: var(--text-secondary);
            }
          }

            em {
              color: var(--primary-color);
              font-style: normal;
              font-weight: var(--font-weight-medium);
          }
        }
      }
    }
  }

  .file-info {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--spacing-md);
    background: var(--bg-secondary);
    border-radius: var(--border-radius-md);
    margin-top: var(--spacing-md);

    .file-details {
      .file-name {
        font-weight: var(--font-weight-medium);
        color: var(--text-primary);
      }

      .file-size {
        color: var(--text-secondary);
        font-size: var(--font-size-sm);
      }
    }
  }

  .text-input {
    :deep(.el-textarea__inner) {
      font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
      font-size: var(--font-size-sm);
      line-height: 1.5;
    }
  }

  .text-toolbar {
    display: flex;
    gap: var(--spacing-sm);
    margin-top: var(--spacing-sm);
  }

  .validation-result {
    margin-top: var(--spacing-lg);
    padding: var(--spacing-md);
    background: var(--bg-secondary);
    border-radius: var(--border-radius-md);

    .validation-header {
      display: flex;
      justify-content: space-between;
      align-items: center;
      margin-bottom: var(--spacing-md);

      h5 {
        margin: 0;
        font-size: var(--font-size-base);
        color: var(--text-primary);
      }
    }

    .preview-info {
      p {
        margin: var(--spacing-xs) 0;

        strong {
          color: var(--text-primary);
        }
      }
    }

    .validation-errors {
      ul {
        margin: var(--spacing-xs) 0;
        padding-left: var(--spacing-lg);
        color: var(--danger-color);
      }
    }
  }

  .url-preview {
    margin-top: var(--spacing-lg);
    padding: var(--spacing-md);
    background: var(--bg-secondary);
    border-radius: var(--border-radius-md);

    h5 {
      margin: 0 0 var(--spacing-sm) 0;
      font-size: var(--font-size-base);
      color: var(--text-primary);
    }

    .preview-info {
      p {
        margin: var(--spacing-xs) 0;

        strong {
          color: var(--text-primary);
        }
      }
    }
  }

  .import-options {
    padding: var(--spacing-lg);
    background: var(--bg-secondary);
    border-radius: var(--border-radius-md);

    .section-title {
      margin-bottom: var(--spacing-lg);

      h4 {
        margin: 0;
        font-size: var(--font-size-lg);
        color: var(--text-primary);
      }
    }
  }

  .dialog-footer {
    display: flex;
    justify-content: flex-end;
    gap: var(--spacing-sm);
  }
}
</style>