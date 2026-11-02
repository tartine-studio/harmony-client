<script lang="ts">
  import { untrack } from "svelte";
  import { page } from "$app/state";
  import ChannelHeader from "$lib/components/chat/ChannelHeader.svelte";
  import Composer from "$lib/components/chat/Composer.svelte";
  import MessageList from "$lib/components/chat/MessageList.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import { errorText, t } from "$lib/i18n/index.svelte";
  import { chat } from "$lib/state/chat.svelte";

  const id = $derived(page.params.id ?? "");
  const channel = $derived(chat.textChannels.find((c) => c.id === id));
  const entry = $derived(chat.messages[id]);

  $effect(() => {
    const channelId = channel?.id;
    // Re-run on channel change only, not when its messages change.
    if (channelId) untrack(() => chat.open(channelId));
  });
</script>

{#if channel}
  {#key channel.id}
    <ChannelHeader name={channel.name} />
    {#if entry?.status === "error" && entry.items.length === 0}
      <div class="state">
        <p>{t("channel.loadFailed")} {errorText(entry.error?.code)}</p>
        <Button variant="secondary" onclick={() => chat.loadMessages(channel.id)}>{t("channel.retry")}</Button>
      </div>
    {:else if !entry || (entry.status === "loading" && entry.items.length === 0)}
      <div class="state" aria-live="polite"><p>{t("channel.loading")}</p></div>
    {:else}
      <MessageList channelName={channel.name} items={entry.items} />
    {/if}
    <Composer channelId={channel.id} channelName={channel.name} />
  {/key}
{:else if chat.channelsLoaded}
  <div class="state"><p>{t("channel.notFound")}</p></div>
{/if}

<style>
  .state {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 12px;
    padding: 24px;
    text-align: center;
    color: var(--color-text-muted);
  }

  .state p {
    margin: 0;
  }
</style>
