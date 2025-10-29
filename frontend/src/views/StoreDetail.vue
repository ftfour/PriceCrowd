<template>
  <div class="grid grid-cols-12 gap-6" v-if="store">
    <!-- ÐžÑÐ½Ð¾Ð²Ð½Ð¾Ð¹ ÐºÐ¾Ð½Ñ‚ÐµÐ½Ñ‚ -->
    <div class="col-span-12 lg:col-span-9 space-y-8">
      <!-- Ð˜Ð½Ñ„Ð¾ Ð¾ Ð¼Ð°Ð³Ð°Ð·Ð¸Ð½Ðµ -->
      <div class="p-6 flex items-center gap-6 rounded-lg border bg-white">
        <img :src="toAbs(store.image_url)" alt="store" class="w-20 h-20 rounded-2xl object-contain bg-slate-50 p-2" />
        <div class="flex-1">
          <h1 class="text-2xl font-bold text-slate-900">{{ store.name }}</h1>
          <p class="text-slate-600 mt-1">{{ store.addr }}</p>
          <p class="mt-3 text-slate-700 text-sm">{{ store.desc }}</p>
        </div>
      </div>

      <!-- ÐÐ½Ð°Ð»Ð¸Ñ‚Ð¸ÐºÐ° Ð¿Ð¾ Ð¼Ð°Ð³Ð°Ð·Ð¸Ð½Ñƒ -->
      <div>
        <h2 class="text-lg font-semibold mb-3">ÐÐ½Ð°Ð»Ð¸Ñ‚Ð¸ÐºÐ° Ð¿Ð¾ Ð¼Ð°Ð³Ð°Ð·Ð¸Ð½Ñƒ</h2>
        <div class="grid grid-cols-2 md:grid-cols-4 gap-4 mb-4">
          <div v-for="(k,i) in kpis" :key="i" class="rounded-lg border bg-white p-4">
            <div class="text-sm text-slate-500">{{ k.title }}</div>
            <div class="text-xl font-semibold text-slate-900 mt-1">{{ k.value }}</div>
          </div>
        </div>

        <div class="grid grid-cols-1 lg:grid-cols-2 gap-4">
          <!-- Ð¡Ð»Ð°Ð¹Ð´: ÑÑ€Ð°Ð²Ð½ÐµÐ½Ð¸Ðµ, Ð³Ñ€Ð°Ñ„Ð¸Ðº, Ñ‚Ð¾Ð²Ð°Ñ€ -->
          <div class="rounded-lg border bg-white p-3 sm:p-4 space-y-3">
            <!-- Ð¡Ñ€ÐµÐ´Ð½ÑÑ Ñ†ÐµÐ½Ð° Ð¿Ð¾ Ð³Ð¾Ñ€Ð¾Ð´Ñƒ -->
            <div class="flex items-baseline justify-between gap-3">
              <div class="text-sm sm:text-base text-slate-600">Ð¡Ñ€ÐµÐ´Ð½ÑÑ Ñ†ÐµÐ½Ð° Ð¿Ð¾ Ð³Ð¾Ñ€Ð¾Ð´Ñƒ</div>
              <div class="text-lg sm:text-2xl font-semibold text-emerald-600">
                {{ Number(currentSlide.cityAvg).toLocaleString('ru-RU') }} â‚½
              </div>
            </div>

            <!-- Ð“Ñ€Ð°Ñ„Ð¸Ðº -->
            <div>
              <div class="text-[11px] sm:text-xs text-slate-500 mb-1">Ð“Ñ€Ð°Ñ„Ð¸Ðº Ñ†ÐµÐ½Ñ‹ (Ð² ÑÑ‚Ð¾Ð¼ Ð¼Ð°Ð³Ð°Ð·Ð¸Ð½Ðµ)</div>
              <div class="w-full h-16 sm:h-20">
                <svg viewBox="0 0 100 50" class="w-full h-full">
                  <polyline fill="none" stroke="#94a3b8" stroke-width="2" :points="currentSlide.citySpark || ''" />
                  <polyline fill="none" stroke="#10b981" stroke-width="2" :points="currentSlide.spark" />
                </svg>
              </div>
            </div>

            <!-- ÐšÐ°Ñ€Ñ‚Ð¾Ñ‡ÐºÐ° Ñ‚Ð¾Ð²Ð°Ñ€Ð° -->
            <div class="grid grid-cols-[56px_1fr_auto] sm:grid-cols-[64px_1fr_auto_auto] items-center gap-3">
              <div class="h-14 w-14 sm:h-16 sm:w-16 rounded-xl bg-slate-50 border flex items-center justify-center overflow-hidden">
                <img :src="toAbs(currentSlide.image_url)" alt="prod" class="max-h-12 sm:max-h-14 object-contain" />
              </div>
              <div class="min-w-0">
                <div class="text-base sm:text-lg font-semibold text-slate-900 truncate">{{ currentSlide.title }}</div>
                <div
                  v-if="currentSlide.cheapest && currentSlide.cheapest.store_id && Number(currentSlide.cheapest.price) < Number(currentSlide.price)"
                  class="text-[11px] sm:text-xs mt-1 truncate"
                >
                  Ð”ÐµÑˆÐµÐ²Ð»Ðµ Ð²
                  <RouterLink :to="`/stores/${currentSlide.cheapest.store_id}`" class="text-blue-600 hover:underline">
                    {{ currentSlide.cheapest.store_name || 'Ð´Ñ€ÑƒÐ³Ð¾Ð¼ Ð¼Ð°Ð³Ð°Ð·Ð¸Ð½Ðµ' }}
                  </RouterLink>
                  â€” {{ Number(currentSlide.cheapest.price).toLocaleString('ru-RU') }} â‚½
                </div>
              </div>
              <div class="text-emerald-600 font-semibold text-sm sm:text-base">
                {{ Number(currentSlide.price).toLocaleString('ru-RU') }} â‚½
              </div>
              <div class="hidden sm:flex flex-col gap-1 ml-1">
                <button @click="prevSlide" title="ÐÐ°Ð·Ð°Ð´" class="h-7 w-7 rounded border text-slate-600">â€¹</button>
                <button @click="nextSlide" title="Ð’Ð¿ÐµÑ€Ñ‘Ð´" class="h-7 w-7 rounded border text-slate-600">â€º</button>
              </div>
            </div>
            <div class="sm:hidden flex justify-end gap-2">
              <button @click="prevSlide" title="ÐÐ°Ð·Ð°Ð´" class="h-8 px-3 rounded border text-slate-600">â€¹</button>
              <button @click="nextSlide" title="Ð’Ð¿ÐµÑ€Ñ‘Ð´" class="h-8 px-3 rounded border text-slate-600">â€º</button>
            </div>
          </div>

          <!-- ÐÐºÑ‚Ð¸Ð²Ð½Ð¾ÑÑ‚ÑŒ -->
          <div class="rounded-lg border bg-white p-4">
            <div class="flex items-center justify-between">
              <div class="text-sm text-slate-500">ÐÐºÑ‚Ð¸Ð²Ð½Ð¾ÑÑ‚ÑŒ</div>
              <RouterLink :to="`/stores/${id}/activity`" class="text-sm text-blue-600 hover:underline">Ð’ÑÑ Ð°ÐºÑ‚Ð¸Ð²Ð½Ð¾ÑÑ‚ÑŒ</RouterLink>
            </div>
            <ul class="mt-2 text-sm text-slate-700 space-y-2">
              <li v-for="a in activities.slice(0,4)" :key="a.key" class="flex items-start justify-between gap-2">
                <div class="flex-1 min-w-0">
                  <div class="font-medium text-slate-800">{{ activityTitle(a) }}</div>
                  <div class="text-xs text-slate-500">{{ formatDate(a.ts_ms) }}</div>
                </div>
                <RouterLink
                  v-if="a.product_id"
                  :to="`/products/${a.product_id}`"
                  class="shrink-0 text-xs text-blue-600 hover:underline"
                >Ð¢Ð¾Ð²Ð°Ñ€</RouterLink>
              </li>
              <li v-if="activities.length===0" class="text-slate-500 text-sm">Ð¡Ð¾Ð±Ñ‹Ñ‚Ð¸Ð¹ Ð¿Ð¾ÐºÐ° Ð½ÐµÑ‚</li>
            </ul>
          </div>
        </div>
      </div>

      <!-- Ð¢Ð¾Ð²Ð°Ñ€Ñ‹ Ð¼Ð°Ð³Ð°Ð·Ð¸Ð½Ð° -->
      <div class="space-y-3">
        <h2 class="text-lg font-semibold">Ð¢Ð¾Ð²Ð°Ñ€Ñ‹ Ð¼Ð°Ð³Ð°Ð·Ð¸Ð½Ð°</h2>
        <div class="rounded-lg border bg-white p-4">
          <div class="flex justify-end mb-2">
            <button v-if="isAuthed" @click="editMode=!editMode" class="text-xs border rounded px-2 py-1">
              {{ editMode ? 'Ð“Ð¾Ñ‚Ð¾Ð²Ð¾' : 'Ð ÐµÐ´Ð°ÐºÑ‚Ð¸Ñ€Ð¾Ð²Ð°Ñ‚ÑŒ ÑÐ¿Ð¸ÑÐ¾Ðº Ñ‚Ð¾Ð²Ð°Ñ€Ð¾Ð²' }}
            </button>
          </div>
          <form v-if="isAuthed && editMode" @submit.prevent="addItem" class="flex flex-wrap items-center gap-3">
            <select v-model="newItem.product_id" class="h-9 border rounded px-2 text-sm min-w-[240px]">
              <option value="" disabled>Ð’Ñ‹Ð±ÐµÑ€Ð¸Ñ‚Ðµ Ñ‚Ð¾Ð²Ð°Ñ€</option>
              <option v-for="p in productsForSelect" :key="p._id" :value="p._id">{{ p.title }}</option>
            </select>
            <input v-model.number="newItem.price" type="number" step="0.01" min="0" placeholder="Ð¦ÐµÐ½Ð°" class="h-9 w-28 border rounded px-2 text-sm" />
            <button type="submit" class="h-9 px-3 rounded bg-blue-600 text-white text-sm">Ð”Ð¾Ð±Ð°Ð²Ð¸Ñ‚ÑŒ</button>
          </form>
        </div>

        <div class="rounded-lg border bg-white divide-y">
          <div v-for="it in storeItems" :key="it.key" class="p-4 flex items-center gap-4">
            <div class="h-12 w-12 rounded bg-slate-50 border flex items-center justify-center overflow-hidden">
              <img :src="toAbs(it.product?.image_url)" class="max-h-10 object-contain" alt="" />
            </div>
            <div class="min-w-0 flex-1">
              <RouterLink :to="`/products/${it.product_id}`" class="font-medium text-slate-900 hover:underline truncate block">
                {{ it.product?.title || it.product_id }}
              </RouterLink>
              <div class="text-xs text-slate-500 break-all">{{ it.product_id }}</div>
            </div>
            <div class="flex items-center gap-2" v-if="isAuthed && editMode">
              <input v-model.number="priceEdit[it.key]" type="number" step="0.01" min="0" class="h-9 w-24 border rounded px-2 text-sm" />
              <button @click="savePrice(it)" class="h-9 px-3 rounded border text-sm">Ð¡Ð¾Ñ…Ñ€Ð°Ð½Ð¸Ñ‚ÑŒ</button>
              <button @click="removeItem(it)" class="h-9 px-3 rounded bg-red-600 text-white text-sm">Ð£Ð´Ð°Ð»Ð¸Ñ‚ÑŒ</button>
            </div>
          </div>
          <div v-if="storeItems.length===0" class="p-8 text-center text-slate-500">ÐÐµÑ‚ Ñ‚Ð¾Ð²Ð°Ñ€Ð¾Ð²</div>
        </div>
      </div>
    </div>

    <!-- Ð¡Ð°Ð¹Ð´Ð±Ð°Ñ€ -->
    <aside class="col-span-12 lg:col-span-3 space-y-4">
      <div>
        <h2 class="text-lg font-semibold mb-3">ÐÐ° ÐºÐ°Ñ€Ñ‚Ðµ</h2>
        <div class="rounded-lg border bg-[linear-gradient(180deg,#f5f7fb,#eef2f7)] h-64 flex items-center justify-center text-slate-400">
          ÐšÐ°Ñ€Ñ‚Ð° (Ð·Ð°Ð³Ð»ÑƒÑˆÐºÐ°)
        </div>
      </div>
      <div>
        <h2 class="text-lg font-semibold mb-3">Ð§Ð°ÑÑ‹ Ð¸ ÐºÐ¾Ð½Ñ‚Ð°ÐºÑ‚Ñ‹</h2>
        <div class="rounded-lg border bg-white p-4 text-sm text-slate-700 space-y-2">
          <div><span class="text-slate-500">Ð§Ð°ÑÑ‹:</span> ÐŸÐ½-Ð’Ñ 09:00â€“22:00</div>
          <div><span class="text-slate-500">Ð¢ÐµÐ».:</span> +7 (411) xxx-xx-xx</div>
          <div><span class="text-slate-500">Ð¡Ñ‚Ð°Ñ‚ÑƒÑ:</span> Ð¾Ñ‚ÐºÑ€Ñ‹Ñ‚</div>
          <div class="text-slate-400 text-xs">Ð˜Ð½Ñ„Ð¾Ñ€Ð¼Ð°Ñ†Ð¸Ñ Ð½Ð¾ÑÐ¸Ñ‚ Ð¾Ð·Ð½Ð°ÐºÐ¾Ð¼Ð¸Ñ‚ÐµÐ»ÑŒÐ½Ñ‹Ð¹ Ñ…Ð°Ñ€Ð°ÐºÑ‚ÐµÑ€</div>
        </div>
      </div>
    </aside>
  </div>

  <div v-else class="text-slate-500">Ð—Ð°Ð³Ñ€ÑƒÐ·ÐºÐ°...</div>
</template>

<script setup lang="ts">
import { onMounted, ref, computed } from 'vue';
import { useRoute, RouterLink } from 'vue-router';
import { API, authHeaders } from '../api';
const route = useRoute();
const id = route.params.id as string;
const placeholderUrl = '/placeholder-can.svg';

const store = ref<any | null>(null);
const editMode = ref(false);
const allProducts = ref<any[]>([]);
const storeItems = ref<any[]>([]);
const newItem = ref<{ product_id: string; price: number | null }>({ product_id: '', price: null });
const priceEdit = ref<Record<string, number>>({});

// Ð·Ð°Ð³Ñ€ÑƒÐ·ÐºÐ° Ð¼Ð°Ð³Ð°Ð·Ð¸Ð½Ð°
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

// KPI â€” Ð·Ð°Ð³Ð»ÑƒÑˆÐºÐ°
const kpis = [
  { title: 'Ð¡Ñ€ÐµÐ´Ð½Ð¸Ð¹ Ñ‡ÐµÐº', value: '480 â‚½' },
  { title: 'Ð§ÐµÐºÐ¾Ð² Ð·Ð° 7 Ð´Ð½ÐµÐ¹', value: '12' },
  { title: 'Ð˜Ð·Ð¼. Ñ†ÐµÐ½ (7Ð´)', value: '-3%' },
  { title: 'Ð£Ð½Ð¸ÐºÐ°Ð»ÑŒÐ½Ñ‹Ñ… Ñ‚Ð¾Ð²Ð°Ñ€Ð¾Ð²', value: '35' },
];

// Ð²ÑÐ¿Ð¾Ð¼Ð¾Ð³Ð°Ñ‚ÐµÐ»ÑŒÐ½Ñ‹Ðµ Ñ„ÑƒÐ½ÐºÑ†Ð¸Ð¸
function idOf(x:any){ return typeof x === 'string' ? x : x?.$oid; }
function makeSparkFromHistory(hist: Array<{ts_ms:number, price:number}>){
  if (!hist || hist.length===0) return '';
  const vals = hist.map(h=> Number(h.price));
  const min = Math.min(...vals), max = Math.max(...vals);
  const range = Math.max(1, max - min);
  return vals.map((v,i)=> `${(i/(vals.length-1))*100},${50-((v-min)/range)*50}`).join(' ');
}

const slides = ref<any[]>([]);
const slideIndex = ref(0);
const currentSlide = computed(()=> slides.value.length
  ? slides.value[slideIndex.value % slides.value.length]
  : { title: 'ÐÐµÑ‚ Ð´Ð°Ð½Ð½Ñ‹Ñ…', image_url: '', price: 0, cityAvg: 0, spark: '', cheapest: null }
);
function prevSlide(){ if (slides.value.length) slideIndex.value = (slideIndex.value - 1 + slides.value.length) % slides.value.length; }
function nextSlide(){ if (slides.value.length) slideIndex.value = (slideIndex.value + 1) % slides.value.length; }

// Ð¿Ñ€Ð¾Ð´ÑƒÐºÑ‚Ñ‹ Ð¸ Ñ†ÐµÐ½Ñ‹
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
    storeItems.value = arr.map((it:any)=> ({
      key: `${idOf(it._id)||idOf(it.product_id)}`,
      product_id: idOf(it.product_id),
      price: it.price,
      product: it.product ? { _id: idOf(it.product._id), title: it.product.title, image_url: it.product.image_url } : null
    }));
    const map: Record<string, number> = {};
    for (const it of storeItems.value){ map[it.key] = Number(it.price) || 0; }
    priceEdit.value = map;
  }

  // Ð¸Ð½ÑÐ°Ð¹Ñ‚Ñ‹
  const ins = await fetch(`${API}/stores/${id}/products/insights`);
  if (ins.ok){
    const data = await ins.json();
    slides.value = data.map((x:any)=> ({
      key: idOf(x.product_id),
      title: x.product_title,
      image_url: x.product_image_url,
      price: x.store_price ?? 0,
      cityAvg: x.city_avg ? Math.round(Number(x.city_avg)) : 0,
      cheapest: x.cheapest ? { store_id: idOf(x.cheapest.store_id), store_name: x.cheapest.store_name, price: x.cheapest.price } : null,
      spark: makeSparkFromHistory(x.history || []),
    }));
  }
}

const productsForSelect = computed(()=> {
  const exist = new Set(storeItems.value.map(it=> it.product_id));
  return allProducts.value.filter((p:any)=> !exist.has(p._id));
});

async function addItem(){
  if (!newItem.value.product_id || newItem.value.price==null) return;
  await fetch(`${API}/stores/${id}/products`, {
    method: 'POST',
    headers: authHeaders({ 'Content-Type': 'application/json' }),
    body: JSON.stringify({ product_id: newItem.value.product_id, price: Number(newItem.value.price) })
  });
  newItem.value = { product_id: '', price: null };
  await loadStoreItems();
}

async function savePrice(it:any){
  const price = Number(priceEdit.value[it.key]);
  await fetch(`${API}/stores/${id}/products/${it.product_id}`, {
    method: 'PUT',
    headers: authHeaders({ 'Content-Type': 'application/json' }),
    body: JSON.stringify({ price })
  });
  await loadStoreItems();
}

async function removeItem(it:any){
  await fetch(`${API}/stores/${id}/products/${it.product_id}`, { method: 'DELETE', headers: authHeaders() });
  await loadStoreItems();
}

onMounted(async()=>{ await loadProducts(); await loadStoreItems(); });

// ÐÐºÑ‚Ð¸Ð²Ð½Ð¾ÑÑ‚ÑŒ
const activities = ref<any[]>([]);
function formatDate(ms:number){ const d = new Date(ms); return d.toLocaleString(); }
function activityTitle(a:any){
  if (a.kind==='item_added') return `Ð”Ð¾Ð±Ð°Ð²Ð»ÐµÐ½ Ñ‚Ð¾Ð²Ð°Ñ€ ${a.product_name || ''}`.trim();
  if (a.kind==='price_updated' || a.kind==='price_set') return `ÐžÐ±Ð½Ð¾Ð²Ð»ÐµÐ½Ð° Ñ†ÐµÐ½Ð° Ð½Ð° Ñ‚Ð¾Ð²Ð°Ñ€ ${a.product_name || ''}`.trim();
  if (a.kind==='item_removed') return `Ð¢Ð¾Ð²Ð°Ñ€ ÑƒÐ´Ð°Ð»Ñ‘Ð½ ${a.product_name || ''}`.trim();
  return 'Ð¡Ð¾Ð±Ñ‹Ñ‚Ð¸Ðµ';
}

async function loadActivities(){
  const res = await fetch(`${API}/stores/${id}/activities`);
  if (res.ok){
    const arr = await res.json();
    activities.value = arr.map((x:any)=> ({
      key: (typeof x._id==='string'? x._id : x._id?.$oid) || `${x.ts_ms}`,
      product_id: (typeof x.product_id==='string'? x.product_id : x.product_id?.$oid) || null,
      kind: x.kind,
      ts_ms: x.ts_ms,
      price: x.price,
      product_name: x.product_name,
      store_name: x.store_name
    }));
  }
}
onMounted(loadActivities);

import { useAuth } from '../auth';
import { computed } from 'vue';
const auth = useAuth();
const isAuthed = computed(() => !!auth.state.token && !!auth.state.username && !!auth.state.role);
</script>
