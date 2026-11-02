<script lang="ts">
  import { goto } from "$app/navigation";
  import Button from "$lib/components/ui/Button.svelte";
  import Logo from "$lib/components/ui/Logo.svelte";
  import { errorText, t } from "$lib/i18n/index.svelte";
  import { session } from "$lib/state/session.svelte";

  async function boot() {
    await session.restore();
    if (session.status === "authenticated") await goto("/app", { replaceState: true });
    else if (session.status === "anonymous") await goto("/login", { replaceState: true });
  }

  async function logout() {
    await session.logout();
    await goto("/login", { replaceState: true });
  }

  $effect(() => {
    void boot();
  });
</script>

<main class="boot">
  <Logo size={48} />
  {#if session.status === "error"}
    <p class="title">{t("boot.failed")}</p>
    <p class="detail">{errorText(session.error?.code)}</p>
    <div class="actions">
      <Button onclick={boot}>{t("boot.retry")}</Button>
      <Button variant="secondary" onclick={logout}>{t("boot.logout")}</Button>
    </div>
  {:else}
    <p class="detail" aria-live="polite">{t("boot.loading")}</p>
  {/if}
</main>

<style>
  .boot {
    height: 100%;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 12px;
    padding: 24px;
    text-align: center;
    background: var(--color-bg-outer);
  }

  .title {
    margin: 12px 0 0;
    font-size: 18px;
    font-weight: 600;
  }

  .detail {
    margin: 0;
    color: var(--color-text-muted);
  }

  .actions {
    display: flex;
    gap: 12px;
    margin-top: 12px;
  }
</style>
