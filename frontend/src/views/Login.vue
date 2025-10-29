<template>
  <section class="max-w-sm mx-auto bg-white border rounded-lg p-6">
    <h1 class="text-xl font-semibold mb-4">Вход</h1>
    <form @submit.prevent="onSubmit" class="space-y-3">
      <div>
        <label class="block text-sm mb-1">Имя пользователя</label>
        <input v-model="username" required class="w-full rounded-md border px-3 py-2 text-sm" />
      </div>
      <div>
        <label class="block text-sm mb-1">Пароль</label>
        <input v-model="password" type="password" required class="w-full rounded-md border px-3 py-2 text-sm" />
      </div>
      <button :disabled="loading" class="w-full rounded-md bg-blue-600 text-white px-3 py-2 text-sm">Войти</button>
      <p v-if="error" class="text-sm text-rose-600">{{ error }}</p>
    </form>
  </section>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { useRouter } from 'vue-router';
import { API } from '../api';
import { loginAuth } from '../auth';

const router = useRouter();
const username = ref('');
const password = ref('');
const loading = ref(false);
const error = ref('');

async function onSubmit(){
  loading.value = true; error.value = '';
  try{
    const res = await fetch(`${API}/auth/login`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ username: username.value, password: password.value })
    });
    if(!res.ok){ error.value = 'Неверные имя пользователя или пароль'; return; }
    const data = await res.json();
    loginAuth(data.token, data.username, data.role);
    router.push('/');
  } finally { loading.value = false; }
}
</script>

