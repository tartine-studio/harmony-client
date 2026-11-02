import type { AppError } from "$lib/bindings/AppError";
import type { SessionInfo } from "$lib/bindings/SessionInfo";
import { commands, toAppError } from "$lib/core/commands";

const LAST_SERVER_KEY = "harmony.lastServerUrl";

export type SessionStatus = "restoring" | "anonymous" | "authenticated" | "error";

function remember(serverUrl: string) {
  try {
    localStorage.setItem(LAST_SERVER_KEY, serverUrl);
  } catch {
    // Storage unavailable: only the prefill is lost.
  }
}

export function lastServerUrl(): string {
  try {
    return localStorage.getItem(LAST_SERVER_KEY) ?? "";
  } catch {
    return "";
  }
}

class Session {
  status = $state<SessionStatus>("restoring");
  info = $state<SessionInfo | null>(null);
  error = $state<AppError | null>(null);
  /** Set when the server rejected our refresh token. */
  expired = $state(false);

  get me() {
    return this.info?.me ?? null;
  }

  get serverHost(): string {
    if (!this.info) return "";
    try {
      return new URL(this.info.serverUrl).host;
    } catch {
      return this.info.serverUrl;
    }
  }

  private set(info: SessionInfo) {
    this.info = info;
    this.status = "authenticated";
    this.error = null;
    this.expired = false;
    remember(info.serverUrl);
  }

  async restore() {
    this.status = "restoring";
    this.error = null;
    try {
      const info = await commands.sessionRestore();
      if (info) this.set(info);
      else this.status = "anonymous";
    } catch (e) {
      this.error = toAppError(e);
      this.status = "error";
    }
  }

  async login(serverUrl: string, email: string, password: string) {
    this.set(await commands.sessionLogin(serverUrl, email, password));
  }

  async register(serverUrl: string, username: string, email: string, password: string) {
    this.set(await commands.sessionRegister(serverUrl, username, email, password));
  }

  async logout() {
    try {
      await commands.sessionLogout();
    } finally {
      this.info = null;
      this.status = "anonymous";
    }
  }

  async expire() {
    await this.logout();
    this.expired = true;
  }
}

export const session = new Session();
