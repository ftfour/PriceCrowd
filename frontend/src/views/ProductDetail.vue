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
          <span v-for="cid in product.category_ids" :key="cid" class="text-xs bg-slate-100 border border-slate-200 rounded px-2 py-1">{{ categoryName(cid) }}</span>
        </div>
        <div class="flex gap-2">
          <RouterLink :to="`/products/${id}/edit`" class="rounded-md border px-3 py-2 text-sm">Редактировать</RouterLink>
          <RouterLink to="/products" class="rounded-md px-3 py-2 text-sm bg-blue-600 text-white">Назад к списку</RouterLink>
        </div>
      </div>
    </div>
  </section>
  <div v-else class="text-slate-500">Загрузка...</div>
</template>

<script setup lang="ts">
import { onMounted, ref } from 'vue';
import { useRoute } from 'vue-router';

const API = import.meta.env.VITE_API_URL || 'http://localhost:8080';
const route = useRoute();
const id = route.params.id as string;
const placeholderUrl = '/placeholder-can.svg';

const product = ref<any | null>(null);
const categories = ref<any[]>([]);

async function load() {
  const res = await fetch(`${API}/products/${id}`);
  if (res.ok) {
    const p = await res.json();
    product.value = {
      _id: typeof p._id === 'string' ? p._id : p._id?.$oid ?? '',
      title: p.title,
      desc: p.desc,
      image_url: p.image_url,
      category_ids: (p.category_ids || []).map((x:any)=> typeof x === 'string' ? x : x?.$oid),
    };
  }
  const resCats = await fetch(`${API}/categories`);
  if (resCats.ok) categories.value = await resCats.json();
}

onMounted(load);

function toAbs(u?: string) {
  const src = u && u.length > 0 ? u : placeholderUrl;
  if (src.startsWith('http://') || src.startsWith('https://')) return src;
  return src.startsWith('/') ? `${API}${src}` : src;
}

function categoryName(id: string) {
  const c = categories.value.find((x:any)=> (typeof x._id === 'string' ? x._id : x._id?.$oid) === id);
  return c?.name || id;
}
</script>

