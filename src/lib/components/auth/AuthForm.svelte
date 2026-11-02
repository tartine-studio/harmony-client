<script lang="ts">
  import { goto } from "$app/navigation";
  import Button from "$lib/components/ui/Button.svelte";
  import Icon from "$lib/components/ui/Icon.svelte";
  import TextField from "$lib/components/ui/TextField.svelte";
  import { validateCredentials, type Field, type FieldError } from "$lib/chat/validation";
  import { toAppError } from "$lib/core/commands";
  import { errorText, t } from "$lib/i18n/index.svelte";
  import { lastServerUrl, session } from "$lib/state/session.svelte";

  type Mode = "login" | "register";

  let mode = $state<Mode>("login");
  let serverUrl = $state(lastServerUrl());
  let username = $state("");
  let email = $state("");
  let password = $state("");
  let busy = $state(false);
  let submitted = $state(false);
  let formError = $state<string | null>(session.expired ? t("auth.expired") : null);

  const register = $derived(mode === "register");
  const fieldErrors = $derived(
    submitted ? validateCredentials({ serverUrl, username, email, password }, mode) : {},
  );

  function fieldError(field: Field): string | undefined {
    const key: FieldError | undefined = fieldErrors[field];
    return key ? t(key) : undefined;
  }

  function switchTo(next: Mode) {
    mode = next;
    submitted = false;
    formError = null;
  }

  async function submit(event: SubmitEvent) {
    event.preventDefault();
    submitted = true;
    formError = null;
    if (Object.keys(validateCredentials({ serverUrl, username, email, password }, mode)).length > 0) return;
    busy = true;
    try {
      if (register) await session.register(serverUrl, username, email, password);
      else await session.login(serverUrl, email, password);
      await goto("/app", { replaceState: true });
    } catch (e) {
      formError = errorText(toAppError(e).code);
    } finally {
      busy = false;
    }
  }
</script>

<div class="panel">
  <div class="tabs" role="tablist" aria-label={t("auth.tabs")}>
    <button type="button" role="tab" aria-selected={!register} onclick={() => switchTo("login")}>
      {t("auth.tabLogin")}
    </button>
    <button type="button" role="tab" aria-selected={register} onclick={() => switchTo("register")}>
      {t("auth.tabRegister")}
    </button>
  </div>

  <h2>{register ? t("auth.registerHeading") : t("auth.loginHeading")}</h2>
  <p class="sub">{register ? t("auth.registerSub") : t("auth.loginSub")}</p>

  <form onsubmit={submit} novalidate>
    <TextField
      id="server"
      type="url"
      mono
      label={t("auth.server")}
      placeholder={t("auth.serverPlaceholder")}
      autocomplete="url"
      spellcheck="false"
      autocapitalize="off"
      bind:value={serverUrl}
      error={fieldError("serverUrl")}
    />
    {#if register}
      <TextField
        id="username"
        label={t("auth.username")}
        autocomplete="username"
        maxlength={32}
        bind:value={username}
        error={fieldError("username")}
      />
    {/if}
    <TextField
      id="email"
      type="email"
      label={t("auth.email")}
      autocomplete="email"
      autocapitalize="off"
      spellcheck="false"
      bind:value={email}
      error={fieldError("email")}
    />
    <TextField
      id="password"
      type="password"
      label={t("auth.password")}
      autocomplete={register ? "new-password" : "current-password"}
      maxlength={128}
      bind:value={password}
      error={fieldError("password")}
    />

    {#if formError}
      <p class="form-error" role="alert"><Icon name="alert" size={16} />{formError}</p>
    {/if}

    <Button type="submit" {busy}>
      {#if busy}
        {register ? t("auth.submittingRegister") : t("auth.submittingLogin")}
      {:else}
        {register ? t("auth.submitRegister") : t("auth.submitLogin")}
      {/if}
    </Button>
  </form>
</div>

<style>
  .panel {
    width: 100%;
    max-width: 420px;
  }

  .tabs {
    display: flex;
    gap: 4px;
    padding: 4px;
    margin-bottom: 32px;
    border-radius: var(--radius-xl);
    background: var(--color-bg);
    border: 1px solid var(--color-bg-elevated);
  }

  .tabs button {
    flex: 1;
    height: 40px;
    border: 0;
    border-radius: 9px;
    font-weight: 600;
    background: transparent;
    color: var(--color-text-muted);
  }

  .tabs button[aria-selected="true"] {
    background: var(--color-bg-active);
    color: var(--color-text-strong);
  }

  h2 {
    margin: 0 0 6px;
    font-size: 28px;
    font-weight: 700;
    letter-spacing: -0.02em;
  }

  .sub {
    margin: 0 0 28px;
    color: var(--color-text-muted);
  }

  form {
    display: flex;
    flex-direction: column;
    gap: 18px;
  }

  form :global(.button) {
    margin-top: 8px;
  }

  .form-error {
    display: flex;
    gap: 8px;
    align-items: flex-start;
    margin: 0;
    padding: 12px 14px;
    border-radius: var(--radius-lg);
    background: var(--color-danger-soft);
    color: var(--color-danger-text);
    font-size: 14px;
  }

  .form-error :global(svg) {
    margin-top: 2px;
  }
</style>
