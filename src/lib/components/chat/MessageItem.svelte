<script lang="ts">
  import { tintIndex } from "$lib/chat/avatar";
  import type { ChatMessage } from "$lib/chat/messages";
  import { isEdited } from "$lib/chat/messages";
  import { MAX_MESSAGE_CHARS, toAppError } from "$lib/core/commands";
  import Avatar from "$lib/components/ui/Avatar.svelte";
  import Icon from "$lib/components/ui/Icon.svelte";
  import IconButton from "$lib/components/ui/IconButton.svelte";
  import { errorText, formatFull, formatTime, t } from "$lib/i18n/index.svelte";
  import { chat } from "$lib/state/chat.svelte";
  import { session } from "$lib/state/session.svelte";
  import { toasts } from "$lib/state/toasts.svelte";

  let { message, showHeader }: { message: ChatMessage; showHeader: boolean } = $props();

  const author = $derived(chat.user(message.userId));
  const name = $derived(author?.username ?? (message.userId ? "…" : t("message.deletedUser")));
  const mine = $derived(!!message.userId && message.userId === session.me?.id);
  const created = $derived(new Date(message.createdAt));
  const pending = $derived(message.local?.status === "pending");
  const failed = $derived(message.local?.status === "failed" ? message.local : null);

  let editing = $state(false);
  let draft = $state("");
  let saving = $state(false);
  let editor = $state<HTMLTextAreaElement>();

  function startEdit() {
    draft = message.content;
    editing = true;
    queueMicrotask(() => {
      editor?.focus();
      editor?.setSelectionRange(draft.length, draft.length);
      autosize();
    });
  }

  function autosize() {
    if (!editor) return;
    editor.style.height = "auto";
    editor.style.height = `${editor.scrollHeight}px`;
  }

  async function save() {
    const content = draft.trim();
    if (!content || content.length > MAX_MESSAGE_CHARS) return;
    if (content === message.content) {
      editing = false;
      return;
    }
    saving = true;
    try {
      await chat.edit(message.id, content);
      editing = false;
    } catch (e) {
      toasts.show(errorText(toAppError(e).code));
    } finally {
      saving = false;
    }
  }

  function onEditKey(event: KeyboardEvent) {
    if (event.key === "Escape") {
      event.preventDefault();
      editing = false;
    } else if (event.key === "Enter" && !event.shiftKey && !event.isComposing) {
      event.preventDefault();
      void save();
    }
  }
</script>

<article class="message" class:with-header={showHeader} class:pending class:failed={!!failed}>
  {#if showHeader}
    <Avatar userId={message.userId} {name} size={40} />
  {:else}
    <time class="gutter-time" datetime={message.createdAt} title={formatFull(created)}>{formatTime(created)}</time>
  {/if}

  <div class="body">
    {#if showHeader}
      <div class="meta">
        <span class="author" style:color={message.userId ? `var(--avatar-${tintIndex(message.userId)}-fg)` : undefined}>
          {name}
        </span>
        <time datetime={message.createdAt} title={formatFull(created)}>{formatTime(created)}</time>
      </div>
    {/if}

    {#if editing}
      <div class="editor">
        <label class="visually-hidden" for="edit-{message.id}">{t("message.editLabel")}</label>
        <textarea
          id="edit-{message.id}"
          bind:this={editor}
          bind:value={draft}
          rows="1"
          maxlength={MAX_MESSAGE_CHARS}
          disabled={saving}
          oninput={autosize}
          onkeydown={onEditKey}
        ></textarea>
        <p class="hint">{t("message.editHint")}</p>
      </div>
    {:else}
      <p class="content">
        {message.content}{#if isEdited(message) && !message.local}<span class="edited">{t("message.edited")}</span>{/if}
      </p>
    {/if}

    {#if pending}
      <p class="state">{t("message.sending")}</p>
    {:else if failed}
      <p class="state error">
        <Icon name="alert" size={14} />
        {t("message.failed")}
        {errorText(failed.error.code)}
        <button type="button" onclick={() => chat.retry(message.channelId, message.id)}>{t("message.retry")}</button>
        <button type="button" onclick={() => chat.discard(message.channelId, message.id)}>{t("message.discard")}</button>
      </p>
    {/if}
  </div>

  {#if mine && !message.local && !editing}
    <div class="actions">
      <IconButton icon="edit" size="sm" label={t("message.edit")} onclick={startEdit} />
      <IconButton icon="trash" size="sm" label={t("message.delete")} onclick={() => chat.remove(message.id)} />
    </div>
  {/if}
</article>

<style>
  .message {
    position: relative;
    display: flex;
    gap: 14px;
    padding: 2px 20px;
  }

  .message.with-header {
    padding-top: 14px;
  }

  .message:hover,
  .message:focus-within {
    background: var(--color-bg-hover);
  }

  .gutter-time {
    width: 40px;
    flex: 0 0 40px;
    padding-top: 4px;
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--color-text-subtle);
    text-align: center;
    visibility: hidden;
  }

  .message:hover .gutter-time {
    visibility: visible;
  }

  .body {
    flex: 1;
    min-width: 0;
  }

  .meta {
    display: flex;
    align-items: baseline;
    gap: 10px;
  }

  .author {
    font-weight: 600;
    font-size: 15px;
    color: var(--color-text);
  }

  .meta time {
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--color-text-subtle);
  }

  .content {
    margin: 2px 0 0;
    color: var(--color-text-body);
    white-space: pre-wrap;
    overflow-wrap: anywhere;
  }

  .edited {
    margin-left: 6px;
    font-size: 12px;
    color: var(--color-text-subtle);
  }

  .pending .content {
    opacity: 0.6;
  }

  .state {
    margin: 2px 0 0;
    font-size: 12px;
    color: var(--color-text-subtle);
  }

  .state.error {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 6px;
    color: var(--color-danger-text);
  }

  .state button {
    padding: 0;
    border: 0;
    background: none;
    color: var(--color-accent);
    font-size: 12px;
    font-weight: 600;
    text-decoration: underline;
  }

  .actions {
    position: absolute;
    top: 0;
    right: 16px;
    transform: translateY(-40%);
    display: none;
    gap: 2px;
    padding: 2px;
    border-radius: var(--radius-md);
    background: var(--color-surface);
    border: 1px solid var(--color-border-strong);
  }

  .message:hover .actions,
  .message:focus-within .actions {
    display: flex;
  }

  .editor textarea {
    width: 100%;
    margin-top: 4px;
    padding: 10px 12px;
    resize: none;
    border-radius: var(--radius-lg);
    border: 1px solid var(--color-border-strong);
    background: var(--color-bg-elevated);
    color: var(--color-text);
    line-height: 1.45;
  }

  .editor textarea:focus-visible {
    outline: none;
    border-color: var(--color-accent);
  }

  .hint {
    margin: 4px 0 4px;
    font-size: 12px;
    color: var(--color-text-subtle);
  }
</style>
