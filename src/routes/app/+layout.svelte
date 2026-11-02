<script lang="ts">
  import { untrack } from "svelte";
  import { goto } from "$app/navigation";
  import ChannelSidebar from "$lib/components/chat/ChannelSidebar.svelte";
  import MemberList from "$lib/components/chat/MemberList.svelte";
  import Icon from "$lib/components/ui/Icon.svelte";
  import { commands } from "$lib/core/commands";
  import { onSessionExpired } from "$lib/core/events";
  import { t } from "$lib/i18n/index.svelte";
  import { chat } from "$lib/state/chat.svelte";
  import { layout } from "$lib/state/layout.svelte";
  import { session } from "$lib/state/session.svelte";

  let { children } = $props();

  const ready = $derived(session.status === "authenticated");

  $effect(() => {
    if (!ready) {
      void goto("/", { replaceState: true });
      return;
    }
    // Only `ready` should re-run this effect, not the state chat touches.
    untrack(() => void chat.start());
    const expired = onSessionExpired(async () => {
      untrack(() => chat.stop());
      await session.expire();
      await goto("/login", { replaceState: true });
    });
    return () => {
      void expired.then((off) => off());
      untrack(() => chat.stop());
    };
  });

  // Report real user input so the server keeps us "online". Rust throttles
  // the frames to one a minute; this only avoids flooding the IPC bridge.
  let lastActivity = 0;
  function activity() {
    const now = Date.now();
    if (now - lastActivity < 10_000) return;
    lastActivity = now;
    void commands.reportActivity();
  }
</script>

<svelte:window
  bind:innerWidth={layout.width}
  onkeydown={activity}
  onpointerdown={activity}
  onfocus={activity}
/>

{#if ready}
  <div class="shell">
    <ChannelSidebar />
    <main>
      {#if chat.gateway === "reconnecting"}
        <div class="banner" role="status">
          <Icon name="retry" size={15} />
          {t("gateway.reconnecting")}
        </div>
      {/if}
      {@render children()}
    </main>
    {#if layout.showMembers}
      <MemberList />
    {/if}
  </div>
{/if}

<style>
  .shell {
    height: 100%;
    display: flex;
    background: var(--color-bg-outer);
  }

  main {
    position: relative;
    flex: 1 1 0;
    min-width: 0;
    min-height: 0;
    display: flex;
    flex-direction: column;
    background: var(--color-bg);
  }

  .banner {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    padding: 8px 16px;
    background: var(--color-danger-soft);
    color: var(--color-danger-text);
    font-size: 13px;
    font-weight: 500;
  }
</style>
