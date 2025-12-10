<template>
  <section class="space-y-10">
    <!-- Hero -->
    <div class="relative overflow-hidden rounded-3xl bg-gradient-to-r from-slate-900 via-slate-800 to-slate-700 text-white px-6 py-12 shadow-xl">
      <div class="absolute inset-0 opacity-30 pointer-events-none" style="background: radial-gradient(circle at 20% 20%, rgba(255,255,255,0.18), transparent 40%), radial-gradient(circle at 80% 0%, rgba(255,255,255,0.1), transparent 35%);"></div>
      <div class="relative flex flex-col lg:flex-row items-center gap-10">
        <div class="flex-1 space-y-5">
          <div class="inline-flex items-center gap-2 rounded-full bg-white/10 px-3 py-1 text-xs uppercase tracking-wide text-slate-100 border border-white/15">
            Сообщество цен • Без спама
          </div>
          <h1 class="text-3xl sm:text-4xl font-bold leading-tight">
            PriceCrowd помогает находить честные цены и делиться находками
          </h1>
          <p class="text-slate-200 text-lg max-w-2xl">
            Сканируйте чеки, сравнивайте цены по магазинам и следите за снижениями. Бесплатно и прозрачно — мы работаем на данных сообщества.
          </p>
          <div class="flex flex-wrap gap-3">
            <RouterLink to="/scan" class="inline-flex items-center gap-2 rounded-lg bg-emerald-500 px-4 py-2 text-sm font-semibold text-white shadow-lg shadow-emerald-500/30 hover:bg-emerald-400 transition">
              📷 Сканировать чек
            </RouterLink>
            <RouterLink to="/products" class="inline-flex items-center gap-2 rounded-lg bg-white/10 px-4 py-2 text-sm font-semibold text-white border border-white/20 hover:bg-white/20 transition">
              Смотреть товары
            </RouterLink>
            <RouterLink to="/rating" class="inline-flex items-center gap-2 rounded-lg bg-white/10 px-4 py-2 text-sm font-semibold text-white border border-white/20 hover:bg-white/20 transition">
              Рейтинг участников
            </RouterLink>
          </div>
          <div class="flex flex-wrap gap-4 text-sm text-slate-200">
            <div class="flex items-center gap-2"><span class="h-2 w-2 rounded-full bg-emerald-400"></span>Открытый экспорт</div>
            <div class="flex items-center gap-2"><span class="h-2 w-2 rounded-full bg-emerald-400"></span>Без рекламы</div>
            <div class="flex items-center gap-2"><span class="h-2 w-2 rounded-full bg-emerald-400"></span>Приватность QR сохраняется</div>
          </div>
        </div>
        <div class="w-full max-w-sm rounded-2xl bg-white/10 p-5 border border-white/15 backdrop-blur">
          <div class="text-sm text-slate-200 mb-3">Сейчас в базе</div>
          <div class="grid grid-cols-2 gap-3">
            <StatCard title="Товаров" :value="stats.products" />
            <StatCard title="Магазинов" :value="stats.stores" />
            <StatCard title="Активностей" :value="activities.length" />
            <StatCard title="Категорий" :value="stats.categories" />
          </div>
          <div class="mt-4 rounded-lg bg-white/10 p-3 text-xs text-slate-200 border border-white/15">
            Обновляется в реальном времени, данные можно выгрузить в один клик.
          </div>
        </div>
      </div>
    </div>

    <!-- How it works -->
    <div class="grid gap-6 lg:grid-cols-3">
      <div class="rounded-2xl border border-slate-200 bg-white p-5 shadow-sm">
        <div class="text-lg font-semibold mb-2">1. Сканируйте</div>
        <p class="text-slate-600 text-sm">Откройте сканер QR, добавьте чек за секунды. Мы бережно относимся к данным и не просим личных данных.</p>
      </div>
      <div class="rounded-2xl border border-slate-200 bg-white p-5 shadow-sm">
        <div class="text-lg font-semibold mb-2">2. Сравнивайте</div>
        <p class="text-slate-600 text-sm">Видите цены по магазинам и категориям, тепловую карту и свежие изменения.</p>
      </div>
      <div class="rounded-2xl border border-slate-200 bg-white p-5 shadow-sm">
        <div class="text-lg font-semibold mb-2">3. Делитесь выгодой</div>
        <p class="text-slate-600 text-sm">Получайте благодарности в рейтинге и помогайте другим покупать дешевле.</p>
      </div>
    </div>

    <!-- Recent activity -->
    <div class="rounded-3xl border border-slate-200 bg-white shadow-sm">
      <div class="flex items-center justify-between px-6 py-4">
        <div>
          <div class="text-lg font-semibold">Свежие события</div>
          <div class="text-sm text-slate-500">Обновления от сообщества и изменения цен</div>
        </div>
        <RouterLink to="/stores" class="text-sm text-blue-600 hover:underline">К магазинам</RouterLink>
      </div>
      <div class="divide-y">
        <div v-for="a in activities.slice(0,6)" :key="a.key" class="px-6 py-4 flex items-start justify-between gap-3">
          <div class="min-w-0">
            <div class="font-medium text-slate-900">{{ activityTitle(a) }}</div>
            <div class="text-xs text-slate-500">{{ formatDate(a.ts_ms) }}</div>
          </div>
          <div class="shrink-0 flex items-center gap-3 text-xs">
            <RouterLink v-if="a.product_id" :to="`/products/${a.product_id}`" class="text-blue-600 hover:underline">Товар</RouterLink>
            <RouterLink v-if="a.store_id" :to="`/stores/${a.store_id}`" class="text-blue-600 hover:underline">Магазин</RouterLink>
          </div>
        </div>
        <div v-if="activities.length===0" class="px-6 py-12 text-center text-slate-500">Пока нет событий, добавьте свой первый чек</div>
      </div>
    </div>
  </section>
</template>

<script setup lang="ts">
import { onMounted, ref, defineComponent } from 'vue';
import { RouterLink } from 'vue-router';

const API = import.meta.env.VITE_API_URL || 'http://localhost:8080';
const activities = ref<any[]>([]);
const stats = ref<{ products: number; stores: number; categories: number }>({ products: 0, stores: 0, categories: 0 });
const StatCard = defineComponent({
  name: 'StatCard',
  props: { title: { type: String, required: true }, value: { type: Number, required: true } },
  template: `
    <div class="rounded-xl bg-white/15 border border-white/15 text-white px-3 py-3 text-center">
      <div class="text-xs uppercase tracking-wide text-slate-200">{{ title }}</div>
      <div class="text-2xl font-semibold">{{ value }}</div>
    </div>
  `,
});

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
  if (a.kind === 'item_added') return `Новый товар ${a.product_name || ''} (в ${a.store_name || ''})`.trim();
  if (a.kind === 'price_updated' || a.kind === 'price_set') return `Изменилась цена ${a.product_name || ''} (в ${a.store_name || ''})`.trim();
  if (a.kind === 'item_removed') return `Товар удален ${a.product_name || ''} (в ${a.store_name || ''})`.trim();
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

  try {
    const [pRes, sRes, cRes] = await Promise.all([
      fetch(`${API}/products`),
      fetch(`${API}/stores`),
      fetch(`${API}/categories`),
    ]);
    const products = pRes.ok ? await pRes.json() : [];
    const stores = sRes.ok ? await sRes.json() : [];
    const categories = cRes.ok ? await cRes.json() : [];
    stats.value = {
      products: Array.isArray(products) ? products.length : 0,
      stores: Array.isArray(stores) ? stores.length : 0,
      categories: Array.isArray(categories) ? categories.length : 0,
    };
  } catch {}
}

onMounted(load);
</script>
