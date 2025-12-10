<template>
  <section class="space-y-6">
    <header class="flex items-center justify-between">
      <h1 class="text-2xl font-semibold">Аналитика</h1>
      <div class="text-sm text-slate-500" v-if="loaded">Операций: {{ ops.length }}</div>
    </header>

    <div v-if="loading" class="text-center text-slate-500 py-10">Загрузка...</div>

    <div v-else class="space-y-6">
      <!-- Summary cards -->
      <div class="grid grid-cols-1 sm:grid-cols-3 gap-4">
        <div class="rounded-lg border bg-white p-4">
          <div class="text-xs text-slate-500">Всего расходов</div>
          <div class="text-2xl font-semibold">{{ money(totalAmount) }} ₽</div>
        </div>
        <div class="rounded-lg border bg-white p-4">
          <div class="text-xs text-slate-500">Средний чек</div>
          <div class="text-2xl font-semibold">{{ money(avgCheck) }} ₽</div>
        </div>
        <div class="rounded-lg border bg-white p-4">
          <div class="text-xs text-slate-500">Продавцов</div>
          <div class="text-2xl font-semibold">{{ sellersCount }}</div>
        </div>
      </div>

      <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <!-- Sellers chart -->
        <div class="rounded-lg border bg-white p-4">
          <div class="font-medium mb-2">По продавцам (топ {{ topSellers.length }})</div>
          <div v-if="!topSellers.length" class="text-sm text-slate-500">Нет данных</div>
          <div v-else class="space-y-2">
            <div v-for="s in topSellers" :key="s.name" class="">
              <div class="flex justify-between text-sm">
                <div class="truncate pr-2">{{ s.name }}</div>
                <div class="text-slate-600">{{ money(s.total) }} ₽</div>
              </div>
              <div class="h-2 bg-slate-100 rounded">
                <div class="h-2 bg-blue-600 rounded" :style="{ width: (s.total/ maxSellerTotal * 100).toFixed(1) + '%' }"></div>
              </div>
              <div class="text-[11px] text-slate-500 mt-1">Чеков: {{ s.count }}, средний: {{ money(s.avg) }} ₽</div>
            </div>
          </div>
        </div>

        <!-- Categories distribution -->
        <div class="rounded-lg border bg-white p-4">
          <div class="font-medium mb-2">Распределение расходов по категориям</div>
          <div v-if="!categoriesDist.length" class="text-sm text-slate-500">Нет данных</div>
          <div v-else class="space-y-2">
            <div v-for="c in categoriesDist" :key="c.id" class="">
              <div class="flex justify-between text-sm">
                <div class="truncate pr-2">{{ c.name }}</div>
                <div class="text-slate-600">{{ money(c.total) }} ₽ ({{ ((c.total/ itemsTotal) * 100).toFixed(1) }}%)</div>
              </div>
              <div class="h-2 bg-slate-100 rounded">
                <div class="h-2 bg-emerald-600 rounded" :style="{ width: (c.total/ maxCatTotal * 100).toFixed(1) + '%' }"></div>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Raw tables -->
      <div class="rounded-lg border bg-white p-4">
        <div class="font-medium mb-2">Детализация по продавцам</div>
        <table class="min-w-full text-sm">
          <thead class="text-left bg-slate-50">
            <tr>
              <th class="px-2 py-2 border-b">Продавец</th>
              <th class="px-2 py-2 border-b w-32">Чеков</th>
              <th class="px-2 py-2 border-b w-40">Сумма</th>
              <th class="px-2 py-2 border-b w-40">Средний чек</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="s in sellersAll" :key="s.name" class="hover:bg-slate-50">
              <td class="px-2 py-2 border-b">{{ s.name }}</td>
              <td class="px-2 py-2 border-b">{{ s.count }}</td>
              <td class="px-2 py-2 border-b">{{ money(s.total) }} ₽</td>
              <td class="px-2 py-2 border-b">{{ money(s.avg) }} ₽</td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>
  </section>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { API, authHeaders } from '../api';

type OpItem = { name: string; price: number; quantity: number; product_id?: string | { $oid: string } | null };
type Op = { _id?: string; id?: string; date: string; seller: string; amount: number; items: OpItem[] };
type Product = { _id?: string | { $oid: string }; id?: string; title: string; category_ids?: (string | { $oid: string })[] };
type Category = { _id?: string | { $oid: string }; id?: string; name: string; parent_ids?: any[] };

function idOf(x:any): string { return typeof x === 'string' ? x : x?.$oid || x?.id || ''; }
function money(r:number){ return (r||0).toFixed(2); }

const loading = ref(true);
const loaded = ref(false);
const ops = ref<Op[]>([]);
const products = ref<Product[]>([]);
const categories = ref<Category[]>([]);

onMounted(async ()=>{
  loading.value = true;
  try {
    const [opsRes, prodsRes, catsRes] = await Promise.all([
      fetch(`${API}/operations`, { headers: authHeaders() }),
      fetch(`${API}/products`),
      fetch(`${API}/categories`),
    ]);
    const opsData = await opsRes.json();
    const prodsData = await prodsRes.json();
    const catsData = await catsRes.json();
    ops.value = Array.isArray(opsData) ? opsData : [];
    products.value = Array.isArray(prodsData) ? prodsData : [];
    categories.value = Array.isArray(catsData) ? catsData : [];
  } catch {}
  finally { loading.value = false; loaded.value = true; }
});

// Summary
const totalAmount = computed(()=> ops.value.reduce((a,o)=> a + (o.amount||0), 0));
const avgCheck = computed(()=> ops.value.length ? totalAmount.value / ops.value.length : 0);

// Sellers aggregation
type SellerAgg = { name: string; count: number; total: number; avg: number };
const sellersAll = computed<SellerAgg[]>(()=>{
  const map = new Map<string, { count: number; total: number }>();
  for (const o of ops.value){
    const key = o.seller || 'Не указан';
    const agg = map.get(key) || { count: 0, total: 0 };
    agg.count += 1; agg.total += (o.amount||0);
    map.set(key, agg);
  }
  return Array.from(map.entries()).map(([name, v])=> ({ name, count: v.count, total: v.total, avg: v.total / v.count }))
    .sort((a,b)=> b.total - a.total);
});
const sellersCount = computed(()=> sellersAll.value.length);
const topSellers = computed(()=> sellersAll.value.slice(0, 10));
const maxSellerTotal = computed(()=> topSellers.value.length ? topSellers.value[0].total : 1);

// Categories distribution (based on items with product_id)
type CatAgg = { id: string; name: string; total: number };
const categoriesDist = computed<CatAgg[]>(()=>{
  if (!products.value.length) return [];
  const prodToPrimaryCat = new Map<string, string>();
  for (const p of products.value){
    const pid = idOf(p._id) || idOf(p.id);
    const cats = (p.category_ids || []).map(idOf).filter(Boolean);
    if (pid) prodToPrimaryCat.set(pid, cats[0] || '');
  }
  const catName = new Map<string,string>();
  for (const c of categories.value){ const cid = idOf(c._id) || idOf(c.id); if (cid) catName.set(cid, c.name); }

  const totals = new Map<string, number>();
  let other = 0;
  for (const o of ops.value){
    for (const it of (o.items||[])){
      const pid = idOf(it.product_id as any);
      const amt = (it.price||0) * (it.quantity||0);
      if (!pid){ other += amt; continue; }
      const cid = prodToPrimaryCat.get(pid);
      if (!cid){ other += amt; continue; }
      totals.set(cid, (totals.get(cid)||0) + amt);
    }
  }
  const arr: CatAgg[] = Array.from(totals.entries()).map(([id,total])=> ({ id, name: catName.get(id)||'Без категории', total }));
  if (other>0) arr.push({ id: 'other', name: 'Без категории', total: other });
  arr.sort((a,b)=> b.total - a.total);
  return arr.slice(0, 15);
});
const itemsTotal = computed(()=> categoriesDist.value.reduce((a,x)=> a + x.total, 0));
const maxCatTotal = computed(()=> categoriesDist.value.length ? categoriesDist.value[0].total : 1);

</script>

