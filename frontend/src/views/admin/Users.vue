<template>
<section class="space-y-6">
    <div class="flex items-end justify-between gap-4">
      <h2 class="text-lg font-semibold">Пользователи</h2>
      <form @submit.prevent="create" class="flex items-center gap-2">
        <input v-model="newUser.username" placeholder="Логин" class="h-9 border rounded px-2 text-sm" required />
        <input v-model="newUser.password" type="password" placeholder="Пароль" class="h-9 border rounded px-2 text-sm" required />
        <button :disabled="busy" class="h-9 px-3 rounded bg-blue-600 text-white text-sm">Создать</button>
      </form>
    </div>
    <p v-if="msg" class="text-sm" :class="ok ? 'text-emerald-600' : 'text-rose-600'">{{ msg }}</p>

    <div class="rounded-lg border bg-white overflow-hidden">
      <table class="w-full text-left text-sm">
        <thead class="bg-slate-50 text-slate-500">
          <tr>
            <th class="px-4 py-2">ID</th>
            <th class="px-4 py-2">Логин</th>
            <th class="px-4 py-2">Роль</th>
            <th class="px-4 py-2">Действия</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="u in users" :key="u._id_str" class="border-t">
            <td class="px-4 py-2 text-xs text-slate-500">{{ u._id_str }}</td>
            <td class="px-4 py-2">
              <template v-if="editId===u._id_str">
                <input v-model="edit.username" class="h-8 border rounded px-2 text-sm" />
              </template>
              <template v-else>{{ u.username }}</template>
            </td>
            <td class="px-4 py-2 text-slate-700">{{ u.role }}</td>
            <td class="px-4 py-2">
              <template v-if="editId===u._id_str">
                <input v-model="edit.password" type="password" placeholder="Новый пароль (необязательно)" class="h-8 border rounded px-2 text-sm mr-2" />
                <button @click="save(u)" class="rounded border px-3 py-1 text-xs">Сохранить</button>
                <button @click="cancel()" class="ml-2 rounded border px-3 py-1 text-xs">Отмена</button>
              </template>
              <template v-else>
                <button @click="startEdit(u)" class="rounded border px-3 py-1 text-xs">Редактировать</button>
                <button @click="remove(u)" class="ml-2 rounded bg-red-600 text-white px-3 py-1 text-xs">Удалить</button>
              </template>
            </td>
          </tr>
          <tr v-if="users.length===0"><td colspan="4" class="px-4 py-6 text-center text-slate-500">Нет пользователей</td></tr>
        </tbody>
      </table>
    </div>
  </section>
</template>

<script setup lang="ts">
import { onMounted, ref } from 'vue';
import { API, authHeaders } from '../../api';

type User = { _id_str: string; username: string; role: string };

const users = ref<User[]>([]);
const newUser = ref({ username: '', password: '' });
const editId = ref<string>('');
const edit = ref<{ username: string; password: string }>({ username: '', password: '' });
const msg = ref('');
const ok = ref(false);
const busy = ref(false);

async function load(){
  const res = await fetch(`${API}/users`, { headers: authHeaders() });
  if (res.ok){
    const arr = await res.json();
    users.value = (Array.isArray(arr) ? arr : []).map((u:any)=> ({ _id_str: (typeof u._id==='string'? u._id : u._id?.$oid) || '', username: u.username, role: u.role }));
  }
}

async function create(){
  msg.value = '';
  const payload = { username: newUser.value.username.trim(), password: newUser.value.password };
  if (!payload.username || payload.password.length<4) { ok.value = false; msg.value = 'Минимальная длина пароля 4'; return; }
  busy.value = true;
  try{
    const res = await fetch(`${API}/users`, { method: 'POST', headers: authHeaders({ 'Content-Type': 'application/json' }), body: JSON.stringify(payload) });
    if (res.ok){ ok.value = true; msg.value = 'Пользователь создан'; newUser.value = { username: '', password: '' }; await load(); }
    else { ok.value = false; const t = await res.text().catch(()=> ''); msg.value = `Ошибка создания: ${res.status} ${t}`; }
  } finally { busy.value = false; }
}

function startEdit(u: User){ editId.value = u._id_str; edit.value = { username: u.username, password: '' }; }
function cancel(){ editId.value = ''; edit.value = { username: '', password: '' }; }

async function save(u: User){
  msg.value = '';
  const payload: any = {};
  if (edit.value.username && edit.value.username !== u.username) payload.username = edit.value.username;
  if (edit.value.password) payload.password = edit.value.password;
  if (Object.keys(payload).length===0) { cancel(); return; }
  const res = await fetch(`${API}/users/${u._id_str}`, { method: 'PUT', headers: authHeaders({ 'Content-Type': 'application/json' }), body: JSON.stringify(payload) });
  if (res.ok || res.status===204){ ok.value = true; msg.value = 'Сохранено'; cancel(); await load(); }
  else { ok.value = false; const t = await res.text().catch(()=> ''); msg.value = `Ошибка сохранения: ${res.status} ${t}`; }
}

async function remove(u: User){
  if (!confirm(`Удалить пользователя ${u.username}?`)) return;
  const res = await fetch(`${API}/users/${u._id_str}`, { method: 'DELETE', headers: authHeaders() });
  if (res.ok || res.status===204){ ok.value = true; msg.value = 'Удалено'; await load(); }
  else { ok.value = false; const t = await res.text().catch(()=> ''); msg.value = `Ошибка удаления: ${res.status} ${t}`; }
}

onMounted(load);
</script>
