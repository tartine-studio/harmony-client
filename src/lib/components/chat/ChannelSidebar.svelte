<script lang="ts">
  import Icon from "$lib/components/ui/Icon.svelte";
  import Logo from "$lib/components/ui/Logo.svelte";
  import { t } from "$lib/i18n/index.svelte";
  import { chat } from "$lib/state/chat.svelte";
  import { session } from "$lib/state/session.svelte";
  import UserPanel from "./UserPanel.svelte";
</script>

<nav class="sidebar" aria-label={t("sidebar.label")}>
  <header>
    <Logo />
    <div class="titles">
      <span class="name">Harmony</span>
      <span class="host" title={session.info?.serverUrl}>{session.serverHost}</span>
    </div>
  </header>

  <div class="channels">
    <h2 class="section">{t("sidebar.text")}</h2>
    <ul>
      {#each chat.textChannels as channel (channel.id)}
        {@const active = channel.id === chat.activeChannelId}
        {@const unread = chat.unread.has(channel.id)}
        <li>
          <a
            class="row"
            class:active
            class:unread
            href="/app/channels/{channel.id}"
            aria-current={active ? "page" : undefined}
          >
            <Icon name="hash" />
            <span class="label">{channel.name}</span>
            {#if unread}<span class="dot" role="img" aria-label={t("sidebar.unread")}></span>{/if}
          </a>
        </li>
      {/each}
    </ul>

    {#if chat.voiceChannels.length > 0}
      <h2 class="section voice">{t("sidebar.voice")}</h2>
      <ul>
        {#each chat.voiceChannels as channel (channel.id)}
          <li>
            <button type="button" class="row" disabled title={t("sidebar.voiceSoon")}>
              <Icon name="volume" />
              <span class="label">{channel.name}</span>
              <span class="soon">{t("sidebar.soon")}</span>
            </button>
          </li>
        {/each}
      </ul>
    {/if}
  </div>

  <UserPanel />
</nav>

<style>
  .sidebar {
    width: var(--size-sidebar);
    flex: 0 0 auto;
    min-height: 0;
    display: flex;
    flex-direction: column;
    background: var(--color-bg-sunken);
    border-right: 1px solid var(--color-border);
  }

  header {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 16px 16px 14px;
    border-bottom: 1px solid var(--color-border);
  }

  .titles {
    display: flex;
    flex-direction: column;
    min-width: 0;
  }

  .name {
    font-weight: 700;
    font-size: 16px;
    letter-spacing: -0.01em;
  }

  .host {
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--color-text-subtle);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .channels {
    flex: 1 1 0;
    min-height: 0;
    overflow-y: auto;
    padding: 14px 10px;
  }

  .section {
    margin: 0;
    padding: 0 6px 6px 10px;
    font-size: 12px;
    font-weight: 600;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--color-text-subtle);
  }

  .section.voice {
    padding-top: 18px;
  }

  ul {
    list-style: none;
    margin: 0;
    padding: 0;
  }

  .row {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    padding: 7px 10px;
    margin: 1px 0;
    border: 0;
    border-radius: var(--radius-md);
    background: transparent;
    color: var(--color-text-muted);
    font-size: 15px;
    text-align: left;
    text-decoration: none;
  }

  .row :global(svg) {
    color: var(--color-text-subtle);
  }

  a.row:hover {
    background: var(--color-bg-hover);
    color: var(--color-text);
  }

  .row.active {
    background: var(--color-bg-active);
    color: var(--color-text-strong);
  }

  .row.unread .label {
    color: var(--color-text-strong);
    font-weight: 600;
  }

  .label {
    flex: 1;
    min-width: 0;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--color-text-strong);
  }

  button.row:disabled {
    opacity: 0.7;
  }

  .soon {
    padding: 1px 8px;
    border-radius: var(--radius-full);
    background: var(--color-surface);
    color: var(--color-text-subtle);
    font-size: 11px;
    font-weight: 600;
  }
</style>
