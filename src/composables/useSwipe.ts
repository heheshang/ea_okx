import { ref, onMounted, onUnmounted, computed, readonly, Ref } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { useResponsive } from './useResponsive'

export interface SwipeDirection {
  left: boolean
  right: boolean
  up: boolean
  down: boolean
}

export interface SwipeOptions {
  threshold?: number
  preventDefault?: boolean
  onSwipe?: (direction: 'left' | 'right' | 'up' | 'down') => void
  onSwipeLeft?: () => void
  onSwipeRight?: () => void
  onSwipeUp?: () => void
  onSwipeDown?: () => void
}

interface TouchPoint {
  x: number
  y: number
  time: number
}

/**
 * Touch gesture hook for swipe interactions
 * Supports horizontal and vertical swipes with customizable sensitivity
 */
export function useSwipe(elementRef: Ref <HTMLElement | null>, options: SwipeOptions = {}) {
  const {
    threshold = 50,
    preventDefault = true,
    onSwipe,
    onSwipeLeft,
    onSwipeRight,
    onSwipeUp,
    onSwipeDown
  } = options

  const { isTouchDevice } = useResponsive()

  const isSwiping = ref(false)
  const swipeDirection = ref<SwipeDirection>({
    left: false,
    right: false,
    up: false,
    down: false
  })

  const startPoint = ref<TouchPoint | null>(null)
  const lastPoint = ref<TouchPoint | null>(null)

  // Calculate touch distance and direction
  const calculateSwipe = (
    start: TouchPoint,
    end: TouchPoint
  ): { direction: 'left' | 'right' | 'up' | 'down' | null; distance: number; velocity: number } => {
    const deltaX = end.x - start.x
    const deltaY = end.y - start.y
    const deltaTime = end.time - start.time

    const distance = Math.sqrt(deltaX * deltaX + deltaY * deltaY)
    const velocity = distance / (deltaTime || 1)

    // Determine dominant direction
    if (Math.abs(deltaX) > Math.abs(deltaY)) {
      // Horizontal swipe
      if (Math.abs(deltaX) > threshold) {
        return {
          direction: deltaX > 0 ? 'right' : 'left',
          distance: Math.abs(deltaX),
          velocity
        }
      }
    } else {
      // Vertical swipe
      if (Math.abs(deltaY) > threshold) {
        return {
          direction: deltaY > 0 ? 'down' : 'up',
          distance: Math.abs(deltaY),
          velocity
        }
      }
    }

    return { direction: null, distance, velocity }
  }

  const handleTouchStart = (event: TouchEvent) => {
    if (!isTouchDevice.value) return
    if (preventDefault) {
      event.preventDefault()
    }

    const touch = event.touches[0]
    const point: TouchPoint = {
      x: touch.clientX,
      y: touch.clientY,
      time: Date.now()
    }

    startPoint.value = point
    lastPoint.value = point
    isSwiping.value = true

    // Reset direction flags
    swipeDirection.value = {
      left: false,
      right: false,
      up: false,
      down: false
    }
  }

  const handleTouchMove = (event: TouchEvent) => {
    if (!isSwiping.value || !startPoint.value) return

    if (preventDefault) {
      event.preventDefault()
    }

    const touch = event.touches[0]
    const point: TouchPoint = {
      x: touch.clientX,
      y: touch.clientY,
      time: Date.now()
    }

    lastPoint.value = point

    // Calculate provisional direction for visual feedback
    const { direction } = calculateSwipe(startPoint.value, point)

    if (direction) {
      swipeDirection.value = {
        left: direction === 'left',
        right: direction === 'right',
        up: direction === 'up',
        down: direction === 'down'
      }
    }
  }

  const handleTouchEnd = () => {
    if (!isSwiping.value || !startPoint.value || !lastPoint.value) return

    const { direction, distance, velocity } = calculateSwipe(startPoint.value, lastPoint.value)

    isSwiping.value = false

    if (direction) {
      // Execute specific swipe handler
      switch (direction) {
        case 'left':
          onSwipeLeft?.()
          break
        case 'right':
          onSwipeRight?.()
          break
        case 'up':
          onSwipeUp?.()
          break
        case 'down':
          onSwipeDown?.()
          break
      }

      // Execute general swipe handler
      onSwipe?.(direction)
    }

    // Reset start point after a delay
    setTimeout(() => {
      startPoint.value = null
      lastPoint.value = null
      swipeDirection.value = {
        left: false,
        right: false,
        up: false,
        down: false
      }
    }, 100)
  }

  // Add event listeners
  onMounted(() => {
    const element = elementRef.value
    if (!element || !isTouchDevice.value) return

    element.addEventListener('touchstart', handleTouchStart, { passive: !preventDefault })
    element.addEventListener('touchmove', handleTouchMove, { passive: !preventDefault })
    element.addEventListener('touchend', handleTouchEnd, { passive: true })
    element.addEventListener('touchcancel', handleTouchEnd, { passive: true })
  })

  // Remove event listeners
  onUnmounted(() => {
    const element = elementRef.value
    if (!element) return

    element.removeEventListener('touchstart', handleTouchStart)
    element.removeEventListener('touchmove', handleTouchMove)
    element.removeEventListener('touchend', handleTouchEnd)
    element.removeEventListener('touchcancel', handleTouchEnd)
  })

  return {
    isSwiping: readonly(isSwiping),
    swipeDirection: readonly(swipeDirection),
    swipeProgress: computed(() => {
      if (!startPoint.value || !lastPoint.value) return 0
      const distance = Math.sqrt(
        Math.pow(lastPoint.value.x - startPoint.value.x, 2) +
        Math.pow(lastPoint.value.y - startPoint.value.y, 2)
      )
      return Math.min(distance / threshold, 1)
    })
  }
}

/**
 * Specialized drawer swipe hook
 * Enables swipe-to-open and swipe-to-close interactions
 */
export function useDrawerSwipe(drawerRef: Ref <HTMLElement | null>, isOpen: Ref <boolean>) {
  const { onSwipeLeft, onSwipeRight } = useDrawerSwipeHandlers(isOpen)

  return useSwipe(drawerRef, {
    threshold: 30,
    preventDefault: true,
    onSwipeLeft,
    onSwipeRight
  })
}

/**
 * Create drawer-specific swipe handlers
 */
export function useDrawerSwipeHandlers(isOpen: Ref <boolean>, options = {}) {
  const { closeThreshold = 150 } = options

  const onSwipeLeft = () => {
    if (isOpen.value) {
      // Close drawer with right-to-left swipe
      isOpen.value = false
    }
  }

  const onSwipeRight = () => {
    if (!isOpen.value) {
      // Open drawer with left-to-right swipe
      isOpen.value = true
    }
  }

  return { onSwipeLeft, onSwipeRight }
}

/**
 * Pull-to-refresh hook for mobile
 */
export function usePullToRefresh(refreshCallback: () => Promise <void>) {
  const { isTouchDevice } = useResponsive()
  const isRefreshing = ref(false)
  const pullDistance = ref(0)

  const containerRef = ref<HTMLElement | null>(null)

  const handleRefresh = async () => {
    if (isRefreshing.value) return

    isRefreshing.value = true
    await refreshCallback()
    isRefreshing.value = false
    pullDistance.value = 0
  }

  const refreshOptions = computed(() => ({
    threshold: 80,
    preventDefault: true,
    onSwipeUp: () => {
      if (window.scrollY === 0) {
        handleRefresh()
      }
    }
  }))

  const { isSwiping, swipeDirection } = useSwipe(containerRef, refreshOptions.value)

  return {
    containerRef,
    isRefreshing,
    pullDistance,
    pullProgress: computed(() => Math.min(pullDistance.value / 80, 1))
  }
}

/**
 * Swipe-based navigation hook for page transitions
 */
export function useNavigationSwipe() {
  const router = useRouter()
  const route = useRoute()

  const { onSwipeLeft, onSwipeRight } = useNavigationSwipeHandlers(route, router)

  return {
    swipeOptions: {
      threshold: 100,
      preventDefault: true,
      onSwipeLeft,
      onSwipeRight
    }
  }
}

export function useNavigationSwipeHandlers(route: any, router: any) {
  const onSwipeLeft = () => {
    // Navigate forward (if possible)
    const nextPath = getNextPath(route.path)
    if (nextPath) {
      router.push(nextPath)
    }
  }

  const onSwipeRight = () => {
    // Navigate backward (if possible)
    if (window.history.length > 1) {
      router.back()
    }
  }

  const getNextPath = (currentPath: string) => {
    const navOrder = ['/dashboard', '/strategies', '/backtest', '/trading', '/risk', '/settings']
    const currentIndex = navOrder.indexOf(currentPath)

    if (currentIndex !== -1 && currentIndex < navOrder.length - 1) {
      return navOrder[currentIndex + 1]
    }
    return null
  }

  return { onSwipeLeft, onSwipeRight }
}