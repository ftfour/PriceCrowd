<template>
  <section class="space-y-4">
    <header class="flex items-center justify-between">
      <h1 class="text-2xl font-semibold">Операции</h1>
      <div class="text-sm text-slate-500">Всего: {{ ops.length }}</div>
    </header>

    <div class="flex items-center gap-2">
      <button class="px-3 py-1.5 rounded-md border" @click="openModal('/stores/new','stores')">Новый магазин</button>
      <button class="px-3 py-1.5 rounded-md border" @click="openModal('/products/new','products')">Новый товар</button>
    </div>

    <div class="rounded-lg border overflow-hidden bg-white">
      <table class="min-w-full text-sm">
        <thead class="bg-slate-50 text-left">
          <tr>
            <th class="px-3 py-2 border-b w-48">Время</th>
            <th class="px-3 py-2 border-b">Продавец</th>
            <th class="px-3 py-2 border-b w-32">Сумма</th>
            <th class="px-3 py-2 border-b w-32">Статус</th>
            <th class="px-3 py-2 border-b w-64">Магазин</th>
            <th class="px-3 py-2 border-b w-56">Действия</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="o in ops" :key="o._id || o.id" class="hover:bg-slate-50 align-top">
            <td class="px-3 py-2 border-b whitespace-nowrap">{{ formatDate(o.date) }}</td>
            <td class="px-3 py-2 border-b">{{ o.seller }}</td>
            <td class="px-3 py-2 border-b">{{ money(o.amount) }}</td>
            <td class="px-3 py-2 border-b">
              <span :class="o.status==='draft' ? 'text-amber-700' : (o.status==='posted' ? 'text-green-700' : 'text-slate-500')">{{ o.status }}</span>
            </td>
            <td class="px-3 py-2 border-b">
              <select v-model="storeSelect[o._id || o.id]" class="border rounded px-2 py-1 text-sm bg-white text-slate-900">
                <option value="">—</option>
                <option v-for="s in stores" :key="s.id" :value="s.id">{{ s.name }}</option>
              </select>
              <button class="ml-2 px-2 py-1 rounded-md border text-xs" @click="saveStore(o)">?????????</button> <button class="ml-2 px-2 py-1 rounded-md border text-xs" @click="openModalStore(o.seller)">????? ??????? (?? ????????)</button>
            </td>
            <td class="px-3 py-2 border-b">
              <button class="mr-2 px-3 py-1.5 rounded-md text-white bg-green-600 hover:bg-green-700" @click="postOperation(o)">Опубликовать</button>
              <button class="px-3 py-1.5 rounded-md text-white bg-red-600 hover:bg-red-700" @click="deleteOperation(o)">Удалить</button>
            </td>
          </tr>
          <tr v-for="o in ops" :key="(o._id || o.id)+'-items'">
            <td colspan="6" class="px-3 py-2 border-b bg-slate-50">
              <div class="text-sm text-slate-700 mb-2">Позиции</div>
              <div class="space-y-2">
                <div v-for="(it, idx) in edited[o._id || o.id]?.items || []" :key="idx" class="grid grid-cols-12 gap-2 items-center">
                  <div class="col-span-5 truncate">{{ it.name }}</div>
                  <div class="col-span-2 text-right">{{ money(it.price) }}</div>
                  <div class="col-span-1 text-right">{{ it.quantity }}</div>
                  <div class="col-span-4">
                    <input type="text" v-model="productQuery[o._id || o.id][idx]" placeholder="Поиск товара..." class="w-full border rounded px-2 py-1 text-sm" />
                    <select v-model="edited[o._id || o.id].items[idx].product_id" class="mt-1 w-full border rounded px-2 py-1 text-sm bg-white text-slate-900">
                      <option :value="null">—</option>
                      <option v-for="p in filteredProducts(o, idx)" :key="p.id" :value="p.id">{{ p.title }}</option>
                    </select>
                    <button class="mt-1 px-2 py-1 rounded-md border text-xs" @click="openModalProduct(it.name)">????? ????? (?? ????????)</button>
                  </div>
                </div>
              </div>
              <div class="mt-3 flex items-center gap-2">
                <button class="px-3 py-1.5 rounded-md text-white bg-blue-600 hover:bg-blue-700" @click="saveItems(o)">Сохранить позиции</button>
                <span class="text-xs text-slate-500">Назначьте товары для строк операции и сохраните</span>
              </div>
            </td>
          </tr>
          <tr v-if="ops.length===0">
            <td colspan="6" class="px-3 py-6 text-center text-slate-500">Пока нет операций</td>
          </tr>
        </tbody>
      </table>
    </div>

    <!-- Popup с формой добавления -->
    <div v-if="showModal" class="fixed inset-0 bg-black/50 z-50 flex items-center justify-center">
      <div class="bg-white rounded-md shadow-xl w-[90vw] h-[90vh] overflow-hidden flex flex-col">
        <div class="p-2 border-b flex items-center justify-between text-sm">
          <div>Быстрое добавление</div>
          <div class="flex items-center gap-2">
            <button class="rounded-md border px-2 py-1" @click="closeModal">Закрыть</button>
          </div>
        </div>
        <div class="p-4 overflow-auto flex-1">
          <div v-if="modalKind==='stores'" class="max-w-md space-y-3">
            <label class="block text-sm">
              <span class="block mb-1">Название магазина</span>
              <input v-model="quickStoreName" class="w-full border rounded px-3 py-2 text-sm" />
            </label>
            <label class="block text-sm">
              <span class="block mb-1">Адрес (необязательно)</span>
              <input v-model="quickStoreAddr" class="w-full border rounded px-3 py-2 text-sm" />
            </label>
            <div class="pt-2">
              <button class="px-3 py-1.5 rounded-md text-white bg-blue-600 hover:bg-blue-700" @click="submitQuickStore">Создать</button>
            </div>
          </div>
          <div v-else-if="modalKind==='products'" class="max-w-md space-y-3">
            <label class="block text-sm">
              <span class="block mb-1">Название товара</span>
              <input v-model="quickProductTitle" class="w-full border rounded px-3 py-2 text-sm" />
            </label>
            <div class="pt-2">
              <button class="px-3 py-1.5 rounded-md text-white bg-blue-600 hover:bg-blue-700" @click="submitQuickProduct">Создать</button>
            </div>
          </div>
        </div>
      </div>
    </div>
  </section>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { API, authHeaders } from '../api';

type Op = { _id?: string; id?: string; date: string; seller: string; amount: number; status: 'draft'|'posted'|'deleted'; store_id?: string|null };
type StoreRef = { id: string; name: string };
type ProductRef = { id: string; title: string };

const ops = ref<Op[]>([]);
const stores = ref<StoreRef[]>([]);
const storeSelect = ref<Record<string, string>>({});
const products = ref<ProductRef[]>([]);
const edited = ref<Record<string, { items: { name:string; price:number; quantity:number; product_id: string | null }[] }>>({});
const productQuery = ref<Record<string, Record<number, string>>>({});
const showModal = ref(false);
const modalUrl = ref('');
const modalKind = ref<'stores'|'products'|''>('');
const quickStoreName = ref('');
const quickStoreAddr = ref('');
const quickProductTitle = ref('');

function getId(o: any){ return (typeof o._id==='string'? o._id : o._id?.$oid) || o.id; }
function formatDate(iso: string){ try { return new Date(iso).toLocaleString(); } catch { return iso; } }
function money(rub: number){ return (rub || 0).toFixed(2); }

async function loadOps(){
  try {
    const res = await fetch(`${API}/operations`, { headers: authHeaders() });
    const data = await res.json();
    ops.value = (Array.isArray(data)? data: []).map((o: any)=> ({ _id: getId(o), date: o.date, seller: o.seller, amount: o.amount, status: o.status, store_id: (typeof o.store_id==='string'? o.store_id : o.store_id?.$oid) || null }));
    for (const oraw of (Array.isArray(data)? data: [])){
      const id = getId(oraw);
      if ((typeof oraw.store_id==='string'? oraw.store_id : oraw.store_id?.$oid)) storeSelect.value[id] = (typeof oraw.store_id==='string'? oraw.store_id : oraw.store_id?.$oid);
      const items = (oraw.items || []).map((i:any)=> ({ name: i.name, price: i.price, quantity: i.quantity, product_id: (typeof i.product_id==='string'? i.product_id : i.product_id?.$oid) || null }));
      edited.value[id] = { items };
      productQuery.value[id] = {} as any;
    }
  } catch {}
}

async function loadStores(){
  try {
    const res = await fetch(`${API}/stores`);
    const data = await res.json();
    stores.value = (Array.isArray(data)? data: []).map((s:any)=> ({ id: (typeof s._id==='string'? s._id : s._id?.$oid) || '', name: s.name }));
  } catch {}
}

async function loadProducts(){
  try { const res = await fetch(`${API}/products`); const data = await res.json(); products.value = (Array.isArray(data)? data: []).map((p:any)=> ({ id: (typeof p._id==='string'? p._id : p._id?.$oid) || '', title: p.title })); } catch {}
}

async function postOperation(o: Op){
  const id = getId(o as any);
  try { await fetch(`${API}/operations/${id}/status`, { method: 'PUT', headers: authHeaders({ 'Content-Type': 'application/json' }), body: JSON.stringify({ status: 'posted' }) }); await loadOps(); } catch {}
}

async function deleteOperation(o: Op){
  const id = getId(o as any);
  if (!confirm("??????? ?????????")) return;
  try { await fetch(`${API}/operations/${id}`, { method: "DELETE", headers: authHeaders() }); await loadOps(); } catch {}
}
{
  const id = getId(o as any);
  if (!confirm('Удалить операцию?')) return;
  try { await fetch(`${API}/operations/${id}/status`, { method: 'PUT', headers: authHeaders({ 'Content-Type': 'application/json' }), body: JSON.stringify({ status: 'deleted' }) }); await loadOps(); } catch {}
}

async function saveStore(o: Op){
  const id = getId(o as any);
  const sid = storeSelect.value[id] || '';
  try { await fetch(`${API}/operations/${id}`, { method: 'PUT', headers: authHeaders({ 'Content-Type': 'application/json' }), body: JSON.stringify({ store_id: sid }) }); await loadOps(); } catch {}
}

function filteredProducts(o: any, idx: number): ProductRef[]{
  const id = getId(o);
  const q = (productQuery.value[id]?.[idx] || '').toLowerCase();
  if (!q) return products.value;
  return products.value.filter(p => p.title.toLowerCase().includes(q));
}

async function saveItems(o: any){
  const id = getId(o);
  const payload = { items: edited.value[id]?.items || [] };
  try { await fetch(`${API}/operations/${id}`, { method: 'PUT', headers: authHeaders({ 'Content-Type': 'application/json' }), body: JSON.stringify(payload) }); alert('Позиции сохранены'); } catch{}
}

function openModal(url: string, kind: 'stores'|'products'){
  modalUrl.value = url;
  modalKind.value = kind;
  showModal.value = true;
  if (kind==='stores') { quickStoreName.value = ''; quickStoreAddr.value=''; }
  if (kind==='products') { quickProductTitle.value = ''; }
}
function closeModal(){
  showModal.value = false;
  if (modalKind.value === "stores") loadStores();
  if (modalKind.value === "products") loadProducts();
}

function openModalStore(name){
  openModal("/stores/new","stores");
  try { quickStoreName.value = String(name||""); } catch {}
}
function openModalProduct(title){
  openModal("/products/new","products");
  try { quickProductTitle.value = String(title||""); } catch {}
}


async function submitQuickStore(){
  try {
    await fetch(`${API}/stores`, { method:'POST', headers: authHeaders({ 'Content-Type':'application/json' }), body: JSON.stringify({ name: quickStoreName.value, addr: quickStoreAddr.value||'' }) });
    await loadStores();
    showModal.value = false;
  } catch {}
}
async function submitQuickProduct(){
  try {
    await fetch(`${API}/products`, { method:'POST', headers: authHeaders({ 'Content-Type':'application/json' }), body: JSON.stringify({ title: quickProductTitle.value, desc:'' }) });
    await loadProducts();
    showModal.value = false;
  } catch {}
}

onMounted(()=>{ loadOps(); loadStores(); loadProducts(); });
</script>




