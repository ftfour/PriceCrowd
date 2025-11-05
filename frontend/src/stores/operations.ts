import { defineStore } from 'pinia';

export type OperationItem = {
  name: string;
  price: number;
  quantity: number;
  product_id?: string | null;
};

export type Operation = {
  id: string;
  date: string;
  seller: string;
  amount: number;
  items: OperationItem[];
  status: 'draft' | 'posted';
  raw?: any;
  store_id?: string | null;
  uploaded_by?: string;
  qr?: string | null;
};

const STORAGE_KEY = 'operations';

function loadFromStorage(): Operation[] {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    return raw ? (JSON.parse(raw) as Operation[]) : [];
  } catch {
    return [];
  }
}

function saveToStorage(ops: Operation[]) {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(ops));
  } catch {}
}

export const useOperationsStore = defineStore('operations', {
  state: () => ({
    operations: loadFromStorage() as Operation[],
  }),
  actions: {
    load() {
      this.operations = loadFromStorage();
    },
    save() {
      saveToStorage(this.operations);
    },
    add(op: Operation) {
      this.operations.unshift(op);
      this.save();
    },
    update(id: string, patch: Partial<Operation>) {
      const idx = this.operations.findIndex(o => o.id === id);
      if (idx !== -1) {
        this.operations[idx] = { ...this.operations[idx], ...patch } as Operation;
        this.save();
      }
    },
    setItemProduct(opId: string, itemIndex: number, productId: string | null) {
      const op = this.operations.find(o => o.id === opId);
      if (!op) return;
      if (!op.items[itemIndex]) return;
      op.items[itemIndex].product_id = productId;
      this.save();
    },
    setStore(opId: string, storeId: string | null) {
      const op = this.operations.find(o => o.id === opId);
      if (!op) return;
      op.store_id = storeId;
      this.save();
    },
    remove(id: string) {
      this.operations = this.operations.filter(o => o.id !== id);
      this.save();
    },
  },
});
