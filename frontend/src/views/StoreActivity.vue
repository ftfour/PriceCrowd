<template>
  <section class="space-y-6">
    <div class="flex items-center justify-between">
      <h1 class="text-2xl font-semibold">Активность магазина</h1>
      <RouterLink :to="`/stores/${id}`" class="text-sm text-blue-600 hover:underline">Назад к магазину</RouterLink>
    </div>

    <div class="rounded-lg border bg-white divide-y">
      <div v-for="a in activities" :key="a.key" class="p-4 flex items-start justify-between gap-3">
        <div class="min-w-0">
          <div class="font-medium text-slate-800">{{ activityTitle(a) }}</div>
          <div class="text-xs text-slate-500">{{ formatDate(a.ts_ms) }}</div>
        </div>
        <RouterLink v-if="a.product_id" :to="`/products/${a.product_id}`" class="shrink-0 text-xs text-blue-600 hover:underline">Товар</RouterLink>
      </div>
      <div v-if="activities.length===0" class="p-8 text-center text-slate-500">Событий пока нет</div>
    </div>
  </section>
</template>

<script setup lang="ts">
import { onMounted, ref } from 'vue';
import { useRoute, RouterLink } from 'vue-router';

const API = import.meta.env.VITE_API_URL || 'http://localhost:8080';
const route = useRoute();
const id = route.params.id as string;
const activities = ref<any[]>([]);

function formatDate(ms:number){ const d = new Date(ms); return d.toLocaleString(); }
function activityTitle(a:any){
  if (a.kind==='item_added') return `Добавлен товар ${a.product_name || ''}`.trim();
  if (a.kind==='price_updated' || a.kind==='price_set') return `Обновлена цена на товар ${a.product_name || ''}`.trim();
  if (a.kind==='item_removed') return `Товар удален ${a.product_name || ''}`.trim();
  return 'Событие';
}

async function load(){
  const res = await fetch(`${API}/stores/${id}/activities`);
  if (res.ok){
    const arr = await res.json();
    activities.value = arr.map((x:any)=> ({ key: (typeof x._id==='string'? x._id : x._id?.$oid) || `${x.ts_ms}`, product_id: (typeof x.product_id==='string'? x.product_id : x.product_id?.$oid) || null, kind: x.kind, ts_ms: x.ts_ms, price: x.price, product_name: x.product_name, store_name: x.store_name }));
  }
}

onMounted(load);
</script>
