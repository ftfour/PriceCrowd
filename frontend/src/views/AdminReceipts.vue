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
            <th class="px-3 py-2 border-b w-48">Действия</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="(r, i) in filtered" :key="i" class="hover:bg-slate-50">
            <td class="px-3 py-2 border-b whitespace-nowrap">{{ formatDate(r.timestamp) }}</td>
            <td class="px-3 py-2 border-b break-all">
              {{ r.qr }}
              <span v-if="used.has(r.qr)" class="ml-2 text-xs text-slate-500">� ??? ? ?????????</span> <span v-if="blockedUsers.has(r.user)" class="ml-2 text-xs text-slate-500">� ? ???????????? ???? ????????</span>
            </td>
            <td class="px-3 py-2 border-b">{{ r.source }}</td>
            <td class="px-3 py-2 border-b">
              <button
                class="inline-flex items-center gap-2 px-3 py-1.5 rounded-md text-white bg-blue-600 hover:bg-blue-700 disabled:opacity-60"
                :disabled="loading[i]"
                @click="createOperation(r.qr, i)"
              >
                <span v-if="loading[i]" class="inline-block h-4 w-4 border-2 border-white/50 border-t-white rounded-full animate-spin"></span>
                <span>{{ loading[i] ? 'Создание...' : 'Создать операцию' }}</span>
              </button>
            </td>
          </tr>
          <tr v-if="filtered.length===0">
            <td colspan="4" class="px-3 py-6 text-center text-slate-500">Нет данных</td>
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
import { API, authHeaders } from '../api';
import { getCheckByQR } from '../services/fnsApi';
import { useOperationsStore } from '../stores/operations';
import { useRouter } from 'vue-router';

type Receipt = { qr: string; timestamp: string; source: string; user: string };
const receipts = ref<Receipt[]>([]);
const timer = ref<number | null>(null);
const source = ref('');
const loading = ref<Record<number, boolean>>({});
const router = useRouter();
const store = useOperationsStore();
const used = computed(() => new Set((store.operations || []).map(o => (o as any).qr).filter(Boolean) as string[]));

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

async function createOperation(qr: string, idx: number) {
  try {
    if (used.value.has(qr)) return; const user = receipts.value[idx]?.user || ""; if (user && blockedUsers.value.has(user)) { alert("? ????? ???????????? ??? ???? ????????"); return; }
    loading.value[idx] = true; const { normalized: data, raw } = await getCheckByQR(qr);\n    try {\n      const res = await fetch(${API}/operations, { method: 'POST', headers: authHeaders({ 'Content-Type': 'application/json' }), body: JSON.stringify({ date: data.dateTime, seller: data.seller?.name || '', amount: (data.totalSum ?? 0) / 100, items: (data.items ?? []).map((i: any) => ({ name: i.name, price: (i.price ?? 0) / 100, quantity: i.quantity ?? 1 })), qr, uploaded_by: user || null }) });\n      if (!res.ok) { if (res.status === 409) { const j = await res.json().catch(() => ({})); if (j?.error === 'user_has_operation') { alert('? ???????????? ??? ???? ????????'); return; } if (j?.error === 'qr_used') { alert('???? ??? ??? ???????????'); return; } } throw new Error(?????? ???????: ); }\n    } catch (e) { alert((e as any)?.message || '?? ??????? ??????? ???????? ?? ???????'); return; }
    const op = {
      id: (globalThis.crypto?.randomUUID?.() || Math.random().toString(36).slice(2)),
      date: data.dateTime,
      seller: data.seller?.name || '—',
      amount: (data.totalSum ?? 0) / 100,
      items: (data.items ?? []).map((i: any) => ({
        name: i.name,
        price: (i.price ?? 0) / 100,
        quantity: i.quantity ?? 1,
      })),
      qr: qr,
      status: 'draft' as const,
      raw,
      uploaded_by: receipts.value[idx]?.user || '',
    };
    store.add(op);
    router.push('/admin/operations');
  } catch (e: any) {
    alert(e?.message || 'Не удалось создать операцию');
  } finally {
    loading.value[idx] = false;
  }
}
</script>







