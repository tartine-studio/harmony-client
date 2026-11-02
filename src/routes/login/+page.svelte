<script lang="ts">
  import AuthForm from "$lib/components/auth/AuthForm.svelte";
  import Hero from "$lib/components/auth/Hero.svelte";
</script>

<div class="login">
  <Hero />
  <section class="form">
    <AuthForm />
  </section>
</div>

<style>
  .login {
    min-height: 100vh;
    display: flex;
    flex-wrap: wrap;
    background: var(--color-bg-outer);
  }

  .form {
    flex: 1 1 480px;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 56px 24px;
  }
</style>
