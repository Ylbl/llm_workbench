import { createRouter, createWebHistory } from 'vue-router'
import WorkspaceShell from '../views/WorkspaceShell.vue'

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/',
      redirect: '/workspace',
    },
    {
      path: '/workspace',
      name: 'workspace',
      component: WorkspaceShell,
    },
    {
      path: '/settings',
      name: 'settings',
      component: WorkspaceShell,
    },
  ],
})

export default router
