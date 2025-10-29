<template>
  <section class="space-y-6" v-if="product">
    <div class="grid grid-cols-1 md:grid-cols-[200px_1fr] gap-6 items-start">
      <div class="bg-white border rounded-lg p-2 flex items-center justify-center">
        <img :src="toAbs(product.image_url)" alt="image" class="max-h-60 object-contain" />
      </div>
      <div class="space-y-3">
        <h1 class="text-2xl font-semibold text-slate-900">{{ product.title }}</h1>
        <p class="text-slate-700 leading-relaxed">{{ product.desc }}</p>
        <div class="flex flex-wrap gap-2">
          <span
            v-for="cid in product.category_ids"
            :key="cid"
            class="text-xs bg-slate-100 border border-slate-200 rounded px-2 py-1"
            >{{ categoryName(cid) }}</span
          >
        </div>
        <div class="flex gap-2">
          <RouterLink
            v-if="isAuthed"
            :to="`/products/${id}/edit`"
            class="rounded-md border px-3 py-2 text-sm"
            >Редактировать</RouterLink
          >
          <RouterLink
            to="/products"
            class="rounded-md px-3 py-2 text-sm bg-blue-600 text-white"
            >Назад к списку</RouterLink
          >
        </div>
      </div>
    </div>

    <!-- Где купить и цены -->
    <div v-if="insights" class="space-y-4">
      <h2 class="text-lg font-semibold">Где купить и цена</h2>
      <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
        <div
          v-for="(k, i) in kpis"
          :key="i"
          class="rounded-lg border bg-white p-4"
        >
          <div class="text-sm text-slate-500">{{ k.title }}</div>
          <div class="text-xl font-semibold text-slate-900 mt-1">
            {{ k.value }}
          </div>
        </div>
      </div>

      <div class="rounded-lg border bg-white divide-y">
        <div
          v-for="s in (insights?.stores || [])
            .slice()
            .sort((a, b) => Number(a.price) - Number(b.price))"
          :key="String(s.store_id)"
          class="p-4 flex flex-col gap-2"
        >
          <div class="flex items-center justify-between gap-3">
            <RouterLink
              :to="`/stores/${typeof s.store_id === 'string' ? s.store_id : s.store_id?.$oid}`"
              class="font-medium text-slate-900 hover:underline"
              >{{ s.store_name || 'Магазин' }}</RouterLink
            >
            <div class="text-emerald-600 font-semibold">
              {{ Number(s.price).toLocaleString('ru-RU') }} ₽
            </div>
          </div>
          <div class="w-full h-16">
            <svg viewBox="0 0 100 50" class="w-full h-full text-emerald-500">
              <polyline
                fill="none"
                stroke="currentColor"
                stroke-width="2"
                :points="makeSparkFromHistory(s.history || [])"
              />
            </svg>
          </div>
        </div>
        <div
          v-if="!insights?.stores || !insights.stores.length"
          class="p-8 text-center text-slate-500"
        >
          Пока нет данных по магазинам
        </div>
      </div>
    </div>
  </section>
  <div v-else class="text-slate-500">Загрузка...</div>
</template>

<script setup lang="ts">
import { onMounted, ref, computed } from 'vue';
import { useRoute, RouterLink } from 'vue-router';

const API = import.meta.env.VITE_API_URL || 'http://localhost:8080';
const route = useRoute();
const id = route.params.id as string;
const placeholderUrl = '/placeholder-can.svg';

const product = ref<any | null>(null);
const categories = ref<any[]>([]);
const insights = ref<any | null>(null);

async function load() {
  const res = await fetch(`${API}/products/${id}`);
  if (res.ok) {
    const p = await res.json();
    product.value = {
      _id: typeof p._id === 'string' ? p._id : p._id?.$oid ?? '',
      title: p.title,
      desc: p.desc,
      image_url: p.image_url,
      category_ids: (p.category_ids || []).map((x: any) =>
        typeof x === 'string' ? x : x?.$oid
      ),
    };
  }
  const resCats = await fetch(`${API}/categories`);
  if (resCats.ok) categories.value = await resCats.json();

  const resIns = await fetch(`${API}/products/${id}/insights`);
  if (resIns.ok) insights.value = await resIns.json();
}

onMounted(load);

function toAbs(u?: string) {
  const src = u && u.length > 0 ? u : placeholderUrl;
  if (src.startsWith('http://') || src.startsWith('https://')) return src;
  return src.startsWith('/') ? `${API}${src}` : src;
}

function categoryName(id: string) {
  const c = categories.value.find(
    (x: any) =>
      (typeof x._id === 'string' ? x._id : x._id?.$oid) === id
  );
  return c?.name || id;
}

const kpis = computed(() => {
  const s = insights.value;
  if (!s) return [] as any[];
  const count = Array.isArray(s.stores) ? s.stores.length : 0;
  const cityAvg = s.city_avg ? Math.round(Number(s.city_avg)) : null;
  return [
    { title: 'В магазинах', value: String(count) },
    {
      title: 'Средняя по городу',
      value: cityAvg != null
        ? `${cityAvg.toLocaleString('ru-RU')} ₽`
        : '—',
    },
  ];
});

function makeSparkFromHistory(hist: Array<{ ts_ms: number; price: number }>) {
  if (!hist || hist.length === 0) return '';
  const vals = hist.map((h) => Number(h.price));
  const min = Math.min(...vals),
    max = Math.max(...vals);
  const range = Math.max(1, max - min);
  return vals
    .map((v, i) => `${(i / (vals.length - 1)) * 100},${50 - ((v - min) / range) * 50}`)
    .join(' ');
}

import { useAuth } from '../auth';
const auth = useAuth();
const isAuthed = computed(
  () => !!auth.state.token && !!auth.state.username && !!auth.state.role
);
</script>
