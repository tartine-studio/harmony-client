/**
 * Everything the chat screens show: users, channels, presence and messages.
 * Fed by Rust commands for the initial loads and by gateway events after.
 */
import { SvelteSet } from "svelte/reactivity";
import type { AppError } from "$lib/bindings/AppError";
import type { Channel } from "$lib/bindings/Channel";
import type { GatewayStatus } from "$lib/bindings/GatewayStatus";
import type { ServerEvent } from "$lib/bindings/ServerEvent";
import type { User } from "$lib/bindings/User";
import { commands, toAppError } from "$lib/core/commands";
import { onGatewayEvent, onGatewayStatus } from "$lib/core/events";
import * as list from "$lib/chat/messages";
import type { ChatMessage } from "$lib/chat/messages";
import { applyBroadcast, emptyPresence, fromSnapshot, statusOf, type PresenceView } from "$lib/chat/presence";
import { initials, tintIndex } from "$lib/chat/avatar";
import { errorText } from "$lib/i18n/index.svelte";
import { session } from "./session.svelte";
import { toasts } from "./toasts.svelte";

export interface ChannelMessages {
  status: "loading" | "ready" | "error";
  error?: AppError;
  items: ChatMessage[];
}

class Chat {
  users = $state<Record<string, User>>({});
  channels = $state<Channel[]>([]);
  channelsLoaded = $state(false);
  presence = $state<PresenceView>(emptyPresence());
  gateway = $state<GatewayStatus>("connecting");
  messages = $state<Record<string, ChannelMessages>>({});
  activeChannelId = $state<string | null>(null);
  unread = new SvelteSet<string>();

  private unlisten: (() => void)[] = [];
  private everConnected = false;
  private usersReload: ReturnType<typeof setTimeout> | null = null;

  get textChannels() {
    return this.channels.filter((c) => c.type === "text");
  }

  get voiceChannels() {
    return this.channels.filter((c) => c.type === "voice");
  }

  private get myId() {
    return session.me?.id ?? "";
  }

  // ----- Lifecycle -----------------------------------------------------------

  async start() {
    this.stop();
    this.unlisten = await Promise.all([
      onGatewayEvent((e) => this.onEvent(e)),
      onGatewayStatus((s) => this.onStatus(s)),
    ]);
    await Promise.all([this.loadUsers(), this.loadChannels()]);
  }

  stop() {
    this.unlisten.forEach((fn) => fn());
    this.unlisten = [];
    this.users = {};
    this.channels = [];
    this.channelsLoaded = false;
    this.presence = emptyPresence();
    this.gateway = "connecting";
    this.messages = {};
    this.activeChannelId = null;
    this.unread.clear();
    this.everConnected = false;
  }

  async loadUsers() {
    try {
      const users = await commands.listUsers();
      this.users = Object.fromEntries(users.map((u) => [u.id, u]));
    } catch (e) {
      toasts.show(errorText(toAppError(e).code));
    }
  }

  async loadChannels() {
    try {
      this.channels = await commands.listChannels();
    } catch (e) {
      toasts.show(errorText(toAppError(e).code));
    } finally {
      this.channelsLoaded = true;
    }
  }

  private scheduleUsersReload() {
    if (this.usersReload) return;
    this.usersReload = setTimeout(() => {
      this.usersReload = null;
      void this.loadUsers();
    }, 500);
  }

  // ----- Users & presence ----------------------------------------------------

  user(id: string | null): User | null {
    return id ? (this.users[id] ?? null) : null;
  }

  status(userId: string) {
    return statusOf(this.presence, userId);
  }

  avatar(userId: string | null, name: string) {
    return { initials: initials(name), tint: userId ? tintIndex(userId) : 5 };
  }

  async setInvisible(invisible: boolean) {
    const previous = this.presence.invisible;
    this.presence.invisible = invisible;
    try {
      await commands.setInvisible(invisible);
    } catch (e) {
      this.presence.invisible = previous;
      toasts.show(errorText(toAppError(e).code));
    }
  }

  // ----- Channels ------------------------------------------------------------

  open(channelId: string) {
    this.activeChannelId = channelId;
    this.unread.delete(channelId);
    const entry = this.messages[channelId];
    if (!entry || entry.status === "error") void this.loadMessages(channelId);
  }

  async loadMessages(channelId: string) {
    const current = this.messages[channelId];
    const knownBefore = new Set(current?.items.map((m) => m.id) ?? []);
    this.messages[channelId] = { status: "loading", items: current?.items ?? [] };
    try {
      const history = await commands.listMessages(channelId);
      const items = list.mergeHistory(history, this.messages[channelId]?.items ?? [], knownBefore);
      this.messages[channelId] = { status: "ready", items };
      if (history.some((m) => m.userId && !this.users[m.userId])) this.scheduleUsersReload();
    } catch (e) {
      this.messages[channelId] = { status: "error", error: toAppError(e), items: current?.items ?? [] };
    }
  }

  private update(channelId: string, fn: (items: ChatMessage[]) => ChatMessage[]) {
    const entry = this.messages[channelId];
    if (entry) entry.items = fn(entry.items);
  }

  private findChannelOf(messageId: string): string | null {
    for (const [channelId, entry] of Object.entries(this.messages)) {
      if (entry.items.some((m) => m.id === messageId)) return channelId;
    }
    return null;
  }

  // ----- Messages ------------------------------------------------------------

  async send(channelId: string, content: string) {
    const local = list.makeLocal(channelId, this.myId, content);
    this.update(channelId, (items) => [...items, local]);
    await this.deliver(channelId, local.id, content);
  }

  async retry(channelId: string, localId: string) {
    const target = this.messages[channelId]?.items.find((m) => m.id === localId);
    if (!target) return;
    this.update(channelId, (items) => list.retrying(items, localId));
    await this.deliver(channelId, localId, target.content);
  }

  discard(channelId: string, localId: string) {
    this.update(channelId, (items) => list.removeById(items, localId));
  }

  private async deliver(channelId: string, localId: string, content: string) {
    try {
      const message = await commands.sendMessage(channelId, content);
      this.update(channelId, (items) => list.confirm(items, localId, message));
    } catch (e) {
      this.update(channelId, (items) => list.fail(items, localId, toAppError(e)));
    }
  }

  /** Throws the AppError so the editor can stay open. */
  async edit(messageId: string, content: string) {
    const message = await commands.editMessage(messageId, content);
    this.update(message.channelId, (items) => list.upsert(items, message));
  }

  async remove(messageId: string) {
    try {
      await commands.deleteMessage(messageId);
      const channelId = this.findChannelOf(messageId);
      if (channelId) this.update(channelId, (items) => list.removeById(items, messageId));
    } catch (e) {
      toasts.show(errorText(toAppError(e).code));
    }
  }

  // ----- Gateway -------------------------------------------------------------

  private onStatus(status: GatewayStatus) {
    this.gateway = status;
    if (status !== "connected") return;
    if (this.everConnected) void this.resync();
    this.everConnected = true;
  }

  /** After a reconnection we may have missed events: reload what is shown. */
  private async resync() {
    const active = this.activeChannelId;
    for (const id of Object.keys(this.messages)) {
      if (id !== active) delete this.messages[id];
    }
    await Promise.all([this.loadUsers(), this.loadChannels(), active ? this.loadMessages(active) : null]);
  }

  private onEvent(event: ServerEvent) {
    switch (event.type) {
      case "presenceSnapshot":
        this.presence = fromSnapshot(event.data, this.myId);
        break;
      case "presenceBroadcast":
        this.presence = applyBroadcast(this.presence, event.data, this.myId);
        break;
      case "messageCreated":
      case "messageUpdated": {
        const m = event.data;
        if (m.userId && !this.users[m.userId]) this.scheduleUsersReload();
        // Channels never opened have no cache: their history loads on open.
        this.update(m.channelId, (items) => list.upsert(items, m));
        if (
          event.type === "messageCreated" &&
          m.channelId !== this.activeChannelId &&
          m.userId !== this.myId
        ) {
          this.unread.add(m.channelId);
        }
        break;
      }
      case "messageDeleted": {
        const channelId = this.findChannelOf(event.data.messageId);
        if (channelId) this.update(channelId, (items) => list.removeById(items, event.data.messageId));
        break;
      }
    }
  }
}

export const chat = new Chat();
