import { createRouter, createWebHistory } from 'vue-router'
import HomeView from '../views/home/HomeView.vue'

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: '/',
      name: 'home',
      component: HomeView,
    },
    {
      path: '/ftp',
      name: 'ftp',
      component: () => import('../views/ftp/FTP.vue'),
    },
    {
      path: '/connect',
      name: 'connect',
      component: () => import('../views/connections/Connection.vue'),
    },
  ],
})

export default router
