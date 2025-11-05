<template>
  <section class="space-y-4">
    <header class="flex items-center justify-between">
      <h1 class="text-2xl font-semibold">Операции</h1>
      <div class="text-sm text-slate-500">Всего: {{ ops.length }}</div>
    </header>

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
          <tr v-for="o in ops" :key="o._id || o.id" class="hover:bg-slate-50">
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
              <button class="ml-2 px-2 py-1 rounded-md border text-xs" @click="saveStore(o)">Сохранить</button>
            </td>
            <td class="px-3 py-2 border-b">
              <button class="mr-2 px-3 py-1.5 rounded-md text-white bg-green-600 hover:bg-green-700" @click="postOperation(o)">Опубликовать</button>
              <button class="px-3 py-1.5 rounded-md text-white bg-red-600 hover:bg-red-700" @click="deleteOperation(o)">Удалить</button>
            </td>
          </tr>
          <tr v-if="ops.length===0">
            <td colspan="6" class="px-3 py-6 text-center text-slate-500">Пока нет операций</td>
          </tr>
        </tbody>
      </table>
    </div>
  </section>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { API, authHeaders } from '../api';

type Op = { _id?: string; id?: string; date: string; seller: string; amount: number; status: 'draft'|'posted'|'deleted'; store_id?: string|null };
type StoreRef = { id: string; name: string };

const ops = ref<Op[]>([]);
const stores = ref<StoreRef[]>([]);
const storeSelect = ref<Record<string, string>>({});

function getId(o: any){ return (typeof o._id==='string'? o._id : o._id?.$oid) || o.id; }
function formatDate(iso: string){ try { return new Date(iso).toLocaleString(); } catch { return iso; } }
function money(rub: number){ return (rub || 0).toFixed(2); }

async function loadOps(){
  try {
    const res = await fetch(`${API}/operations`, { headers: authHeaders() });
    const data = await res.json();
    ops.value = (Array.isArray(data)? data: []).map((o: any)=> ({ _id: getId(o), date: o.date, seller: o.seller, amount: o.amount, status: o.status, store_id: (typeof o.store_id==='string'? o.store_id : o.store_id?.$oid) || null }));
    for (const o of ops.value){ const id = getId(o as any); if (o.store_id) storeSelect.value[id] = o.store_id; }
  } catch {}
}

async function loadStores(){
  try {
    const res = await fetch(`${API}/stores`);
    const data = await res.json();
    stores.value = (Array.isArray(data)? data: []).map((s:any)=> ({ id: (typeof s._id==='string'? s._id : s._id?.$oid) || '', name: s.name }));
  } catch {}
}

async function postOperation(o: Op){
  const id = getId(o as any);
  try { await fetch(`${API}/operations/${id}/status`, { method: 'PUT', headers: authHeaders({ 'Content-Type': 'application/json' }), body: JSON.stringify({ status: 'posted' }) }); await loadOps(); } catch {}
}

async function deleteOperation(o: Op){
  const id = getId(o as any);
  if (!confirm('Удалить операцию?')) return;
  try { await fetch(`${API}/operations/${id}/status`, { method: 'PUT', headers: authHeaders({ 'Content-Type': 'application/json' }), body: JSON.stringify({ status: 'deleted' }) }); await loadOps(); } catch {}
}

async function saveStore(o: Op){
  const id = getId(o as any);
  const sid = storeSelect.value[id] || '';
  try { await fetch(`${API}/operations/${id}`, { method: 'PUT', headers: authHeaders({ 'Content-Type': 'application/json' }), body: JSON.stringify({ store_id: sid }) }); await loadOps(); } catch {}
}

onMounted(()=>{ loadOps(); loadStores(); });
</script>
