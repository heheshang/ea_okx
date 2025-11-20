<template>
  <header class="responsive-header" :class="headerClasses">
    <!-- 渐进式背景 -->
    <div
      class="header-backdrop"
      :style="{ opacity: backdropOpacity, backgroundColor: themeBackgroundColor }"
    />

    <!-- Desktop Header -->
    <div class="desktop-header" v-if="!isMobile">
      <div class="header-container">
        <!-- Logo Area with Animation -->
        <div class="logo-section" @click="handleLogoClick">
          <div class="logo-icon" :class="{ 'logo-pulse': isLogoHovered }">
            <span class="logo-text">{{ brandName.charAt(0) }}</span>
            <div v-if="notifications.length > 0" class="logo-notification-badge">
              {{ notifications.length > 99 ? '99+' : notifications.length }}
            </div>
          </div>
          <transition name="brand-name-fade">
            <span class="brand-name" v-if="!isScrolled || showFullTitle">{{ brandName }}</span>
          </transition>
        </div>

        <!-- Desktop Navigation with Active Indicators -->
        <nav class="nav-section">
          <div
            v-for="item in primaryNavItems"
            :key="item.path"
            class="nav-item"
            :class="{
              active: isActive(item.path),
              'has-notification': item.badge && item.badge > 0
            }"
            @click="handleNavClick(item.path)"
            @mouseenter="handleNavHover(item)"
            @mouseleave="handleNavLeave"
          >
            <el-icon :size="20">
              <component :is="item.icon" />
            </el-icon>
            <span class="nav-label">{{ item.label }}</span>
            <el-badge
              v-if="item.badge && item.badge > 0"
              :value="item.badge"
              :max="99"
              class="nav-badge"
            />
            <!-- 激活指示器 -->
            <div class="nav-indicator"></div>
          </div>

          <!-- 更多菜单（当导航项过多时） -->
          <el-dropdown
            v-if="secondaryNavItems.length > 0"
            trigger="hover"
            @command="handleNavClick"
            class="nav-more-dropdown"
          >
            <div class="nav-item nav-more">
              <el-icon :size="20">
                <MoreFilled />
              </el-icon>
              <span class="nav-label">更多</span>
            </div>
            <template #dropdown>
              <el-dropdown-menu>
                <el-dropdown-item
                  v-for="item in secondaryNavItems"
                  :key="item.path"
                  :command="item.path"
                  :class="{ active: isActive(item.path) }"
                >
                  <el-icon>
                    <component :is="item.icon" />
                  </el-icon>
                  {{ item.label }}
                  <el-badge
                    v-if="item.badge && item.badge > 0"
                    :value="item.badge"
                    :max="99"
                    class="dropdown-badge"
                  />
                </el-dropdown-item>
              </el-dropdown-menu>
            </template>
          </el-dropdown>
        </nav>

        <!-- User Actions with Status -->
        <div class="user-section">
          <!-- 搜索框 -->
          <transition name="search-expand">
            <div v-if="showSearchBar || searchExpanded" class="search-container">
              <el-input
                v-model="searchQuery"
                :placeholder="searchPlaceholder"
                class="search-input"
                size="default"
                clearable
                @keyup.enter="handleSearch"
                @blur="handleSearchBlur"
                ref="searchInputRef"
              >
                <template #prefix>
                  <el-icon>
                    <Search />
                  </el-icon>
                </template>
              </el-input>
            </div>
          </transition>

          <!-- 操作按钮 -->
          <div class="action-buttons">
            <!-- 搜索按钮 -->
            <el-tooltip :content="searchTooltip" placement="bottom">
              <el-icon
                class="action-icon search-toggle"
                :class="{ active: searchExpanded }"
                @click="toggleSearch"
              >
                <Search />
              </el-icon>
            </el-tooltip>

            <!-- 通知 -->
            <el-dropdown
              trigger="click"
              @command="handleNotificationAction"
              class="notification-dropdown"
            >
              <el-icon class="action-icon notification-trigger">
                <el-badge
                  :value="unreadNotifications"
                  :hidden="unreadNotifications === 0"
                  class="notification-badge"
                >
                  <Bell />
                </el-badge>
              </el-icon>
              <template #dropdown>
                <el-dropdown-menu class="notification-menu">
                  <div class="notification-header">
                    <span>通知</span>
                    <el-button
                      v-if="notifications.length > 0"
                      text
                      size="small"
                      @click="markAllAsRead"
                    >
                      全部已读
                    </el-button>
                  </div>
                  <el-divider />
                  <div
                    v-for="notification in notifications.slice(0, 5)"
                    :key="notification.id"
                    class="notification-item"
                    :class="{ unread: !notification.read }"
                  >
                    <div class="notification-content">
                      <div class="notification-title">{{ notification.title }}</div>
                      <div class="notification-desc">{{ notification.description }}</div>
                      <div class="notification-time">{{ formatTime(notification.time) }}</div>
                    </div>
                  </div>
                  <el-empty
                    v-if="notifications.length === 0"
                    description="暂无通知"
                    :image-size="60"
                  />
                </el-dropdown-menu>
              </template>
            </el-dropdown>

            <!-- 主题切换 -->
            <el-tooltip :content="themeTooltip" placement="bottom">
              <el-icon
                class="action-icon theme-switcher"
                :class="{ rotating: isThemeChanging }"
                @click="handleThemeToggle"
              >
                <Sunny v-if="isDark" />
                <Moon v-else />
              </el-icon>
            </el-tooltip>

            <!-- 用户菜单 -->
            <el-dropdown trigger="click" @command="handleUserAction" class="user-dropdown">
              <div class="user-avatar">
                <el-avatar :size="32" :src="userAvatar">
                  {{ userName.charAt(0).toUpperCase() }}
                </el-avatar>
              </div>
              <template #dropdown>
                <el-dropdown-menu>
                  <el-dropdown-item command="profile">
                    <el-icon>
                      <User />
                    </el-icon>
                    个人资料
                  </el-dropdown-item>
                  <el-dropdown-item command="settings">
                    <el-icon>
                      <Setting />
                    </el-icon>
                    设置
                  </el-dropdown-item>
                  <el-divider />
                  <el-dropdown-item command="logout" divided>
                    <el-icon>
                      <SwitchButton />
                    </el-icon>
                    退出登录
                  </el-dropdown-item>
                </el-dropdown-menu>
              </template>
            </el-dropdown>
          </div>
        </div>
      </div>
    </div>

    <!-- Mobile Header -->
    <div class="mobile-header" v-if="isMobile">
      <!-- Mobile Top Bar -->
      <div class="mobile-top-bar" :class="{ 'scrolled': isScrolled }">
        <div class="mobile-left">
          <!-- 汉堡菜单 -->
          <el-icon class="action-icon mobile-menu" @click="toggleMobileMenu">
            <Menu v-if="!mobileMenuOpen" />
            <Close v-else />
          </el-icon>

          <!-- Logo和标题 -->
          <div class="mobile-logo" @click="handleLogoClick">
            <div class="logo-icon mobile-icon">
              <span class="logo-text">{{ brandName.charAt(0) }}</span>
            </div>
            <transition name="mobile-title">
              <span class="brand-name mobile" v-if="showMobileTitle">{{ mobileTitle }}</span>
            </transition>
          </div>
        </div>

        <div class="mobile-actions">
          <!-- 搜索 -->
          <el-icon class="action-icon" @click="toggleMobileSearch">
            <Search />
          </el-icon>

          <!-- 通知 -->
          <el-badge
            :value="unreadNotifications"
            :hidden="unreadNotifications === 0"
            class="notification-badge"
          >
            <el-icon class="action-icon" @click="showMobileNotifications">
              <Bell />
            </el-icon>
          </el-badge>
        </div>
      </div>

      <!-- Mobile Search Bar -->
      <transition name="mobile-search">
        <div v-if="showMobileSearchBar" class="mobile-search-bar">
          <el-input
            v-model="searchQuery"
            :placeholder="mobileSearchPlaceholder"
            class="search-input"
            size="large"
            clearable
            @keyup.enter="handleSearch"
            ref="mobileSearchInputRef"
            autofocus
          >
            <template #prefix>
              <el-icon>
                <Search />
              </el-icon>
            </template>
          </el-input>
        </div>
      </transition>

      <!-- Mobile Slide Menu -->
      <transition name="mobile-menu">
        <div v-if="mobileMenuOpen" class="mobile-slide-menu">
          <div class="menu-header">
            <div class="user-info">
              <el-avatar :size="40" :src="userAvatar">
                {{ userName.charAt(0).toUpperCase() }}
              </el-avatar>
              <div class="user-details">
                <div class="user-name">{{ userName }}</div>
                <div class="user-status">{{ userStatus }}</div>
              </div>
            </div>
            <el-icon class="menu-close" @click="toggleMobileMenu">
              <Close />
            </el-icon>
          </div>

          <div class="menu-items">
            <div
              v-for="item in navItems"
              :key="item.path"
              class="mobile-menu-item"
              :class="{ active: isActive(item.path) }"
              @click="handleMobileNavClick(item.path)"
            >
              <el-icon :size="20">
                <component :is="item.icon" />
              </el-icon>
              <span class="menu-label">{{ item.label }}</span>
              <div class="menu-right">
                <el-badge
                  v-if="item.badge && item.badge > 0"
                  :value="item.badge"
                  :max="99"
                  class="menu-badge"
                />
              </div>
            </div>
          </div>

          <div class="menu-footer">
            <div class="menu-item theme-toggle" @click="handleThemeToggle">
              <el-icon :size="20">
                <Sunny v-if="isDark" />
                <Moon v-else />
              </el-icon>
              <span>{{ isDark ? '浅色模式' : '深色模式' }}</span>
            </div>
            <div class="menu-item" @click="handleUserAction('logout')">
              <el-icon :size="20">
                <SwitchButton />
              </el-icon>
              <span>退出登录</span>
            </div>
          </div>
        </div>
      </transition>
    </div>

    <!-- Mobile Bottom Tab Navigation -->
    <div class="mobile-tab-bar" :class="{ 'menu-open': mobileMenuOpen }">
      <div
        v-for="item in primaryNavItems"
        :key="item.path"
        class="tab-item"
        :class="{ active: isActive(item.path) }"
        @click="handleNavClick(item.path)"
      >
        <div class="tab-icon">
          <el-icon :size="22">
            <component :is="item.icon" />
          </el-icon>
          <el-badge
            v-if="item.badge && item.badge > 0"
            :value="item.badge"
            :max="9"
            class="tab-badge"
          />
        </div>
        <span class="tab-label">{{ item.label }}</span>
        <div v-if="isActive(item.path)" class="tab-indicator"></div>
      </div>
    </div>

    <!-- Mobile Backdrop -->
    <div
      v-if="isMobile && (mobileMenuOpen || showMobileSearchBar)"
      class="mobile-backdrop"
      @click="closeMobileOverlays"
    />
  </header>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch, nextTick } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useResponsive } from '@/composables/useResponsive'
import { useTheme } from '@/composables/useTheme'
import dayjs from 'dayjs'
import relativeTime from 'dayjs/plugin/relativeTime'

// Icons
import { Search, Sunny, Moon, Bell, MoreFilled, Menu, Close, User, Setting, SwitchButton } from '@element-plus/icons-vue'
import { ElMessage, ElNotification } from 'element-plus'

dayjs.extend(relativeTime)

interface NavItem {
  label: string
  path: string
  icon: any
  badge?: number
}

interface Notification {
  id: string
  title: string
  description: string
  time: Date
  read: boolean
  type: 'info' | 'success' | 'warning' | 'error'
}

interface Props {
  brandName?: string
  navItems?: NavItem[]
  theme?: {
    primaryColor?: string
  }
  user?: {
    name?: string
    avatar?: string
    status?: string
  }
  notifications?: Notification[]
  showSearch?: boolean
  showNotifications?: boolean
  stickyHeader?: boolean
  collapseOnScroll?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  brandName: 'EA OKX',
  navItems: () => [
    { label: '仪表板', path: '/dashboard', icon: 'DataAnalysis', badge: 0 },
    { label: '策略', path: '/strategies', icon: 'TrendCharts', badge: 0 },
    { label: '回测', path: '/backtest', icon: 'Timer', badge: 0 },
    { label: '交易', path: '/trading', icon: 'Money', badge: 2 },
    { label: '风控', path: '/risk', icon: 'Warning', badge: 1 },
    { label: '设置', path: '/settings', icon: 'Setting', badge: 0 },
  ],
  theme: () => ({ primaryColor: '#539bf5' }),
  user: () => ({ name: '用户', status: '在线' }),
  notifications: () => [],
  showSearch: true,
  showNotifications: true,
  stickyHeader: true,
  collapseOnScroll: true
})

const emit = defineEmits<{
  navClick: [path: string]
  search: [query: string]
  notificationClick: [notification: Notification]
  userAction: [action: string]
  themeChange: [isDark: boolean]
}>()

// Composables
const route = useRoute()
const router = useRouter()
const { isMobile, isTablet } = useResponsive()
const { isDark, toggleTheme } = useTheme()

// State
const showSearch = ref(false)
const searchQuery = ref('')
const searchExpanded = ref(false)
const isScrolling = ref(false)
const isScrolled = ref(false)
const scrollY = ref(0)
const lastScrollY = ref(0)
const scrollDirection = ref<'up' | 'down'>('up')

// Mobile state
const mobileMenuOpen = ref(false)
const showMobileSearchBar = ref(false)

// User state
const userName = computed(() => props.user.name || '用户')
const userAvatar = computed(() => props.user.avatar || '')
const userStatus = computed(() => props.user.status || '在线')

// Notification state
const notifications = ref<Notification[]>(props.notifications)
const isThemeChanging = ref(false)

// UI state
const isLogoHovered = ref(false)
const hoveredNavItem = ref<NavItem | null>(null)
const searchInputRef = ref()
const mobileSearchInputRef = ref()

// Computed
const primaryNavItems = computed(() => {
  const maxDesktopItems = isTablet.value ? 4 : 6
  return props.navItems.slice(0, maxDesktopItems)
})

const secondaryNavItems = computed(() => {
  const maxDesktopItems = isTablet.value ? 4 : 6
  return props.navItems.slice(maxDesktopItems)
})

const unreadNotifications = computed(() => {
  return notifications.value.filter(n => !n.read).length
})

const activeTab = computed(() => route.path)

const headerClasses = computed(() => ({
  'is-mobile': isMobile.value,
  'is-tablet': isTablet.value,
  'is-scrolled': isScrolled.value,
  'is-scrolling': isScrolling.value,
  'sticky': props.stickyHeader,
  'collapsed': props.collapseOnScroll && isScrolled.value,
  'search-expanded': searchExpanded.value,
  'mobile-menu-open': mobileMenuOpen.value
}))

// Backdrop opacity for scroll effect
const backdropOpacity = computed(() => {
  if (!props.stickyHeader) return 1
  
  if (isScrolled.value) {
    return Math.min(0.95, 0.9 + (scrollY.value / 1000))
  }
  return 0.98
})

// Theme background color
const themeBackgroundColor = computed(() => {
  return isDark.value ? 'rgba(28, 33, 40, 0.95)' : 'rgba(246, 248, 250, 0.95)'
})

// Mobile title logic
const mobileTitle = computed(() => {
  const currentItem = props.navItems.find(item => isActive(item.path))
  return currentItem ? currentItem.label : props.brandName
})

const showMobileTitle = computed(() => {
  return !isScrolled.value || !mobileMenuOpen.value
})

const showFullTitle = computed(() => {
  return !props.collapseOnScroll || !isScrolled.value
})

// Search tooltips
const searchTooltip = computed(() => {
  return searchExpanded.value ? '收起搜索' : '搜索策略、指标或数据...'
})

const searchPlaceholder = computed(() => {
  return '搜索策略、指标或数据...'
})

const mobileSearchPlaceholder = computed(() => {
  return '搜索...'
})

const themeTooltip = computed(() => {
  return isDark.value ? '切换到浅色模式' : '切换到深色模式'
})

// Methods
const isActive = (path: string) => {
  return path === activeTab.value || route.path.startsWith(path + '/')
}

const handleNavClick = (path: string) => {
  emit('navClick', path)
  closeMobileOverlays()
}

const handleMobileNavClick = (path: string) => {
  handleNavClick(path)
  mobileMenuOpen.value = false
}

const handleNavHover = (item: NavItem) => {
  hoveredNavItem.value = item
}

const handleNavLeave = () => {
  hoveredNavItem.value = null
}

const handleSearch = () => {
  if (searchQuery.value.trim()) {
    emit('search', searchQuery.value)
    searchExpanded.value = false
  }
}

const handleSearchBlur = () => {
  if (!searchQuery.value.trim()) {
    searchExpanded.value = false
  }
}

const handleLogoClick = () => {
  router.push('/dashboard')
}

const handleThemeToggle = async () => {
  isThemeChanging.value = true
  try {
    toggleTheme()
    emit('themeChange', !isDark.value)
    
    // 显示主题切换提示
    ElMessage.success(`已切换到${!isDark.value ? '浅色' : '深色'}模式`)
  } finally {
    setTimeout(() => {
      isThemeChanging.value = false
    }, 300)
  }
}

const handleNotificationClick = (notification: Notification) => {
  notification.read = true
  emit('notificationClick', notification)
}

const handleNotificationAction = (action: string) => {
  if (action.startsWith('mark-read-')) {
    const notificationId = action.replace('mark-read-', '')
    const notification = notifications.value.find(n => n.id === notificationId)
    if (notification) {
      notification.read = true
    }
  }
}

const handleUserAction = (action: string) => {
  emit('userAction', action)
}

// Search functionality
const toggleSearch = () => {
  searchExpanded.value = !searchExpanded.value
  
  if (searchExpanded.value) {
    nextTick(() => {
      searchInputRef.value?.focus()
    })
  }
}

const toggleMobileSearch = () => {
  showMobileSearchBar.value = !showMobileSearchBar.value
  
  if (showMobileSearchBar.value) {
    nextTick(() => {
      mobileSearchInputRef.value?.focus()
    })
  }
}

// Mobile menu functionality
const toggleMobileMenu = () => {
  mobileMenuOpen.value = !mobileMenuOpen.value
  
  // 防止背景滚动
  if (mobileMenuOpen.value) {
    document.body.style.overflow = 'hidden'
  } else {
    document.body.style.overflow = ''
  }
}

const closeMobileOverlays = () => {
  mobileMenuOpen.value = false
  showMobileSearchBar.value = false
  document.body.style.overflow = ''
}

const showMobileNotifications = () => {
  // 这里可以显示移动端通知面板
  ElMessage.info('通知功能开发中...')
}

// Notification methods
const markAllAsRead = () => {
  notifications.value.forEach(notification => {
    notification.read = true
  })
  ElMessage.success('所有通知已标记为已读')
}

const formatTime = (time: Date) => {
  return dayjs(time).fromNow()
}

// Scroll handling
const handleScroll = () => {
  const currentScrollY = window.scrollY
  scrollY.value = currentScrollY
  
  // 检测滚动方向
  if (currentScrollY > lastScrollY.value) {
    scrollDirection.value = 'down'
  } else {
    scrollDirection.value = 'up'
  }
  
  // 检测是否已滚动
  isScrolled.value = currentScrollY > 50
  
  // 滚动状态检测
  if (!isScrolling.value) {
    isScrolling.value = true
  }
  
  lastScrollY.value = currentScrollY
}

const throttledHandleScroll = () => {
  let timeoutId: number | null = null
  return () => {
    if (timeoutId) {
      clearTimeout(timeoutId)
    }
    timeoutId = window.setTimeout(() => {
      handleScroll()
      isScrolling.value = false
    }, 16) // ~60fps
  }
}

// Keyboard shortcuts
const handleKeydown = (event: KeyboardEvent) => {
  // Esc 键关闭所有覆盖层
  if (event.key === 'Escape') {
    closeMobileOverlays()
    searchExpanded.value = false
  }
  
  // Ctrl/Cmd + K 打开搜索
  if ((event.ctrlKey || event.metaKey) && event.key === 'k') {
    event.preventDefault()
    if (isMobile.value) {
      toggleMobileSearch()
    } else {
      toggleSearch()
    }
  }
}

// Lifecycle
onMounted(() => {
  window.addEventListener('scroll', throttledHandleScroll(), { passive: true })
  document.addEventListener('keydown', handleKeydown)
})

onUnmounted(() => {
  window.removeEventListener('scroll', throttledHandleScroll())
  document.removeEventListener('keydown', handleKeydown)
  document.body.style.overflow = ''
})

// Watch for route changes
watch(() => route.path, () => {
  closeMobileOverlays()
  searchExpanded.value = false
})

// Watch for notification changes
watch(() => props.notifications, (newNotifications) => {
  notifications.value = [...newNotifications]
}, { deep: true })
</script>

<style lang="scss" scoped>
@import '@/styles/variables.scss';

.responsive-header {
  position: sticky;
  top: 0;
  z-index: $z-sticky;
  background-color: var(--bg-secondary);
  border-bottom: 1px solid var(--border-color);
}

/* Desktop Header */
.desktop-header {
  .header-container {
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: $header-height;
    padding: 0 $spacing-xl;
    max-width: 1200px;
    margin: 0 auto;
  }

  .logo-section {
    display: flex;
    align-items: center;
    gap: $spacing-md;
  }

  .logo-icon {
    width: 36px;
    height: 36px;
    border-radius: $radius-md;
    display: flex;
    align-items: center;
    justify-content: center;
    background: linear-gradient(135deg, var(--accent-color) 0%, var(--info-color) 100%);
    box-shadow: $shadow-sm;
    transition: all $transition-fast;

    &:hover {
      transform: translateY(-2px);
      box-shadow: $shadow-md;
    }
  }

  .logo-text {
    color: white;
    font-weight: $font-weight-bold;
    font-size: $font-size-lg;
  }

  .brand-name {
    font-size: $font-size-xl;
    font-weight: $font-weight-bold;
    color: var(--accent-color);
  }

  .nav-section {
    display: flex;
    align-items: center;
    gap: $spacing-sm;
  }

  .nav-item {
    display: flex;
    align-items: center;
    gap: $spacing-sm;
    padding: $spacing-md $spacing-lg;
    border-radius: $radius-md;
    color: var(--text-secondary);
    cursor: pointer;
    transition: all $transition-fast;
    position: relative;
    font-weight: $font-weight-medium;

    &:hover {
      background-color: var(--hover-bg);
      color: var(--accent-color);
      transform: translateY(-1px);
    }

    &.active {
      background-color: var(--info-bg);
      color: var(--accent-color);
      font-weight: $font-weight-semibold;

      &::after {
        content: '';
        position: absolute;
        bottom: -1px;
        left: 50%;
        transform: translateX(-50%);
        width: 70%;
        height: 3px;
        background-color: var(--accent-color);
        border-radius: 2px 2px 0 0;
      }
    }
  }

  .nav-label {
    font-size: $font-size-sm;
  }

  .user-section {
    display: flex;
    align-items: center;
    gap: $spacing-sm;
  }
}

/* Mobile Header */
.mobile-header {
  .mobile-top-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: $header-height-mobile;
    padding: 0 $spacing-lg;
  }

  .mobile-logo {
    display: flex;
    align-items: center;
    gap: $spacing-sm;
  }

  .mobile-actions {
    display: flex;
    align-items: center;
    gap: $spacing-md;
  }

  .mobile-tab-bar {
    position: fixed;
    bottom: 0;
    left: 0;
    right: 0;
    height: 64px;
    background-color: var(--bg-secondary);
    border-top: 1px solid var(--border-color);
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    align-items: center;
    box-shadow: 0 -4px 12px rgba(0, 0, 0, 0.1);
    z-index: $z-fixed;
    padding-bottom: env(safe-area-inset-bottom);
  }

  .tab-item {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: $spacing-xs;
    padding: $spacing-sm;
    cursor: pointer;
    transition: all $transition-fast;
    position: relative;
    min-height: 44px; // Touch target size

    &:active {
      transform: scale(0.95);
      background-color: var(--hover-bg);
    }

    &.active {
      color: var(--accent-color);
    }
  }

  .tab-icon {
    position: relative;
    height: 24px;
  }

  .tab-label {
    font-size: 11px;
    font-weight: $font-weight-medium;
    margin-top: 2px;
  }

  .tab-indicator {
    position: absolute;
    bottom: 0;
    left: 50%;
    transform: translateX(-50%);
    width: 24px;
    height: 3px;
    background-color: var(--accent-color);
    border-radius: 3px 3px 0 0;
  }
}

/* Search Bar */
.search-bar {
  display: flex;
  align-items: center;
  gap: $spacing-md;
  padding: $spacing-lg $spacing-xl;
  border-top: 1px solid var(--border-color);
  background-color: var(--bg-primary);

  &.desktop {
    position: absolute;
    top: 100%;
    left: 0;
    right: 0;
    z-index: 10;
  }

  &.mobile {
    padding: $spacing-md $spacing-lg;
  }

  .search-input {
    flex: 1;
  }
}

/* Action Icons */
.action-icon {
  font-size: 20px;
  cursor: pointer;
  color: var(--text-secondary);
  transition: all $transition-fast;
  padding: $spacing-sm;
  border-radius: $radius-sm;
  min-width: 36px;
  min-height: 36px;
  display: flex;
  align-items: center;
  justify-content: center;

  &:hover {
    color: var(--accent-color);
    background-color: var(--hover-bg);
  }

  &:active {
    transform: scale(0.95);
  }

  &.theme-switcher {
    font-size: 22px;

    &:hover {
      transform: rotate(20deg);
    }
  }
}

.notification-badge {
  :deep(.el-badge__content) {
    background-color: #f56c6c;
    border: none;
    min-width: 16px;
    height: 16px;
    padding: 0 4px;
    font-size: 10px;
  }
}

/* Animations */
.slide-down-enter-active,
.slide-down-leave-active {
  transition: all 0.2s ease-out;
}

.slide-down-enter-from,
.slide-down-leave-to {
  opacity: 0;
  transform: translateY(-10px);
}

/* Badges */
.nav-badge {
  :deep(.el-badge__content) {
    position: absolute;
    top: -4px;
    right: -4px;
    background-color: #f56c6c;
    border: none;
    min-width: 16px;
    height: 16px;
    padding: 0 4px;
    font-size: 10px;
    transform: scale(0.8);
  }
}

tab-badge {
  :deep(.el-badge__content) {
    position: absolute;
    top: -8px;
    right: -6px;
    background-color: #f56c6c;
    border: none;
    min-width: 14px;
    height: 14px;
    padding: 0 3px;
    font-size: 9px;
  }
}
</style>