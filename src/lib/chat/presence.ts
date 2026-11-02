import type { Presence } from "$lib/bindings/Presence";
import type { PresenceSnapshot } from "$lib/bindings/PresenceSnapshot";
import type { PresenceStatus } from "$lib/bindings/PresenceStatus";

export interface PresenceView {
  statuses: Record<string, PresenceStatus>;
  /** My own "appear offline" setting. */
  invisible: boolean;
}

export function emptyPresence(): PresenceView {
  return { statuses: {}, invisible: false };
}

/** Rebuilds presence from the snapshot sent on (re)connection. Users not
 *  listed are offline. */
export function fromSnapshot(snapshot: PresenceSnapshot, me: string): PresenceView {
  const view = emptyPresence();
  for (const p of snapshot.presences) {
    view.statuses[p.userId] = p.status;
    if (p.userId === me && p.invisible !== undefined) view.invisible = p.invisible;
  }
  return view;
}

export function applyBroadcast(view: PresenceView, update: Presence, me: string): PresenceView {
  const statuses = { ...view.statuses, [update.userId]: update.status };
  const invisible = update.userId === me && update.invisible !== undefined ? update.invisible : view.invisible;
  return { statuses, invisible };
}

export function statusOf(view: PresenceView, userId: string): PresenceStatus {
  return view.statuses[userId] ?? "offline";
}
