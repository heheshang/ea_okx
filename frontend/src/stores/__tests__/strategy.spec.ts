import { describe, it, expect, beforeEach } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { useStrategyStore } from '../strategy'

describe('Strategy Store', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  it('should initialize with empty state', () => {
    const store = useStrategyStore()
    expect(store.strategies).toEqual([])
    expect(store.activeStrategy).toBeNull()
    expect(store.signals).toEqual([])
  })

  it('should set strategies', () => {
    const store = useStrategyStore()
    const strategies = [
      { id: '1', name: 'Strategy 1' },
      { id: '2', name: 'Strategy 2' }
    ]
    
    store.setStrategies(strategies)
    
    expect(store.strategies).toEqual(strategies)
  })

  it('should set active strategy', () => {
    const store = useStrategyStore()
    const strategy = { id: '1', name: 'Test Strategy' }
    
    store.setActiveStrategy(strategy)
    
    expect(store.activeStrategy).toEqual(strategy)
  })

  it('should add strategy', () => {
    const store = useStrategyStore()
    const strategy = { id: '1', name: 'New Strategy' }
    
    store.addStrategy(strategy)
    
    expect(store.strategies).toHaveLength(1)
    expect(store.strategies[0]).toEqual(strategy)
  })

  it('should update existing strategy', () => {
    const store = useStrategyStore()
    store.setStrategies([
      { id: '1', name: 'Old Name', status: 'Draft' }
    ])
    
    store.updateStrategy('1', { name: 'New Name', status: 'Active' })
    
    expect(store.strategies[0].name).toBe('New Name')
    expect(store.strategies[0].status).toBe('Active')
  })

  it('should remove strategy', () => {
    const store = useStrategyStore()
    store.setStrategies([
      { id: '1', name: 'Strategy 1' },
      { id: '2', name: 'Strategy 2' }
    ])
    
    store.removeStrategy('1')
    
    expect(store.strategies).toHaveLength(1)
    expect(store.strategies[0].id).toBe('2')
  })

  it('should handle signals and keep only last 100', () => {
    const store = useStrategyStore()
    
    // Add 105 signals
    for (let i = 0; i < 105; i++) {
      store.handleSignal({ id: i, type: 'buy' })
    }
    
    expect(store.signals).toHaveLength(100)
    expect(store.signals[0].id).toBe(104) // Most recent first
  })
})
