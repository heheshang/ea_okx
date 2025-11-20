import { describe, it, expect, vi, beforeEach } from 'vitest'
import { mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import Dashboard from '../Dashboard.vue'
import ElementPlus from 'element-plus'

// Mock Tauri API
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn().mockResolvedValue({
    cpu_usage: 35.5,
    memory_usage: 62.3,
    network_latency: 45.2,
    active_strategies: 3,
    total_orders: 150
  })
}))

// Mock ECharts
vi.mock('echarts', () => ({
  default: {
    init: vi.fn(() => ({
      setOption: vi.fn(),
      resize: vi.fn(),
      dispose: vi.fn()
    }))
  }
}))

describe('Dashboard View', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  it('should render dashboard', () => {
    const wrapper = mount(Dashboard, {
      global: {
        plugins: [ElementPlus]
      }
    })
    
    expect(wrapper.find('.dashboard').exists()).toBe(true)
  })

  it('should display metric cards', () => {
    const wrapper = mount(Dashboard, {
      global: {
        plugins: [ElementPlus]
      }
    })
    
    expect(wrapper.findAll('.metric-card')).toHaveLength(4)
  })

  it('should format PnL correctly', async () => {
    const wrapper = mount(Dashboard, {
      global: {
        plugins: [ElementPlus]
      }
    })
    
    const vm = wrapper.vm as any
    
    expect(vm.formatPnL(1234.56)).toBe('+$1234.56')
    expect(vm.formatPnL(-500.00)).toBe('-$500.00')
    expect(vm.formatPnL(0)).toBe('+$0.00')
  })

  it('should get correct status type', () => {
    const wrapper = mount(Dashboard, {
      global: {
        plugins: [ElementPlus]
      }
    })
    
    const vm = wrapper.vm as any
    
    expect(vm.getStatusType('Active')).toBe('success')
    expect(vm.getStatusType('Paused')).toBe('warning')
    expect(vm.getStatusType('Stopped')).toBe('info')
  })

  it('should get correct alert type', () => {
    const wrapper = mount(Dashboard, {
      global: {
        plugins: [ElementPlus]
      }
    })
    
    const vm = wrapper.vm as any
    
    expect(vm.getAlertType('INFO')).toBe('primary')
    expect(vm.getAlertType('WARNING')).toBe('warning')
    expect(vm.getAlertType('ERROR')).toBe('danger')
    expect(vm.getAlertType('CRITICAL')).toBe('danger')
  })
})
