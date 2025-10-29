<template>
  <section class="space-y-6">
    <h1 class="text-2xl font-semibold">Рейтинг пользователей</h1>
    <div class="rounded-lg border bg-white overflow-hidden">
      <table class="w-full text-left">
        <thead class="bg-slate-50 text-slate-500 text-sm">
          <tr>
            <th class="px-4 py-2 w-16">#</th>
            <th class="px-4 py-2">Имя (Telegram)</th>
            <th class="px-4 py-2 w-32">Очки</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="(u,i) in list" :key="i" class="border-t">
            <td class="px-4 py-2">{{ i+1 }}</td>
            <td class="px-4 py-2">{{ u.name || '—' }}</td>
            <td class="px-4 py-2">{{ u.points }}</td>
          </tr>
          <tr v-if="list.length===0"><td colspan="3" class="px-4 py-6 text-center text-slate-500">Нет записей</td></tr>
        </tbody>
      </table>
    </div>
  </section>
</template>

<script setup lang="ts">
import { onMounted, ref } from 'vue';
import { API } from '../api';
const list = ref<Array<{ name: string; points: number }>>([]);
async function load(){
  const res = await fetch(`${API}/ratings/users`);
  if (res.ok){ list.value = await res.json(); }
}
onMounted(load);
</script>

