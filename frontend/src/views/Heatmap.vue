<template>
  <section class="space-y-6">
    <header class="flex items-center justify-between">
      <div>
        <h1 class="text-2xl font-semibold">Тепловая карта цен</h1>
        <p class="text-sm text-slate-600">Сравнение средних цен по магазинам и категориям</p>
      </div>
      <div class="text-sm text-slate-500" v-if="!loading && !error">
        Магазинов: {{ stores.length }} · Категорий: {{ categoryOrder.length }}
      </div>
    </header>

    <div v-if="loading" class="rounded-lg border bg-white p-6 text-center text-slate-500">Загружаем данные...</div>
    <div v-else-if="error" class="rounded-lg border border-rose-200 bg-rose-50 p-4 text-rose-700">{{ error }}</div>
    <div v-else class="space-y-4">
      <div class="flex flex-wrap items-center gap-3 text-sm text-slate-600">
        <div class="flex items-center gap-2">
          <span class="inline-block h-3 w-24 rounded-full" :style="legendStyle"></span>
          <span>min</span>
          <span class="text-xs text-slate-400">→</span>
          <span>max</span>
        </div>
        <div class="text-xs text-slate-500">Цвет = средняя цена в категории для магазина</div>
      </div>

      <div class="overflow-auto rounded-lg border bg-white">
        <table class="min-w-full text-sm">
          <thead class="bg-slate-50 text-slate-600">
            <tr>
              <th class="px-3 py-2 text-left w-52">Категория</th>
              <th v-for="store in storeOrder" :key="store.id" class="px-3 py-2 text-left whitespace-nowrap">{{ store.name }}</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="catId in categoryOrder" :key="catId" class="border-t">
              <td class="px-3 py-2 font-medium text-slate-700">
                {{ categoryName(catId) }}
              </td>
              <td v-for="store in storeOrder" :key="store.id + '_' + catId" class="px-2 py-2">
                <div
                  class="rounded-md border text-center px-2 py-3"
                  :style="cellStyle(avgPrice(store.id, catId))"
                >
                  <div class="font-semibold">
                    <span v-if="avgPrice(store.id, catId) !== null">{{ money(avgPrice(store.id, catId) || 0) }} ₽</span>
                    <span v-else class="text-slate-400">—</span>
                  </div>
                  <div class="text-[11px] text-slate-600" v-if="cellCount(store.id, catId) > 0">
                    {{ cellCount(store.id, catId) }} поз.
                  </div>
                </div>
              </td>
            </tr>
          </tbody>
        </table>
      </div>

      <div class="text-xs text-slate-500">
        Источник: store_items (средняя цена по товарам, привязанным к категории). Берём топ категорий и магазинов с наибольшим числом позиций.
      </div>
    </div>
  </section>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue';
import { API } from '../api';

type Store = { _id?: any; id?: any; name: string };
type Category = { _id?: any; id?: any; name: string };
type StoreItemPayload = { price: number; product?: { category_ids?: any[] } };

function idOf(x: any): string {
  if (!x) return '';
  if (typeof x === 'string') return x;
  return x.$oid || x.id || '';
}
function money(v: number) {
  return (v || 0).toFixed(2);
}

const loading = ref(true);
const error = ref('');
const stores = ref<Store[]>([]);
const categories = ref<Category[]>([]);
const storeItems = ref<Record<string, StoreItemPayload[]>>({});

onMounted(async () => {
  loading.value = true;
  error.value = '';
  try {
    const [storesRes, catsRes] = await Promise.all([
      fetch(`${API}/stores`),
      fetch(`${API}/categories`),
    ]);
    const storesData = await storesRes.json();
    const catsData = await catsRes.json();
    stores.value = Array.isArray(storesData) ? storesData : [];
    categories.value = Array.isArray(catsData) ? catsData : [];

    const payloads = await Promise.all(
      stores.value.map(async (s) => {
        const res = await fetch(`${API}/stores/${idOf(s._id) || idOf(s.id)}/products`);
        if (!res.ok) return { id: idOf(s._id) || idOf(s.id), items: [] as StoreItemPayload[] };
        const data = await res.json();
        return { id: idOf(s._id) || idOf(s.id), items: Array.isArray(data) ? data : [] };
      }),
    );
    const map: Record<string, StoreItemPayload[]> = {};
    for (const p of payloads) {
      if (!p.id) continue;
      map[p.id] = p.items;
    }
    storeItems.value = map;
  } catch (e: any) {
    error.value = e?.message || 'Не удалось загрузить данные';
  } finally {
    loading.value = false;
  }
});

type CellAgg = { sum: number; count: number };

const heat = computed(() => {
  const cells = new Map<string, CellAgg>();
  const catUsage = new Map<string, number>();
  const storeUsage = new Map<string, number>();
  let min = Number.POSITIVE_INFINITY;
  let max = Number.NEGATIVE_INFINITY;

  for (const s of stores.value) {
    const sid = idOf(s._id) || idOf(s.id);
    const items = storeItems.value[sid] || [];
    storeUsage.set(sid, items.length);
    for (const it of items) {
      const price = Number(it.price || 0);
      if (!(price > 0)) continue;
      const cats = (it.product?.category_ids || []).map(idOf).filter(Boolean);
      const cid = cats[0] || 'uncategorized';

      const key = `${sid}__${cid}`;
      const agg = cells.get(key) || { sum: 0, count: 0 };
      agg.sum += price;
      agg.count += 1;
      cells.set(key, agg);

      catUsage.set(cid, (catUsage.get(cid) || 0) + 1);
      min = Math.min(min, price);
      max = Math.max(max, price);
    }
  }

  const storeOrder = stores.value
    .slice()
    .sort((a, b) => {
      const sa = storeUsage.get(idOf(a._id) || idOf(a.id)) || 0;
      const sb = storeUsage.get(idOf(b._id) || idOf(b.id)) || 0;
      return sb - sa;
    })
    .slice(0, 8); // ограничим визуальный шум

  const categoryOrder = Array.from(catUsage.entries())
    .sort((a, b) => b[1] - a[1])
    .slice(0, 12)
    .map(([cid]) => cid);

  return {
    cells,
    storeOrder,
    categoryOrder,
    min: Number.isFinite(min) ? min : null,
    max: Number.isFinite(max) ? max : null,
  };
});

const storeOrder = computed(() => heat.value.storeOrder);
const categoryOrder = computed(() => heat.value.categoryOrder);

function avgPrice(storeId: string, catId: string): number | null {
  const key = `${storeId}__${catId}`;
  const cell = heat.value.cells.get(key);
  if (!cell || !cell.count) return null;
  return cell.sum / cell.count;
}
function cellCount(storeId: string, catId: string): number {
  const key = `${storeId}__${catId}`;
  const cell = heat.value.cells.get(key);
  return cell?.count || 0;
}

function categoryName(id: string) {
  const found = categories.value.find((c) => idOf(c._id) === id || idOf(c.id) === id);
  if (found) return found.name;
  if (id === 'uncategorized') return 'Без категории';
  return 'Категория';
}

function cellStyle(value: number | null) {
  if (value === null || heat.value.min === null || heat.value.max === null) {
    return { backgroundColor: '#f8fafc' };
  }
  const min = heat.value.min;
  const max = heat.value.max;
  if (max === min) return { backgroundColor: 'hsl(200, 90%, 75%)' };
  const ratio = Math.min(1, Math.max(0, (value - min) / (max - min)));
  const hue = 200 - 140 * ratio; // от синего к оранжево-розовому
  const light = 90 - 35 * ratio;
  return { backgroundColor: `hsl(${hue}, 90%, ${light}%)` };
}

const legendStyle = computed(() => ({
  background: 'linear-gradient(90deg, hsl(200,90%,90%) 0%, hsl(60,90%,75%) 50%, hsl(16,90%,60%) 100%)',
}));
</script>
