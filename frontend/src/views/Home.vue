<template>
  <section class="space-y-6">
    <h2 class="text-2xl font-semibold">Алло</h2>
    <p class="text-gray-700">Добро пожаловать в PriceCrowd. Сервис для экономии среднего чека.</p>

    <div class="rounded-lg border bg-white divide-y">
      <div v-for="a in activities" :key="a.key" class="p-4 flex items-start justify-between gap-3">
        <div class="min-w-0">
          <div class="font-medium text-slate-800">{{ activityTitle(a) }}</div>
          <div class="text-xs text-slate-500">{{ formatDate(a.ts_ms) }}</div>
        </div>
        <div class="shrink-0 flex items-center gap-3">
          <RouterLink v-if="a.product_id" :to="`/products/${a.product_id}`" class="text-xs text-blue-600 hover:underline">Товар</RouterLink>
          <RouterLink v-if="a.store_id" :to="`/stores/${a.store_id}`" class="text-xs text-blue-600 hover:underline">Магазин</RouterLink>
        </div>
      </div>
      <div v-if="activities.length===0" class="p-8 text-center text-slate-500">???? ??? ???????</div>
    </div>
  </section>
</template>

<script setup lang="ts">
import { onMounted, ref } from 'vue';
import { RouterLink } from 'vue-router';

const API = import.meta.env.VITE_API_URL || 'http://localhost:8080';
const activities = ref<any[]>([]);

function formatDate(ms:number){ const d = new Date(ms); return d.toLocaleString(); }
function activityTitle(a:any){
  if (a.kind==='item_added') return `Доавлен товар ${a.product_name || ''} (в магазине ${a.store_name || ''})`.trim();
  if (a.kind==='price_updated' || a.kind==='price_set') return `Обновлена цена ${a.product_name || ''} (в магазине ${a.store_name || ''})`.trim();
  if (a.kind==='item_removed') return `Удален товар ${a.product_name || ''} (в магазине ${a.store_name || ''})`.trim();
  return '???????';
}

async function load(){
  const res = await fetch(`${API}/activities`);
  if (res.ok){
    const arr = await res.json();
    activities.value = arr.map((x:any)=> ({ key: (typeof x._id==='string'? x._id : x._id?.$oid) || `${x.ts_ms}`, store_id: (typeof x.store_id==='string'? x.store_id : x.store_id?.$oid), product_id: (typeof x.product_id==='string'? x.product_id : x.product_id?.$oid) || null, kind: x.kind, ts_ms: x.ts_ms, product_name: x.product_name, store_name: x.store_name }));
  }
}

onMounted(load);
</script>
