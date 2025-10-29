<template>
  <section class="space-y-6">
    <div class="flex items-end justify-between gap-4">
      <h2 class="text-2xl font-semibold">Товары</h2>
      <div class="flex items-center gap-3">
        <input
          v-model="query"
          type="text"
          placeholder="Поиск по товарам..."
          class="w-64 rounded-md border px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
        />
        <RouterLink
          v-if="isAdmin"
          to="/products/new"
          class="rounded-md bg-blue-600 text-white px-3 py-2 text-sm"
        >
          Добавить товар
        </RouterLink>
      </div>
    </div>

    <div class="rounded-lg border bg-white divide-y">
      <div
        v-for="p in filtered"
        :key="p._id"
        class="p-4 flex items-center gap-4"
      >
        <div
          class="h-16 w-16 sm:h-20 sm:w-20 rounded-xl bg-slate-50 border flex items-center justify-center overflow-hidden"
        >
          <img
            :src="toAbs(p.image_url)"
            alt="product"
            class="max-h-14 sm:max-h-16 object-contain"
          />
        </div>
        <div class="flex-1 min-w-0">
          <div class="flex items-start justify-between gap-3">
            <div class="min-w-0">
              <RouterLink
                :to="`/products/${p._id}`"
                class="font-medium text-gray-900 hover:underline truncate block"
              >
                {{ p.title }}
              </RouterLink>
              <p class="mt-1 text-xs text-gray-500 break-all">ID: {{ p._id }}</p>
            </div>
            <div class="flex gap-2 shrink-0" v-if="isAdmin">
              <RouterLink
                :to="`/products/${p._id}/edit`"
                class="rounded-md border px-3 py-1.5 text-xs"
              >
                Редактировать
              </RouterLink>
              <button
                @click="remove(p)"
                class="rounded-md bg-red-600 text-white px-3 py-1.5 text-xs"
              >
                Удалить
              </button>
            </div>
          </div>
          <p class="text-sm text-gray-700 mt-2 line-clamp-2">
            {{ p.desc }}
          </p>
          <div class="mt-2 flex flex-wrap gap-2">
            <span
              v-for="cid in p.category_ids"
              :key="cid"
              class="text-xs bg-slate-100 border border-slate-200 rounded px-2 py-1"
            >
              {{ categoryName(cid) }}
            </span>
          </div>
        </div>
      </div>
      <div
        v-if="filtered.length === 0"
        class="p-8 text-center text-slate-500"
      >
        Нет товаров
      </div>
    </div>
  </section>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue';
import { useAuth } from '../auth';
import { API, authHeaders } from '../api';

type Product = {
  _id: string;
  title: string;
  desc: string;
  image_url?: string;
  category_ids?: string[];
};

const products = ref<Product[]>([]);
const categories = ref<any[]>([]);
const query = ref('');
const placeholderUrl = '/placeholder-can.svg';

const filtered = computed(() => {
  const q = query.value.trim().toLowerCase();
  if (!q) return products.value;
  return products.value.filter(
    (p) =>
      p.title.toLowerCase().includes(q) ||
      p.desc.toLowerCase().includes(q)
  );
});

function toAbs(u?: string) {
  const src = u && u.length > 0 ? u : placeholderUrl;
  if (src.startsWith('http://') || src.startsWith('https://')) return src;
  return src.startsWith('/') ? `${API}${src}` : src;
}

async function fetchProducts() {
  const res = await fetch(`${API}/products`);
  const data = await res.json();
  products.value = (Array.isArray(data) ? data : []).map((p: any) => ({
    _id: typeof p._id === 'string' ? p._id : p._id?.$oid ?? '',
    title: p.title,
    desc: p.desc,
    image_url: p.image_url,
    category_ids: (p.category_ids || []).map((x: any) =>
      typeof x === 'string' ? x : x?.$oid
    ),
  }));

  const resCats = await fetch(`${API}/categories`);
  if (resCats.ok) {
    const list = await resCats.json();
    categories.value = list.map((c: any) => ({
      ...c,
      _id_str:
        typeof c._id === 'string' ? c._id : c._id?.$oid,
    }));
  }
}

async function remove(p: Product) {
  await fetch(`${API}/products/${p._id}`, {
    method: 'DELETE',
    headers: authHeaders(),
  });
  await fetchProducts();
}

onMounted(fetchProducts);

function categoryName(id: string) {
  const c = categories.value.find((x: any) => x._id_str === id);
  return c?.name || id;
}

const auth = useAuth();
const isAdmin = computed(
  () => auth.isAdmin.value === true || auth.state.role === 'admin'
);
</script>

<style scoped>
.line-clamp-2 {
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}
</style>
