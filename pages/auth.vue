<template>
  <div>
    <NuxtPage class="auth-container universal-card" :route="route" />
  </div>
</template>

<script setup>
const route = useRoute()
</script>

<style lang="scss">
.auth-container {
  width: 25rem;
  padding: var(--gap-xl);
  background-color: var(--color-raised-bg);
  border-radius: var(--radius-lg);
  margin: 2rem auto;

  h1 {
    margin: 0;
    color: var(--color-contrast);
  }

  h2 {
    font-size: 1.25rem;
    font-weight: 500;
    margin: 0;
    color: var(--color-contrast);
  }

  p {
    margin: 0;
  }

  .btn {
    font-weight: 700;
    min-height: 2.5rem;
    text-decoration: none;
  }

  input {
    width: 100%;
    border: none;
    outline: none;
  }

  .btn.right-icon svg {
    margin-left: var(--gap-sm);
  }

  .btn.left-icon svg {
    margin-right: var(--gap-sm);
  }

  .input-group {
    display: flex;
    gap: var(--gap-md);
    flex-wrap: wrap;
  }

  button.checkbox {
    appearance: none !important;
    border: none;
  }

  .continue-btn {
    margin-left: auto;
    margin-right: auto;
    margin-block-start: 0;
  }

  .continue-btn svg {
    margin: 0 0 0 0.5rem;
  }

  // login styles
  .third-party {
    display: grid;
    gap: var(--gap-md);
    grid-template-columns: repeat(2, 1fr);
    width: 100%;
  }
  .third-party .btn {
    width: 100%;
    justify-content: center;
    vertical-align: middle;
  }
  .third-party .btn svg {
    margin-right: var(--gap-sm);
    width: 1.25rem;
    height: 1.25rem;
  }

  .discord-btn {
    color: #ffffff;
    background-color: #5865f2;
  }

  .apple-btn {
    color: var(--color-accent-contrast);
    background-color: var(--color-contrast);
  }

  .google-btn {
    color: #ffffff;
    background-color: #4285f4;
  }

  .gitlab-btn {
    color: #ffffff;
    background-color: #fc6d26;
  }

  .github-btn {
    color: #ffffff;
    background-color: #8740f1;
  }

  .microsoft-btn {
    color: var(--color-accent-contrast);
    background-color: var(--color-contrast);
  }

  .text-divider {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 100%;
  }

  .text-divider div {
    height: 2px;
    width: 100%;
    max-width: 5rem;
    opacity: 40%;
    border-radius: var(--radius-max);
    background-color: var(--color-base);
  }

  .text-divider span {
    margin-inline: var(--gap-sm);
  }

  @media screen and (max-width: 25.5rem) {
    width: auto;
    margin: 1rem;

    .third-party .btn {
      grid-column: 1 / 3;
    }
  }
}

.auth-page-container {
  display: flex;
  flex-direction: column;
  gap: var(--gap-md);
}
</style>
