/** Typed subscriptions to the events emitted by the Rust core. */
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { GatewayStatus } from "$lib/bindings/GatewayStatus";
import type { ServerEvent } from "$lib/bindings/ServerEvent";

export function onGatewayEvent(handler: (event: ServerEvent) => void): Promise<UnlistenFn> {
  return listen<ServerEvent>("gateway:event", (e) => handler(e.payload));
}

export function onGatewayStatus(handler: (status: GatewayStatus) => void): Promise<UnlistenFn> {
  return listen<GatewayStatus>("gateway:status", (e) => handler(e.payload));
}

export function onSessionExpired(handler: () => void): Promise<UnlistenFn> {
  return listen("session:expired", () => handler());
}
