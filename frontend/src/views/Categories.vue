<template>
  <section class="space-y-8">
    <div class="flex items-end justify-between gap-4">
      <h2 class="text-2xl font-semibold">Категории</h2>
      <RouterLink to="/categories/new" class="rounded-md bg-blue-600 text-white px-3 py-2 text-sm">Добавить категорию</RouterLink>
    </div>

    <div>
      <h3 class="text-lg font-semibold mb-2">Дерево категорий</h3>
      <div class="rounded-lg border bg-white p-4">
        <CategoryTree :categories="categories" :selectable="false" />
      </div>
    </div>

    <div class="rounded-lg border bg-white overflow-hidden">
      <table class="w-full text-left">
        <thead class="bg-slate-50 text-slate-500 text-sm">
          <tr>
            <th class="px-5 py-3">Название</th>
            <th class="px-5 py-3">Родители</th>
            <th class="px-5 py-3">Описание</th>
            <th class="px-5 py-3">Действия</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="c in categories" :key="c._id_str" class="border-t border-slate-100">
            <td class="px-5 py-3 font-medium text-slate-800">{{ c.name }}</td>
            <td class="px-5 py-3 text-slate-600">
              <span v-if="c.parent_ids.length===0" class="text-slate-400 text-xs">—</span>
              <span v-else class="flex flex-wrap gap-1">
                <span v-for="pid in c.parent_ids" :key="pid" class="text-xs bg-slate-100 border border-slate-200 rounded px-2 py-1">{{ nameById(pid) }}</span>
              </span>
            </td>
            <td class="px-5 py-3 text-slate-600">{{ c.desc }}</td>
            <td class="px-5 py-3">
              <RouterLink :to="`/categories/${c._id_str}/edit`" class="text-blue-600 hover:underline text-sm">Изменить</RouterLink>
              <button @click="remove(c)" class="ml-3 text-rose-600 hover:underline text-sm">Удалить</button>
            </td>
          </tr>
        </tbody>
      </table>
    </div>
  </section>
</template>

<script setup lang="ts">
import { onMounted, ref } from 'vue';
import { authHeaders } from '../api';
import CategoryTree from '../components/CategoryTree.vue';

import { API } from '../api';
const categories = ref<any[]>([]);

function mapId(x:any){ return (typeof x === 'string' ? x : x?.$oid); }

async function load(){
  const res = await fetch(`${API}/categories`);
  if (res.ok){
    const list = await res.json();
    categories.value = list.map((c:any)=> ({...c, _id_str: mapId(c._id), parent_ids: (c.parent_ids||[]).map(mapId)}));
  }
}

function nameById(id:string){
  const c = categories.value.find((x:any)=> x._id_str===id);
  return c?.name || id;
}

async function remove(c:any){
  await fetch(`${API}/categories/${c._id_str}`, { method: 'DELETE', headers: authHeaders() });
  await load();
}

onMounted(load);
</script>
