<template>
  <section class="grid grid-cols-12 gap-6">
    <!-- Товары с категориями -->
    <div class="col-span-12 lg:col-span-9 space-y-4">
      <h1 class="text-2xl font-semibold">Корзина</h1>
      <div class="rounded-lg border bg-white p-4">
        <div class="flex items-center justify-between mb-3">
          <div class="text-slate-600">Выберите товары (по категориям)</div>
          <div class="text-xs text-slate-500">Отмечено: {{ selectedProducts.size }}</div>
        </div>
        <div class="space-y-2">
          <div v-for="node in tree" :key="node.id" class="">
            <CategoryNode :node="node" :products-by-cat="productsByCat" :selected-products="selectedProducts"
              :expanded="expandedCats" @toggle="toggleCat" @toggle-product="toggleProduct" />
          </div>
        </div>
      </div>

      <div class="flex items-center gap-3">
        <button @click="calculate" :disabled="selectedProducts.size===0 || selectedStores.size===0"
          class="rounded-md bg-blue-600 text-white px-4 py-2 text-sm disabled:opacity-60">
          Рассчитать выгодную корзину
        </button>
        <div v-if="calcError" class="text-sm text-rose-600">{{ calcError }}</div>
      </div>

      <!-- Результат расчёта -->
      <div v-if="result.items.length" class="rounded-lg border bg-white p-4 space-y-3">
        <h2 class="text-lg font-semibold">Результат</h2>
        <div class="overflow-auto">
          <table class="w-full text-left text-sm">
            <thead class="bg-slate-50 text-slate-500">
              <tr>
                <th class="px-3 py-2">Товар</th>
                <th class="px-3 py-2">Магазин</th>
                <th class="px-3 py-2">Цена</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="it in result.items" :key="it.product_id" class="border-t border-slate-100">
                <td class="px-3 py-2">
                  <RouterLink :to="`/products/${it.product_id}`" class="text-slate-800 hover:underline">{{ it.product_title }}</RouterLink>
                </td>
                <td class="px-3 py-2">
                  <RouterLink v-if="it.store_id" :to="`/stores/${it.store_id}`" class="text-blue-600 hover:underline">{{ it.store_name }}</RouterLink>
                  <span v-else class="text-slate-400">нет в выбранных магазинах</span>
                </td>
                <td class="px-3 py-2">{{ it.price!=null ? Number(it.price).toLocaleString('ru-RU') + ' ₽' : '—' }}</td>
              </tr>
            </tbody>
          </table>
        </div>
        <div class="grid sm:grid-cols-2 gap-3">
          <div class="rounded border p-3">
            <div class="text-sm text-slate-500">Итого по магазинам</div>
            <ul class="mt-2 text-sm text-slate-700 space-y-1">
              <li v-for="s in result.summary" :key="s.store_id" class="flex items-center justify-between">
                <RouterLink :to="`/stores/${s.store_id}`" class="text-slate-800 hover:underline">{{ s.store_name }}</RouterLink>
                <div class="font-semibold">{{ Number(s.total).toLocaleString('ru-RU') }} ₽</div>
              </li>
            </ul>
          </div>
          <div class="rounded border p-3">
            <div class="text-sm text-slate-500">Итого</div>
            <div class="text-2xl font-semibold text-emerald-600 mt-1">{{ Number(result.total).toLocaleString('ru-RU') }} ₽</div>
          </div>
        </div>
      </div>
    </div>

    <!-- Сайдбар магазинов -->
    <aside class="col-span-12 lg:col-span-3 space-y-4">
      <div class="rounded-lg border bg-white p-4">
        <div class="flex items-center justify-between mb-2">
          <h2 class="text-lg font-semibold">Магазины</h2>
          <button @click="toggleAllStores" class="text-xs text-blue-600 hover:underline">{{ allStoresSelected ? 'Снять все' : 'Выбрать все' }}</button>
        </div>
        <div class="space-y-2 max-h-[50vh] overflow-auto pr-1">
          <label v-for="s in stores" :key="s._id" class="flex items-center justify-between gap-3">
            <span class="text-sm text-slate-800 truncate">{{ s.name }}</span>
            <input type="checkbox" :checked="selectedStores.has(s._id)" @change="toggleStore(s._id)" />
          </label>
        </div>
      </div>
    </aside>
  </section>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue';
import { RouterLink } from 'vue-router';

const API = import.meta.env.VITE_API_URL || 'http://localhost:8080';

// Data
const products = ref<any[]>([]);
const categories = ref<any[]>([]);
const stores = ref<any[]>([]);
const selectedProducts = ref<Set<string>>(new Set());
const expandedCats = ref<Set<string>>(new Set());
const selectedStores = ref<Set<string>>(new Set());
const storeItems = ref<Record<string, Record<string, number>>>({}); // store_id -> { product_id: price }
const storeNameById = ref<Record<string, string>>({});
const calcError = ref<string | null>(null);

// Helpers
function idOf(x:any){ return typeof x === 'string' ? x : x?.$oid; }

// Load data
async function loadProducts(){
  const res = await fetch(`${API}/products`);
  if (res.ok){
    const arr = await res.json();
    products.value = arr.map((p:any)=> ({ _id: idOf(p._id), title: p.title, category_ids: (p.category_ids||[]).map(idOf) }));
  }
}
async function loadCategories(){
  const res = await fetch(`${API}/categories`);
  if (res.ok){
    const arr = await res.json();
    categories.value = arr.map((c:any)=> ({ _id: idOf(c._id), name: c.name, parent_ids: (c.parent_ids||[]).map(idOf) }));
  }
}
async function loadStores(){
  const res = await fetch(`${API}/stores`);
  if (res.ok){
    const arr = await res.json();
    stores.value = arr.map((s:any)=> ({ _id: idOf(s._id), name: s.name }));
    storeNameById.value = Object.fromEntries(stores.value.map((s:any)=> [s._id, s.name]));
  }
}

onMounted(async()=>{ await Promise.all([loadProducts(), loadCategories(), loadStores()]); });

// Build category tree
const normalizedCats = computed(()=> categories.value.map(c=> ({ id: c._id, name: c.name, parents: c.parent_ids })));
const childrenMap = computed(()=>{
  const m = new Map<string, {id:string;name:string}[]>();
  for (const c of normalizedCats.value){
    for (const p of (c.parents||[])){
      const arr = m.get(p)||[]; arr.push({id:c.id,name:c.name}); m.set(p,arr);
    }
  }
  for (const [k,arr] of m){ arr.sort((a,b)=> a.name.localeCompare(b.name)); m.set(k,arr); }
  return m;
});
const roots = computed(()=> normalizedCats.value.filter(c=> !c.parents || c.parents.length===0).map(c=> ({id:c.id,name:c.name})).sort((a,b)=> a.name.localeCompare(b.name)));
const productsByCat = computed(()=>{
  const mp: Record<string, { _id:string; title:string }[]> = {};
  for (const p of products.value){
    for (const cid of (p.category_ids||[])){
      if (!mp[cid]) mp[cid]=[];
      mp[cid].push({_id:p._id, title:p.title});
    }
  }
  for (const cid in mp){ mp[cid].sort((a,b)=> a.title.localeCompare(b.title)); }
  return mp;
});
const tree = computed(()=> roots.value.map(buildNode));
function buildNode(root:{id:string;name:string}){
  const n:any = { id: root.id, name: root.name, children: [] as any[] };
  const kids = childrenMap.value.get(root.id)||[];
  n.children = kids.map(buildNode);
  return n;
}

// Expand/collapse and select
function toggleCat(id:string){ const s = new Set(expandedCats.value); if (s.has(id)) s.delete(id); else s.add(id); expandedCats.value = s; }
function toggleProduct(id:string){ const s = new Set(selectedProducts.value); if (s.has(id)) s.delete(id); else s.add(id); selectedProducts.value = s; }

// Stores selection
function toggleStore(id:string){ const s = new Set(selectedStores.value); if (s.has(id)) s.delete(id); else s.add(id); selectedStores.value = s; }
const allStoresSelected = computed(()=> selectedStores.value.size===stores.value.length && stores.value.length>0);
function toggleAllStores(){ if (allStoresSelected.value) selectedStores.value = new Set(); else selectedStores.value = new Set(stores.value.map((s:any)=> s._id)); }

// Fetch store items for selected stores
async function ensureStoreItemsLoaded(){
  for (const sid of selectedStores.value){
    if (storeItems.value[sid]) continue;
    const res = await fetch(`${API}/stores/${sid}/products`);
    if (res.ok){
      const arr = await res.json();
      const mp: Record<string, number> = {};
      for (const it of arr){ const pid = idOf(it.product_id); mp[pid] = Number(it.price); }
      storeItems.value[sid] = mp;
    }
  }
}

// Calculation
const result = ref<{ items: any[]; summary: any[]; total: number }>({ items: [], summary: [], total: 0 });
async function calculate(){
  calcError.value = null;
  if (selectedProducts.value.size===0) { calcError.value = 'Выберите товары'; return; }
  if (selectedStores.value.size===0) { calcError.value = 'Выберите магазины'; return; }
  await ensureStoreItemsLoaded();
  const items:any[] = [];
  const totals: Record<string, number> = {};
  for (const pid of selectedProducts.value){
    // find cheapest among selected stores
    let best: { store_id: string|null; store_name?: string; price: number|null } = { store_id: null, price: null };
    for (const sid of selectedStores.value){
      const price = storeItems.value[sid]?.[pid];
      if (price==null) continue;
      if (best.price==null || price < best.price){ best = { store_id: sid, store_name: storeNameById.value[sid], price }; }
    }
    const pInfo = products.value.find(x=> x._id===pid);
    items.push({ product_id: pid, product_title: pInfo?.title || pid, store_id: best.store_id, store_name: best.store_name, price: best.price });
    if (best.store_id && best.price!=null){ totals[best.store_id] = (totals[best.store_id]||0) + best.price; }
  }
  const summary = Object.entries(totals).map(([sid,total])=> ({ store_id: sid, store_name: storeNameById.value[sid], total }));
  const total = summary.reduce((a,x)=> a + x.total, 0);
  result.value = { items, summary, total };
}
</script>

<script lang="ts">
// Inline component: category with products and nested categories
import { defineComponent } from 'vue';
export default {
  components: {
    CategoryNode: defineComponent({
      name: 'CategoryNode',
      props: {
        node: { type: Object, required: true },
        productsByCat: { type: Object, required: true },
        selectedProducts: { type: Object, required: true },
        expanded: { type: Object, required: true },
      },
      emits: ['toggle','toggle-product'],
      setup(props, { emit }){
        function toggle(){ emit('toggle', (props as any).node.id); }
        function onProd(id:string){ emit('toggle-product', id); }
        return { toggle, onProd };
      },
      template: `
      <div class="space-y-1">
        <div class="flex items-center gap-2">
          <button type="button" class="text-slate-600 hover:text-slate-900 text-xs w-5" @click="toggle">
            {{ expanded.has(node.id) ? '▾' : '▸' }}
          </button>
          <div class="font-medium text-sm text-slate-800">{{ node.name }}</div>
        </div>
        <div v-if="expanded.has(node.id)" class="pl-6 border-l border-slate-200 ml-[10px] space-y-2">
          <div v-if="productsByCat[node.id] && productsByCat[node.id].length" class="space-y-1">
            <label v-for="p in productsByCat[node.id]" :key="p._id" class="flex items-center justify-between gap-3 text-sm">
              <span class="truncate">{{ p.title }}</span>
              <input type="checkbox" :checked="selectedProducts.has(p._id)" @change="onProd(p._id)" />
            </label>
          </div>
          <CategoryNode v-for="child in node.children" :key="child.id" :node="child" :products-by-cat="productsByCat"
            :selected-products="selectedProducts" :expanded="expanded" @toggle="$emit('toggle', $event)" @toggle-product="$emit('toggle-product', $event)" />
        </div>
      </div>
      `
    })
  }
}
</script>

