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
      path: '/workspace/:itemId',
      name: 'workspace-item',
      component: WorkspaceShell,
      props: true,
    },
    {
      path: '/settings',
      name: 'settings',
      component: WorkspaceShell,
    },
    {
      path: '/math-demo',
      redirect: '/settings',
    },
    {
      path: '/llm-settings',
      redirect: '/settings',
    },
  ],
})

export default router
