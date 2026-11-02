<script lang="ts">
  import { goto } from "$app/navigation";
  import Avatar from "$lib/components/ui/Avatar.svelte";
  import Icon from "$lib/components/ui/Icon.svelte";
  import { t } from "$lib/i18n/index.svelte";
  import { chat } from "$lib/state/chat.svelte";
  import { session } from "$lib/state/session.svelte";

  let open = $state(false);
  let panel: HTMLDivElement;

  const me = $derived(session.me);
  const invisible = $derived(chat.presence.invisible);
  const status = $derived(invisible ? "invisible" : me ? chat.status(me.id) : "offline");
  const statusText = $derived(
    invisible ? t("status.invisible") : status === "idle" ? t("status.idle") : t("status.online"),
  );

  function onWindowClick(event: MouseEvent) {
    if (open && !panel.contains(event.target as Node)) open = false;
  }

  function onKeydown(event: KeyboardEvent) {
    if (open && event.key === "Escape") open = false;
  }

  async function logout() {
    open = false;
    chat.stop();
    await session.logout();
    await goto("/login", { replaceState: true });
  }
</script>

<svelte:window onclick={onWindowClick} onkeydown={onKeydown} />

<div class="panel" bind:this={panel}>
  {#if open}
    <div class="popover" role="dialog" aria-label={t("status.title")}>
      <div class="title">{t("status.title")}</div>
      <p>{t("status.explanation")}</p>
      <button
        type="button"
        class="toggle-row"
        role="switch"
        aria-checked={invisible}
        onclick={() => chat.setInvisible(!invisible)}
      >
        <span>{t("status.appearOffline")}</span>
        <span class="track" class:on={invisible}><span class="knob"></span></span>
      </button>
      <button type="button" class="logout" onclick={logout}>
        <Icon name="logout" size={16} />
        {t("status.logout")}
      </button>
    </div>
  {/if}

  {#if me}
    <button
      type="button"
      class="me"
      aria-label={t("status.edit")}
      aria-expanded={open}
      onclick={() => (open = !open)}
    >
      <Avatar userId={me.id} name={me.username} size={36} {status} ring="var(--color-bg-deep)" />
      <div class="text">
        <div class="username">{me.username}</div>
        <div class="status">{statusText}</div>
      </div>
    </button>
  {/if}
</div>

<style>
  .panel {
    position: relative;
    display: flex;
    align-items: center;
    padding: 10px;
    border-top: 1px solid var(--color-border);
    background: var(--color-bg-deep);
  }

  .me {
    flex: 1;
    min-width: 0;
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 6px;
    border: 0;
    border-radius: var(--radius-md);
    background: transparent;
    text-align: left;
  }

  .me:hover {
    background: var(--color-bg-hover);
  }

  .text {
    min-width: 0;
  }

  .username {
    font-size: 14px;
    font-weight: 600;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .status {
    font-size: 12px;
    color: var(--color-text-muted);
  }

  .popover {
    position: absolute;
    left: 10px;
    right: 10px;
    bottom: 66px;
    z-index: 2;
    padding: 16px;
    border-radius: var(--radius-xl);
    background: var(--color-surface);
    border: 1px solid var(--color-border-strong);
    box-shadow: 0 12px 32px var(--color-overlay-shadow);
  }

  .title {
    font-size: 13px;
    font-weight: 600;
    margin-bottom: 6px;
  }

  p {
    margin: 0 0 14px;
    font-size: 13px;
    color: var(--color-text-muted);
  }

  .toggle-row,
  .logout {
    display: flex;
    align-items: center;
    gap: 12px;
    width: 100%;
    padding: 10px 12px;
    border: 0;
    border-radius: var(--radius-md);
    background: var(--color-bg);
    font-size: 14px;
    text-align: left;
  }

  .toggle-row {
    justify-content: space-between;
  }

  .logout {
    margin-top: 8px;
    background: transparent;
    color: var(--color-danger-text);
  }

  .logout:hover {
    background: var(--color-danger-soft);
  }

  .track {
    position: relative;
    width: 40px;
    height: 24px;
    flex: 0 0 auto;
    border-radius: 12px;
    background: var(--color-control-off);
    transition: background 0.2s;
  }

  .track.on {
    background: var(--color-accent);
  }

  .knob {
    position: absolute;
    top: 3px;
    left: 3px;
    width: 18px;
    height: 18px;
    border-radius: 50%;
    background: var(--color-text-strong);
    transition: left 0.2s;
  }

  .track.on .knob {
    left: 19px;
  }

  @media (prefers-reduced-motion: reduce) {
    .track,
    .knob {
      transition: none;
    }
  }
</style>
