import { computed, watch } from 'vue'
import { useConfigStore } from '@/stores/config'

export function useTheme() {
  const configStore = useConfigStore()
  
  const isDark = computed(() => configStore.theme === 'dark')
  const theme = computed(() => configStore.theme)
  
  // Fast theme toggle
  const toggleTheme = () => {
    const newTheme = isDark.value ? 'light' : 'dark'
    configStore.setTheme(newTheme)
    applyTheme(newTheme)
  }
  
  // Apply theme to DOM instantly
  const applyTheme = (themeValue: 'light' | 'dark') => {
    const html = document.documentElement
    html.setAttribute('data-theme', themeValue)
    html.classList.toggle('dark', themeValue === 'dark')
    
    // Dispatch custom event for charts to update
    window.dispatchEvent(new CustomEvent('theme-changed', { 
      detail: { theme: themeValue } 
    }))
  }
  
  // Initialize theme
  const initTheme = () => {
    applyTheme(configStore.theme)
  }
  
  // Watch for external theme changes (from settings page, etc.)
  watch(() => configStore.theme, (newTheme) => {
    applyTheme(newTheme)
  })
  
  return {
    isDark,
    theme,
    toggleTheme,
    initTheme,
    applyTheme
  }
}
