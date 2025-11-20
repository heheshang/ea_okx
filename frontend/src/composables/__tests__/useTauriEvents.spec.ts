import { describe, it, expect, vi, beforeEach } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { useTauriEvents } from '../useTauriEvents'

// Mock Tauri event API
const mockListen = vi.fn()
vi.mock('@tauri-apps/api/event', () => ({
  listen: mockListen
}))

describe('useTauriEvents Composable', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    mockListen.mockClear()
  })

  it('should initialize event listeners', async () => {
    const { initializeEventListeners } = useTauriEvents()
    
    await initializeEventListeners()
    
    // Should register 6 event listeners
    expect(mockListen).toHaveBeenCalledTimes(6)
    expect(mockListen).toHaveBeenCalledWith('market-data-update', expect.any(Function))
    expect(mockListen).toHaveBeenCalledWith('order-status-change', expect.any(Function))
    expect(mockListen).toHaveBeenCalledWith('position-update', expect.any(Function))
    expect(mockListen).toHaveBeenCalledWith('strategy-signal', expect.any(Function))
    expect(mockListen).toHaveBeenCalledWith('risk-alert', expect.any(Function))
    expect(mockListen).toHaveBeenCalledWith('system-health', expect.any(Function))
  })

  it('should return initializeEventListeners function', () => {
    const { initializeEventListeners } = useTauriEvents()
    
    expect(typeof initializeEventListeners).toBe('function')
  })
})
