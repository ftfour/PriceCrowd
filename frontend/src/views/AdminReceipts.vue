<template>
  <section class="space-y-4">
    <header class="flex items-center justify-between">
      <h1 class="text-2xl font-semibold">📥 Отправленные данные</h1>
      <div class="flex items-center gap-2">
        <label class="text-sm text-slate-600">Источник:</label>
        <select v-model="source" class="rounded-md border px-2 py-1 text-sm">
          <option value="">Все</option>
          <option value="telegram_webapp">webapp</option>
          <option value="telegram">telegram</option>
        </select>
      </div>
    </header>

    <div class="rounded-lg border overflow-hidden bg-white">
      <table class="min-w-full text-sm">
        <thead class="bg-slate-50 text-left">
          <tr>
            <th class="px-3 py-2 border-b w-48">Дата</th>
            <th class="px-3 py-2 border-b">QR-строка</th>
            <th class="px-3 py-2 border-b w-40">Источник</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="(r, i) in filtered" :key="i" class="hover:bg-slate-50">
            <td class="px-3 py-2 border-b whitespace-nowrap">{{ formatDate(r.timestamp) }}</td>
            <td class="px-3 py-2 border-b break-all">{{ r.qr }}</td>
            <td class="px-3 py-2 border-b">{{ r.source }}</td>
          </tr>
          <tr v-if="filtered.length===0">
            <td colspan="3" class="px-3 py-6 text-center text-slate-500">Нет данных</td>
          </tr>
        </tbody>
      </table>
    </div>
    <p class="text-xs text-slate-500">Автообновление каждые 30 секунд.</p>
  </section>
</template>

<script setup lang="ts">
import { onMounted, onBeforeUnmount, ref, computed } from 'vue';
import axios from 'axios';
import { API } from '../api';

type Receipt = { qr: string; timestamp: string; source: string; user: string };
const receipts = ref<Receipt[]>([]);
const timer = ref<number | null>(null);
const source = ref('');

async function fetchReceipts() {
  try {
    const res = await axios.get(`${API}/receipts/list`);
    const data = Array.isArray(res.data) ? res.data : [];
    receipts.value = data;
  } catch {}
}

const filtered = computed(() => {
  const s = source.value.trim();
  if (!s) return receipts.value;
  return receipts.value.filter(r => r.source === s);
});

function formatDate(iso: string) {
  try { return new Date(iso).toLocaleString(); } catch { return iso; }
}

onMounted(async () => {
  await fetchReceipts();
  timer.value = window.setInterval(fetchReceipts, 30000);
});
onBeforeUnmount(() => { if (timer.value) window.clearInterval(timer.value); });
</script>

