import { ref, onMounted, onUnmounted, computed } from 'vue'

export interface ResponsiveBreakpoints {
  isMobile: boolean
  isTablet: boolean
  isDesktop: boolean
  isDesktopLg: boolean
  currentBreakpoint: 'mobile' | 'tablet' | 'desktop' | 'desktop-lg'
}

const BREAKPOINTS = {
  mobile: 0,
  tablet: 768,
  desktop: 1024,
  desktopLg: 1440,
}

export function useResponsive() {
  const windowWidth = ref(window.innerWidth)
  const windowHeight = ref(window.innerHeight)
  
  const isMobile = computed(() => windowWidth.value < BREAKPOINTS.tablet)
  const isTablet = computed(() => 
    windowWidth.value >= BREAKPOINTS.tablet && windowWidth.value < BREAKPOINTS.desktop
  )
  const isDesktop = computed(() => 
    windowWidth.value >= BREAKPOINTS.desktop && windowWidth.value < BREAKPOINTS.desktopLg
  )
  const isDesktopLg = computed(() => windowWidth.value >= BREAKPOINTS.desktopLg)
  
  const currentBreakpoint = computed<'mobile' | 'tablet' | 'desktop' | 'desktop-lg'>(() => {
    if (isMobile.value) return 'mobile'
    if (isTablet.value) return 'tablet'
    if (isDesktop.value) return 'desktop'
    return 'desktop-lg'
  })
  
  const isPortrait = computed(() => windowHeight.value > windowWidth.value)
  const isLandscape = computed(() => windowWidth.value > windowHeight.value)
  
  const isTouchDevice = computed(() => {
    return 'ontouchstart' in window || navigator.maxTouchPoints > 0
  })
  
  // Update window dimensions
  const updateDimensions = () => {
    windowWidth.value = window.innerWidth
    windowHeight.value = window.innerHeight
  }
  
  // Debounced resize handler
  let resizeTimeout: number | null = null
  const handleResize = () => {
    if (resizeTimeout) {
      clearTimeout(resizeTimeout)
    }
    resizeTimeout = window.setTimeout(() => {
      updateDimensions()
    }, 150)
  }
  
  onMounted(() => {
    updateDimensions()
    window.addEventListener('resize', handleResize)
  })
  
  onUnmounted(() => {
    window.removeEventListener('resize', handleResize)
    if (resizeTimeout) {
      clearTimeout(resizeTimeout)
    }
  })
  
  return {
    windowWidth,
    windowHeight,
    isMobile,
    isTablet,
    isDesktop,
    isDesktopLg,
    currentBreakpoint,
    isPortrait,
    isLandscape,
    isTouchDevice,
  }
}
