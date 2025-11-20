import { defineStore } from 'pinia'
import { ref } from 'vue'

export const useMarketStore = defineStore('market', () => {
  const marketData = ref<Record<string, any>>({})
  const latestPrices = ref<Record<string, number>>({})

  function updateMarketData(data: any) {
    if (data.symbol) {
      marketData.value[data.symbol] = data
      if (data.price) {
        latestPrices.value[data.symbol] = data.price
      }
    }
  }

  function getPrice(symbol: string): number | undefined {
    return latestPrices.value[symbol]
  }

  return {
    marketData,
    latestPrices,
    updateMarketData,
    getPrice
  }
})
