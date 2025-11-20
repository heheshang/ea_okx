<template>
  <div class="settings">
    <el-row :gutter="20">
      <el-col :span="24">
        <el-tabs v-model="activeTab" type="border-card">
          <!-- API Keys -->
          <el-tab-pane label="API Keys" name="api">
            <el-card shadow="never">
              <template #header>
                <div class="card-header">
                  <div class="header-title">
                    <el-icon><Key /></el-icon>
                    <span>OKX API Configuration</span>
                  </div>
                  <el-button type="primary" size="small" @click="showAddApiDialog = true" class="add-api-btn">
                    <el-icon><Plus /></el-icon>
                    <span class="btn-text">Add API Key</span>
                  </el-button>
                </div>
              </template>

              <el-alert
                title="Security Notice"
                type="warning"
                :closable="false"
                style="margin-bottom: 20px"
              >
                API keys are encrypted and stored securely. Never share your secret key with anyone.
              </el-alert>

              <el-table :data="apiKeys" style="width: 100%">
                <el-table-column prop="name" label="Name" width="150" />
                <el-table-column prop="apiKey" label="API Key" width="300">
                  <template #default="{ row }">
                    <span class="mono">{{ maskApiKey(row.apiKey) }}</span>
                  </template>
                </el-table-column>
                <el-table-column prop="environment" label="Environment" width="120">
                  <template #default="{ row }">
                    <el-tag :type="row.environment === 'Live' ? 'danger' : 'success'">
                      {{ row.environment }}
                    </el-tag>
                  </template>
                </el-table-column>
                <el-table-column prop="permissions" label="Permissions">
                  <template #default="{ row }">
                    <el-tag v-for="perm in row.permissions" :key="perm" size="small" style="margin-right: 5px">
                      {{ perm }}
                    </el-tag>
                  </template>
                </el-table-column>
                <el-table-column prop="createdAt" label="Created" width="180" />
                <el-table-column label="Actions" width="150" fixed="right">
                  <template #default="{ row }">
                    <el-button size="small" type="primary" text @click="testConnection(row.id)">
                      Test
                    </el-button>
                    <el-button size="small" type="danger" text @click="deleteApiKey(row.id)">
                      Delete
                    </el-button>
                  </template>
                </el-table-column>
              </el-table>
            </el-card>
          </el-tab-pane>

          <!-- System Settings -->
          <el-tab-pane label="System Settings" name="system">
            <el-card shadow="never">
              <el-form :model="systemSettings" :label-width="isMobile ? '180px' : '200px'" label-position="left">
                <el-divider content-position="left">
                  <el-icon><Setting /></el-icon>
                  <span>General</span>
                </el-divider>
                <el-form-item label="Theme">
                  <el-radio-group v-model="systemSettings.theme">
                    <el-radio label="Dark">Dark</el-radio>
                    <el-radio label="Light">Light</el-radio>
                    <el-radio label="Auto">Auto</el-radio>
                  </el-radio-group>
                </el-form-item>
                <el-form-item label="Language">
                  <el-select v-model="systemSettings.language" style="width: 200px">
                    <el-option label="English" value="en" />
                    <el-option label="中文" value="zh" />
                  </el-select>
                </el-form-item>
                <el-form-item label="Time Zone">
                  <el-select v-model="systemSettings.timezone" style="width: 200px">
                    <el-option label="UTC" value="UTC" />
                    <el-option label="UTC+8 (Shanghai)" value="Asia/Shanghai" />
                    <el-option label="EST" value="America/New_York" />
                  </el-select>
                </el-form-item>

                <el-divider content-position="left">
                  <el-icon><Connection /></el-icon>
                  <span>Trading</span>
                </el-divider>
                <el-form-item label="Auto-Reconnect">
                  <el-switch v-model="systemSettings.autoReconnect" />
                </el-form-item>
                <el-form-item label="Enable Paper Trading">
                  <el-switch v-model="systemSettings.paperTrading" />
                </el-form-item>
                <el-form-item label="Order Timeout (seconds)">
                  <el-input-number
                    v-model="systemSettings.orderTimeout"
                    :min="5"
                    :max="300"
                    controls-position="right"
                  />
                </el-form-item>
                <el-form-item label="Reconciliation Interval (seconds)">
                  <el-input-number
                    v-model="systemSettings.reconciliationInterval"
                    :min="10"
                    :max="300"
                    controls-position="right"
                  />
                </el-form-item>

                <el-divider content-position="left">
                  <el-icon><InfoFilled /></el-icon>
                  <span>Notifications</span>
                </el-divider>
                <el-form-item label="Enable Desktop Notifications">
                  <el-switch v-model="systemSettings.desktopNotifications" />
                </el-form-item>
                <el-form-item label="Email Notifications">
                  <el-switch v-model="systemSettings.emailNotifications" />
                </el-form-item>
                <el-form-item label="Alert Sound">
                  <el-switch v-model="systemSettings.alertSound" />
                </el-form-item>

                <el-form-item>
                  <el-space>
                    <el-button type="primary" @click="saveSystemSettings" class="settings-action-btn">
                      <el-icon><Setting /></el-icon>
                      <span class="btn-text">Save Settings</span>
                    </el-button>
                    <el-button @click="resetSystemSettings" class="settings-action-btn">
                      <span class="btn-text">Reset to Default</span>
                    </el-button>
                  </el-space>
                </el-form-item>
              </el-form>
            </el-card>
          </el-tab-pane>

          <!-- Database -->
          <el-tab-pane label="Database" name="database">
            <el-card shadow="never">
              <template #header>
                <div class="card-header">
                  <el-icon><Connection /></el-icon>
                  <span>Database Configuration</span>
                </div>
              </template>
              <el-form :model="databaseConfig" :label-width="isMobile ? '140px' : '200px'" label-position="left">
                <el-form-item label="PostgreSQL Host">
                  <el-input v-model="databaseConfig.host" placeholder="localhost" />
                </el-form-item>
                <el-form-item label="Port">
                  <el-input-number
                    v-model="databaseConfig.port"
                    :min="1024"
                    :max="65535"
                    controls-position="right"
                  />
                </el-form-item>
                <el-form-item label="Database Name">
                  <el-input v-model="databaseConfig.database" placeholder="ea_okx" />
                </el-form-item>
                <el-form-item label="Username">
                  <el-input v-model="databaseConfig.username" placeholder="postgres" />
                </el-form-item>
                <el-form-item label="Password">
                  <el-input
                    v-model="databaseConfig.password"
                    type="password"
                    show-password
                    placeholder="Enter password"
                  />
                </el-form-item>

                <el-divider content-position="left">Redis Cache</el-divider>
                <el-form-item label="Redis Host">
                  <el-input v-model="databaseConfig.redisHost" placeholder="localhost" />
                </el-form-item>
                <el-form-item label="Redis Port">
                  <el-input-number
                    v-model="databaseConfig.redisPort"
                    :min="1024"
                    :max="65535"
                    controls-position="right"
                  />
                </el-form-item>

                <el-form-item>
                  <el-space>
                    <el-button type="primary" @click="testDatabaseConnection" class="db-action-btn">
                      <el-icon><Connection /></el-icon>
                      <span class="btn-text">Test Connection</span>
                    </el-button>
                    <el-button type="primary" @click="saveDatabaseConfig" class="db-action-btn">
                      <span class="btn-text">Save Configuration</span>
                    </el-button>
                  </el-space>
                </el-form-item>
              </el-form>
            </el-card>
          </el-tab-pane>

          <!-- About -->
          <el-tab-pane label="About" name="about">
            <el-card shadow="never">
              <el-descriptions :column="1" border>
                <el-descriptions-item label="Application Name">
                  EA OKX Quantitative Trading System
                </el-descriptions-item>
                <el-descriptions-item label="Version">0.1.0</el-descriptions-item>
                <el-descriptions-item label="Build Date">2024-11-20</el-descriptions-item>
                <el-descriptions-item label="Rust Version">1.75+</el-descriptions-item>
                <el-descriptions-item label="Tauri Version">2.0</el-descriptions-item>
                <el-descriptions-item label="License">MIT</el-descriptions-item>
                <el-descriptions-item label="Repository">
                  <el-link href="https://github.com/ea-okx/quantitative-trading" target="_blank">
                    GitHub Repository
                  </el-link>
                </el-descriptions-item>
              </el-descriptions>

              <el-divider />

              <div style="text-align: center; padding: 20px">
                <el-button type="primary" @click="checkForUpdates">Check for Updates</el-button>
                <el-button @click="viewLogs">View Logs</el-button>
              </div>

              <el-alert
                title="Disclaimer"
                type="warning"
                :closable="false"
                style="margin-top: 20px"
              >
                This software is for educational and research purposes. Trading cryptocurrencies
                involves substantial risk of loss. Past performance does not guarantee future
                results.
              </el-alert>
            </el-card>
          </el-tab-pane>
        </el-tabs>
      </el-col>
    </el-row>

    <!-- Add API Key Dialog -->
    <el-dialog v-model="showAddApiDialog" title="Add API Key" :width="isMobile ? '90%' : '600px'">
      <el-form :model="newApiKey" label-width="120px">
        <el-form-item label="Name">
          <el-input v-model="newApiKey.name" placeholder="e.g., Main Trading Account" />
        </el-form-item>
        <el-form-item label="Environment">
          <el-radio-group v-model="newApiKey.environment">
            <el-radio label="Testnet">Testnet (Recommended)</el-radio>
            <el-radio label="Live">Live Trading</el-radio>
          </el-radio-group>
        </el-form-item>
        <el-form-item label="API Key">
          <el-input v-model="newApiKey.apiKey" placeholder="Enter API key" />
        </el-form-item>
        <el-form-item label="Secret Key">
          <el-input
            v-model="newApiKey.secretKey"
            type="password"
            show-password
            placeholder="Enter secret key"
          />
        </el-form-item>
        <el-form-item label="Passphrase">
          <el-input
            v-model="newApiKey.passphrase"
            type="password"
            show-password
            placeholder="Enter passphrase"
          />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="showAddApiDialog = false">Cancel</el-button>
        <el-button type="primary" @click="addApiKey">Add API Key</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { ElMessage, ElMessageBox } from 'element-plus'
import { Plus, Key, Setting, Connection, InfoFilled } from '@element-plus/icons-vue'
import { useResponsive } from '@/composables/useResponsive'

const { isMobile } = useResponsive()

// Define component name for keep-alive
defineOptions({
  name: 'Settings'
})

interface ApiKey {
  id: string
  name: string
  apiKey: string
  environment: string
  permissions: string[]
  createdAt: string
}

const activeTab = ref('api')
const showAddApiDialog = ref(false)

const apiKeys = ref<ApiKey[]>([
  {
    id: '1',
    name: 'Main Account',
    apiKey: 'a1b2c3d4-e5f6-7890-abcd-1234567890ab',
    environment: 'Testnet',
    permissions: ['Read', 'Trade'],
    createdAt: '2024-11-01 10:30:00',
  },
])

const systemSettings = ref({
  theme: 'Dark',
  language: 'en',
  timezone: 'UTC',
  autoReconnect: true,
  paperTrading: true,
  orderTimeout: 30,
  reconciliationInterval: 30,
  desktopNotifications: true,
  emailNotifications: false,
  alertSound: true,
})

const databaseConfig = ref({
  host: 'localhost',
  port: 5432,
  database: 'ea_okx',
  username: 'postgres',
  password: '',
  redisHost: 'localhost',
  redisPort: 6379,
})

const newApiKey = ref({
  name: '',
  environment: 'Testnet',
  apiKey: '',
  secretKey: '',
  passphrase: '',
})

const maskApiKey = (key: string) => {
  if (key.length <= 8) return key
  return key.substring(0, 8) + '...' + key.substring(key.length - 4)
}

const addApiKey = async () => {
  if (!newApiKey.value.name || !newApiKey.value.apiKey || !newApiKey.value.secretKey) {
    ElMessage.warning('Please fill in all required fields')
    return
  }

  try {
    await invoke('add_api_key', {
      name: newApiKey.value.name,
      apiKey: newApiKey.value.apiKey,
      secretKey: newApiKey.value.secretKey,
      passphrase: newApiKey.value.passphrase,
      environment: newApiKey.value.environment,
    })
    ElMessage.success('API key added successfully')
    showAddApiDialog.value = false
    // Reset form
    newApiKey.value = {
      name: '',
      environment: 'Testnet',
      apiKey: '',
      secretKey: '',
      passphrase: '',
    }
    // Reload API keys
  } catch (error) {
    console.error('Failed to add API key:', error)
    ElMessage.error('Failed to add API key')
  }
}

const testConnection = async (id: string) => {
  try {
    const result = await invoke('test_api_connection', { apiKeyId: id })
    ElMessage.success('Connection test successful')
    console.log('Test result:', result)
  } catch (error) {
    console.error('Connection test failed:', error)
    ElMessage.error('Connection test failed')
  }
}

const deleteApiKey = async (id: string) => {
  try {
    await ElMessageBox.confirm('Are you sure to delete this API key?', 'Warning', {
      type: 'warning',
    })
    await invoke('delete_api_key', { apiKeyId: id })
    apiKeys.value = apiKeys.value.filter((k) => k.id !== id)
    ElMessage.success('API key deleted')
  } catch (error) {
    if (error !== 'cancel') {
      console.error('Failed to delete API key:', error)
      ElMessage.error('Failed to delete API key')
    }
  }
}

const saveSystemSettings = async () => {
  try {
    await invoke('save_system_settings', { settings: systemSettings.value })
    ElMessage.success('Settings saved successfully')
  } catch (error) {
    console.error('Failed to save settings:', error)
    ElMessage.error('Failed to save settings')
  }
}

const resetSystemSettings = () => {
  systemSettings.value = {
    theme: 'Dark',
    language: 'en',
    timezone: 'UTC',
    autoReconnect: true,
    paperTrading: true,
    orderTimeout: 30,
    reconciliationInterval: 30,
    desktopNotifications: true,
    emailNotifications: false,
    alertSound: true,
  }
  ElMessage.info('Settings reset to default')
}

const testDatabaseConnection = async () => {
  try {
    await invoke('test_database_connection', { config: databaseConfig.value })
    ElMessage.success('Database connection successful')
  } catch (error) {
    console.error('Database connection failed:', error)
    ElMessage.error('Database connection failed')
  }
}

const saveDatabaseConfig = async () => {
  try {
    await invoke('save_database_config', { config: databaseConfig.value })
    ElMessage.success('Database configuration saved')
  } catch (error) {
    console.error('Failed to save database configuration:', error)
    ElMessage.error('Failed to save database configuration')
  }
}

const checkForUpdates = () => {
  ElMessage.info('Checking for updates...')
  // Implement update check
}

const viewLogs = () => {
  // Open logs directory or show logs in dialog
  ElMessage.info('Opening logs...')
}
</script>

<style lang="scss" scoped>
@import '@/styles/variables.scss';
@import '@/styles/utilities.scss';

.settings {
  .card-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    flex-wrap: wrap;
    gap: $spacing-md;
    
    .header-title {
      display: flex;
      align-items: center;
      gap: $spacing-sm;
    }
  }
  
  .add-api-btn,
  .settings-action-btn,
  .db-action-btn {
    min-height: $touch-target-min;
    
    @include mobile {
      min-height: $touch-target-comfortable;
    }
    
    @media (max-width: 480px) {
      .btn-text {
        display: none;
      }
    }
  }

  .mono {
    font-family: 'Courier New', monospace;
    
    @include mobile {
      font-size: $font-size-xs;
    }
  }
  
  :deep(.el-form-item__label) {
    @include mobile {
      font-size: $font-size-sm;
    }
  }
  
  :deep(.el-divider__text) {
    background-color: var(--bg-secondary);
    color: var(--text-secondary);
    display: flex;
    align-items: center;
    gap: $spacing-sm;
    
    .el-icon {
      font-size: $font-size-base;
    }
  }

  :deep(.el-tabs--border-card) {
    background-color: var(--bg-secondary);
    border: 1px solid var(--border-color);
  }

  :deep(.el-tabs__header) {
    background-color: var(--bg-tertiary);
  }

  :deep(.el-tabs__item) {
    color: var(--text-secondary);

    &.is-active {
      color: var(--text-primary);
    }
  }

  :deep(.el-card) {
    background-color: var(--bg-secondary);
    border: none;
    color: var(--text-primary);
  }

  :deep(.el-form-item__label) {
    color: var(--text-secondary);
  }

  :deep(.el-table) {
    background-color: var(--bg-secondary);
    color: var(--text-primary);

    th {
      background-color: var(--bg-tertiary);
      color: var(--text-secondary);
    }

    tr {
      background-color: var(--bg-secondary);

      &:hover > td {
        background-color: var(--bg-tertiary);
      }
    }
  }

  :deep(.el-descriptions) {
    --el-descriptions-item-bordered-label-background: var(--bg-tertiary);
  }
}
</style>
