export const API = import.meta.env.VITE_API_URL || 'http://localhost:8080';

export function getToken(): string | null {
  return localStorage.getItem('token');
}

export function authHeaders(headers: Record<string,string> = {}): HeadersInit {
  const t = getToken();
  return t ? { ...headers, Authorization: `Bearer ${t}` } : headers;
}

