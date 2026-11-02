<script lang="ts">
  import type { Snippet } from "svelte";
  import type { HTMLButtonAttributes } from "svelte/elements";

  interface Props extends HTMLButtonAttributes {
    variant?: "primary" | "secondary";
    busy?: boolean;
    children: Snippet;
  }

  let { variant = "primary", busy = false, disabled, children, ...rest }: Props = $props();
</script>

<button class="button {variant}" disabled={disabled || busy} aria-busy={busy || undefined} {...rest}>
  {#if busy}<span class="spinner" aria-hidden="true"></span>{/if}
  {@render children()}
</button>

<style>
  .button {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 10px;
    height: 50px;
    padding: 0 22px;
    border: 0;
    border-radius: var(--radius-lg);
    font-weight: 600;
  }

  .primary {
    background: var(--color-accent);
    color: var(--color-accent-contrast);
  }

  .primary:hover:not(:disabled) {
    background: var(--color-accent-hover);
  }

  .secondary {
    background: var(--color-bg-elevated);
    color: var(--color-text);
    border: 1px solid var(--color-border-strong);
  }

  .button:disabled {
    opacity: 0.75;
  }

  .spinner {
    width: 16px;
    height: 16px;
    border-radius: 50%;
    border: 2px solid currentColor;
    border-right-color: transparent;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  @media (prefers-reduced-motion: reduce) {
    .spinner {
      animation-duration: 2.4s;
    }
  }
</style>
