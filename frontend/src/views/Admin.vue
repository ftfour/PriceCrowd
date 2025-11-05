<template>
  <section class="space-y-6">
    <header class="flex items-center justify-between">
      <h1 class="text-2xl font-semibold">Админка</h1>
    </header>

    <div class="rounded-lg border bg-white">
      <nav class="border-b px-4 py-2 text-sm flex gap-4">
        <button :class="tabClass('telegram')" @click="tab='telegram'">Телеграм-бот</button>
        <button :class="tabClass('users')" @click="tab='users'">Пользователи</button>
        <button :class="tabClass('about')" @click="tab='about'">О системе</button>
        <div class="ml-auto flex items-center gap-4">
          <RouterLink to="/admin/receipts" class="text-gray-700 hover:text-black">Чеки</RouterLink>
          <RouterLink to="/admin/operations" class="text-gray-700 hover:text-black">Операции</RouterLink>
        </div>
      </nav>
      <div class="p-4">
        <TelegramSettings v-if="tab==='telegram'" />
        <AdminUsers v-else-if="tab==='users'" />
        <div v-else class="text-sm text-slate-700 space-y-3">
  <p>SPA: Vue 3 + Vite. Backend: Rust + Axum + MongoDB.</p>
  <div class="rounded-md border p-3 bg-slate-50">
    <div class="font-medium mb-2">??????? ????????</div>
    <button @click="clearAll" class="rounded-md bg-red-600 text-white px-3 py-1.5">???????? ???????? ??????</button>
    <div v-if="actionMsg" class="text-xs text-slate-600 mt-2">{{ actionMsg }}</div>
  </div>
</div>
      </div>
    </div>
  </section>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import TelegramSettings from './admin/TelegramSettings.vue';
import AdminUsers from './admin/Users.vue';
import { API, authHeaders } from '../api';

const tab = ref<'telegram'|'users'|'about'>('telegram');
const actionMsg = ref('');
async function clearAll() {
  if (!confirm('???????? ???????? ??????? ????????, ????????, ?????????, ????, ??????? ????? ???????.')) return;
  try {
    const res = await fetch(${API}/dev/clear, { method: 'POST', headers: authHeaders() });
    actionMsg.value = res.ok ? '?????? ???????' : ?????? ???????: ;
  } catch (e) {
    actionMsg.value = (e as any)?.message || '??????? ??????';
  }
}/dev/clear, { method: 'POST', headers: authHeaders() });
    actionMsg.value = res.ok ? '?????? ???????' : ?????? ???????: ;
  } catch (e) {
    actionMsg.value = (e as any)?.message || '??????? ??????';
  }
}

function tabClass(name: string){
  return [
    'px-3 py-1.5 rounded-md',
    tab.value===name ? 'bg-blue-600 text-white' : 'text-gray-700 hover:text-black'
  ];
}
</script>



