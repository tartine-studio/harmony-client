<script lang="ts">
  import type { User } from "$lib/bindings/User";
  import Avatar from "$lib/components/ui/Avatar.svelte";
  import { t } from "$lib/i18n/index.svelte";
  import { chat } from "$lib/state/chat.svelte";
  import { session } from "$lib/state/session.svelte";

  interface Row {
    user: User;
    status: "online" | "idle" | "offline" | "invisible";
    sub: string;
  }

  const groups = $derived.by(() => {
    const meId = session.me?.id;
    const online: Row[] = [];
    const offline: Row[] = [];
    const users = Object.values(chat.users).sort((a, b) =>
      a.username.localeCompare(b.username, undefined, { sensitivity: "base" }),
    );
    for (const user of users) {
      if (user.id === meId) {
        const invisible = chat.presence.invisible;
        const status = invisible ? "invisible" : chat.status(user.id) === "idle" ? "idle" : "online";
        online.unshift({ user, status, sub: invisible ? t("members.youInvisible") : t("members.you") });
        continue;
      }
      const status = chat.status(user.id);
      if (status === "offline") offline.push({ user, status, sub: "" });
      else online.push({ user, status, sub: status === "idle" ? t("status.idle") : "" });
    }
    return { online, offline };
  });
</script>

<aside class="members" aria-label={t("members.label")}>
  <h2>{t("members.online", { count: groups.online.length })}</h2>
  <ul>
    {#each groups.online as row (row.user.id)}
      <li>
        <Avatar
          userId={row.user.id}
          name={row.user.username}
          size={32}
          status={row.status}
          dim={row.status === "invisible"}
        />
        <div class="text">
          <div class="name">{row.user.username}</div>
          {#if row.sub}<div class="sub">{row.sub}</div>{/if}
        </div>
      </li>
    {/each}
  </ul>

  {#if groups.offline.length > 0}
    <h2 class="offline-title">{t("members.offline", { count: groups.offline.length })}</h2>
    <ul>
      {#each groups.offline as row (row.user.id)}
        <li class="offline">
          <Avatar userId={row.user.id} name={row.user.username} size={32} status="offline" dim />
          <div class="name">{row.user.username}</div>
        </li>
      {/each}
    </ul>
  {/if}
</aside>

<style>
  .members {
    width: var(--size-members);
    flex: 0 0 auto;
    min-height: 0;
    overflow-y: auto;
    padding: 20px 12px;
    background: var(--color-bg-sunken);
    border-left: 1px solid var(--color-border);
  }

  h2 {
    margin: 0;
    padding: 0 8px 8px;
    font-size: 12px;
    font-weight: 600;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--color-text-subtle);
  }

  .offline-title {
    padding-top: 18px;
  }

  ul {
    list-style: none;
    margin: 0;
    padding: 0;
  }

  li {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 7px 8px;
    border-radius: var(--radius-md);
  }

  .text {
    min-width: 0;
  }

  .name {
    font-size: 14px;
    font-weight: 500;
    color: var(--color-text);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .offline .name {
    font-weight: 400;
    color: var(--color-text-subtle);
  }

  .sub {
    font-size: 12px;
    color: var(--color-text-muted);
  }
</style>
