import { reactive, computed } from 'vue';

type Role = 'admin' | 'user' | '';

const state = reactive({
  token: localStorage.getItem('token') || '',
  username: localStorage.getItem('username') || '',
  role: (localStorage.getItem('role') as Role) || '',
});

export const isAuthenticated = computed(() => !!state.token);
export const isAdmin = computed(() => state.role === 'admin');

export function loginAuth(token: string, username: string, role: string) {
  state.token = token;
  state.username = username;
  state.role = (role as Role) || '';
  localStorage.setItem('token', token);
  localStorage.setItem('username', username);
  localStorage.setItem('role', role);
}

export function logoutAuth() {
  state.token = '';
  state.username = '';
  state.role = '' as Role;
  localStorage.removeItem('token');
  localStorage.removeItem('username');
  localStorage.removeItem('role');
}

export function useAuth() {
  return { state, isAuthenticated, isAdmin, loginAuth, logoutAuth };
}

// Синхронизация между вкладками
window.addEventListener('storage', (e) => {
  if (e.key === 'token' || e.key === 'username' || e.key === 'role') {
    state.token = localStorage.getItem('token') || '';
    state.username = localStorage.getItem('username') || '';
    state.role = (localStorage.getItem('role') as Role) || '';
  }
});

// Локальная проверка токена: exp из JWT payload
function decodeJwtExp(t: string): number | null {
  try {
    const parts = t.split('.');
    if (parts.length !== 3) return null;
    const payload = parts[1]
      .replace(/-/g, '+')
      .replace(/_/g, '/');
    const json = atob(payload);
    const obj = JSON.parse(json);
    if (typeof obj.exp === 'number') return obj.exp as number;
    return null;
  } catch { return null; }
}

export function ensureAuthFresh() {
  if (!state.token) return;
  const exp = decodeJwtExp(state.token);
  const now = Math.floor(Date.now() / 1000);
  if (!exp || exp <= now) {
    // протухший/битый токен — выходим
    logoutAuth();
  }
}

// Автопроверка при загрузке модуля
ensureAuthFresh();
