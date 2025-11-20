<template>
  <router-view />
</template>

<script setup lang="ts">
import { onMounted } from 'vue'
import { useTauriEvents } from '@/composables/useTauriEvents'
import { useTheme } from '@/composables/useTheme'

// Initialize services
const { initializeEventListeners } = useTauriEvents()
const { initTheme } = useTheme()

onMounted(() => {
  initTheme()
  initializeEventListeners()
})
</script>

<style lang="scss">
#app {
  width: 100%;
  height: 100vh;
  overflow: hidden;
}
</style>
