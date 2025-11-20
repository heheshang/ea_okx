import { createRouter, createWebHistory } from 'vue-router'
import type { RouteRecordRaw } from 'vue-router'

const routes: RouteRecordRaw[] = [
  {
    path: '/',
    name: 'Layout',
    component: () => import('@/layouts/MainLayout.vue'),
    redirect: '/dashboard',
    children: [
      {
        path: '/dashboard',
        name: 'Dashboard',
        component: () => import('@/views/Dashboard.vue'),
        meta: { title: 'Dashboard', icon: 'DataAnalysis' }
      },
      {
        path: '/strategies',
        name: 'Strategies',
        component: () => import('@/views/Strategies.vue'),
        meta: { title: 'Strategy Management', icon: 'TrendCharts' }
      },
      {
        path: '/templates',
        name: 'Templates',
        component: () => import('@/views/Templates.vue'),
        meta: { title: 'Strategy Templates', icon: 'Document' }
      },
      {
        path: '/strategies/:id',
        name: 'StrategyDetail',
        component: () => import('@/views/StrategyDetail.vue'),
        meta: { title: 'Strategy Detail', hidden: true }
      },
      {
        path: '/backtest',
        name: 'Backtest',
        component: () => import('@/views/Backtest.vue'),
        meta: { title: 'Backtest Analysis', icon: 'TrendCharts' }
      },
      {
        path: '/trading',
        name: 'Trading',
        component: () => import('@/views/Trading.vue'),
        meta: { title: 'Trading Monitor', icon: 'Money' }
      },
      {
        path: '/risk',
        name: 'Risk',
        component: () => import('@/views/Risk.vue'),
        meta: { title: 'Risk Center', icon: 'Warning' }
      },
      {
        path: '/settings',
        name: 'Settings',
        component: () => import('@/views/Settings.vue'),
        meta: { title: 'Settings', icon: 'Setting' }
      }
    ]
  }
]

const router = createRouter({
  history: createWebHistory(),
  routes
})

export default router
