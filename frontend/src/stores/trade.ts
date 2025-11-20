import { defineStore } from 'pinia'
import { ref } from 'vue'

export const useTradeStore = defineStore('trade', () => {
  const orders = ref<any[]>([])
  const positions = ref<any[]>([])

  function updateOrderStatus(orderData: any) {
    const index = orders.value.findIndex(o => o.id === orderData.order_id)
    if (index >= 0) {
      orders.value[index] = { ...orders.value[index], ...orderData }
    }
  }

  function updatePosition(positionData: any) {
    const index = positions.value.findIndex(p => p.symbol === positionData.symbol)
    if (index >= 0) {
      positions.value[index] = positionData
    } else {
      positions.value.push(positionData)
    }
  }

  function addOrder(order: any) {
    orders.value.unshift(order)
  }

  return {
    orders,
    positions,
    updateOrderStatus,
    updatePosition,
    addOrder
  }
}, {
  persist: {
    paths: ['orders', 'positions']
  }
})
