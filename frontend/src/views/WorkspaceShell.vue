<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import {
  Bell,
  Bot,
  Database,
  FilePlus2,
  FileText,
  Files,
  ListTree,
  MessageSquareText,
  PanelLeftClose,
  PanelRightClose,
  RefreshCcw,
  Settings,
  Trash2,
} from 'lucide-vue-next'
import { useHealthStore } from '../stores/health'
import { useWorkspaceStore } from '../stores/workspace'
import type { WorkspaceItem } from '../api/workspace'
import { createNote } from '../api/notes'
import { createConversation } from '../api/chat'
import { createAgent } from '../api/agents'
import SettingsPane from './SettingsPane.vue'
import NoteEditorPane from '../components/NoteEditorPane.vue'
import ChatPane from '../components/ChatPane.vue'
import AgentConfigPane from '../components/AgentConfigPane.vue'
import { WbContextMenu, WbContextMenuItem, WbIconButton } from '../ui'

type LeftTool = 'project' | 'settings'
type RightTool = 'outline' | 'notifications' | 'database'

const health = useHealthStore()
const workspace = useWorkspaceStore()
const route = useRoute()
const router = useRouter()

const minPanelWidth = 220
const maxPanelWidth = 520
const leftPanelOpen = ref(true)
const activeLeftTool = ref<LeftTool>('project')
const rightPanelOpen = ref(true)
const activeRightTool = ref<RightTool>('outline')
const leftPanelWidth = ref(loadPanelSize('left', 292))
const rightPanelWidth = ref(loadPanelSize('right', 270))
const projectContextTargetId = ref<string | null>(null)

let activeResize:
  | {
      side: 'left' | 'right'
      startX: number
      startWidth: number
    }
  | null = null

const itemTypeLabels: Record<string, string> = {
  note: '笔记',
  chat: '聊天',
  agent_config: '代理',
  file: '文件',
  task: '任务',
  settings_view: '设置',
}

const itemTypeIcons: Record<string, string> = {
  note: '笔',
  chat: '聊',
  agent_config: '代',
  file: '文',
  task: '任',
  settings_view: '设',
}

const placeholderText: Record<string, string> = {
  note: '笔记编辑器',
  chat: '聊天面板',
  agent_config: '代理配置',
  file: '文件面板（批次 13）',
  task: '任务面板（批次 14）',
  settings_view: '设置视图',
}

const isSettingsRoute = computed(() => route.name === 'settings')
const isWorkspaceRoute = computed(
  () => route.name === 'workspace' || route.name === 'workspace-item',
)

const rootItems = computed(() =>
  workspace.items
    .filter((item) => !item.parent_id)
    .sort((a, b) => a.sort_order - b.sort_order),
)

const selectedItem = computed(() => workspace.selectedItem)

const databaseStatus = computed(() => health.data?.database.status ?? 'not_configured')
const backendStatus = computed(() => {
  if (health.isLoading && !health.data) return 'checking'
  return health.data?.status ?? 'offline'
})

const statusMessage = computed(() => {
  if (health.error) return health.error
  if (health.isLoading) return '正在检查后端健康状态'
  if (health.data) return `${health.data.status} ${health.data.app.host}:${health.data.app.port}`
  return '后端健康状态未检查'
})

const outlineRows = computed(() => {
  if (!selectedItem.value) {
    return [
      { label: '工作区', value: `${workspace.items.length} 个项目` },
      { label: '后端', value: backendStatus.value },
      { label: '数据库', value: databaseStatus.value },
    ]
  }

  return [
    { label: '标题', value: selectedItem.value.title },
    { label: '类型', value: selectedItem.value.item_type },
    { label: '父节点', value: selectedItem.value.parent_id ?? '根' },
    { label: '排序', value: String(selectedItem.value.sort_order) },
    { label: '更新时间', value: new Date(selectedItem.value.updated_at).toLocaleString() },
  ]
})

const projectContextTarget = computed(() =>
  projectContextTargetId.value
    ? (workspace.items.find((item) => item.id === projectContextTargetId.value) ?? null)
    : null,
)

function loadPanelSize(side: 'left' | 'right', fallback: number) {
  if (typeof window === 'undefined') return fallback
  const storedValue = window.localStorage.getItem(`llm-workbench:${side}-panel-width`)
  if (storedValue === null) return fallback
  const stored = Number(storedValue)
  if (!Number.isFinite(stored)) return fallback
  return clamp(stored, minPanelWidth, maxPanelWidth)
}

function savePanelSize(side: 'left' | 'right', width: number) {
  if (typeof window === 'undefined') return
  window.localStorage.setItem(`llm-workbench:${side}-panel-width`, String(width))
}

function clamp(value: number, min: number, max: number) {
  return Math.min(max, Math.max(min, value))
}

function getChildren(parentId: string): WorkspaceItem[] {
  return workspace.items
    .filter((item) => item.parent_id === parentId)
    .sort((a, b) => a.sort_order - b.sort_order)
}

function selectLeftTool(tool: LeftTool) {
  activeLeftTool.value = tool
  leftPanelOpen.value = true

  if (tool === 'project') void router.push('/workspace')
  if (tool === 'settings') void router.push('/settings')
}

function selectRightTool(tool: RightTool) {
  if (activeRightTool.value === tool) {
    rightPanelOpen.value = !rightPanelOpen.value
    return
  }

  activeRightTool.value = tool
  rightPanelOpen.value = true
}

function startPanelResize(side: 'left' | 'right', event: PointerEvent) {
  activeResize = {
    side,
    startX: event.clientX,
    startWidth: side === 'left' ? leftPanelWidth.value : rightPanelWidth.value,
  }
  event.preventDefault()
  document.body.classList.add('is-resizing-panel')
  window.addEventListener('pointermove', handlePanelResize)
  window.addEventListener('pointerup', stopPanelResize)
}

function handlePanelResize(event: PointerEvent) {
  if (!activeResize) return

  const delta =
    activeResize.side === 'left'
      ? event.clientX - activeResize.startX
      : activeResize.startX - event.clientX
  const nextWidth = clamp(activeResize.startWidth + delta, minPanelWidth, maxPanelWidth)

  if (activeResize.side === 'left') {
    leftPanelWidth.value = nextWidth
  } else {
    rightPanelWidth.value = nextWidth
  }
}

function stopPanelResize() {
  if (activeResize) {
    const side = activeResize.side
    savePanelSize(side, side === 'left' ? leftPanelWidth.value : rightPanelWidth.value)
  }

  activeResize = null
  document.body.classList.remove('is-resizing-panel')
  window.removeEventListener('pointermove', handlePanelResize)
  window.removeEventListener('pointerup', stopPanelResize)
}

function handleSelect(id: string) {
  workspace.selectItem(id)
  void router.push(`/workspace/${id}`)
}

async function handleDelete(id: string) {
  await workspace.deleteItem(id)
}

function setProjectContextTarget(itemId: string | null) {
  projectContextTargetId.value = itemId
}

function getProjectContextParentId() {
  return projectContextTarget.value?.parent_id ?? null
}

async function createProjectItem(itemType: 'note' | 'chat' | 'agent_config') {
  const label = itemTypeLabels[itemType] ?? itemType
  const title = `未命名${label}`
  const parentId = getProjectContextParentId()

  try {
    if (itemType === 'note') {
      const note = await createNote({ title, parent_id: parentId, sort_order: 0 })
      handleSelect(note.workspace_item_id)
    } else if (itemType === 'chat') {
      const conv = await createConversation(title)
      handleSelect(conv.workspace_item_id)
    } else {
      const agent = await createAgent({ name: title })
      handleSelect(agent.workspace_item_id!)
    }
    await workspace.loadItems()
  } catch (e) {
    console.error('Failed to create workspace item:', e)
  }
}

async function deleteProjectContextTarget() {
  if (!projectContextTarget.value) return
  await handleDelete(projectContextTarget.value.id)
  projectContextTargetId.value = null
}

function syncToolFromRoute() {
  if (isSettingsRoute.value) activeLeftTool.value = 'settings'
  else activeLeftTool.value = 'project'
}

onMounted(() => {
  syncToolFromRoute()
  void health.refresh()
  void workspace.loadItems()
})

onBeforeUnmount(() => {
  stopPanelResize()
})

watch(
  () => route.name,
  () => syncToolFromRoute(),
)

watch(
  () => route.params.itemId,
  (itemId) => {
    if (itemId && typeof itemId === 'string' && isWorkspaceRoute.value) {
      workspace.selectItem(itemId)
    } else if (!itemId && route.name === 'workspace') {
      workspace.selectItem(null)
    }
  },
  { immediate: true },
)
</script>

<template>
  <div class="ide-shell">
    <div class="ide-workbench">
      <aside class="activity-bar activity-bar-left" aria-label="Tool windows">
        <WbIconButton
          class="activity-button"
          :active="activeLeftTool === 'project' && leftPanelOpen"
          title="项目"
          @click="selectLeftTool('project')"
        >
          <Files :size="21" />
        </WbIconButton>
        <WbIconButton
          class="activity-button"
          :active="activeLeftTool === 'settings' && leftPanelOpen"
          title="设置"
          @click="selectLeftTool('settings')"
        >
          <Settings :size="20" />
        </WbIconButton>
      </aside>

      <aside
        v-if="leftPanelOpen"
        class="tool-window left-tool-window"
        :style="{ width: `${leftPanelWidth}px` }"
      >
        <div class="tool-window-header">
          <div>
            <span class="tool-title">{{ activeLeftTool === 'project' ? '项目' : '设置' }}</span>
            <span class="tool-subtitle">llm_workbench</span>
          </div>
          <WbIconButton class="icon-button compact" size="compact" title="隐藏面板" @click="leftPanelOpen = false">
            <PanelLeftClose :size="16" />
          </WbIconButton>
        </div>

        <WbContextMenu v-if="activeLeftTool === 'project'">
          <template #trigger>
            <div class="project-panel" @contextmenu.capture="setProjectContextTarget(null)">
              <div v-if="rootItems.length === 0 && !workspace.isLoading" class="tool-empty">
                无工作区项目
              </div>

              <div class="tree-list" role="tree">
                <template v-for="item in rootItems" :key="item.id">
                  <div
                    class="tree-item"
                    :class="{ active: workspace.selectedItemId === item.id }"
                    role="treeitem"
                    tabindex="0"
                    @click="handleSelect(item.id)"
                    @contextmenu="setProjectContextTarget(item.id)"
                    @keydown.enter="handleSelect(item.id)"
                    @keydown.space.prevent="handleSelect(item.id)"
                  >
                    <span class="tree-chevron">›</span>
                    <span class="tree-icon">{{ itemTypeIcons[item.item_type] ?? '?' }}</span>
                    <span class="tree-label">{{ item.title }}</span>
                  </div>

                  <div
                    v-for="child in getChildren(item.id)"
                    :key="child.id"
                    class="tree-item child"
                    :class="{ active: workspace.selectedItemId === child.id }"
                    role="treeitem"
                    tabindex="0"
                    @click="handleSelect(child.id)"
                    @contextmenu="setProjectContextTarget(child.id)"
                    @keydown.enter="handleSelect(child.id)"
                    @keydown.space.prevent="handleSelect(child.id)"
                  >
                    <span class="tree-chevron"></span>
                    <span class="tree-icon">{{ itemTypeIcons[child.item_type] ?? '?' }}</span>
                    <span class="tree-label">{{ child.title }}</span>
                  </div>
                </template>
              </div>
            </div>
          </template>

            <WbContextMenuItem @click="createProjectItem('note')">
            <FileText :size="15" />
            新建笔记
          </WbContextMenuItem>
          <WbContextMenuItem @click="createProjectItem('chat')">
            <MessageSquareText :size="15" />
            新建聊天
          </WbContextMenuItem>
          <WbContextMenuItem @click="createProjectItem('agent_config')">
            <Bot :size="15" />
            新建代理
          </WbContextMenuItem>
          <div class="wb-menu-separator"></div>
          <WbContextMenuItem @click="workspace.loadItems()">
            <RefreshCcw :size="15" />
            刷新
          </WbContextMenuItem>
          <div class="wb-menu-separator"></div>
          <WbContextMenuItem :disabled="!projectContextTarget" @click="deleteProjectContextTarget()">
            <Trash2 :size="15" />
            删除
          </WbContextMenuItem>
        </WbContextMenu>

        <div v-else class="aux-panel">
          <RouterLink class="tool-link" to="/settings">打开设置</RouterLink>
          <div class="tool-empty">在中心面板中编辑设置</div>
        </div>
      </aside>

      <div
        v-if="leftPanelOpen"
        class="panel-resizer panel-resizer-left"
        role="separator"
        aria-orientation="vertical"
        aria-label="调整左侧边栏大小"
        @pointerdown="startPanelResize('left', $event)"
      ></div>

      <main class="editor-area">
        <section class="editor-surface" aria-label="Main editor">
          <SettingsPane v-if="isSettingsRoute" />

          <template v-else-if="isWorkspaceRoute">
            <div v-if="!selectedItem" class="empty-state ide-empty">
              请选择或创建一个工作区项目
            </div>

            <template v-else-if="selectedItem.item_type === 'note'">
              <NoteEditorPane :workspaceItem="selectedItem" />
            </template>
            <template v-else-if="selectedItem.item_type === 'chat'">
              <ChatPane :workspaceItem="selectedItem" />
            </template>
            <template v-else-if="selectedItem.item_type === 'agent_config'">
              <AgentConfigPane :workspaceItem="selectedItem" />
            </template>
            <template v-else>
              <div class="placeholder-pane">
                <div class="placeholder-icon">
                  <FilePlus2 :size="32" />
                </div>
                <h2>{{ itemTypeLabels[selectedItem.item_type] ?? selectedItem.item_type }}</h2>
                <p>{{ placeholderText[selectedItem.item_type] ?? '未知项目类型' }}</p>
                <button class="secondary-button" type="button" @click="handleDelete(selectedItem.id)">
                  删除项目
                </button>
              </div>
            </template>
          </template>
        </section>
      </main>

      <div
        v-if="rightPanelOpen"
        class="panel-resizer panel-resizer-right"
        role="separator"
        aria-orientation="vertical"
        aria-label="调整右侧边栏大小"
        @pointerdown="startPanelResize('right', $event)"
      ></div>

      <aside
        v-if="rightPanelOpen"
        class="tool-window right-tool-window"
        :style="{ width: `${rightPanelWidth}px` }"
      >
        <div class="tool-window-header">
          <div>
            <span class="tool-title">
              {{
                activeRightTool === 'outline'
                  ? '大纲'
                  : activeRightTool === 'notifications'
                    ? '通知'
                    : '数据库'
              }}
            </span>
            <span class="tool-subtitle">{{ selectedItem?.title ?? '工作区' }}</span>
          </div>
          <WbIconButton class="icon-button compact" size="compact"           title="隐藏面板" @click="rightPanelOpen = false">
            <PanelRightClose :size="16" />
          </WbIconButton>
        </div>

        <div v-if="activeRightTool === 'outline'" class="outline-panel">
          <div v-for="row in outlineRows" :key="row.label" class="outline-row">
            <span>{{ row.label }}</span>
            <strong>{{ row.value }}</strong>
          </div>
        </div>

        <div v-else-if="activeRightTool === 'notifications'" class="notifications-panel">
          <div class="notice-card">
            <span class="notice-title">后端</span>
            <span>{{ statusMessage }}</span>
          </div>
          <div class="notice-card">
            <span class="notice-title">数据库</span>
            <span>{{ databaseStatus }}</span>
          </div>
        </div>

        <div v-else class="database-panel">
          <div class="database-node">
            <Database :size="16" />
            postgres@localhost
          </div>
          <div class="database-node muted">database: llm_workbench</div>
          <div class="database-node muted">status: {{ databaseStatus }}</div>
        </div>
      </aside>

      <aside class="activity-bar activity-bar-right" aria-label="Right tool windows">
        <WbIconButton
          class="activity-button"
          :active="activeRightTool === 'outline' && rightPanelOpen"
          title="大纲"
          @click="selectRightTool('outline')"
        >
          <ListTree :size="20" />
        </WbIconButton>
        <WbIconButton
          class="activity-button"
          :active="activeRightTool === 'notifications' && rightPanelOpen"
          title="通知"
          @click="selectRightTool('notifications')"
        >
          <Bell :size="20" />
        </WbIconButton>
        <WbIconButton
          class="activity-button"
          :active="activeRightTool === 'database' && rightPanelOpen"
          title="数据库"
          @click="selectRightTool('database')"
        >
          <Database :size="20" />
        </WbIconButton>
      </aside>
    </div>

    <footer class="ide-statusbar" role="status" aria-live="polite">
      <span>数据库: {{ databaseStatus }}</span>
      <span>后端: {{ backendStatus }}</span>
      <span v-if="health.lastCheckedAt">上次检查 {{ health.lastCheckedAt }}</span>
      <span>UTF-8</span>
      <span>LF</span>
      <span>Cargo Check</span>
      <span class="status-good">无需认证</span>
    </footer>
  </div>
</template>
