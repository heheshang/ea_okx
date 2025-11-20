import { describe, it, expect, beforeEach } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { useConfigStore } from '../config'

describe('Config Store', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  it('should initialize with default values', () => {
    const store = useConfigStore()
    expect(store.theme).toBe('dark')
    expect(store.language).toBe('en')
    expect(store.riskLimits).toEqual({})
    expect(store.apiKeys).toEqual({})
  })

  it('should set theme', () => {
    const store = useConfigStore()
    
    store.setTheme('light')
    expect(store.theme).toBe('light')
    
    store.setTheme('dark')
    expect(store.theme).toBe('dark')
  })

  it('should set language', () => {
    const store = useConfigStore()
    
    store.setLanguage('zh')
    expect(store.language).toBe('zh')
    
    store.setLanguage('en')
    expect(store.language).toBe('en')
  })

  it('should set risk limits', () => {
    const store = useConfigStore()
    const limits = {
      maxLeverage: 3,
      dailyLossLimit: 5000
    }
    
    store.setRiskLimits(limits)
    
    expect(store.riskLimits).toEqual(limits)
  })

  it('should set API keys', () => {
    const store = useConfigStore()
    const keys = {
      apiKey: 'test-key',
      secretKey: 'test-secret'
    }
    
    store.setApiKeys(keys)
    
    expect(store.apiKeys).toEqual(keys)
  })
})
