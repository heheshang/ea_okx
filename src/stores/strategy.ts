import { defineStore } from 'pinia'
import { ref } from 'vue'

export const useStrategyStore = defineStore('strategy', () => {
  const strategies = ref<any[]>([])
  const activeStrategy = ref<any>(null)
  const signals = ref<any[]>([])

  function setStrategies(data: any[]) {
    strategies.value = data
  }

  function setActiveStrategy(strategy: any) {
    activeStrategy.value = strategy
  }

  function handleSignal(signalData: any) {
    signals.value.unshift(signalData)
    // Keep only last 100 signals
    if (signals.value.length > 100) {
      signals.value = signals.value.slice(0, 100)
    }
  }

  function addStrategy(strategy: any) {
    strategies.value.push(strategy)
  }

  function updateStrategy(id: string, updates: any) {
    const index = strategies.value.findIndex(s => s.id === id)
    if (index >= 0) {
      strategies.value[index] = { ...strategies.value[index], ...updates }
    }
  }

  function removeStrategy(id: string) {
    strategies.value = strategies.value.filter(s => s.id !== id)
  }

  return {
    strategies,
    activeStrategy,
    signals,
    setStrategies,
    setActiveStrategy,
    handleSignal,
    addStrategy,
    updateStrategy,
    removeStrategy
  }
}, {
  persist: {
    paths: ['strategies']
  }
})
