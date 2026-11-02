<script lang="ts">
  import { untrack } from "svelte";
  import { dayLabel, groupMessages } from "$lib/chat/grouping";
  import type { ChatMessage } from "$lib/chat/messages";
  import Icon from "$lib/components/ui/Icon.svelte";
  import { formatDay, t } from "$lib/i18n/index.svelte";
  import { session } from "$lib/state/session.svelte";
  import MessageItem from "./MessageItem.svelte";

  let { channelName, items }: { channelName: string; items: ChatMessage[] } = $props();

  const rows = $derived(groupMessages(items));
  const lastId = $derived(items.at(-1)?.id);
  const lastIsMine = $derived(!!session.me && items.at(-1)?.userId === session.me.id);

  let viewport: HTMLDivElement;
  let stuck = true;
  let unseen = $state(false);

  function nearBottom(): boolean {
    return viewport.scrollHeight - viewport.scrollTop - viewport.clientHeight < 48;
  }

  function scrollToBottom() {
    viewport.scrollTop = viewport.scrollHeight;
    unseen = false;
  }

  function onScroll() {
    stuck = nearBottom();
    if (stuck) unseen = false;
  }

  // Stay pinned to the bottom when new messages arrive, unless the user has
  // scrolled up to read history; then offer a jump button instead. Our own
  // messages always scroll into view.
  $effect(() => {
    if (!lastId || !viewport) return;
    if (stuck || untrack(() => lastIsMine)) scrollToBottom();
    else unseen = true;
  });

  function dayText(date: Date): string {
    const label = dayLabel(date);
    return label === "today" ? t("message.today") : label === "yesterday" ? t("message.yesterday") : formatDay(date);
  }
</script>

<div class="viewport" bind:this={viewport} onscroll={onScroll}>
  <div class="intro">
    <div class="intro-icon"><Icon name="hash" size={28} /></div>
    <h2>{t("channel.welcome", { name: channelName })}</h2>
    <p>{t("channel.beginning")}</p>
  </div>

  <ol>
    {#each rows as row (row.message.id)}
      <li>
        {#if row.newDay}
          <div class="day" role="separator"><span>{dayText(row.newDay)}</span></div>
        {/if}
        <MessageItem message={row.message} showHeader={row.showHeader} />
      </li>
    {/each}
  </ol>
</div>

{#if unseen}
  <button type="button" class="jump" onclick={scrollToBottom}>
    {t("channel.newMessages")}
    <Icon name="arrowDown" size={16} />
  </button>
{/if}

<style>
  .viewport {
    flex: 1 1 0;
    min-height: 0;
    overflow-y: auto;
    padding: 24px 0 12px;
  }

  .intro {
    padding: 0 20px 20px;
    margin-bottom: 8px;
    border-bottom: 1px solid var(--color-border);
  }

  .intro-icon {
    width: 56px;
    height: 56px;
    border-radius: 50%;
    background: var(--color-bg-elevated);
    display: flex;
    align-items: center;
    justify-content: center;
    margin-bottom: 12px;
    color: var(--color-text-secondary);
  }

  h2 {
    margin: 0 0 4px;
    font-size: 24px;
    font-weight: 700;
    letter-spacing: -0.01em;
    overflow-wrap: anywhere;
  }

  .intro p {
    margin: 0;
    color: var(--color-text-muted);
  }

  ol {
    list-style: none;
    margin: 0;
    padding: 0;
  }

  .day {
    display: flex;
    align-items: center;
    gap: 12px;
    margin: 16px 20px 0;
    font-size: 12px;
    font-weight: 600;
    color: var(--color-text-subtle);
  }

  .day::before,
  .day::after {
    content: "";
    flex: 1;
    height: 1px;
    background: var(--color-border);
  }

  .jump {
    position: absolute;
    left: 50%;
    bottom: 96px;
    transform: translateX(-50%);
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 8px 14px;
    border: 0;
    border-radius: var(--radius-full);
    background: var(--color-accent);
    color: var(--color-accent-contrast);
    font-size: 13px;
    font-weight: 600;
    box-shadow: 0 8px 24px var(--color-overlay-shadow);
  }
</style>
