import { createRouter, createWebHistory } from 'vue-router'

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/',
      component: () => import('@/components/layout/AppLayout.vue'),
      children: [
        {
          path: '',
          name: 'welcome',
          component: () => import('@/views/workspace/WorkspaceView.vue'),
        },
        {
          path: 'project/:projectId',
          name: 'project-workspace',
          component: () => import('@/views/workspace/ProjectWorkspace.vue'),
          props: true,
        },
        {
          path: 'project/:projectId/chat',
          name: 'project-chat',
          component: () => import('@/views/sessions/ChatView.vue'),
          props: true,
        },
        {
          path: 'project/:projectId/chat/:sessionId',
          name: 'project-session',
          component: () => import('@/views/sessions/ChatView.vue'),
          props: true,
        },
        {
          path: 'agents',
          name: 'agents',
          component: () => import('@/views/agents/AgentView.vue'),
        },
        {
          path: 'capabilities',
          name: 'capabilities',
          component: () => import('@/views/capabilities/CapabilityCenter.vue'),
        },
        {
          path: 'capabilities/mcp',
          name: 'mcp',
          component: () => import('@/views/capabilities/McpList.vue'),
        },
        {
          path: 'capabilities/skill',
          name: 'skill',
          component: () => import('@/views/capabilities/SkillList.vue'),
        },
        {
          path: 'settings',
          name: 'settings',
          component: () => import('@/views/settings/SettingsView.vue'),
        },
      ],
    },
  ],
})

export default router
