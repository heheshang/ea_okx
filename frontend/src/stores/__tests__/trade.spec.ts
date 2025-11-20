import { describe, it, expect, beforeEach } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { useTradeStore } from '../trade'

describe('Trade Store', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  it('should initialize with empty state', () => {
    const store = useTradeStore()
    expect(store.orders).toEqual([])
    expect(store.positions).toEqual([])
  })

  it('should add order', () => {
    const store = useTradeStore()
    const order = { id: '1', symbol: 'BTC-USDT', side: 'Buy' }
    
    store.addOrder(order)
    
    expect(store.orders).toHaveLength(1)
    expect(store.orders[0]).toEqual(order)
  })

  it('should update order status', () => {
    const store = useTradeStore()
    store.addOrder({ id: '1', status: 'Created' })
    
    store.updateOrderStatus({ order_id: '1', status: 'Filled' })
    
    expect(store.orders[0].status).toBe('Filled')
  })

  it('should update existing position', () => {
    const store = useTradeStore()
    store.positions = [
      { symbol: 'BTC-USDT', quantity: 1, pnl: 100 }
    ]
    
    store.updatePosition({ symbol: 'BTC-USDT', quantity: 2, pnl: 200 })
    
    expect(store.positions).toHaveLength(1)
    expect(store.positions[0].quantity).toBe(2)
    expect(store.positions[0].pnl).toBe(200)
  })

  it('should add new position if not exists', () => {
    const store = useTradeStore()
    
    store.updatePosition({ symbol: 'ETH-USDT', quantity: 1, pnl: 50 })
    
    expect(store.positions).toHaveLength(1)
    expect(store.positions[0].symbol).toBe('ETH-USDT')
  })

  it('should add orders to beginning of list', () => {
    const store = useTradeStore()
    
    store.addOrder({ id: '1' })
    store.addOrder({ id: '2' })
    
    expect(store.orders[0].id).toBe('2') // Most recent first
    expect(store.orders[1].id).toBe('1')
  })
})
