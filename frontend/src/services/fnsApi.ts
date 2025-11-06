export type FnsItem = {
  name: string;
  price: number; // in kopecks
  quantity: number;
};

export type FnsResponse = {
  dateTime: string; // ISO string
  seller: { name: string };
  totalSum: number; // in kopecks
  items: FnsItem[];
};

export type FnsCheckResult = { raw: any; normalized: FnsResponse };

function parseQRParams(qr: string): URLSearchParams {
  // QR string format: "t=20250922T1549&s=1500.00&fn=...&i=...&fp=...&n=1"
  // It may contain a leading scheme like "?" or not.
  const clean = qr.trim().replace(/^\?/, '');
  return new URLSearchParams(clean);
}

export async function getCheckByQR(qr: string): Promise<FnsCheckResult> {
  // Go through backend to avoid CORS and keep token secret
  const { API } = await import('../api');
  const url = `${API}/fns/check?` + new URLSearchParams({ qr }).toString();
  const res = await fetch(url);

  if (!res.ok) {
    const text = await res.text().catch(() => '');
    throw new Error(`FNS API error ${res.status}: ${text || res.statusText}`);
  }

  const json = await res.json();
  const normalized = normalizeFnsPayload(json, qr);
  return { raw: json, normalized };
}

function parseQrTimestampMaybe(qr: string): string | null {
  try {
    const params = parseQRParams(qr);
    const t = params.get('t');
    if (!t) return null;
    const m = t.match(/^(\d{4})(\d{2})(\d{2})T(\d{2})(\d{2})(\d{2})?$/);
    if (!m) return null;
    const [, Y, M, D, h, mnt, s] = m;
    const dt = new Date(Number(Y), Number(M) - 1, Number(D), Number(h), Number(mnt), s ? Number(s) : 0);
    return dt.toISOString();
  } catch {
    return null;
  }
}

function toKopecks(n: any): number {
  if (typeof n === 'number') return Math.round(n);
  if (typeof n === 'string') {
    const s = n.replace(',', '.');
    if (/\.|e/i.test(s)) {
      const rub = Number(s);
      return Math.round(isFinite(rub) ? rub * 100 : 0);
    }
    const asInt = Number(s);
    return isFinite(asInt) ? Math.round(asInt) : 0;
  }
  return 0;
}

function normalizeFnsPayload(json: any, qr: string): FnsResponse {
  // Helper to safely access
  const maybe = (path: (obj: any) => any, def?: any) => {
    try { const v = path(json); return v === undefined || v === null ? def : v; } catch { return def; }
  };

  // Depth-first search to find an object that looks like a receipt
  function findReceiptNode(node: any): any | null {
    if (!node || typeof node !== 'object') return null;
    const hasItems = Array.isArray((node as any).items) && (node as any).items.length >= 0;
    const hasTotals = 'totalSum' in node || 'sum' in node;
    const hasDate = 'dateTime' in node || 'date' in node;
    if (hasItems && (hasTotals || hasDate)) return node;
    for (const k of Object.keys(node)) {
      const child = (node as any)[k];
      const found = findReceiptNode(child);
      if (found) return found;
    }
    return null;
  }

  const candidate =
    maybe(j => j.data?.json) ||
    maybe(j => j.json) ||
    maybe(j => j.data?.ticket?.document?.receipt) ||
    maybe(j => j.ticket?.document?.receipt) ||
    findReceiptNode(json) ||
    json;

  // Date
  let dateRaw = candidate?.dateTime ?? candidate?.date ?? maybe(j => j.data?.dateTime);
  let iso = '';
  if (typeof dateRaw === 'number') {
    const ms = dateRaw > 10_000_000_000 ? dateRaw : dateRaw * 1000;
    iso = new Date(ms).toISOString();
  } else if (typeof dateRaw === 'string') {
    const tryDate = new Date(dateRaw);
    if (!isNaN(tryDate.getTime())) iso = tryDate.toISOString();
  }
  if (!iso) iso = parseQrTimestampMaybe(qr) || new Date().toISOString();

  // Seller
  const sellerName = candidate?.seller?.name
    || candidate?.user
    || candidate?.retailPlace
    || candidate?.retailPlaceAddress
    || candidate?.sellerName
    || '—';

  // Items
  const itemsSrc = Array.isArray(candidate?.items) ? candidate.items : [];
  const items: FnsItem[] = itemsSrc.map((i: any) => {
    const qty = Number(i.quantity ?? i.qty ?? 1) || 1;
    const priceK = toKopecks(i.price ?? (i.sum != null ? (Number(i.sum) / qty) : 0));
    return {
      name: i.name || i.itemName || 'Товар',
      price: priceK,
      quantity: qty,
    };
  });

  // Total
  let total = candidate?.totalSum ?? candidate?.sum ?? items.reduce((acc: number, it: any) => acc + (it.price * (it.quantity || 1)), 0);
  total = toKopecks(total);

  return {
    dateTime: iso,
    seller: { name: String(sellerName) },
    totalSum: total,
    items,
  };
}
