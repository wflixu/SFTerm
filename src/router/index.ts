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
      path: '/sftp',
      name: 'sftp',
      component: () => import('../views/sftp/SFTPView.vue'),
    },
    {
      path: '/session/:hostId',
      name: 'session',
      component: () => import('../views/connections/Connection.vue'),
    },
  ],
})

export default router
