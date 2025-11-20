import { listen } from '@tauri-apps/api/event'
import { useMarketStore } from '@/stores/market'
import { useTradeStore } from '@/stores/trade'
import { useStrategyStore } from '@/stores/strategy'

export function useTauriEvents() {
  const marketStore = useMarketStore()
  const tradeStore = useTradeStore()
  const strategyStore = useStrategyStore()

  const initializeEventListeners = async () => {
    // Market data updates
    await listen('market-data-update', (event: any) => {
      marketStore.updateMarketData(event.payload)
    })

    // Order status changes
    await listen('order-status-change', (event: any) => {
      tradeStore.updateOrderStatus(event.payload)
    })

    // Position updates
    await listen('position-update', (event: any) => {
      tradeStore.updatePosition(event.payload)
    })

    // Strategy signals
    await listen('strategy-signal', (event: any) => {
      strategyStore.handleSignal(event.payload)
    })

    // Risk alerts
    await listen('risk-alert', (event: any) => {
      console.warn('Risk Alert:', event.payload)
      // Handle risk alert
    })

    // System health
    await listen('system-health', (event: any) => {
      // Update system health metrics
    })
  }

  return {
    initializeEventListeners
  }
}
