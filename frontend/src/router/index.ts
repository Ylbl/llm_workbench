import { createRouter, createWebHistory } from 'vue-router'
import WorkspaceShell from '../views/WorkspaceShell.vue'
import MathDemo from '../views/MathDemo.vue'
import LlmSettingsPane from '../views/LlmSettingsPane.vue'

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
      name: 'math-demo',
      component: MathDemo,
    },
    {
      path: '/llm-settings',
      name: 'llm-settings',
      component: LlmSettingsPane,
    },
  ],
})

export default router
