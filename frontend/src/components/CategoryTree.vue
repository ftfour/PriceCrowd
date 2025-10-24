<template>
  <div class="space-y-2">
    <div v-for="root in tree" :key="root.id" class="">
      <CategoryTreeNode
        :node="root"
        :selectable="selectable"
        :expanded-ids="expandedSet"
        :selected-ids="selectedSet"
        @toggle="toggle"
        @select="toggleSelect"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import CategoryTreeNode from './CategoryTreeNode.vue';

type RawCat = { _id: any; name: string; parent_ids?: any[] };
type Node = { id: string; name: string; children: Node[] };

const props = defineProps<{ 
  categories: RawCat[];
  modelValue?: string[];
  selectable?: boolean;
}>();
const emit = defineEmits<{ (e:'update:modelValue', v: string[]): void }>();

const selectable = computed(()=> props.selectable ?? true);

function idOf(x:any): string { return typeof x === 'string' ? x : x?.$oid; }

const normalized = computed(()=> props.categories.map(c => ({
  id: idOf(c._id),
  name: c.name,
  parents: (c.parent_ids||[]).map(idOf),
})));

const childrenMap = computed(()=> {
  const map = new Map<string, Node[]>();
  for (const c of normalized.value){
    if (c.parents.length===0){ continue; }
    for (const p of c.parents){
      const arr = map.get(p) || [];
      arr.push({ id: c.id, name: c.name, children: [] });
      map.set(p, arr);
    }
  }
  // sort children
  for (const [k, arr] of map){ arr.sort((a,b)=> a.name.localeCompare(b.name)); map.set(k, arr); }
  return map;
});

const tree = computed<Node[]>(()=> {
  const roots: Node[] = normalized.value
    .filter(c => c.parents.length===0)
    .map(c => ({ id: c.id, name: c.name, children: [] }))
    .sort((a,b)=> a.name.localeCompare(b.name));
  function attach(n: Node, seen: Set<string>){
    if (seen.has(n.id)) return; // avoid cycles
    seen.add(n.id);
    const kids = childrenMap.value.get(n.id) || [];
    n.children = kids.map(k => ({ ...k, children: [] }));
    for (const child of n.children){ attach(child, new Set(seen)); }
  }
  for (const r of roots){ attach(r, new Set()); }
  return roots;
});

const expandedSet = ref<Set<string>>(new Set());
watch(tree, (t)=>{
  const s = new Set<string>();
  for (const r of t){ s.add(r.id); }
  expandedSet.value = s;
}, { immediate: true });

const selectedSet = ref<Set<string>>(new Set(props.modelValue || []));
watch(()=> props.modelValue, (v)=>{
  selectedSet.value = new Set(v || []);
}, { immediate: true });

function toggle(id:string){
  const s = new Set(expandedSet.value);
  if (s.has(id)) s.delete(id); else s.add(id);
  expandedSet.value = s;
}

function toggleSelect(id:string){
  const s = new Set(selectedSet.value);
  if (s.has(id)) s.delete(id); else s.add(id);
  selectedSet.value = s;
  emit('update:modelValue', Array.from(s));
}
</script>

