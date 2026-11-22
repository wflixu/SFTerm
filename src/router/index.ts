import { createRouter, createWebHistory } from 'vue-router'
import HomeView from '../views/home/HomeView.vue'
import SFTPView from '../views/sftp/SFTPView.vue'
import TerminalView from '../views/terminal/TerminalView.vue'

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: '/',
      name: 'home',
      component: HomeView,
    },
    {
      path: '/sftp',
      name: 'sftp',
      component: SFTPView,
    },
    {
      path: '/terminal/:hostId',
      name: 'terminal',
      component: TerminalView,
    },
    {
      path: '/:pathMatch(.*)*',
      redirect: (to) => {
        console.warn('[router] 未匹配的地址，已回到 Home：', to.fullPath)
        return '/'
      },
    },
  ],
})

// 动态 import 失败时至少留下可诊断的日志，而不是静默空白
router.onError((error, to) => {
  console.error('[router] 路由加载失败', to?.fullPath, error)
})

export default router
