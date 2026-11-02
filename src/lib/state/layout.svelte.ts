const MEMBERS_KEY = "harmony.showMembers";
const WIDE = 1100;

function stored(): boolean | null {
  try {
    const v = localStorage.getItem(MEMBERS_KEY);
    return v === null ? null : v === "true";
  } catch {
    return null;
  }
}

class Layout {
  width = $state(typeof window === "undefined" ? WIDE : window.innerWidth);
  /** Explicit user choice; `null` = automatic (hidden on narrow windows). */
  private choice = $state<boolean | null>(stored());

  get showMembers(): boolean {
    return this.choice ?? this.width >= WIDE;
  }

  toggleMembers() {
    this.choice = !this.showMembers;
    try {
      localStorage.setItem(MEMBERS_KEY, String(this.choice));
    } catch {
      // Not persisted; fine.
    }
  }
}

export const layout = new Layout();
