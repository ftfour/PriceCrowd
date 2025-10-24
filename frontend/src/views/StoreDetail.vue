<template>
  <div class="grid grid-cols-12 gap-6" v-if="store">
    <!-- Основной контент -->
    <div class="col-span-12 lg:col-span-9 space-y-8">
      <!-- Инфо о магазине -->
      <div class="p-6 flex items-center gap-6 rounded-lg border bg-white">
        <img :src="toAbs(store.image_url)" alt="store" class="w-20 h-20 rounded-2xl object-contain bg-slate-50 p-2" />
        <div class="flex-1">
          <h1 class="text-2xl font-bold text-slate-900">{{ store.name }}</h1>
          <p class="text-slate-600 mt-1">{{ store.addr }}</p>
          <p class="mt-3 text-slate-700 text-sm">{{ store.desc }}</p>
        </div>
        <div class="text-right min-w-[120px]">
          <div class="text-yellow-500 font-semibold text-xl">★ 4.6</div>
          <div class="text-slate-400 text-sm">1 товар</div>
        </div>
      </div>

      <!-- Аналитика магазина (демо) -->
      <div>
        <h2 class="text-lg font-semibold mb-3">Аналитика по магазину</h2>
        <div class="grid grid-cols-2 md:grid-cols-4 gap-4 mb-4">
          <div v-for="(k,i) in kpis" :key="i" class="rounded-lg border bg-white p-4">
            <div class="text-sm text-slate-500">{{ k.title }}</div>
            <div class="text-xl font-semibold text-slate-900 mt-1">{{ k.value }}</div>
          </div>
        </div>
        <div class="grid grid-cols-1 lg:grid-cols-2 gap-4">
          <div class="rounded-lg border bg-white p-4 flex items-center justify-between">
            <div>
              <div class="text-sm text-slate-500">Цена товара vs средняя по городу</div>
              <div class="text-xl font-semibold text-slate-900 mt-1">{{ priceInsight.product.label }}</div>
              <div class="mt-1 text-emerald-600 font-semibold">{{ priceInsight.product.price }} ₽ <span class="text-slate-400 font-normal">(город: {{ priceInsight.cityAvg }} ₽)</span></div>
              <div class="text-sm mt-1" :class="priceInsight.trend7d<0?'text-emerald-600':'text-rose-600'">
                {{ priceInsight.trend7d<0?'↓':'↑' }} {{ Math.abs(priceInsight.trend7d) }}% за 7 дней
              </div>
            </div>
            <div class="w-40 h-20">
              <svg viewBox="0 0 100 50" class="w-full h-full text-emerald-500">
                <polyline fill="none" stroke="currentColor" stroke-width="2" :points="sparkPoints" />
              </svg>
            </div>
          </div>
          <div class="rounded-lg border bg-white p-4">
            <div class="flex items-center justify-between">
              <div class="text-sm text-slate-500">Активность</div>
              <RouterLink :to="`/stores/${id}/activity`" class="text-sm text-blue-600 hover:underline">Вся активность</RouterLink>
            </div>
            <ul class="mt-2 text-sm text-slate-700 space-y-2">
              <li v-for="a in activities.slice(0,5)" :key="a.key" class="flex items-start justify-between gap-2">
                <div class="flex-1 min-w-0">
                  <div class="font-medium text-slate-800">{{ activityTitle(a) }}</div>
                  <div class="text-xs text-slate-500">{{ formatDate(a.ts_ms) }}</div>
                </div>
                <RouterLink v-if="a.product_id" :to="`/products/${a.product_id}`" class="shrink-0 text-xs text-blue-600 hover:underline">Товар</RouterLink>
              </li>
              <li v-if="activities.length===0" class="text-slate-500 text-sm">Событий пока нет</li>
            </ul>
          </div>
        </div>
      </div>

      <!-- Товары магазина (реальные данные) -->
      <div class="space-y-3">
        <h2 class="text-lg font-semibold">Товары магазина</h2>
        <div class="rounded-lg border bg-white p-4">
          <form @submit.prevent="addItem" class="flex flex-wrap items-center gap-3">
            <select v-model="newItem.product_id" class="h-9 border rounded px-2 text-sm min-w-[240px]">
              <option value="" disabled>Выберите товар</option>
              <option v-for="p in productsForSelect" :key="p._id" :value="p._id">{{ p.title }}</option>
            </select>
            <input v-model.number="newItem.price" type="number" step="0.01" min="0" placeholder="Цена" class="h-9 w-28 border rounded px-2 text-sm" />
            <button type="submit" class="h-9 px-3 rounded bg-blue-600 text-white text-sm">Добавить</button>
          </form>
        </div>
        <div class="rounded-lg border bg-white divide-y">
          <div v-for="it in storeItems" :key="it.key" class="p-4 flex items-center gap-4">
            <div class="h-12 w-12 rounded bg-slate-50 border flex items-center justify-center overflow-hidden">
              <img :src="toAbs(it.product?.image_url)" class="max-h-10 object-contain" alt="" />
            </div>
            <div class="min-w-0 flex-1">
              <RouterLink :to="`/products/${it.product_id}`" class="font-medium text-slate-900 hover:underline truncate block">{{ it.product?.title || it.product_id }}</RouterLink>
              <div class="text-xs text-slate-500 break-all">{{ it.product_id }}</div>
            </div>
            <div class="flex items-center gap-2">
              <input v-model.number="priceEdit[it.key]" type="number" step="0.01" min="0" class="h-9 w-24 border rounded px-2 text-sm" />
              <button @click="savePrice(it)" class="h-9 px-3 rounded border text-sm">Сохранить</button>
              <button @click="removeItem(it)" class="h-9 px-3 rounded bg-red-600 text-white text-sm">Удалить</button>
            </div>
          </div>
          <div v-if="storeItems.length===0" class="p-8 text-center text-slate-500">Нет товаров</div>
        </div>
      </div>
    </div>

    <!-- Сайдбар -->
    <aside class="col-span-12 lg:col-span-3 space-y-4">
      <div>
        <h2 class="text-lg font-semibold mb-3">На карте</h2>
        <div class="rounded-lg border bg-[linear-gradient(180deg,#f5f7fb,#eef2f7)] h-64 flex items-center justify-center text-slate-400">
          Карта (заглушка)
        </div>
      </div>
      <div>
        <h2 class="text-lg font-semibold mb-3">Heatmap по часам (7д)</h2>
        <div class="rounded-lg border bg-white p-3 overflow-auto">
          <div class="grid" :style="{gridTemplateColumns: '40px repeat(24, 1fr)'}">
            <div></div>
            <div v-for="h in 24" :key="h" class="text-[10px] text-slate-500 text-center">{{ h-1 }}</div>
            <template v-for="(row,d) in heatmap" :key="d">
              <div class="text-[11px] text-slate-600 pr-1 text-right flex items-center">{{ days[d] }}</div>
              <div v-for="(v,h) in row" :key="h" class="h-4 m-[2px] rounded" :title="`${days[d]} ${h}:00 — ${v}`" :style="{ background: color(v) }"></div>
            </template>
          </div>
        </div>
      </div>
      <div>
        <h2 class="text-lg font-semibold mb-3">Часы и контакты</h2>
        <div class="rounded-lg border bg-white p-4 text-sm text-slate-700 space-y-2">
          <div><span class="text-slate-500">Часы:</span> Пн-Вс 09:00–22:00</div>
          <div><span class="text-slate-500">Тел.:</span> +7 (411) xxx-xx-xx</div>
          <div><span class="text-slate-500">Статус:</span> Открыто</div>
          <div class="text-slate-400 text-xs">Информация носит ознакомительный характер</div>
        </div>
      </div>
    </aside>
  </div>
  <div v-else class="text-slate-500">Загрузка...</div>
</template>

<script setup lang="ts">
import { onMounted, ref, computed } from 'vue';
import { useRoute } from 'vue-router';

const API = import.meta.env.VITE_API_URL || 'http://localhost:8080';
const route = useRoute();
const id = route.params.id as string;
const placeholderUrl = '/placeholder-can.svg';

const store = ref<any | null>(null);
const allProducts = ref<any[]>([]);
const storeItems = ref<any[]>([]);
const newItem = ref<{ product_id: string; price: number | null }>({ product_id: '', price: null });
const priceEdit = ref<Record<string, number>>({});

async function load() {
  const res = await fetch(`${API}/stores/${id}`);
  if (res.ok) {
    const s = await res.json();
    store.value = {
      _id: typeof s._id === 'string' ? s._id : s._id?.$oid ?? '',
      name: s.name,
      addr: s.addr,
      desc: s.desc,
      image_url: s.image_url,
    };
  }
}

onMounted(load);

function toAbs(u?: string) {
  const src = u && u.length > 0 ? u : placeholderUrl;
  if (src.startsWith('http://') || src.startsWith('https://')) return src;
  return src.startsWith('/') ? `${API}${src}` : src;
}

// Демо-аналитика
const kpis = [
  { title: 'Средний чек', value: '480 ₽' },
  { title: 'Чеков за 7 дней', value: '12' },
  { title: 'Изм. цен (7д)', value: '-3%' },
  { title: 'Уникальных товаров', value: '35' },
];

const priceInsight = {
  product: { label: 'Burn PM Zero', price: 169 },
  cityAvg: 175,
  trend7d: -2,
  spark: [165, 169, 172, 170, 169, 168, 169],
};

const sparkPoints = computed(() => {
  const pts = priceInsight.spark;
  return pts.map((v, i) => `${(i/(pts.length-1))*100},${50-((v-165)/(10))*50}`).join(' ');
});

const days = ['Пн','Вт','Ср','Чт','Пт','Сб','Вс'];
function rnd(seed:number){ let x=seed; return ()=> (x = (x*9301+49297)%233280)/233280 }
const R = rnd(42);
const heatmap:number[][] = days.map((_)=> Array.from({length:24},()=> Math.floor(R()*5)));
function color(v:number){ return `hsl(160 70% ${90 - v*12}%)`; }

// Store items logic
function idOf(x:any){ return typeof x === 'string' ? x : x?.$oid; }

async function loadProducts(){
  const res = await fetch(`${API}/products`);
  if (res.ok){
    const list = await res.json();
    allProducts.value = list.map((p:any)=> ({ _id: idOf(p._id), title: p.title, image_url: p.image_url }));
  }
}

async function loadStoreItems(){
  const res = await fetch(`${API}/stores/${id}/products`);
  if (res.ok){
    const arr = await res.json();
    storeItems.value = arr.map((it:any)=> ({ key: `${idOf(it._id)||idOf(it.product_id)}`, product_id: idOf(it.product_id), price: it.price, product: it.product ? { _id: idOf(it.product._id), title: it.product.title, image_url: it.product.image_url } : null }));
    const map: Record<string, number> = {};
    for (const it of storeItems.value){ map[it.key] = Number(it.price) || 0; }
    priceEdit.value = map;
  }
}

const productsForSelect = computed(()=> {
  const exist = new Set(storeItems.value.map(it=> it.product_id));
  return allProducts.value.filter((p:any)=> !exist.has(p._id));
});

async function addItem(){
  if (!newItem.value.product_id || newItem.value.price==null) return;
  await fetch(`${API}/stores/${id}/products`, { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ product_id: newItem.value.product_id, price: Number(newItem.value.price) }) });
  newItem.value = { product_id: '', price: null };
  await loadStoreItems();
}

async function savePrice(it:any){
  const price = Number(priceEdit.value[it.key]);
  await fetch(`${API}/stores/${id}/products/${it.product_id}`, { method: 'PUT', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ price }) });
  await loadStoreItems();
}

async function removeItem(it:any){
  await fetch(`${API}/stores/${id}/products/${it.product_id}`, { method: 'DELETE' });
  await loadStoreItems();
}

onMounted(async()=>{ await loadProducts(); await loadStoreItems(); });

// Activities
const activities = ref<any[]>([]);
function formatDate(ms:number){ const d = new Date(ms); return d.toLocaleString(); }
function activityTitle(a:any){
  if (a.kind==='item_added') return `Добавлен товар ${a.product_name || ''}`.trim();
  if (a.kind==='price_updated' || a.kind==='price_set') return `Обновлена цена на товар ${a.product_name || ''}`.trim();
  if (a.kind==='item_removed') return `Товар удален ${a.product_name || ''}`.trim();
  return 'Событие';
}
async function loadActivities(){
  const res = await fetch(`${API}/stores/${id}/activities`);
  if (res.ok){
    const arr = await res.json();
    activities.value = arr.map((x:any)=> ({ key: (typeof x._id==='string'? x._id : x._id?.$oid) || `${x.ts_ms}`, product_id: (typeof x.product_id==='string'? x.product_id : x.product_id?.$oid) || null, kind: x.kind, ts_ms: x.ts_ms, price: x.price, product_name: x.product_name }));
  }
}
onMounted(loadActivities);
</script>
