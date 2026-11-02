<script lang="ts" module>
  // Drafts survive switching channels.
  const drafts = new Map<string, string>();
</script>

<script lang="ts">
  import { untrack } from "svelte";
  import { MAX_MESSAGE_CHARS } from "$lib/core/commands";
  import Icon from "$lib/components/ui/Icon.svelte";
  import { t } from "$lib/i18n/index.svelte";
  import { chat } from "$lib/state/chat.svelte";

  let { channelId, channelName }: { channelId: string; channelName: string } = $props();

  // The composer is re-created for each channel (`{#key}`), so reading the
  // initial channel is intended.
  let draft = $state(untrack(() => drafts.get(channelId) ?? ""));
  let input: HTMLTextAreaElement;

  const length = $derived(Array.from(draft.trim()).length);
  const tooLong = $derived(length > MAX_MESSAGE_CHARS);
  const canSend = $derived(length > 0 && !tooLong);

  $effect(() => {
    drafts.set(channelId, draft);
  });

  $effect(() => {
    input.focus();
  });

  function autosize() {
    input.style.height = "auto";
    input.style.height = `${Math.min(input.scrollHeight, window.innerHeight * 0.4)}px`;
  }

  function send() {
    if (!canSend) return;
    const content = draft.trim();
    draft = "";
    queueMicrotask(autosize);
    void chat.send(channelId, content);
  }

  function onKeydown(event: KeyboardEvent) {
    if (event.key === "Enter" && !event.shiftKey && !event.isComposing) {
      event.preventDefault();
      send();
    }
  }
</script>

<div class="composer">
  <div class="box" class:invalid={tooLong}>
    <label class="visually-hidden" for="composer">{t("composer.label")}</label>
    <textarea
      id="composer"
      bind:this={input}
      bind:value={draft}
      rows="1"
      placeholder={t("composer.placeholder", { name: channelName })}
      autocomplete="off"
      oninput={autosize}
      onkeydown={onKeydown}
    ></textarea>
    <button type="button" class="send" aria-label={t("composer.send")} disabled={!canSend} onclick={send}>
      <Icon name="send" strokeWidth={2.2} />
    </button>
  </div>
  {#if length > MAX_MESSAGE_CHARS - 200}
    <p class="counter" class:over={tooLong} aria-live="polite">
      {t("composer.counter", { count: length, max: MAX_MESSAGE_CHARS })}
    </p>
  {/if}
</div>

<style>
  .composer {
    flex: 0 0 auto;
    padding: 4px 20px 20px;
  }

  .box {
    display: flex;
    align-items: flex-end;
    gap: 8px;
    padding: 4px 6px 4px 4px;
    border-radius: var(--radius-xl);
    background: var(--color-bg-elevated);
    border: 1px solid var(--color-border-strong);
  }

  .box:focus-within {
    border-color: var(--color-text-subtle);
  }

  .box.invalid {
    border-color: var(--color-danger);
  }

  textarea {
    flex: 1;
    min-width: 0;
    max-height: 40vh;
    padding: 12px 14px;
    border: 0;
    outline: none;
    resize: none;
    background: transparent;
    color: var(--color-text);
    line-height: 1.45;
  }

  textarea::placeholder {
    color: var(--color-text-subtle);
  }

  .send {
    width: 40px;
    height: 40px;
    margin-bottom: 2px;
    display: flex;
    align-items: center;
    justify-content: center;
    flex: 0 0 auto;
    border: 0;
    border-radius: 9px;
    background: var(--color-accent);
    color: var(--color-accent-contrast);
  }

  .send:disabled {
    background: var(--color-control-off);
    color: var(--color-text-subtle);
  }

  .counter {
    margin: 6px 4px 0;
    font-family: var(--font-mono);
    font-size: 12px;
    text-align: right;
    color: var(--color-text-subtle);
  }

  .counter.over {
    color: var(--color-danger-text);
  }
</style>
