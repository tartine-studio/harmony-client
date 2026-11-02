<script lang="ts">
  import { goto } from "$app/navigation";
  import Icon from "$lib/components/ui/Icon.svelte";
  import { t } from "$lib/i18n/index.svelte";
  import { chat } from "$lib/state/chat.svelte";

  const first = $derived(chat.textChannels[0]);

  $effect(() => {
    if (first) void goto(`/app/channels/${first.id}`, { replaceState: true });
  });
</script>

{#if chat.channelsLoaded && !first}
  <div class="empty">
    <Icon name="hash" size={32} />
    <h1>{t("channel.noChannels")}</h1>
    <p>{t("channel.noChannelsHint")}</p>
  </div>
{/if}

<style>
  .empty {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 8px;
    padding: 24px;
    text-align: center;
    color: var(--color-text-subtle);
  }

  h1 {
    margin: 8px 0 0;
    font-size: 18px;
    color: var(--color-text);
  }

  p {
    margin: 0;
    color: var(--color-text-muted);
  }
</style>
