<script lang="ts">
  import { t } from "$lib/i18n/index.svelte";
  import { toasts } from "$lib/state/toasts.svelte";
  import Icon from "./Icon.svelte";
</script>

<div class="toasts" role="status" aria-live="polite">
  {#each toasts.items as toast (toast.id)}
    <div class="toast">
      <Icon name="alert" size={18} />
      <span>{toast.text}</span>
      <button type="button" aria-label={t("toast.dismiss")} onclick={() => toasts.dismiss(toast.id)}>
        <Icon name="close" size={15} />
      </button>
    </div>
  {/each}
</div>

<style>
  .toasts {
    position: fixed;
    right: 16px;
    bottom: 16px;
    z-index: 10;
    display: flex;
    flex-direction: column;
    gap: 8px;
    max-width: min(420px, calc(100vw - 32px));
  }

  .toast {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 12px 12px 12px 14px;
    border-radius: var(--radius-xl);
    background: var(--color-surface);
    border: 1px solid var(--color-border-strong);
    box-shadow: 0 12px 32px var(--color-overlay-shadow);
    color: var(--color-danger-text);
    font-size: 14px;
  }

  .toast span {
    flex: 1;
    color: var(--color-text);
  }

  button {
    display: flex;
    padding: 4px;
    border: 0;
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--color-text-subtle);
  }
</style>
