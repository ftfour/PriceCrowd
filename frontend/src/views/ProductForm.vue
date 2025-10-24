<template>
  <section class="space-y-6">
    <h1 class="text-2xl font-semibold">{{ isEdit ? 'Изменение товара' : 'Добавление товара' }}</h1>
    <form @submit.prevent="onSubmit" class="space-y-4 bg-white border rounded-lg p-4 max-w-2xl">
      <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
        <div>
          <label class="block text-sm text-slate-600 mb-1">Название</label>
          <input v-model="form.title" required class="w-full rounded-md border px-3 py-2 text-sm" />
        </div>
        <div>
          <label class="block text-sm text-slate-600 mb-1">Категории</label>
          <div class="flex flex-col gap-2">
            <div class="flex flex-wrap gap-2">
              <span v-for="cid in selectedCats" :key="cid" class="text-xs bg-slate-100 border border-slate-200 rounded px-2 py-1 flex items-center gap-1">
                {{ categoryName(cid) }}
                <button type="button" @click="removeCat(cid)" class="text-slate-500 hover:text-rose-600">×</button>
              </span>
              <span v-if="selectedCats.length===0" class="text-xs text-slate-400">Категории не выбраны</span>
            </div>
            <div class="flex items-center gap-2">
              <select v-model="addCat" @change="addCatNow" class="text-sm border rounded px-2 py-2">
                <option value="" disabled>+ Добавить категорию</option>
                <option v-for="c in availableCategories" :key="c._id_str" :value="c._id_str">{{ c.name }}</option>
              </select>
              <RouterLink to="/categories/new" class="text-xs text-blue-600 hover:underline">Создать категорию</RouterLink>
            </div>
          </div>
        </div>
      </div>
      <div>
        <label class="block text-sm text-slate-600 mb-1">Описание</label>
        <textarea v-model="form.desc" required rows="4" class="w-full rounded-md border px-3 py-2 text-sm"></textarea>
      </div>
      <div class="grid grid-cols-1 md:grid-cols-[200px_1fr] gap-4 items-start">
        <div class="bg-white border rounded-lg p-2 flex items-center justify-center">
          <img :src="preview || form.image_url || placeholderUrl" alt="preview" class="max-h-48 object-contain" />
        </div>
        <div>
          <label class="block text-sm text-slate-600 mb-1">Изображение</label>
          <input type="file" accept="image/*" @change="onFileChange" />
          <p class="text-xs text-slate-500 mt-1">Можно не загружать — останется текущее изображение или плейсхолдер.</p>
        </div>
      </div>
      <div class="flex gap-2">
        <button type="submit" :disabled="loading" class="rounded-md bg-blue-600 text-white px-3 py-2 text-sm disabled:opacity-60">
          {{ isEdit ? 'Сохранить' : 'Добавить' }}
        </button>
        <RouterLink to="/products" class="rounded-md border px-3 py-2 text-sm">Отмена</RouterLink>
        <span v-if="loading" class="self-center text-xs text-slate-500">Загрузка...</span>
      </div>
    </form>
  </section>
  
</template>

<script setup lang="ts">
import { onMounted, ref, computed } from 'vue';
import { useRoute, useRouter, RouterLink } from 'vue-router';

const API = import.meta.env.VITE_API_URL || 'http://localhost:8080';
const route = useRoute();
const router = useRouter();
const id = route.params.id as string | undefined;
const isEdit = computed(() => !!id);
const placeholderUrl = '/placeholder-can.svg';

const form = ref<{ title: string; desc: string; image_url?: string }>({ title: '', desc: '' });
const allCats = ref<any[]>([]);
const selectedCats = ref<string[]>([]);
const addCat = ref<string>('');
const file = ref<File | null>(null);
const preview = ref<string | null>(null);
const loading = ref(false);

function onFileChange(e: Event) {
  const input = e.target as HTMLInputElement;
  const f = input.files?.[0] || null;
  file.value = f;
  if (f) {
    const reader = new FileReader();
    reader.onload = () => { preview.value = reader.result as string; };
    reader.readAsDataURL(f);
  } else { preview.value = null; }
}

async function uploadIfNeeded(): Promise<string | undefined> {
  if (!file.value) return undefined;
  const data = new FormData();
  data.append('file', file.value);
  const res = await fetch(`${API}/upload`, { method: 'POST', body: data });
  if (!res.ok) throw new Error('Upload failed');
  const json = await res.json();
  return json.url as string;
}

async function onSubmit() {
  loading.value = true;
  try {
    const image_url = await uploadIfNeeded();
    const payload: any = { ...form.value, category_ids: selectedCats.value };
    if (image_url) payload.image_url = image_url;
    if (isEdit.value && id) {
      await fetch(`${API}/products/${id}`, { method: 'PUT', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(payload) });
      await router.push(`/products/${id}`);
    } else {
      const res = await fetch(`${API}/products`, { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(payload) });
      const created = await res.json();
      const newId = typeof created._id === 'string' ? created._id : created._id?.$oid;
      await router.push(`/products/${newId}`);
    }
  } finally {
    loading.value = false;
  }
}

async function loadIfEdit() {
  if (!isEdit.value || !id) return;
  const res = await fetch(`${API}/products/${id}`);
  if (res.ok) {
    const p = await res.json();
    form.value = { title: p.title, desc: p.desc, image_url: p.image_url };
    selectedCats.value = (p.category_ids || []).map((x:any)=> typeof x==='string'?x:x?.$oid);
  }
}

function idOf(x:any){ return typeof x === 'string' ? x : x?.$oid; }

async function loadCats(){
  const res = await fetch(`${API}/categories`);
  if (res.ok) {
    const list = await res.json();
    allCats.value = list.map((c:any)=> ({...c, _id_str: idOf(c._id)}));
  }
}

const availableCategories = computed(()=> {
  const chosen = new Set(selectedCats.value);
  return allCats.value.filter((c:any)=> !chosen.has(c._id_str));
});

function categoryName(id: string){
  const c = allCats.value.find((x:any)=> x._id_str===id);
  return c?.name || id;
}

function addCatNow(){
  const v = addCat.value;
  if (!v) return;
  if (!selectedCats.value.includes(v)) selectedCats.value = [...selectedCats.value, v];
  addCat.value = '';
}

function removeCat(id: string){
  selectedCats.value = selectedCats.value.filter(x=> x!==id);
}

onMounted(async()=>{ await loadCats(); await loadIfEdit(); });
</script>

