import { defineStore } from 'pinia'
import { ref } from 'vue'

export const useConfigStore = defineStore('config', () => {
  const theme = ref<'light' | 'dark'>('dark')
  const language = ref<'en' | 'zh'>('en')
  const riskLimits = ref<any>({})
  const apiKeys = ref<any>({})

  function setTheme(newTheme: 'light' | 'dark') {
    theme.value = newTheme
  }

  function setLanguage(lang: 'en' | 'zh') {
    language.value = lang
  }

  function setRiskLimits(limits: any) {
    riskLimits.value = limits
  }

  function setApiKeys(keys: any) {
    apiKeys.value = keys
  }

  return {
    theme,
    language,
    riskLimits,
    apiKeys,
    setTheme,
    setLanguage,
    setRiskLimits,
    setApiKeys
  }
}, {
  persist: true
})
