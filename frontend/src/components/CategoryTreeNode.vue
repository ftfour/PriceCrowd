<template>
  <div class="space-y-1">
    <div class="flex items-center gap-2">
      <button v-if="node.children && node.children.length" @click="onToggle" type="button" class="text-slate-600 hover:text-slate-900 text-xs w-5">
        {{ isExpanded ? '▾' : '▸' }}
      </button>
      <span v-else class="inline-block w-5"></span>
      <label class="flex items-center gap-2">
        <input v-if="selectable" type="checkbox" :checked="isSelected" @change="onCheck" />
        <span class="text-sm text-slate-800">{{ node.name }}</span>
      </label>
    </div>
    <div v-if="isExpanded && node.children && node.children.length" class="pl-6 border-l border-slate-200 ml-[10px]">
      <CategoryTreeNode
        v-for="child in node.children"
        :key="child.id"
        :node="child"
        :selectable="selectable"
        :expanded-ids="expandedIds"
        :selected-ids="selectedIds"
        @toggle="$emit('toggle', $event)"
        @select="$emit('select', $event)"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
defineOptions({ name: 'CategoryTreeNode' });

type Node = { id: string; name: string; children?: Node[] };

const props = defineProps<{ 
  node: Node;
  selectable?: boolean;
  expandedIds: Set<string>;
  selectedIds: Set<string>;
}>();

const emit = defineEmits<{ (e:'toggle', id:string): void; (e:'select', id:string): void }>();

const isExpanded = computed(()=> props.expandedIds.has(props.node.id));
const isSelected = computed(()=> props.selectedIds.has(props.node.id));

function onToggle(){ emit('toggle', props.node.id); }
function onCheck(){ emit('select', props.node.id); }
</script>

