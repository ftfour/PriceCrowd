<template>
  <section class="space-y-6">
    <h1 class="text-2xl font-semibold">{{ isEdit ? 'Изменение магазина' : 'Добавление магазина' }}</h1>
    <form @submit.prevent="onSubmit" class="space-y-4 bg-white border rounded-lg p-4 max-w-2xl">
      <p v-if="errorMsg" class="text-sm text-rose-600 bg-rose-50 border border-rose-200 rounded p-2">{{ errorMsg }}</p>
      <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
        <div>
          <label class="block text-sm text-slate-600 mb-1">Название</label>
          <input v-model="form.name" required class="w-full rounded-md border px-3 py-2 text-sm" />
        </div>
        <div>
          <label class="block text-sm text-slate-600 mb-1">Адрес</label>
          <input v-model="form.addr" required class="w-full rounded-md border px-3 py-2 text-sm" />
        </div>
      </div>
      <div>
        <label class="block text-sm text-slate-600 mb-1">Описание</label>
        <textarea v-model="form.desc" rows="4" class="w-full rounded-md border px-3 py-2 text-sm"></textarea>
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
        <RouterLink to="/stores" class="rounded-md border px-3 py-2 text-sm">Отмена</RouterLink>
        <span v-if="loading" class="self-center text-xs text-slate-500">Загрузка...</span>
      </div>
    </form>
  </section>
</template>

<script setup lang="ts">
import { onMounted, ref, computed } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { authHeaders } from '../api';

const API = import.meta.env.VITE_API_URL || 'http://localhost:8080';
const route = useRoute();
const router = useRouter();
const id = route.params.id as string | undefined;
const isEdit = computed(() => !!id);
const placeholderUrl = '/placeholder-can.svg';

const form = ref<{ name: string; addr: string; desc?: string; image_url?: string }>({ name: '', addr: '', desc: '' });
const file = ref<File | null>(null);
const preview = ref<string | null>(null);
const loading = ref(false);
const errorMsg = ref<string | null>(null);

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
  const res = await fetch(`${API}/upload`, { method: 'POST', body: data, headers: authHeaders() });
  if (!res.ok) throw new Error('Upload failed');
  const json = await res.json();
  return json.url as string;
}

async function onSubmit() {
  loading.value = true;
  errorMsg.value = null;
  try {
    const image_url = await uploadIfNeeded();
    const payload: any = { ...form.value };
    if (image_url) payload.image_url = image_url;
    if (isEdit.value && id) {
      const res = await fetch(`${API}/stores/${id}`, { method: 'PUT', headers: authHeaders({ 'Content-Type': 'application/json' }), body: JSON.stringify(payload) });
      if (!res.ok) {
        const txt = await res.text();
        throw new Error(`Ошибка сохранения: ${res.status} ${txt}`);
      }
      router.push(`/stores/${id}`);
    } else {
      const res = await fetch(`${API}/stores`, { method: 'POST', headers: authHeaders({ 'Content-Type': 'application/json' }), body: JSON.stringify(payload) });
      if (!res.ok) {
        const txt = await res.text();
        throw new Error(`Ошибка создания: ${res.status} ${txt}`);
      }
      const created = await res.json();
      const newId = typeof created._id === 'string' ? created._id : created._id?.$oid;
      if (!newId) throw new Error('Сервер не вернул _id');
      router.push(`/stores/${newId}`);
    }
  } catch (e: any) {
    errorMsg.value = e?.message || 'Не удалось выполнить операцию';
  } finally {
    loading.value = false;
  }
}

async function loadIfEdit() {
  if (!isEdit.value || !id) return;
  const res = await fetch(`${API}/stores/${id}`);
  if (res.ok) {
    const s = await res.json();
    form.value = {
      name: s.name,
      addr: s.addr,
      desc: s.desc,
      image_url: s.image_url,
    };
  }
}

onMounted(loadIfEdit);
</script>
