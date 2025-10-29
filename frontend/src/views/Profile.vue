<template>
  <section class="space-y-6">
    <h1 class="text-2xl font-semibold">Профиль</h1>

    <div class="rounded-lg border bg-white p-4">
      <div class="text-sm">Логин: <b>{{ username }}</b></div>
      <div class="text-sm">Роль: <b>{{ role }}</b></div>
    </div>

    <div class="rounded-lg border bg-white p-4 space-y-3">
      <h2 class="text-lg font-semibold">Привязка Telegram</h2>
      <div v-if="status.linked" class="text-sm text-emerald-700">Телеграм привязан (chat id: {{ status.telegram_id }})</div>
      <div v-else class="text-sm text-slate-700">Телеграм не привязан</div>

      <div v-if="!status.linked">
        <button @click="startLink" :disabled="busy" class="rounded bg-blue-600 text-white px-3 py-1.5 text-sm">Привязать</button>
        <div v-if="code" class="mt-3 text-sm">
          Отправьте боту в Telegram: <code class="px-2 py-1 bg-slate-100 border rounded">/link {{ code }}</code>
        </div>
      </div>
      <div v-else>
        <button @click="unlink" class="rounded border px-3 py-1.5 text-sm">Отвязать</button>
      </div>

      <p v-if="msg" class="text-sm" :class="ok ? 'text-emerald-600' : 'text-rose-600'">{{ msg }}</p>
    </div>
  </section>
</template>

<script setup lang="ts">
import { onMounted, ref } from 'vue';
import { API, authHeaders } from '../api';

const username = ref(localStorage.getItem('username') || '');
const role = ref(localStorage.getItem('role') || '');
const status = ref<{ linked: boolean; telegram_id?: number | null }>({ linked: false, telegram_id: null });
const code = ref('');
const msg = ref('');
const ok = ref(false);
const busy = ref(false);

async function loadStatus(){
  const res = await fetch(`${API}/users/link_telegram/status`, { headers: authHeaders() });
  if (res.ok){ status.value = await res.json(); } else { status.value = { linked:false, telegram_id:null }; }
}

async function startLink(){
  busy.value = true; msg.value = ''; code.value='';
  try{
    const res = await fetch(`${API}/users/link_telegram/start`, { method: 'POST', headers: authHeaders() });
    if (res.ok){ const data = await res.json(); code.value = data.code; ok.value = true; msg.value = 'Код сгенерирован'; pollUntilLinked(); }
    else { ok.value = false; msg.value = `Ошибка: ${res.status}`; }
  } finally { busy.value = false; }
}

async function unlink(){
  const res = await fetch(`${API}/users/link_telegram/unlink`, { method: 'POST', headers: authHeaders() });
  if (res.ok || res.status===204){ ok.value = true; msg.value = 'Отвязано'; code.value=''; await loadStatus(); }
  else { ok.value=false; msg.value = `Ошибка: ${res.status}`; }
}

let interval: any;
async function pollUntilLinked(){
  clearInterval(interval);
  interval = setInterval(async()=>{
    await loadStatus();
    if (status.value.linked){ clearInterval(interval); msg.value='Готово!'; ok.value = true; }
  }, 3000);
}

onMounted(loadStatus);
</script>

