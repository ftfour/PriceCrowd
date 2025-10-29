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

    <div class="mt-8 space-y-3">
      <h3 class="text-base font-semibold">Статус бота</h3>
      <div class="rounded-lg border bg-white p-4 text-sm text-slate-700">
        <div>Включен (polling): <b>{{ status.enabled ? 'Да' : 'Нет' }}</b></div>
        <div>Вебхук: <b>{{ status.webhook_enabled ? 'Да' : 'Нет' }}</b></div>
        <div>Пуллинг активен: <b>{{ status.polling ? 'Да' : 'Нет' }}</b></div>
        <div>Последний опрос: <b>{{ formatTs(status.last_poll_ms) }}</b></div>
      </div>

      <h3 class="text-base font-semibold">Последние логи</h3>
      <div class="rounded-lg border bg-white p-0 max-h-64 overflow-auto">
        <table class="w-full text-left text-xs">
          <thead class="bg-slate-50 text-slate-500">
            <tr><th class="px-3 py-2 w-40">Время</th><th class="px-3 py-2 w-16">Ур.</th><th class="px-3 py-2">Сообщение</th></tr>
          </thead>
          <tbody>
            <tr v-for="(l,i) in logs" :key="i" class="border-t">
              <td class="px-3 py-2">{{ formatTs(l.ts_ms) }}</td>
              <td class="px-3 py-2 uppercase text-slate-500">{{ l.level }}</td>
              <td class="px-3 py-2">{{ l.message }}</td>
            </tr>
            <tr v-if="logs.length===0"><td colspan="3" class="px-3 py-3 text-slate-500">Нет записей</td></tr>
          </tbody>
        </table>
      </div>
    </div>
  </div>
  </template>

<script setup lang="ts">
import { onMounted, reactive, ref } from 'vue';
import { API, authHeaders } from '../../api';

const form = reactive<{ token?: string; webhook_url?: string; enabled: boolean; webhook_enabled: boolean }>({ enabled: false, webhook_enabled: false });
const saving = ref(false);
const msg = ref('');
const ok = ref(false);
const status = reactive<{ enabled: boolean; webhook_enabled: boolean; polling: boolean; last_poll_ms?: number | null }>({ enabled: false, webhook_enabled: false, polling: false, last_poll_ms: null });
const logs = ref<Array<{ ts_ms: number; level: string; message: string }>>([]);

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

async function loadStatus(){
  const res = await fetch(`${API}/settings/telegram/status`, { headers: authHeaders() });
  if (res.ok){
    const data = await res.json();
    const st = data.status || {};
    status.enabled = !!st.enabled;
    status.webhook_enabled = !!st.webhook_enabled;
    status.polling = !!st.polling;
    status.last_poll_ms = st.last_poll_ms ?? null;
    logs.value = Array.isArray(data.logs) ? data.logs : [];
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
onMounted(() => { loadStatus(); setInterval(loadStatus, 5000); });

function formatTs(ms?: number | null){
  if (!ms) return '-';
  const d = new Date(ms);
  return d.toLocaleString();
}
</script>
