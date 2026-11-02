<script lang="ts">
  import type { HTMLButtonAttributes } from "svelte/elements";
  import Icon, { type IconName } from "./Icon.svelte";

  interface Props extends HTMLButtonAttributes {
    icon: IconName;
    label: string;
    size?: "sm" | "md";
    pressed?: boolean;
  }

  let { icon, label, size = "md", pressed, ...rest }: Props = $props();
</script>

<button
  type="button"
  class="icon-button {size}"
  aria-label={label}
  title={label}
  aria-pressed={pressed}
  {...rest}
>
  <Icon name={icon} size={size === "sm" ? 15 : 20} />
</button>

<style>
  .icon-button {
    display: flex;
    align-items: center;
    justify-content: center;
    flex: 0 0 auto;
    border: 0;
    border-radius: var(--radius-md);
    background: transparent;
    color: var(--color-text-subtle);
  }

  .icon-button.md {
    width: 36px;
    height: 36px;
  }

  .icon-button.sm {
    width: 30px;
    height: 30px;
    border-radius: var(--radius-sm);
  }

  .icon-button:hover {
    background: var(--color-bg-hover);
    color: var(--color-text);
  }

  .icon-button[aria-pressed="true"] {
    color: var(--color-text-strong);
  }
</style>
