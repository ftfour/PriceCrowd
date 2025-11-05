<template>
  <section class="space-y-6">
    <h2 class="text-2xl font-semibold">Лента</h2>
    <p class="text-gray-700">Свежие события и активности в PriceCrowd.</p>

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
      <div v-if="activities.length===0" class="p-8 text-center text-slate-500">Пока нет событий</div>
    </div>
  </section>
</template>

<script setup lang="ts">
import { onMounted, ref } from 'vue';
import { RouterLink } from 'vue-router';

const API = import.meta.env.VITE_API_URL || 'http://localhost:8080';
const activities = ref<any[]>([]);

function formatDate(ms: number) {
  try { return new Date(ms).toLocaleString(); } catch { return String(ms); }
}

function activityTitle(a: any) {
  if (a._type === 'event') {
    if (a.kind === 'receipt_uploaded') return `${a.user ? a.user + ': ' : ''}Загрузил чек`;
    if (a.kind === 'receipt_verified') return `${a.user ? a.user + ': ' : ''}Чек подтвержден`;
    if (a.kind === 'user_registered') return `${a.user ? a.user + ': ' : ''}Регистрация`;
    return a.message || 'Событие';
  }
  if (a.kind === 'item_added') return `Добавлен товар ${a.product_name || ''} (в магазине ${a.store_name || ''})`.trim();
  if (a.kind === 'price_updated' || a.kind === 'price_set') return `Изменена цена ${a.product_name || ''} (в магазине ${a.store_name || ''})`.trim();
  if (a.kind === 'item_removed') return `Удален товар ${a.product_name || ''} (в магазине ${a.store_name || ''})`.trim();
  return 'Активность';
}

async function load() {
  try {
    const [aRes, eRes] = await Promise.all([
      fetch(`${API}/activities`),
      fetch(`${API}/events`),
    ]);
    const acts = aRes.ok ? await aRes.json() : [];
    const evs = eRes.ok ? await eRes.json() : [];
    const mappedActs = (acts || []).map((x: any) => ({
      _type: 'activity',
      key: (typeof x._id === 'string' ? x._id : x._id?.$oid) || `${x.ts_ms}`,
      store_id: (typeof x.store_id === 'string' ? x.store_id : x.store_id?.$oid) || null,
      product_id: (typeof x.product_id === 'string' ? x.product_id : x.product_id?.$oid) || null,
      kind: x.kind,
      ts_ms: x.ts_ms,
      product_name: x.product_name,
      store_name: x.store_name,
    }));
    const mappedEvents = (evs || []).map((e: any, i: number) => ({
      _type: 'event',
      key: `ev_${i}_${e.ts_ms}`,
      kind: e.kind,
      ts_ms: e.ts_ms,
      message: e.message,
      user: e.user,
    }));
    activities.value = [...mappedEvents, ...mappedActs].sort((a: any, b: any) => (b.ts_ms || 0) - (a.ts_ms || 0));
  } catch {}
}

onMounted(load);
</script>
