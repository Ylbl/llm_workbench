<script setup lang="ts">
import { computed } from 'vue'
import type { WorkspaceItem } from '../api/workspace'

const props = defineProps<{
  items: WorkspaceItem[]
  selectedId: string | null
  parentId?: string | null
}>()

const emit = defineEmits<{
  select: [id: string]
  delete: [id: string]
  create: [parentId: string | null, itemType: string]
}>()

interface TreeNode {
  item: WorkspaceItem
  children: TreeNode[]
}

const tree = computed<TreeNode[]>(() => {
  return buildTree(props.items, props.parentId ?? null)
})

function buildTree(items: WorkspaceItem[], parentId: string | null): TreeNode[] {
  return items
    .filter((item) => item.parent_id === parentId)
    .sort((a, b) => a.sort_order - b.sort_order)
    .map((item) => ({
      item,
      children: buildTree(items, item.id),
    }))
}

const itemTypeLabels: Record<string, string> = {
  note: 'Note',
  chat: 'Chat',
  agent_config: 'Agent',
  file: 'File',
  task: 'Task',
  settings_view: 'Settings',
}
</script>

<template>
  <div class="workspace-tree" role="tree" aria-label="Workspace items">
    <div v-if="tree.length === 0" class="tree-empty muted-label">No items</div>

    <div v-for="node in tree" :key="node.item.id" class="tree-node">
      <button
        class="tree-item"
        :class="{ 'is-selected': node.item.id === selectedId }"
        type="button"
        role="treeitem"
        :aria-selected="node.item.id === selectedId"
        @click="emit('select', node.item.id)"
      >
        <span class="tree-item-type">{{ itemTypeLabels[node.item.item_type] ?? node.item.item_type }}</span>
        <span class="tree-item-title">{{ node.item.title }}</span>

        <button
          class="tree-item-delete"
          type="button"
          title="Delete"
          @click.stop="emit('delete', node.item.id)"
        >
          &times;
        </button>
      </button>

      <WorkspaceTree
        v-if="node.children.length > 0"
        :items="items"
        :selectedId="selectedId"
        :parentId="node.item.id"
        @select="(id: string) => emit('select', id)"
        @delete="(id: string) => emit('delete', id)"
        @create="(parentId: string | null, itemType: string) => emit('create', parentId, itemType)"
      />
    </div>
  </div>
</template>

<style scoped>
.workspace-tree {
  padding-left: 0;
}

.tree-empty {
  padding: 8px 12px;
  font-size: 13px;
}

.tree-node {
  margin: 0;
}

.tree-item {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  padding: 6px 12px;
  border: none;
  background: none;
  color: var(--color-text);
  font-size: 14px;
  text-align: left;
  cursor: pointer;
  border-radius: 4px;
}

.tree-item:hover {
  background: var(--color-surface);
}

.tree-item.is-selected {
  background: var(--color-accent);
  color: #fff;
}

.tree-item.is-selected .tree-item-type {
  opacity: 0.8;
}

.tree-item.is-selected .tree-item-delete {
  color: rgba(255, 255, 255, 0.7);
}

.tree-item-type {
  font-size: 11px;
  font-weight: 600;
  text-transform: uppercase;
  opacity: 0.5;
  min-width: 48px;
}

.tree-item-title {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.tree-item-delete {
  display: none;
  padding: 0 4px;
  border: none;
  background: none;
  color: var(--color-muted);
  font-size: 16px;
  line-height: 1;
  cursor: pointer;
  border-radius: 2px;
}

.tree-item:hover .tree-item-delete {
  display: inline;
}

.tree-item-delete:hover {
  background: var(--color-danger);
  color: #fff;
}

.workspace-tree .workspace-tree {
  padding-left: 16px;
}
</style>
