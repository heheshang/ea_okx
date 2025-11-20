import { describe, it, expect, beforeEach } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { useMarketStore } from '../market'

describe('Market Store', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  it('should initialize with empty market data', () => {
    const store = useMarketStore()
    expect(store.marketData).toEqual({})
    expect(store.latestPrices).toEqual({})
  })

  it('should update market data correctly', () => {
    const store = useMarketStore()
    const data = {
      symbol: 'BTC-USDT',
      price: 45000,
      volume: 1000,
      timestamp: '2024-11-20T10:00:00Z'
    }
    
    store.updateMarketData(data)
    
    expect(store.marketData['BTC-USDT']).toEqual(data)
    expect(store.latestPrices['BTC-USDT']).toBe(45000)
  })

  it('should update price when market data is updated', () => {
    const store = useMarketStore()
    
    store.updateMarketData({ symbol: 'ETH-USDT', price: 3000 })
    expect(store.latestPrices['ETH-USDT']).toBe(3000)
    
    store.updateMarketData({ symbol: 'ETH-USDT', price: 3100 })
    expect(store.latestPrices['ETH-USDT']).toBe(3100)
  })

  it('should get price for symbol', () => {
    const store = useMarketStore()
    
    store.updateMarketData({ symbol: 'BTC-USDT', price: 45000 })
    
    expect(store.getPrice('BTC-USDT')).toBe(45000)
    expect(store.getPrice('ETH-USDT')).toBeUndefined()
  })

  it('should handle data without symbol', () => {
    const store = useMarketStore()
    
    store.updateMarketData({ price: 45000 })
    
    expect(Object.keys(store.marketData)).toHaveLength(0)
  })
})
