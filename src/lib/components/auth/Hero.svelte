<script lang="ts">
  import Icon from "$lib/components/ui/Icon.svelte";
  import Logo from "$lib/components/ui/Logo.svelte";
  import { t } from "$lib/i18n/index.svelte";

  const features = [
    { icon: "hash", title: "hero.textTitle", body: "hero.textBody" },
    { icon: "volume", title: "hero.voiceTitle", body: "hero.voiceBody" },
    { icon: "servers", title: "hero.selfHostTitle", body: "hero.selfHostBody" },
  ] as const;
</script>

<section class="hero">
  <div class="brand">
    <Logo size={36} />
    <span>Harmony</span>
  </div>

  <div class="pitch">
    <h1>{t("hero.title1")}<br />{t("hero.title2")}</h1>
    <p class="lead">{t("hero.lead")}</p>
    <ul>
      {#each features as f (f.title)}
        <li>
          <span class="badge"><Icon name={f.icon} /></span>
          <span><strong>{t(f.title)}</strong><span class="muted">{t(f.body)}</span></span>
        </li>
      {/each}
    </ul>
  </div>

  <p class="footer">
    {t("hero.footer")}
    <a href="https://github.com/tartine-studio/harmony-server" target="_blank" rel="noreferrer">{t("hero.source")}</a>
  </p>
</section>

<style>
  .hero {
    flex: 1 1 520px;
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    gap: 48px;
    padding: 56px clamp(24px, 6vw, 88px);
    background: var(--color-bg-sunken);
    border-right: 1px solid var(--color-border);
  }

  .brand {
    display: flex;
    align-items: center;
    gap: 12px;
    font-size: 20px;
    font-weight: 700;
    letter-spacing: -0.01em;
  }

  .pitch {
    max-width: 520px;
  }

  h1 {
    margin: 0 0 20px;
    font-size: clamp(36px, 4.4vw, 56px);
    line-height: 1.05;
    font-weight: 700;
    letter-spacing: -0.03em;
  }

  .lead {
    margin: 0 0 40px;
    font-size: 18px;
    color: var(--color-text-muted);
  }

  ul {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 18px;
  }

  li {
    display: flex;
    gap: 14px;
    align-items: flex-start;
  }

  .badge {
    width: 36px;
    height: 36px;
    flex: 0 0 auto;
    border-radius: var(--radius-lg);
    background: var(--color-surface);
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--color-accent);
  }

  strong {
    display: block;
    font-weight: 600;
  }

  .muted {
    color: var(--color-text-muted);
  }

  .footer {
    margin: 0;
    font-size: 13px;
    color: var(--color-text-subtle);
  }

  @media (max-width: 900px) {
    .hero {
      display: none;
    }
  }
</style>
