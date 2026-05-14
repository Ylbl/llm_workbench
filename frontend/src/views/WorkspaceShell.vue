<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import {
  Bell,
  Bot,
  BookOpen,
  Braces,
  Database,
  FilePlus2,
  FileText,
  Files,
  ListTree,
  MessageSquareText,
  PanelLeftClose,
  PanelRightClose,
  RefreshCcw,
  Search,
  Settings,
  Trash2,
  Workflow,
} from 'lucide-vue-next'
import { useHealthStore } from '../stores/health'
import { useWorkspaceStore } from '../stores/workspace'
import type { WorkspaceItem } from '../api/workspace'
import SettingsPane from './SettingsPane.vue'
import LlmSettingsPane from './LlmSettingsPane.vue'
import MathDemo from './MathDemo.vue'
import NoteEditorPane from '../components/NoteEditorPane.vue'
import ChatPane from '../components/ChatPane.vue'
import AgentConfigPane from '../components/AgentConfigPane.vue'
import { WbContextMenu, WbContextMenuItem, WbIconButton } from '../ui'

type LeftTool = 'project' | 'settings' | 'llm' | 'math' | 'search'
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
  note: 'Note',
  chat: 'Chat',
  agent_config: 'Agent',
  file: 'File',
  task: 'Task',
  settings_view: 'Settings',
}

const itemTypeIcons: Record<string, string> = {
  note: 'N',
  chat: 'C',
  agent_config: 'A',
  file: 'F',
  task: 'T',
  settings_view: 'S',
}

const placeholderText: Record<string, string> = {
  note: 'Note Editor',
  chat: 'Chat Pane',
  agent_config: 'Agent Config',
  file: 'File Pane (Batch 13)',
  task: 'Task Pane (Batch 14)',
  settings_view: 'Settings View',
}

const isSettingsRoute = computed(() => route.name === 'settings')
const isLlmRoute = computed(() => route.name === 'llm-settings')
const isMathRoute = computed(() => route.name === 'math-demo')
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
  if (health.isLoading) return 'Checking backend health'
  if (health.data) return `${health.data.status} ${health.data.app.host}:${health.data.app.port}`
  return 'Backend health not checked'
})

const editorTitle = computed(() => {
  if (isSettingsRoute.value) return 'Settings'
  if (isLlmRoute.value) return 'LLM Settings'
  if (isMathRoute.value) return 'Math Demo'
  return selectedItem.value?.title ?? 'Workspace'
})

const editorSubtitle = computed(() => {
  if (isWorkspaceRoute.value && selectedItem.value) {
    return itemTypeLabels[selectedItem.value.item_type] ?? selectedItem.value.item_type
  }

  if (isSettingsRoute.value) return 'Raw persisted JSON'
  if (isLlmRoute.value) return 'Providers and request profiles'
  if (isMathRoute.value) return 'Shared formula renderer'
  return 'Project workspace'
})

const activeRouteLabel = computed(() => {
  if (isSettingsRoute.value) return 'settings'
  if (isLlmRoute.value) return 'llm'
  if (isMathRoute.value) return 'math'
  return 'workspace'
})

const outlineRows = computed(() => {
  if (!selectedItem.value) {
    return [
      { label: 'Workspace', value: `${workspace.items.length} items` },
      { label: 'Backend', value: backendStatus.value },
      { label: 'Database', value: databaseStatus.value },
    ]
  }

  return [
    { label: 'Title', value: selectedItem.value.title },
    { label: 'Type', value: selectedItem.value.item_type },
    { label: 'Parent', value: selectedItem.value.parent_id ?? 'root' },
    { label: 'Sort', value: String(selectedItem.value.sort_order) },
    { label: 'Updated', value: new Date(selectedItem.value.updated_at).toLocaleString() },
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
  if (tool === 'llm') void router.push('/llm-settings')
  if (tool === 'math') void router.push('/math-demo')
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
  const item = await workspace.createItem({
    item_type: itemType,
    title: `Untitled ${label}`,
    parent_id: getProjectContextParentId(),
  })
  if (item) {
    handleSelect(item.id)
  }
}

async function deleteProjectContextTarget() {
  if (!projectContextTarget.value) return
  await handleDelete(projectContextTarget.value.id)
  projectContextTargetId.value = null
}

function syncToolFromRoute() {
  if (isSettingsRoute.value) activeLeftTool.value = 'settings'
  else if (isLlmRoute.value) activeLeftTool.value = 'llm'
  else if (isMathRoute.value) activeLeftTool.value = 'math'
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
          title="Project"
          @click="selectLeftTool('project')"
        >
          <Files :size="21" />
        </WbIconButton>
        <WbIconButton
          class="activity-button"
          :active="activeLeftTool === 'settings' && leftPanelOpen"
          title="Settings"
          @click="selectLeftTool('settings')"
        >
          <Settings :size="20" />
        </WbIconButton>
        <WbIconButton
          class="activity-button"
          :active="activeLeftTool === 'llm' && leftPanelOpen"
          title="LLM"
          @click="selectLeftTool('llm')"
        >
          <Workflow :size="20" />
        </WbIconButton>
        <WbIconButton
          class="activity-button"
          :active="activeLeftTool === 'math' && leftPanelOpen"
          title="Math"
          @click="selectLeftTool('math')"
        >
          <Braces :size="20" />
        </WbIconButton>
        <WbIconButton
          class="activity-button"
          :active="activeLeftTool === 'search' && leftPanelOpen"
          title="Search"
          @click="selectLeftTool('search')"
        >
          <Search :size="20" />
        </WbIconButton>
      </aside>

      <aside
        v-if="leftPanelOpen"
        class="tool-window left-tool-window"
        :style="{ width: `${leftPanelWidth}px` }"
      >
        <div class="tool-window-header">
          <div>
            <span class="tool-title">
              {{
                activeLeftTool === 'project'
                  ? 'Project'
                  : activeLeftTool === 'settings'
                    ? 'Settings'
                    : activeLeftTool === 'llm'
                      ? 'LLM'
                      : activeLeftTool === 'math'
                        ? 'Math'
                        : 'Search'
              }}
            </span>
            <span class="tool-subtitle">llm_workbench</span>
          </div>
          <WbIconButton class="icon-button compact" size="compact" title="Hide tool window" @click="leftPanelOpen = false">
            <PanelLeftClose :size="16" />
          </WbIconButton>
        </div>

        <WbContextMenu v-if="activeLeftTool === 'project'">
          <template #trigger>
            <div class="project-panel" @contextmenu.capture="setProjectContextTarget(null)">
              <div v-if="rootItems.length === 0 && !workspace.isLoading" class="tool-empty">
                No workspace items
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
            New Note
          </WbContextMenuItem>
          <WbContextMenuItem @click="createProjectItem('chat')">
            <MessageSquareText :size="15" />
            New Chat
          </WbContextMenuItem>
          <WbContextMenuItem @click="createProjectItem('agent_config')">
            <Bot :size="15" />
            New Agent
          </WbContextMenuItem>
          <div class="wb-menu-separator"></div>
          <WbContextMenuItem @click="workspace.loadItems()">
            <RefreshCcw :size="15" />
            Refresh
          </WbContextMenuItem>
          <div class="wb-menu-separator"></div>
          <WbContextMenuItem :disabled="!projectContextTarget" @click="deleteProjectContextTarget()">
            <Trash2 :size="15" />
            Delete
          </WbContextMenuItem>
        </WbContextMenu>

        <div v-else-if="activeLeftTool === 'search'" class="aux-panel">
          <input class="tool-search-input" type="search" placeholder="Search everywhere" />
          <div class="tool-empty">Search UI placeholder</div>
        </div>

        <div v-else class="aux-panel">
          <RouterLink
            class="tool-link"
            :to="
              activeLeftTool === 'settings'
                ? '/settings'
                : activeLeftTool === 'llm'
                  ? '/llm-settings'
                  : '/math-demo'
            "
          >
            {{
              activeLeftTool === 'settings'
                ? 'Open Settings'
                : activeLeftTool === 'llm'
                  ? 'Open LLM Settings'
                  : 'Open Math Demo'
            }}
          </RouterLink>
          <div class="tool-empty">Tool window content can be expanded here</div>
        </div>
      </aside>

      <div
        v-if="leftPanelOpen"
        class="panel-resizer panel-resizer-left"
        role="separator"
        aria-orientation="vertical"
        aria-label="Resize left sidebar"
        @pointerdown="startPanelResize('left', $event)"
      ></div>

      <main class="editor-area">
        <div class="editor-tabs">
          <button class="editor-tab active" type="button">
            <component
              :is="
                isSettingsRoute
                  ? Settings
                  : isLlmRoute
                    ? Workflow
                    : isMathRoute
                      ? Braces
                      : selectedItem?.item_type === 'chat'
                        ? MessageSquareText
                        : BookOpen
              "
              :size="15"
            />
            <span>{{ editorTitle }}</span>
            <span class="tab-close">×</span>
          </button>
        </div>

        <div class="editor-toolbar">
          <span class="breadcrumb">
            llm_workbench / {{ activeRouteLabel }} / {{ editorTitle }}
          </span>
          <span class="editor-context">{{ editorSubtitle }}</span>
        </div>

        <section class="editor-surface" aria-label="Main editor">
          <SettingsPane v-if="isSettingsRoute" />
          <LlmSettingsPane v-else-if="isLlmRoute" />
          <MathDemo v-else-if="isMathRoute" />

          <template v-else-if="isWorkspaceRoute">
            <div v-if="!selectedItem" class="empty-state ide-empty">
              Select or create a workspace item
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
                <p>{{ placeholderText[selectedItem.item_type] ?? 'Unknown item type' }}</p>
                <button class="secondary-button" type="button" @click="handleDelete(selectedItem.id)">
                  Delete item
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
        aria-label="Resize right sidebar"
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
                  ? 'Outline'
                  : activeRightTool === 'notifications'
                    ? 'Notifications'
                    : 'Database'
              }}
            </span>
            <span class="tool-subtitle">{{ editorTitle }}</span>
          </div>
          <WbIconButton class="icon-button compact" size="compact" title="Hide tool window" @click="rightPanelOpen = false">
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
            <span class="notice-title">Backend</span>
            <span>{{ statusMessage }}</span>
          </div>
          <div class="notice-card">
            <span class="notice-title">Database</span>
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
          title="Outline"
          @click="selectRightTool('outline')"
        >
          <ListTree :size="20" />
        </WbIconButton>
        <WbIconButton
          class="activity-button"
          :active="activeRightTool === 'notifications' && rightPanelOpen"
          title="Notifications"
          @click="selectRightTool('notifications')"
        >
          <Bell :size="20" />
        </WbIconButton>
        <WbIconButton
          class="activity-button"
          :active="activeRightTool === 'database' && rightPanelOpen"
          title="Database"
          @click="selectRightTool('database')"
        >
          <Database :size="20" />
        </WbIconButton>
      </aside>
    </div>

    <footer class="ide-statusbar" role="status" aria-live="polite">
      <span>Database: {{ databaseStatus }}</span>
      <span>Backend: {{ backendStatus }}</span>
      <span v-if="health.lastCheckedAt">Last check {{ health.lastCheckedAt }}</span>
      <span>UTF-8</span>
      <span>LF</span>
      <span>Cargo Check</span>
      <span class="status-good">No auth</span>
    </footer>
  </div>
</template>
