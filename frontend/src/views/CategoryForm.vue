<template>
  <section class="space-y-6">
    <h1 class="text-2xl font-semibold">{{ isEdit ? 'Изменение категории' : 'Добавление категории' }}</h1>
    <form @submit.prevent="onSubmit" class="space-y-4 bg-white border rounded-lg p-4 max-w-2xl">
      <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
        <div>
          <label class="block text-sm text-slate-600 mb-1">Название</label>
          <input v-model="form.name" required class="w-full rounded-md border px-3 py-2 text-sm" />
        </div>
        <div>
          <label class="block text-sm text-slate-600 mb-1">Родительские категории</label>
          <select v-model="parentsSelected" multiple class="w-full rounded-md border px-3 py-2 text-sm h-32">
            <option v-for="c in allCats" :key="c._id_str" :value="c._id_str">{{ c.name }}</option>
          </select>
          <p class="text-xs text-slate-500 mt-1">Удерживайте Ctrl/Cmd для выбора нескольких.</p>
        </div>
      </div>
      <div>
        <label class="block text-sm text-slate-600 mb-1">Описание</label>
        <textarea v-model="form.desc" rows="4" class="w-full rounded-md border px-3 py-2 text-sm"></textarea>
      </div>
      <div class="flex gap-2">
        <button type="submit" :disabled="loading" class="rounded-md bg-blue-600 text-white px-3 py-2 text-sm disabled:opacity-60">
          {{ isEdit ? 'Сохранить' : 'Добавить' }}
        </button>
        <RouterLink to="/categories" class="rounded-md border px-3 py-2 text-sm">Отмена</RouterLink>
        <span v-if="loading" class="self-center text-xs text-slate-500">Загрузка...</span>
      </div>
    </form>
  </section>
</template>

<script setup lang="ts">
import { onMounted, ref, computed } from 'vue';
import { useRoute, useRouter } from 'vue-router';

const API = import.meta.env.VITE_API_URL || 'http://localhost:8080';
const route = useRoute();
const router = useRouter();
const id = route.params.id as string | undefined;
const isEdit = computed(() => !!id);

const form = ref<{ name: string; desc?: string; parent_ids?: string[] }>({ name: '', desc: '' });
const allCats = ref<any[]>([]);
const parentsSelected = ref<string[]>([]);
const loading = ref(false);

async function loadCats(){
  const res = await fetch(`${API}/categories`);
  if (res.ok){
    const list = await res.json();
    allCats.value = list.map((c:any)=> ({...c, _id_str: (typeof c._id==='string'? c._id : c._id?.$oid)}));
  }
}

async function loadIfEdit(){
  if (!isEdit.value || !id) return;
  const res = await fetch(`${API}/categories/${id}`);
  if (res.ok){
    const c = await res.json();
    form.value = { name: c.name, desc: c.desc };
    parentsSelected.value = (c.parent_ids || []).map((x:any)=> (typeof x==='string'?x:x?.$oid));
  }
}

async function onSubmit(){
  loading.value = true;
  try {
    const payload:any = { name: form.value.name, desc: form.value.desc, parent_ids: parentsSelected.value };
    if (isEdit.value && id){
      await fetch(`${API}/categories/${id}`, { method: 'PUT', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(payload) });
    } else {
      await fetch(`${API}/categories`, { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(payload) });
    }
    router.push('/categories');
  } finally { loading.value = false; }
}

onMounted(async()=>{ await loadCats(); await loadIfEdit(); });
</script>

