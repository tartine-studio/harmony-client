<script lang="ts">
  let { size = 30 }: { size?: number } = $props();
</script>

<svg width={size} height={size} viewBox="0 0 32 32" fill="none" aria-hidden="true">
  <circle cx="16" cy="16" r="15" stroke="currentColor" stroke-width="2" />
  <path d="M9 19c2.5-6 4.5-6 7 0s4.5 6 7 0" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" />
</svg>

<style>
  svg {
    flex: 0 0 auto;
    color: var(--color-accent);
  }
</style>
