<script lang="ts">
  import type { HTMLInputAttributes } from "svelte/elements";

  interface Props extends HTMLInputAttributes {
    id: string;
    label: string;
    value: string;
    error?: string;
    mono?: boolean;
  }

  let { id, label, value = $bindable(), error, mono = false, ...rest }: Props = $props();
</script>

<div class="field">
  <label for={id}>{label}</label>
  <input
    {id}
    class:mono
    class:invalid={!!error}
    bind:value
    aria-invalid={error ? "true" : undefined}
    aria-describedby={error ? `${id}-error` : undefined}
    {...rest}
  />
  {#if error}
    <p id="{id}-error" class="error">{error}</p>
  {/if}
</div>

<style>
  .field {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  label {
    font-size: 13px;
    font-weight: 600;
    color: var(--color-text-secondary);
  }

  input {
    height: 46px;
    padding: 0 14px;
    border-radius: var(--radius-lg);
    border: 1px solid var(--color-border-strong);
    background: var(--color-bg);
    color: var(--color-text);
  }

  input.mono {
    font-family: var(--font-mono);
    font-size: 14px;
  }

  input:focus-visible {
    outline: none;
    border-color: var(--color-accent);
  }

  input.invalid {
    border-color: var(--color-danger);
  }

  .error {
    margin: 0;
    font-size: 13px;
    color: var(--color-danger-text);
  }
</style>
