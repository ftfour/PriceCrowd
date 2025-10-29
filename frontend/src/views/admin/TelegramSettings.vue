<template>
  <div class="max-w-xl">
    <h2 class="text-lg font-semibold mb-4">Настройки Telegram-бота</h2>

    <form @submit.prevent="save" class="space-y-4">
      <div>
        <label class="block text-sm mb-1">Токен бота</label>
        <input v-model="form.token" class="w-full rounded-md border px-3 py-2 text-sm" placeholder="123456:ABC-DEF..." />
      </div>
      <div>
        <label class="block text-sm mb-1">Webhook URL (необязательно)</label>
        <input v-model="form.webhook_url" class="w-full rounded-md border px-3 py-2 text-sm" placeholder="https://.../telegram/webhook" />
      </div>
      <div class="flex flex-col gap-2">
        <label class="inline-flex items-center gap-2 text-sm"><input type="checkbox" v-model="form.enabled" /> Бот включен (polling)</label>
        <label class="inline-flex items-center gap-2 text-sm"><input type="checkbox" v-model="form.webhook_enabled" /> Использовать вебхук</label>
      </div>

      <div class="flex items-center gap-3 pt-2">
        <button :disabled="saving" class="rounded-md bg-blue-600 text-white px-4 py-2 text-sm">Сохранить</button>
        <span v-if="msg" class="text-sm" :class="ok ? 'text-emerald-600' : 'text-rose-600'">{{ msg }}</span>
      </div>
    </form>
  </div>
  </template>

<script setup lang="ts">
import { onMounted, reactive, ref } from 'vue';
import { API, authHeaders } from '../../api';

const form = reactive<{ token?: string; webhook_url?: string; enabled: boolean; webhook_enabled: boolean }>({ enabled: false, webhook_enabled: false });
const saving = ref(false);
const msg = ref('');
const ok = ref(false);

async function load(){
  msg.value = '';
  const res = await fetch(`${API}/settings/telegram`, { headers: authHeaders() });
  if (res.ok){
    const data = await res.json();
    form.token = data.token || '';
    form.webhook_url = data.webhook_url || '';
    form.enabled = !!data.enabled;
    form.webhook_enabled = !!data.webhook_enabled;
  } else {
    const t = await res.text().catch(()=> '');
    msg.value = `Ошибка загрузки: ${res.status} ${t}`;
    ok.value = false;
  }
}

async function save(){
  saving.value = true; msg.value = '';
  try{
    const payload: any = { enabled: !!form.enabled, webhook_enabled: !!form.webhook_enabled };
    if (form.token !== undefined) payload.token = form.token;
    if (form.webhook_url !== undefined) payload.webhook_url = form.webhook_url;
    const res = await fetch(`${API}/settings/telegram`, { method: 'PUT', headers: authHeaders({ 'Content-Type': 'application/json' }), body: JSON.stringify(payload) });
    ok.value = res.ok;
    if (res.ok) {
      msg.value = 'Сохранено';
    } else {
      const t = await res.text().catch(()=> '');
      msg.value = `Ошибка сохранения: ${res.status} ${t}`;
    }
    if (res.ok) await load();
  } finally { saving.value = false; }
}

onMounted(load);
</script>
