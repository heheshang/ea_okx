import { defineStore } from 'pinia'
import { ref } from 'vue'

export const useUserStore = defineStore('user', () => {
  const token = ref<string>('')
  const userInfo = ref<any>(null)

  function setToken(newToken: string) {
    token.value = newToken
  }

  function setUserInfo(info: any) {
    userInfo.value = info
  }

  function logout() {
    token.value = ''
    userInfo.value = null
  }

  return {
    token,
    userInfo,
    setToken,
    setUserInfo,
    logout
  }
}, {
  persist: true
})
