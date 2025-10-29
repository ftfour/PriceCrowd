<template>
  <section class="space-y-8">
    <div class="flex items-end justify-between gap-4">
      <h2 class="text-2xl font-semibold">ÐœÐ°Ð³Ð°Ð·Ð¸Ð½Ñ‹</h2>
      <div class="flex items-center gap-3">
        <input
          v-model="query"
          type="text"
          placeholder="ÐŸÐ¾Ð¸ÑÐº Ð¿Ð¾ Ð¼Ð°Ð³Ð°Ð·Ð¸Ð½Ð°Ð¼..."
          class="w-64 rounded-md border px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
        />
        <RouterLink v-if="isAdmin" to="/stores/new" class="rounded-md bg-blue-600 text-white px-3 py-2 text-sm">Ð”Ð¾Ð±Ð°Ð²Ð¸Ñ‚ÑŒ Ð¼Ð°Ð³Ð°Ð·Ð¸Ð½</RouterLink>
      </div>
    </div>

    <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
      <article v-for="s in filtered" :key="s._id" class="rounded-lg border bg-white p-0 overflow-hidden flex flex-col">
        <div class="h-40 bg-slate-50 flex items-center justify-center border-b">
          <img :src="toAbs(s.image_url)" alt="store" class="max-h-36 object-contain" />
        </div>
        <div class="p-4 flex flex-col gap-3">
          <header class="flex items-start justify-between gap-2">
            <div class="min-w-0">
              <RouterLink :to="`/stores/${s._id}`" class="font-medium text-gray-900 truncate hover:underline">{{ s.name }}</RouterLink>
              <p class="mt-1 text-[11px] text-gray-500 break-all">ID: {{ s._id }}</p>
            </div>
            <div class="flex gap-2" v-if="isAdmin">
              <RouterLink :to="`/stores/${s._id}/edit`" class="rounded-md border px-2 py-1 text-xs">Ð ÐµÐ´Ð°ÐºÑ‚Ð¸Ñ€Ð¾Ð²Ð°Ñ‚ÑŒ</RouterLink>
              <button @click="removeStore(s)" class="rounded-md bg-red-600 text-white px-2 py-1 text-xs">Ð£Ð´Ð°Ð»Ð¸Ñ‚ÑŒ</button>
            </div>
          </header>
          <p class="text-sm text-gray-700">{{ s.addr }}</p>
          <p class="text-xs text-gray-500">{{ s.desc }}</p>
        </div>
      </article>
    </div>
  </section>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue';
import { useAuth } from '../auth';
import { authHeaders } from '../api';

type Store = { _id: string; name: string; addr: string; desc?: string; image_url?: string };

const API = import.meta.env.VITE_API_URL || 'http://localhost:8080';
const stores = ref<Store[]>([]);
const query = ref('');
const placeholderUrl = '/placeholder-can.svg';

const filtered = computed(() => {
  const q = query.value.trim().toLowerCase();
  if (!q) return stores.value;
  return stores.value.filter(s =>
    s.name.toLowerCase().includes(q) ||
    s.addr.toLowerCase().includes(q) ||
    (s.desc || '').toLowerCase().includes(q)
  );
});

function toAbs(u?: string) {
  const src = u && u.length > 0 ? u : placeholderUrl;
  if (src.startsWith('http://') || src.startsWith('https://')) return src;
  return src.startsWith('/') ? `${API}${src}` : src;
}

async function fetchStores() {
  const res = await fetch(`${API}/stores`);
  const data = await res.json();
  stores.value = (Array.isArray(data) ? data : []).map((s: any) => ({
    _id: typeof s._id === 'string' ? s._id : s._id?.$oid ?? '',
    name: s.name,
    addr: s.addr,
    desc: s.desc,
    image_url: s.image_url,
  }));
}

async function removeStore(s: Store) {
  await fetch(`${API}/stores/${s._id}`, { method: 'DELETE', headers: authHeaders() });
  await fetchStores();
}

onMounted(fetchStores);

const auth = useAuth();
const isAdmin = computed(() => auth.isAdmin.value === true || auth.state.role === "admin");
</script>



