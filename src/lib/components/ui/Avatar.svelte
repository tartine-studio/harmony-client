<script lang="ts">
  import type { PresenceStatus } from "$lib/bindings/PresenceStatus";
  import { initials, tintIndex } from "$lib/chat/avatar";

  interface Props {
    userId: string | null;
    name: string;
    size?: 24 | 32 | 36 | 40;
    /** Shows a presence dot. "invisible" renders like offline. */
    status?: PresenceStatus | "invisible";
    /** Background behind the dot, to cut it out of the avatar. */
    ring?: string;
    dim?: boolean;
  }

  let { userId, name, size = 40, status, ring = "var(--color-bg-sunken)", dim = false }: Props = $props();

  const tint = $derived(userId ? tintIndex(userId) : 5);
</script>

<div
  class="avatar"
  class:dim
  style:--size="{size}px"
  style:--bg="var(--avatar-{tint}-bg)"
  style:--fg="var(--avatar-{tint}-fg)"
  style:--ring={ring}
  aria-hidden="true"
>
  {initials(name)}
  {#if status}
    <span class="dot {status}"></span>
  {/if}
</div>

<style>
  .avatar {
    position: relative;
    flex: 0 0 auto;
    width: var(--size);
    height: var(--size);
    border-radius: 50%;
    background: var(--bg);
    color: var(--fg);
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: calc(var(--size) * 0.38);
    font-weight: 600;
    user-select: none;
  }

  .avatar.dim {
    opacity: 0.6;
  }

  .dot {
    position: absolute;
    right: -3px;
    bottom: -3px;
    width: 14px;
    height: 14px;
    border-radius: 50%;
    border: 3px solid var(--ring);
  }

  .dot.online {
    background: var(--color-status-online);
  }

  .dot.idle {
    background: var(--color-status-idle);
  }

  .dot.offline,
  .dot.invisible {
    background: var(--ring);
    box-shadow: inset 0 0 0 2px var(--color-status-offline);
  }
</style>
