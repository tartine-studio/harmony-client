/**
 * Typed wrappers around the Rust commands (src-tauri/src/commands.rs).
 * Every network call goes through Rust; tokens never reach the webview.
 */
import { invoke } from "@tauri-apps/api/core";
import type { AppError } from "$lib/bindings/AppError";
import type { Channel } from "$lib/bindings/Channel";
import type { Message } from "$lib/bindings/Message";
import type { SessionInfo } from "$lib/bindings/SessionInfo";
import type { User } from "$lib/bindings/User";

export const MAX_MESSAGE_CHARS = 2000;

export function isAppError(value: unknown): value is AppError {
  return typeof value === "object" && value !== null && "code" in value && "message" in value;
}

/** Normalizes anything thrown by `invoke` into an `AppError`. */
export function toAppError(value: unknown): AppError {
  if (isAppError(value)) return value;
  return { code: "unknown", status: null, message: String(value) };
}

async function call<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  try {
    return await invoke<T>(command, args);
  } catch (e) {
    throw toAppError(e);
  }
}

export const commands = {
  sessionRestore: () => call<SessionInfo | null>("session_restore"),
  sessionLogin: (serverUrl: string, email: string, password: string) =>
    call<SessionInfo>("session_login", { serverUrl, email, password }),
  sessionRegister: (serverUrl: string, username: string, email: string, password: string) =>
    call<SessionInfo>("session_register", { serverUrl, username, email, password }),
  sessionLogout: () => call<void>("session_logout"),

  listUsers: () => call<User[]>("list_users"),
  listChannels: () => call<Channel[]>("list_channels"),
  listMessages: (channelId: string) => call<Message[]>("list_messages", { channelId }),

  sendMessage: (channelId: string, content: string) => call<Message>("send_message", { channelId, content }),
  editMessage: (messageId: string, content: string) => call<Message>("edit_message", { messageId, content }),
  deleteMessage: (messageId: string) => call<void>("delete_message", { messageId }),
  setInvisible: (invisible: boolean) => call<void>("set_invisible", { invisible }),
  reportActivity: () => call<void>("report_activity"),
};
