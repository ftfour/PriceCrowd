<template>
  <section class="space-y-4">
    <header class="flex items-center justify-between">
      <h1 class="text-2xl font-semibold">Операции</h1>
      <div class="text-sm text-slate-500">Всего: {{ ops.length }}</div>
    </header>

    <div class="flex items-center gap-2">
      <button class="px-3 py-1.5 rounded-md border" @click="openModal('/stores/new', 'stores')">Новый магазин</button>
      <button class="px-3 py-1.5 rounded-md border" @click="openModal('/products/new', 'products')">Новый товар</button>
    </div>

    <div v-if="loading" class="text-center text-slate-500 py-10">Загрузка...</div>

    <div v-else class="rounded-lg border overflow-hidden bg-white">
      <table class="min-w-full text-sm">
        <thead class="bg-slate-50 text-left">
          <tr>
            <th class="px-3 py-2 border-b w-48">Время</th>
            <th class="px-3 py-2 border-b">Продавец</th>
            <th class="px-3 py-2 border-b w-32">Сумма</th>
            <th class="px-3 py-2 border-b w-32">Статус</th>
            <th class="px-3 py-2 border-b w-64">Магазин</th>
            <th class="px-3 py-2 border-b w-56">Действия</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="o in ops" :key="o._id || o.id" class="hover:bg-slate-50 align-top">
            <td class="px-3 py-2 border-b whitespace-nowrap">{{ formatDate(o.date) }}</td>
            <td class="px-3 py-2 border-b">{{ o.seller }}</td>
            <td class="px-3 py-2 border-b">{{ money(o.amount) }}</td>
            <td class="px-3 py-2 border-b">
              <span
                :class="o.status === 'draft'
                  ? 'text-amber-700'
                  : o.status === 'posted'
                  ? 'text-green-700'
                  : 'text-slate-500'"
              >
                {{ o.status }}
              </span>
              <div class="mt-2 flex gap-2">
                <button class="px-2 py-1 rounded border text-xs" @click="openJson(o)">JSON</button>
                <button class="px-2 py-1 rounded border text-xs" @click="openHtml(o)">HTML</button>
                <button class="px-2 py-1 rounded border text-xs" @click="toggleItems(o)">
                  {{ expanded[o._id || o.id] ? 'Скрыть позиции' : 'Показать позиции' }}
                </button>
              </div>
            </td>
            <td class="px-3 py-2 border-b">
              <select
                v-model="storeSelect[o._id || o.id]"
                class="border rounded px-2 py-1 text-sm bg-white text-slate-900"
              >
                <option value="">—</option>
                <option v-for="s in stores" :key="s.id" :value="s.id">{{ s.name }}</option>
              </select>
              <button
                class="ml-2 px-2 py-1 rounded-md border text-xs"
                @click="saveStore(o)"
              >
                💾 Сохранить
              </button>
              <button
                class="ml-2 px-2 py-1 rounded-md border text-xs"
                @click="openModalStore(o.seller)"
              >
                ➕ Новый (по продавцу)
              </button>
            </td>
            <td class="px-3 py-2 border-b">
              <button
                class="mr-2 px-3 py-1.5 rounded-md text-white bg-green-600 hover:bg-green-700"
                @click="postOperation(o)"
              >
                Опубликовать
              </button>
              <button
                class="px-3 py-1.5 rounded-md text-white bg-red-600 hover:bg-red-700"
                @click="deleteOperation(o)"
              >
                Удалить
              </button>
            </td>
          </tr>

          <!-- Таблица позиций -->
          <tr v-for="o in ops" :key="(o._id || o.id) + '-items'" v-show="expanded[o._id || o.id]">
            <td colspan="6" class="px-3 py-2 border-b bg-slate-50">
              <div class="text-sm text-slate-700 mb-2">Позиции ({{ (edited[o._id || o.id]?.items || []).length }})</div>
              <div class="space-y-2">
                <div
                  v-for="(it, idx) in edited[o._id || o.id]?.items || []"
                  :key="idx"
                  class="grid grid-cols-12 gap-2 items-center border-l-2 border-blue-200 pl-3"
                >
                  <div class="col-span-5 truncate">{{ it.name }}</div>
                  <div class="col-span-2 text-right">{{ money(it.price) }}</div>
                  <div class="col-span-1 text-right">{{ it.quantity }}</div>
                  <div class="col-span-4">
                    <input
                      type="text"
                      v-model="productQuery[o._id || o.id][idx]"
                      placeholder="Поиск товара..."
                      class="w-full border rounded px-2 py-1 text-sm"
                    />
                    <select
                      v-model="edited[o._id || o.id].items[idx].product_id"
                      class="mt-1 w-full border rounded px-2 py-1 text-sm bg-white text-slate-900"
                    >
                      <option :value="null">—</option>
                      <option
                        v-for="p in filteredProducts(o, idx)"
                        :key="p.id"
                        :value="p.id"
                      >
                        {{ p.title }}
                      </option>
                    </select>
                    <button
                      class="mt-1 px-2 py-1 rounded-md border text-xs"
                      @click="openModalProduct(it.name)"
                    >
                      ➕ Новый товар (по названию)
                    </button>
                  </div>
                </div>
              </div>

              <div class="mt-3 flex items-center gap-2">
                <button
                  class="px-3 py-1.5 rounded-md text-white bg-blue-600 hover:bg-blue-700"
                  @click="saveItems(o)"
                >
                  💾 Сохранить позиции
                </button>
                <span class="text-xs text-slate-500">
                  Назначьте товары для строк операции и сохраните
                </span>
              </div>
            </td>
          </tr>

          <tr v-if="ops.length === 0">
            <td colspan="6" class="px-3 py-6 text-center text-slate-500">
              Пока нет операций
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <!-- Popup -->
    <div
      v-if="showModal"
      class="fixed inset-0 bg-black/50 z-50 flex items-center justify-center"
    >
      <div
        class="bg-white rounded-md shadow-xl w-[90vw] h-[90vh] overflow-hidden flex flex-col"
      >
        <div class="p-2 border-b flex items-center justify-between text-sm">
          <div>Быстрое добавление</div>
          <button class="rounded-md border px-2 py-1" @click="closeModal">
            Закрыть
          </button>
        </div>

        <div class="p-4 overflow-auto flex-1">
          <div v-if="modalKind === 'stores'" class="max-w-md space-y-3">
            <label class="block text-sm">
              <span class="block mb-1">Название магазина</span>
              <input
                v-model="quickStoreName"
                class="w-full border rounded px-3 py-2 text-sm"
              />
            </label>
            <label class="block text-sm">
              <span class="block mb-1">Адрес (необязательно)</span>
              <input
                v-model="quickStoreAddr"
                class="w-full border rounded px-3 py-2 text-sm"
              />
            </label>
            <div class="pt-2">
              <button
                class="px-3 py-1.5 rounded-md text-white bg-blue-600 hover:bg-blue-700"
                @click="submitQuickStore"
              >
                Создать
              </button>
            </div>
          </div>

          <div v-else-if="modalKind === 'products'" class="max-w-md space-y-3">
            <label class="block text-sm">
              <span class="block mb-1">Название товара</span>
              <input
                v-model="quickProductTitle"
                class="w-full border rounded px-3 py-2 text-sm"
              />
            </label>
            <div class="pt-2">
              <button
                class="px-3 py-1.5 rounded-md text-white bg-blue-600 hover:bg-blue-700"
                @click="submitQuickProduct"
              >
                Создать
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Modal for JSON/HTML -->
    <div v-if="rawModalVisible" class="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
      <div class="bg-white rounded-md shadow-xl w-[90vw] h-[90vh] overflow-hidden flex flex-col">
        <div class="p-2 border-b flex items-center justify-between text-sm">
          <div class="font-medium">{{ rawModalTitle }}</div>
          <button class="rounded-md border px-2 py-1" @click="closeRawModal">Закрыть</button>
        </div>
        <div class="flex-1 overflow-auto p-4 bg-slate-50">
          <pre v-if="rawModalKind==='json'" class="text-xs whitespace-pre-wrap bg-white border rounded p-3">{{ rawModalJson }}</pre>
          <div v-else-if="rawModalKind==='html'" class="bg-white border rounded p-3 text-xs overflow-auto" v-html="rawModalHtml"></div>
        </div>
      </div>
    </div>
  </section>
</template>

<script setup lang="ts">
import { ref, onMounted } from "vue";
import { API, authHeaders } from "../api";

type Op = {
  _id?: string;
  id?: string;
  date: string;
  seller: string;
  amount: number;
  status: "draft" | "posted" | "deleted";
  store_id?: string | null;
  raw?: any;
};

type StoreRef = { id: string; name: string };
type ProductRef = { id: string; title: string };

const ops = ref<Op[]>([]);
const stores = ref<StoreRef[]>([]);
const products = ref<ProductRef[]>([]);
const storeSelect = ref<Record<string, string>>({});
const edited = ref<
  Record<string, { items: { name: string; price: number; quantity: number; product_id: string | null }[] }>
>({});
const productQuery = ref<Record<string, Record<number, string>>>({});
const expanded = ref<Record<string, boolean>>({});

const showModal = ref(false);
const modalUrl = ref("");
const modalKind = ref<"stores" | "products" | "">("");
const quickStoreName = ref("");
const quickStoreAddr = ref("");
const quickProductTitle = ref("");
const loading = ref(false);

function getId(o: any) {
  return (typeof o._id === "string" ? o._id : o._id?.$oid) || o.id;
}
function formatDate(iso: string) {
  try {
    return new Date(iso).toLocaleString();
  } catch {
    return iso;
  }
}
function money(rub: number) {
  return (rub || 0).toFixed(2);
}

async function ensureRaw(o: any) {
  try {
    const id = getId(o);
    if (o && o.raw) return;
    const res = await fetch(`${API}/operations/${id}`, { headers: authHeaders() });
    if (!res.ok) return;
    const full = await res.json();
    const idx = ops.value.findIndex((x: any) => getId(x) === id);
    if (idx >= 0) {
      ops.value[idx] = { ...ops.value[idx], raw: full?.raw } as any;
    }
  } catch {}
}

async function loadOps() {
  loading.value = true;
  try {
    const res = await fetch(`${API}/operations`, { headers: authHeaders() });
    const data = await res.json();
    ops.value = (Array.isArray(data) ? data : []).map((o: any) => ({
      _id: getId(o),
      date: o.date,
      seller: o.seller,
      amount: o.amount,
      status: o.status,
      store_id:
        (typeof o.store_id === "string" ? o.store_id : o.store_id?.$oid) || null,
      raw: o.raw,
    }));

    for (const oraw of data) {
      const id = getId(oraw);
      if (oraw.store_id)
        storeSelect.value[id] =
          (typeof oraw.store_id === "string"
            ? oraw.store_id
            : oraw.store_id?.$oid) || "";
      const items = (oraw.items || []).map((i: any) => ({
        name: i.name,
        price: i.price,
        quantity: i.quantity,
        product_id:
          (typeof i.product_id === "string"
            ? i.product_id
            : i.product_id?.$oid) || null,
      }));
      edited.value[id] = { items };
      productQuery.value[id] = {} as any;
    }
  } catch {
  } finally {
    loading.value = false;
  }
}

async function loadStores() {
  try {
    const res = await fetch(`${API}/stores`);
    const data = await res.json();
    stores.value = (Array.isArray(data) ? data : []).map((s: any) => ({
      id: (typeof s._id === "string" ? s._id : s._id?.$oid) || "",
      name: s.name,
    }));
  } catch {}
}

async function loadProducts() {
  try {
    const res = await fetch(`${API}/products`);
    const data = await res.json();
    products.value = (Array.isArray(data) ? data : []).map((p: any) => ({
      id: (typeof p._id === "string" ? p._id : p._id?.$oid) || "",
      title: p.title,
    }));
  } catch {}
}

async function postOperation(o: Op) {
  const id = getId(o);
  const selectedStore = (storeSelect.value[id] || o.store_id || "") as string;
  if (!selectedStore) {
    alert("Сначала выберите магазин.");
    return;
  }
  // If UI has a selection that isn't persisted yet, persist it first
  if (!o.store_id || o.store_id !== selectedStore) {
    try {
      const resSave = await fetch(`${API}/operations/${id}`, {
        method: "PUT",
        headers: authHeaders({ "Content-Type": "application/json" }),
        body: JSON.stringify({ store_id: selectedStore }),
      });
      if (!resSave.ok) {
        try { const err = await resSave.json(); alert(err?.error || "Не удалось сохранить магазин для операции"); } catch {}
        return;
      }
    } catch {
      return;
    }
  }
  try {
    const res = await fetch(`${API}/operations/${id}/status`, {
      method: "PUT",
      headers: authHeaders({ "Content-Type": "application/json" }),
      body: JSON.stringify({ status: "posted" }),
    });
    if (!res.ok) {
      try {
        const err = await res.json();
        alert(err?.error || "Не удалось обновить статус операции");
      } catch {}
      return;
    }
    await loadOps();
  } catch {}
}

async function deleteOperation(o: Op) {
  const id = getId(o);
  if (!confirm("Удалить операцию?")) return;
  try {
    await fetch(`${API}/operations/${id}`, {
      method: "DELETE",
      headers: authHeaders(),
    });
    await loadOps();
  } catch {}
}

async function saveStore(o: Op) {
  const id = getId(o);
  const sid = storeSelect.value[id] || "";
  try {
    const res = await fetch(`${API}/operations/${id}`, {
      method: "PUT",
      headers: authHeaders({ "Content-Type": "application/json" }),
      body: JSON.stringify({ store_id: sid }),
    });
    if (!res.ok) {
      try { const err = await res.json(); alert(err?.error || "Не удалось сохранить магазин для операции"); } catch {}
      return;
    }
    await loadOps();
  } catch {}
}

function filteredProducts(o: any, idx: number): ProductRef[] {
  const id = getId(o);
  const q = (productQuery.value[id]?.[idx] || "").toLowerCase();
  if (!q) return products.value;
  return products.value.filter((p) => p.title.toLowerCase().includes(q));
}

async function saveItems(o: any) {
  const id = getId(o);
  const payload = { items: edited.value[id]?.items || [] };
  try {
    await fetch(`${API}/operations/${id}`, {
      method: "PUT",
      headers: authHeaders({ "Content-Type": "application/json" }),
      body: JSON.stringify(payload),
    });
    alert("Позиции сохранены");
  } catch {}
}

function openModal(url: string, kind: "stores" | "products") {
  modalUrl.value = url;
  modalKind.value = kind;
  showModal.value = true;
  if (kind === "stores") {
    quickStoreName.value = "";
    quickStoreAddr.value = "";
  }
  if (kind === "products") quickProductTitle.value = "";
}
function closeModal() {
  showModal.value = false;
  if (modalKind.value === "stores") loadStores();
  if (modalKind.value === "products") loadProducts();
}

function openModalStore(name: string) {
  openModal("/stores/new", "stores");
  quickStoreName.value = String(name || "");
}
function openModalProduct(title: string) {
  openModal("/products/new", "products");
  quickProductTitle.value = String(title || "");
}

async function submitQuickStore() {
  try {
    await fetch(`${API}/stores`, {
      method: "POST",
      headers: authHeaders({ "Content-Type": "application/json" }),
      body: JSON.stringify({
        name: quickStoreName.value,
        addr: quickStoreAddr.value || "",
      }),
    });
    await loadStores();
    showModal.value = false;
  } catch {}
}

async function submitQuickProduct() {
  try {
    await fetch(`${API}/products`, {
      method: "POST",
      headers: authHeaders({ "Content-Type": "application/json" }),
      body: JSON.stringify({ title: quickProductTitle.value, desc: "" }),
    });
    await loadProducts();
    showModal.value = false;
  } catch {}
}

onMounted(() => {
  loadOps();
  loadStores();
  loadProducts();
});

function prettyRaw(o: any): string {
  try {
    if (!o || !('raw' in o)) return '';
    const raw = (o as any).raw;
    if (raw == null) return '';
    // If string, try to pretty-print JSON
    if (typeof raw === 'string') {
      try { const obj = JSON.parse(raw); return JSON.stringify(obj, null, 2); } catch { return raw; }
    }
    return JSON.stringify(raw, null, 2);
  } catch {
    try { return String((o as any)?.raw ?? ''); } catch { return ''; }
  }
}

function rawHtml(o: any): string {
  try {
    let raw: any = (o as any)?.raw;
    if (!raw) return '';
    if (typeof raw === 'string') { try { raw = JSON.parse(raw); } catch { /* keep as string */ } }
    // Try common paths first
    const direct = raw?.data?.html || raw?.html || raw?.body;
    if (typeof direct === 'string' && /</.test(direct)) return direct;
    // Fallback: deep search for any string that looks like HTML
    const stack: any[] = [raw];
    const seen = new Set<any>();
    while (stack.length) {
      const cur = stack.pop();
      if (!cur || typeof cur !== 'object' || seen.has(cur)) continue;
      seen.add(cur);
      for (const k of Object.keys(cur)) {
        const v: any = (cur as any)[k];
        if (typeof v === 'string' && v.length > 20 && /<[^>]+>/.test(v)) return v;
        if (v && typeof v === 'object') stack.push(v);
      }
    }
    return '';
  } catch {
    return '';
  }
}

function toggleItems(o: any) {
  const id = getId(o);
  expanded.value[id] = !expanded.value[id];
}

const rawModalVisible = ref(false);
const rawModalKind = ref<"json" | "html" | "">("");
const rawModalTitle = ref("");
const rawModalJson = ref("");
const rawModalHtml = ref("");

async function openJson(o: any) {
  await ensureRaw(o);
  rawModalKind.value = "json";
  rawModalTitle.value = `Ответ API — ${o.seller}`;
  rawModalJson.value = prettyRaw(o);
  rawModalVisible.value = true;
}

async function openHtml(o: any) {
  await ensureRaw(o);
  rawModalKind.value = "html";
  rawModalTitle.value = `HTML из чека — ${o.seller}`;
  rawModalHtml.value = rawHtml(o);
  rawModalVisible.value = true;
}

function closeRawModal() {
  rawModalVisible.value = false;
  rawModalKind.value = "";
  rawModalJson.value = "";
  rawModalHtml.value = "";
}
</script>

