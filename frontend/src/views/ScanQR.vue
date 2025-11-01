<template>
  <section class="p-4 space-y-4">
    <h1 class="text-xl font-semibold">Сканирование QR чека</h1>

    <div class="rounded-lg overflow-hidden border bg-black aspect-video max-w-md">
      <video ref="videoEl" autoplay playsinline class="w-full h-full object-cover"></video>
    </div>

    <div v-if="qrText" class="p-3 rounded-md bg-green-50 border border-green-200 text-sm">
      <div class="font-medium">Распознано:</div>
      <div class="break-all mt-1">{{ qrText }}</div>
    </div>

    <div v-if="statusMsg" :class="statusOk ? 'text-green-700' : 'text-red-700'" class="text-sm">
      {{ statusMsg }}
    </div>

    <div class="flex gap-2">
      <button @click="closeApp" class="rounded-md bg-slate-700 text-white px-3 py-2 text-sm">Закрыть</button>
      <button v-if="!scanning" @click="startScanner" class="rounded-md border px-3 py-2 text-sm">Сканировать снова</button>
    </div>

    <canvas ref="canvasEl" class="hidden"></canvas>
  </section>
</template>

<script setup lang="ts">
import { onMounted, onBeforeUnmount, ref } from 'vue';
import jsQR from 'jsqr';
import { API } from '../api';

const videoEl = ref<HTMLVideoElement | null>(null);
const canvasEl = ref<HTMLCanvasElement | null>(null);
const streamRef = ref<MediaStream | null>(null);
const rafId = ref<number | null>(null);
const qrText = ref('');
const statusMsg = ref('');
const statusOk = ref(false);
const scanning = ref(false);

function getTelegramUserId(): string | null {
  try {
    const w = window as any;
    const uid = w?.Telegram?.WebApp?.initDataUnsafe?.user?.id;
    if (uid) return String(uid);
  } catch {}
  return localStorage.getItem('username');
}

async function sendToBackend(qr: string) {
  try {
    const user = getTelegramUserId() ?? 'anonymous';
    const res = await fetch(`${API}/receipts/upload`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ qr, user, source: 'telegram_webapp' }),
    });
    const data = await res.json().catch(() => ({}));
    if (res.ok && data?.status === 'ok') {
      statusOk.value = true;
      statusMsg.value = '✅ Чек отправлен';
    } else {
      statusOk.value = false;
      statusMsg.value = 'Ошибка отправки на сервер';
    }
  } catch (e) {
    statusOk.value = false;
    statusMsg.value = 'Сеть недоступна';
  }
}

function sendToTelegram(qr: string) {
  try {
    const w = window as any;
    if (w?.Telegram?.WebApp?.sendData) {
      w.Telegram.WebApp.sendData(qr);
    }
  } catch {}
}

function closeApp() {
  try {
    const w = window as any;
    if (w?.Telegram?.WebApp?.close) w.Telegram.WebApp.close();
  } catch {}
}

function stopStream() {
  try {
    if (rafId.value) cancelAnimationFrame(rafId.value);
    rafId.value = null;
    if (streamRef.value) {
      streamRef.value.getTracks().forEach(t => t.stop());
      streamRef.value = null;
    }
  } catch {}
}

async function startScanner() {
  statusMsg.value = '';
  statusOk.value = false;
  qrText.value = '';
  scanning.value = true;
  try {
    const stream = await navigator.mediaDevices.getUserMedia({ video: { facingMode: { ideal: 'environment' } }, audio: false });
    streamRef.value = stream;
    const video = videoEl.value!;
    video.srcObject = stream;
    await video.play();
    const canvas = canvasEl.value!;
    const ctx = canvas.getContext('2d')!;
    const scan = () => {
      if (!video.videoWidth || !video.videoHeight) {
        rafId.value = requestAnimationFrame(scan);
        return;
      }
      canvas.width = video.videoWidth;
      canvas.height = video.videoHeight;
      ctx.drawImage(video, 0, 0, canvas.width, canvas.height);
      const img = ctx.getImageData(0, 0, canvas.width, canvas.height);
      const result = jsQR(img.data as any, img.width, img.height);
      if (result && result.data) {
        qrText.value = result.data;
        stopStream();
        scanning.value = false;
        sendToBackend(result.data);
        sendToTelegram(result.data);
      } else {
        rafId.value = requestAnimationFrame(scan);
      }
    };
    rafId.value = requestAnimationFrame(scan);
  } catch (e) {
    scanning.value = false;
    statusOk.value = false;
    statusMsg.value = 'Нет доступа к камере';
  }
}

onMounted(() => {
  // Try to expand WebApp viewport if available
  try { (window as any)?.Telegram?.WebApp?.expand?.(); } catch {}
  startScanner();
});

onBeforeUnmount(() => {
  stopStream();
});
</script>

<style scoped>
.aspect-video { aspect-ratio: 16 / 9; }
</style>

