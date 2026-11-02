<script lang="ts">
  import Icon from "$lib/components/ui/Icon.svelte";
  import IconButton from "$lib/components/ui/IconButton.svelte";
  import { t } from "$lib/i18n/index.svelte";
  import { layout } from "$lib/state/layout.svelte";

  let { name }: { name: string } = $props();
</script>

<header>
  <span class="icon"><Icon name="hash" size={20} /></span>
  <h1>{name}</h1>
  <IconButton
    icon="users"
    label={t("channel.toggleMembers")}
    pressed={layout.showMembers}
    onclick={() => layout.toggleMembers()}
  />
</header>

<style>
  header {
    display: flex;
    align-items: center;
    gap: 10px;
    height: var(--size-header);
    flex: 0 0 auto;
    padding: 0 12px 0 20px;
    border-bottom: 1px solid var(--color-bg-elevated);
  }

  .icon {
    color: var(--color-text-subtle);
  }

  h1 {
    flex: 1;
    min-width: 0;
    margin: 0;
    font-size: 16px;
    font-weight: 600;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
</style>
