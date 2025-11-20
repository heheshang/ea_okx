import { describe, it, expect, beforeEach } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { useUserStore } from '../user'

describe('User Store', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  it('should initialize with empty state', () => {
    const store = useUserStore()
    expect(store.token).toBe('')
    expect(store.userInfo).toBeNull()
  })

  it('should set token correctly', () => {
    const store = useUserStore()
    const testToken = 'test-token-123'
    
    store.setToken(testToken)
    
    expect(store.token).toBe(testToken)
  })

  it('should set user info correctly', () => {
    const store = useUserStore()
    const userInfo = {
      id: '1',
      name: 'Test User',
      email: 'test@example.com'
    }
    
    store.setUserInfo(userInfo)
    
    expect(store.userInfo).toEqual(userInfo)
  })

  it('should logout and clear state', () => {
    const store = useUserStore()
    
    store.setToken('test-token')
    store.setUserInfo({ id: '1', name: 'Test' })
    
    store.logout()
    
    expect(store.token).toBe('')
    expect(store.userInfo).toBeNull()
  })
})
