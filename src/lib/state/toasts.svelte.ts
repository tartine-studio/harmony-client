export interface Toast {
  id: number;
  text: string;
}

let nextId = 1;

class Toasts {
  items = $state<Toast[]>([]);

  show(text: string, durationMs = 5000) {
    const id = nextId++;
    this.items.push({ id, text });
    setTimeout(() => this.dismiss(id), durationMs);
  }

  dismiss(id: number) {
    this.items = this.items.filter((t) => t.id !== id);
  }
}

export const toasts = new Toasts();
