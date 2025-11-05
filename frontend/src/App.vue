<template>
  <div>
    <header class="border-b bg-white">
      <div class="mx-auto max-w-5xl px-4 py-3 flex items-center justify-between gap-4">
        <h1 class="text-lg font-semibold text-gray-900">PriceCrowd</h1>
        <nav class="flex items-center gap-4 text-sm">
          <RouterLink to="/" class="text-gray-700 hover:text-black">Главная</RouterLink>
          <RouterLink to="/products" class="text-gray-700 hover:text-black">Товары</RouterLink>
          <RouterLink to="/stores" class="text-gray-700 hover:text-black">Магазины</RouterLink>
          <RouterLink to="/rating" class="text-gray-700 hover:text-black">Рейтинг</RouterLink>
          <RouterLink to="/cart" class="text-gray-700 hover:text-black">Корзина</RouterLink>
          <RouterLink v-if="auth.isAdmin" to="/admin" class="text-gray-700 hover:text-black">Админка</RouterLink>
          <RouterLink to="/scan" class="text-gray-700 hover:text-black">Сканер</RouterLink>
        </nav>
        <div class="flex items-center gap-3 text-sm">
          <template v-if="isAuthed">
            <div class="flex items-center gap-2">
              <div class="h-8 w-8 rounded-full bg-slate-200 flex items-center justify-center text-xs font-semibold text-slate-700">
                {{ auth.state.username.slice(0,1).toUpperCase() }}
              </div>
              <div class="leading-tight">
                <div class="font-medium text-gray-900">{{ auth.state.username }}</div>
                <div class="text-[11px] text-slate-500">{{ auth.state.role === 'admin' ? 'Администратор' : 'Пользователь' }}</div>
              </div>
            </div>
            <RouterLink to="/profile" class="rounded-md border px-3 py-1.5">Профиль</RouterLink>
            <button @click="onLogout" class="rounded-md border px-3 py-1.5">Выйти</button>
          </template>
          <template v-else>
            <RouterLink to="/login" class="rounded-md bg-blue-600 text-white px-3 py-1.5">Войти</RouterLink>
          </template>
        </div>
      </div>
    </header>

    <main class="mx-auto max-w-5xl px-4 py-8">
      <RouterView />
    </main>

    <footer class="mx-auto max-w-5xl px-4 py-8 text-xs text-gray-600">
      <div class="space-y-2">
        <div class="text-sm font-medium text-gray-800">? ???????</div>
        <p>
          PriceCrowd � ?????? ????????????? ???: ?????????? QR-????, ???????? ??????? ? ????????,
          ????????? ????????? ???? ? ???????????? ???????? ? ??????. ?????? ????? ??????????, ?????? ? API ?????????.
        </p>
        <p>
          ????????: Vue 3 + Vite + Tailwind. ??????: Rust (Axum) + MongoDB. ?????????????? ?????????? ? Telegram-?????.
        </p>
        <div class="pt-2">
          <RouterLink to="/privacy" class="hover:underline">???????? ??????????????????</RouterLink>
        </div>
      </div>
    </footer>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useRouter } from 'vue-router';
import { useAuth, logoutAuth, ensureAuthFresh } from './auth';

const router = useRouter();
const auth = useAuth();

// Авторизован — только если есть токен, логин и роль
const isAuthed = computed(() => !!auth.state.token && !!auth.state.username && !!auth.state.role);

ensureAuthFresh();

function onLogout() {
  logoutAuth();
  router.push('/');
}
</script>
