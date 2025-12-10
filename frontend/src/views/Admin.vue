<template>
  <section class="space-y-6">
    <header class="flex items-center justify-between">
      <h1 class="text-2xl font-semibold">Администрирование</h1>
    </header>

    <div class="rounded-lg border bg-white">
      <nav class="border-b px-4 py-2 text-sm flex gap-4 items-center">
        <button :class="tabClass('telegram')" @click="tab='telegram'">Телеграм-бот</button>
        <button :class="tabClass('users')" @click="tab='users'">Пользователи</button>
        <button :class="tabClass('about')" @click="tab='about'">О системе</button>
        <div class="ml-auto flex items-center gap-4">
          <RouterLink to="/admin/receipts" class="text-gray-700 hover:text-black">Чеки</RouterLink>
          <RouterLink to="/admin/operations" class="text-gray-700 hover:text-black">Операции</RouterLink>
          <RouterLink to="/admin/reports" class="text-gray-700 hover:text-black">Отчёты</RouterLink>
          <RouterLink to="/admin/heatmap" class="text-gray-700 hover:text-black">Теплокарта</RouterLink>
        </div>
      </nav>
      <div class="p-4">
        <TelegramSettings v-if="tab==='telegram'" />
        <AdminUsers v-else-if="tab==='users'" />
        <div v-else class="text-sm text-slate-700 space-y-3">
          <p>SPA: Vue 3 + Vite. Backend: Rust + Axum + MongoDB.</p>
          <div class="rounded-md border p-3 bg-slate-50">
            <div class="font-medium mb-2">Быстрые действия</div>
            <div class="flex gap-2 flex-wrap">
              <button @click="seed" class="rounded-md bg-green-600 text-white px-3 py-1.5">Заполнить сайт тысячами тестовыми записями</button>
              <button @click="clearAll" class="rounded-md bg-red-600 text-white px-3 py-1.5">Показать только оригинальные данные</button>
              <button @click="downloadExport" class="rounded-md bg-blue-600 text-white px-3 py-1.5">Выгрузить данные</button>
            </div>
            <div v-if="actionMsg" class="text-xs text-slate-600 mt-2">{{ actionMsg }}</div>
          </div>
        </div>
      </div>
    </div>
  </section>
  
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { RouterLink } from 'vue-router';
import TelegramSettings from './admin/TelegramSettings.vue';
import AdminUsers from './admin/Users.vue';
import { API, authHeaders } from '../api';

const tab = ref<'telegram'|'users'|'about'>('about');
const actionMsg = ref('');

async function seed() {
  if (!confirm('Заполнить сайт тестовыми данными? Будет создано большое количество записей.')) return;
  try {
    actionMsg.value = 'Начинаю создание тестовых данных...';
    const res = await fetch(`${API}/dev/seed`, { method: 'POST', headers: authHeaders() });
    actionMsg.value = res.ok ? 'Тестовые данные создаются в фоновом режиме' : `Ошибка создания: ${res.status}`;
  } catch (e: any) {
    actionMsg.value = e?.message || 'Сетевая ошибка';
  }
}

async function clearAll() {
  if (!confirm('Удалить тестовые данные? Продукты, магазины, категории, чеки, события, созданные для теста, будут удалены.')) return;
  try {
    const res = await fetch(`${API}/dev/clear`, { method: 'POST', headers: authHeaders() });
    actionMsg.value = res.ok ? 'Данные очищены' : `Ошибка очистки: ${res.status}`;
  } catch (e: any) {
    actionMsg.value = e?.message || 'Сетевая ошибка';
  }
}

async function downloadExport() {
  try {
    const res = await fetch(`${API}/export`, { headers: authHeaders() });
    if (!res.ok) { actionMsg.value = `Ошибка выгрузки: ${res.status}`; return; }
    const blob = await res.blob();
    // Try to extract filename from headers
    let filename = 'pricecrowd_export.json';
    const cd = res.headers.get('content-disposition') || '';
    const m = /filename="?([^";]+)"?/i.exec(cd);
    if (m && m[1]) filename = m[1];
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url; a.download = filename; document.body.appendChild(a); a.click(); a.remove();
    URL.revokeObjectURL(url);
    actionMsg.value = 'Выгрузка началась';
  } catch (e: any) {
    actionMsg.value = e?.message || 'Сетевая ошибка';
  }
}

function tabClass(name: string){
  return [
    'px-3 py-1.5 rounded-md',
    tab.value===name ? 'bg-blue-600 text-white' : 'text-gray-700 hover:text-black'
  ];
}
</script>
