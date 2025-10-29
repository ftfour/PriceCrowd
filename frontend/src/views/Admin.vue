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
      </nav>
      <div class="p-4">
        <TelegramSettings v-if="tab==='telegram'" />
        <AdminUsers v-else-if="tab==='users'" />
        <div v-else class="text-sm text-slate-700">
          <p>Версия SPA: Vue 3 + Vite. Бэкенд: Rust + Axum + MongoDB.</p>
        </div>
      </div>
    </div>
  </section>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import TelegramSettings from './admin/TelegramSettings.vue';
import AdminUsers from './admin/Users.vue';

const tab = ref<'telegram'|'users'|'about'>('telegram');

function tabClass(name: string){
  return [
    'px-3 py-1.5 rounded-md',
    tab.value===name ? 'bg-blue-600 text-white' : 'text-gray-700 hover:text-black'
  ];
}
</script>
