<template>
  <section class="space-y-4">
    <header class="flex items-center justify-between">
      <h1 class="text-2xl font-semibold">📄 Операции</h1>
      <div class="text-sm text-slate-500">Всего: {{ ops.length }}</div>
    </header>

    <div class="rounded-lg border overflow-hidden bg-white">
      <table class="min-w-full text-sm">
        <thead class="bg-slate-50 text-left">
          <tr>
            <th class="px-3 py-2 border-b w-8"></th>
            <th class="px-3 py-2 border-b w-48">Дата</th>
            <th class="px-3 py-2 border-b">Продавец</th>
            <th class="px-3 py-2 border-b w-32">Сумма</th>
            <th class="px-3 py-2 border-b w-28">Статус</th>
            <th class="px-3 py-2 border-b w-56">Действия</th>
          </tr>
        </thead>
        <tbody>
          <template v-for="o in ops" :key="o.id">
            <tr class="hover:bg-slate-50">
              <td class="px-3 py-2 border-b align-top">
                <button
                  class="text-slate-600 hover:text-black"
                  @click="toggle(o.id)"
                  :title="expanded[o.id] ? 'Скрыть' : 'Показать детали'"
                >
                  <span v-if="expanded[o.id]">▾</span>
                  <span v-else>▸</span>
                </button>
              </td>
              <td class="px-3 py-2 border-b whitespace-nowrap align-top">{{ formatDate(o.date) }}</td>
              <td class="px-3 py-2 border-b align-top">
                <div class="font-medium">{{ o.seller }}</div>
                <div class="text-xs text-slate-500">ID: {{ o.id }}</div>
                <div v-if="o.qr" class="text-xs text-slate-500 break-all">QR: {{ o.qr }}</div>
              </td>
              <td class="px-3 py-2 border-b align-top">{{ money(o.amount) }}</td>
              <td class="px-3 py-2 border-b align-top">
                <span :class="o.status==='draft' ? 'text-amber-700' : 'text-green-700'">
                  {{ o.status === 'draft' ? 'Черновик' : 'Проведено' }}
                </span>
              </td>
              <td class="px-3 py-2 border-b align-top">
                <button
                  class="mr-2 px-3 py-1.5 rounded-md text-white bg-green-600 hover:bg-green-700"
                  @click="postOperation(o.id)"
                >Провести</button>
                <button
                  class="px-3 py-1.5 rounded-md text-white bg-red-600 hover:bg-red-700"
                  @click="removeOperation(o.id)"
                >Удалить</button>
              </td>
            </tr>
            <tr v-if="expanded[o.id]">
              <td colspan="6" class="px-3 py-3 border-b bg-slate-50">
                <div class="flex items-center justify-between mb-2">
                  <div class="text-sm text-slate-700">Позиции чека ({{ o.items.length }})</div>
                  <div class="flex items-center gap-2">
                    <label class="text-sm text-slate-700">Магазин:</label>
                    <select v-model="storeSelect[o.id]" class="border rounded px-2 py-1 text-sm bg-white text-slate-900">
                      <option value="">— Выберите —</option>
                      <option v-for="s in stores" :key="s.id" :value="s.id">{{ s.name }}</option>
                    </select>
                    <button class="px-3 py-1.5 rounded-md text-white bg-blue-600 hover:bg-blue-700" @click="onCreateStore(o.id, o.seller)">Новый магазин</button>
                    <button class="px-3 py-1.5 rounded-md text-white bg-green-600 hover:bg-green-700" @click="onSaveOperation(o)">Сохранить</button>
                  </div>
                </div>
                <div class="flex items-center gap-2 mb-3">
                  <label class="text-sm text-slate-700">Категория:</label>
                  <select v-model="catFilter[o.id]" class="border rounded px-2 py-1 text-sm bg-white text-slate-900" @change="onCategoryChange(o.id)">
                    <option value="">Все</option>
                    <option v-for="c in topCategories" :key="c.id" :value="c.id">{{ c.name }}</option>
                  </select>
                  <label class="text-sm text-slate-700">Подкатегория:</label>
                  <select v-model="subcatFilter[o.id]" class="border rounded px-2 py-1 text-sm bg-white text-slate-900">
                    <option value="">Все</option>
                    <option v-for="c in subcategoriesOf(catFilter[o.id])" :key="c.id" :value="c.id">{{ c.name }}</option>
                  </select>
                </div>
                <div class="overflow-x-auto">
                  <table class="min-w-full text-xs">
                    <thead>
                      <tr class="text-left text-slate-600">
                        <th class="px-2 py-1 border-b">Наименование</th>
                        <th class="px-2 py-1 border-b w-24">Кол-во</th>
                        <th class="px-2 py-1 border-b w-28">Цена</th>
                        <th class="px-2 py-1 border-b w-28">Сумма</th>
                      </tr>
                    </thead>
                    <tbody>
                      <ItemRow v-for="(it, idx) in o.items" :key="idx" :op-id="o.id" :index="idx" :item="it" :products="filteredProducts(o.id)" :store-id="o.store_id || storeSelect[o.id] || null" v-model:selectedProductId="pendingProducts[o.id][idx]" />
                      <tr>
                        <td colspan="3" class="px-2 py-2 text-right font-medium">Итого по позициям:</td>
                        <td class="px-2 py-2 font-medium">{{ money(sumItems(o)) }}</td>
                      </tr>
                    </tbody>
                  </table>
                </div>
                <div class="mt-4 space-y-2">
                  <details>
                    <summary class="cursor-pointer text-sm text-slate-700">HTML из API (если доступно)</summary>
                    <div class="bg-white border rounded p-2 text-xs overflow-x-auto mt-2" v-html="rawHtml(o.raw)"></div>
                  </details>
                  <details>
                    <summary class="cursor-pointer text-sm text-slate-700">Сырой ответ API (JSON)</summary>
                    <pre class="text-xs bg-white border rounded p-2 overflow-x-auto whitespace-pre-wrap mt-2">{{ pretty(o.raw) }}</pre>
                  </details>
                </div>
              </td>
            </tr>
          </template>
          <tr v-if="ops.length===0">
            <td colspan="6" class="px-3 py-6 text-center text-slate-500">Нет операций</td>
          </tr>
        </tbody>
      </table>
    </div>
  </section>
    <!-- Modal for quick create forms -->
    <div v-if="showModal" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <div class="bg-white rounded-md shadow-xl w-[90vw] h-[90vh] overflow-hidden flex flex-col">
        <div class="p-2 border-b flex items-center justify-between">
          <div class="text-sm text-slate-700">??????? ??????????</div>
          <div class="flex items-center gap-2">
            <button class="rounded-md border px-2 py-1 text-sm" @click="onModalRefresh">???????? ??????</button>
            <button class="rounded-md bg-slate-800 text-white px-2 py-1 text-sm" @click="closeModal">???????</button>
          </div>
        </div>
        <iframe :src="modalUrl" class="flex-1"></iframe>
      </div>
    </div>
</template>

<script setup lang="ts">
import { computed, ref, onMounted } from 'vue';
import { useOperationsStore } from '../stores/operations';
import ItemRow from '../components/ItemRow.vue';
import { API, authHeaders } from '../api';

type StoreRef = { id: string; name: string };
type ProductRef = { id: string; title: string; category_ids: string[] };
type CategoryRef = { id: string; name: string; parent_ids: string[] };

const store = useOperationsStore();
const ops = computed(() => store.operations);
const expanded = ref<Record<string, boolean>>({});
const stores = ref<StoreRef[]>([]);
const products = ref<ProductRef[]>([]);
const storeSelect = ref<Record<string, string>>({});
const categories = ref<CategoryRef[]>([]);
const catFilter = ref<Record<string, string>>({});
const subcatFilter = ref<Record<string, string>>({});
  const pendingProducts = ref<Record<string, Record<number, string | null>>>({});
  const showModal = ref(false);
  const modalUrl = ref('');
  function openModal(url: string) { modalUrl.value = url; showModal.value = true; }
  function closeModal(){ showModal.value = false; }
  function onModalRefresh(){ fetchStores(); fetchProducts(); }

function toggle(id: string) {
  expanded.value[id] = !expanded.value[id];
}

async function postOperation(id: string) {
  store.update(id, { status: 'posted' });
  try {
    const op = ops.value.find(o => o.id === id);
    const username = op?.uploaded_by || '';
    if (username) {
      await fetch(`${API}/ratings/grant`, { method: 'POST', headers: authHeaders({ 'Content-Type': 'application/json' }), body: JSON.stringify({ username, delta: 1 }) });
    }
  } catch {}
}

function removeOperation(id: string) {
  if (confirm('Удалить операцию?')) store.remove(id);
}

function formatDate(iso: string) {
  try { return new Date(iso).toLocaleString(); } catch { return iso; }
}

function money(rub: number) {
  return (rub || 0).toFixed(2);
}

function sumItems(o: { items: { price: number; quantity: number }[] }) {
  return o.items.reduce((s, it) => s + (it.price * it.quantity), 0);
}

function pretty(v: any) {
  try { return JSON.stringify(v ?? {}, null, 2); } catch { return String(v); }
}

function rawHtml(v: any) {
  try { return (v && (v.data?.html || v.html)) || ''; } catch { return ''; }
}

async function onLinkStore(opId: string) {
  const sid = storeSelect.value[opId] || null;
  console.log('linkStore', { opId, storeId: sid });
  store.setStore(opId, sid);
}

async function onCreateStore(opId: string, defaultName: string) {
  const name = (prompt('Название магазина', defaultName || '') || '').trim();
  if (!name) return;
  const addr = (prompt('Адрес магазина', '') || '').trim();
  console.log('createStore', { name, addr });
  try {
    const res = await fetch(`${API}/stores`, { method: 'POST', headers: authHeaders({ 'Content-Type': 'application/json' }), body: JSON.stringify({ name, addr }) });
    if (!res.ok) {
      const txt = await res.text();
      throw new Error(`Ошибка создания магазина: ${res.status} ${txt}`);
    }
    const created = await res.json();
    const id = typeof created._id === 'string' ? created._id : created._id?.$oid;
    if (id) {
      stores.value.unshift({ id, name });
      store.setStore(opId, id);
      storeSelect.value[opId] = id;
    }
  } catch (e) {
    alert((e as any)?.message || 'Не удалось создать магазин');
  }
}

async function fetchStores() {
  try {
    const res = await fetch(`${API}/stores`);
    const data = await res.json();
    stores.value = (Array.isArray(data) ? data : []).map((s: any) => ({ id: typeof s._id === 'string' ? s._id : s._id?.$oid ?? '', name: s.name }));
  } catch {}
}

async function fetchProducts() {
  try {
    const res = await fetch(`${API}/products`);
    const data = await res.json();
    products.value = (Array.isArray(data) ? data : []).map((p: any) => ({ id: typeof p._id === 'string' ? p._id : p._id?.$oid ?? '', title: p.title, category_ids: (p.category_ids || []).map((x: any) => (typeof x === 'string' ? x : x?.$oid ?? '')) }));
  } catch {}
}

async function fetchCategories() {
  try {
    const res = await fetch(`${API}/categories`);
    const data = await res.json();
    categories.value = (Array.isArray(data) ? data : []).map((c: any) => ({ id: typeof c._id === 'string' ? c._id : c._id?.$oid ?? '', name: c.name, parent_ids: (c.parent_ids || []).map((x: any) => (typeof x === 'string' ? x : x?.$oid ?? '')) }));
  } catch {}
}

const topCategories = computed(() => categories.value.filter(c => !c.parent_ids || c.parent_ids.length === 0));
function subcategoriesOf(parentId?: string) {
  if (!parentId) return categories.value.filter(c => c.parent_ids && c.parent_ids.length > 0);
  return categories.value.filter(c => c.parent_ids?.includes(parentId));
}

function filteredProducts(opId: string): ProductRef[] {
  const cat = catFilter.value[opId];
  const sub = subcatFilter.value[opId];
  if (!cat && !sub) return products.value;
  let list = products.value;
  if (cat) list = list.filter(p => p.category_ids?.includes(cat));
  if (sub) list = list.filter(p => p.category_ids?.includes(sub));
  return list;
}

function onCategoryChange(opId: string) {
  subcatFilter.value[opId] = '';
}

async function onSaveOperation(o: any) {
  const sid = storeSelect.value[o.id] || null;
  if (sid !== (o.store_id || null)) {
    store.setStore(o.id, sid);
  }
  const pending = pendingProducts.value[o.id] || {};
  for (const [idxStr, pid] of Object.entries(pending)) {
    const idx = Number(idxStr);
    if (pid) {
      store.setItemProduct(o.id, idx, pid);
      const storeId = (sid || o.store_id || null) as string | null;
      if (storeId) {
        try {
          await fetch(`${API}/stores/${storeId}/products`, { method: 'POST', headers: authHeaders({ 'Content-Type': 'application/json' }), body: JSON.stringify({ product_id: pid, price: o.items[idx].price }) });
        } catch {}
      }
    }
  }
  console.log('saved operation', { opId: o.id, store_id: sid, items: pending });
}

onMounted(() => {
  fetchStores();
  fetchProducts();
  fetchCategories();
  // Prefill selected store from persisted operations
  try {
    for (const o of ops.value) {
      if (o.store_id) storeSelect.value[o.id] = o.store_id as string;
      if (!pendingProducts.value[o.id]) pendingProducts.value[o.id] = {};
    }
  } catch {}
});
</script>
