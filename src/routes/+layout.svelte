<script lang="ts">
  import "@fontsource-variable/instrument-sans";
  import "@fontsource/jetbrains-mono/400.css";
  import "@fontsource/jetbrains-mono/500.css";
  import "$lib/styles/global.css";
  import Toasts from "$lib/components/ui/Toasts.svelte";
  import { getLocale } from "$lib/i18n/index.svelte";

  let { children } = $props();

  $effect(() => {
    document.documentElement.lang = getLocale();
  });
</script>

{@render children()}
<Toasts />
