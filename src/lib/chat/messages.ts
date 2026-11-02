/**
 * Pure helpers on a channel's message list. Lists are kept sorted oldest
 * first and never contain two entries with the same id.
 */
import type { AppError } from "$lib/bindings/AppError";
import type { Message } from "$lib/bindings/Message";

export type LocalState = { status: "pending" } | { status: "failed"; error: AppError };

/** A message as displayed: either confirmed by the server or still local. */
export type ChatMessage = Message & { local?: LocalState };

const LOCAL_PREFIX = "local:";

export function isLocalId(id: string): boolean {
  return id.startsWith(LOCAL_PREFIX);
}

export function makeLocal(channelId: string, userId: string, content: string, now = new Date()): ChatMessage {
  const stamp = now.toISOString();
  return {
    id: `${LOCAL_PREFIX}${crypto.randomUUID()}`,
    userId,
    channelId,
    content,
    createdAt: stamp,
    updatedAt: stamp,
    local: { status: "pending" },
  };
}

/** Inserts or replaces a server message, keeping the order. */
export function upsert(list: ChatMessage[], message: Message): ChatMessage[] {
  const existing = list.findIndex((m) => m.id === message.id);
  if (existing >= 0) {
    const next = list.slice();
    next[existing] = message;
    return next;
  }
  // Pending messages stay at the bottom, after every confirmed one.
  let i = list.length;
  while (i > 0 && (list[i - 1].local || list[i - 1].createdAt > message.createdAt)) i--;
  return [...list.slice(0, i), message, ...list.slice(i)];
}

/** Swaps a local message for the confirmed one (or drops it if the
 *  broadcast already inserted it). */
export function confirm(list: ChatMessage[], localId: string, message: Message): ChatMessage[] {
  return upsert(
    list.filter((m) => m.id !== localId),
    message,
  );
}

export function fail(list: ChatMessage[], localId: string, error: AppError): ChatMessage[] {
  return list.map((m) => (m.id === localId ? { ...m, local: { status: "failed", error } } : m));
}

export function retrying(list: ChatMessage[], localId: string): ChatMessage[] {
  const target = list.find((m) => m.id === localId);
  if (!target) return list;
  return [...list.filter((m) => m.id !== localId), { ...target, local: { status: "pending" } }];
}

export function removeById(list: ChatMessage[], id: string): ChatMessage[] {
  return list.filter((m) => m.id !== id);
}

/**
 * Merges a freshly loaded history with what the list holds now: local
 * messages still in flight, and server messages that arrived while the
 * history was loading (`knownBefore` = ids present when loading started).
 */
export function mergeHistory(history: Message[], current: ChatMessage[], knownBefore: Set<string>): ChatMessage[] {
  let merged: ChatMessage[] = history;
  for (const m of current) {
    if (!m.local && !knownBefore.has(m.id)) merged = upsert(merged, m);
  }
  return [...merged, ...current.filter((m) => m.local)];
}

export function isEdited(message: Message): boolean {
  return message.updatedAt !== message.createdAt;
}
