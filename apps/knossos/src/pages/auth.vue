<template>
  <NuxtPage class="auth-container universal-card" />
</template>

<style>
.auth-container {
  width: 26rem;
  max-width: calc(100% - 2rem);
  margin: 1rem auto;
  display: flex;
  flex-direction: column;
  gap: 2rem;
}

.auth-container h1 {
  font-size: var(--font-size-xl);
  margin: 0 0 -1rem 0;
  color: var(--color-contrast);
}

.auth-container p {
  margin: 0;
}

.auth-container .btn {
  font-weight: 700;
  min-height: 2.5rem;
  text-decoration: none;
}

.centered-btn {
  margin-inline: auto;
}

.btn.continue-btn svg {
  margin: 0 0 0 0.5rem;
}

.third-party {
  display: grid;
  gap: var(--gap-md);
  grid-template-columns: repeat(2, 1fr);
  width: 100%;
}

.third-party .btn {
  width: 100%;
  vertical-align: middle;
}

.third-party .btn svg {
  margin-right: var(--gap-sm);
  width: 1.25rem;
  height: 1.25rem;
}

@media screen and (max-width: 25.5rem) {
  .third-party .btn {
    grid-column: 1 / 3;
  }
}

.turnstile {
  display: flex;
  justify-content: center;
  overflow: hidden;
  border-radius: var(--radius-md);
  border: 2px solid var(--color-button-bg);
  height: 66px;

  iframe {
    margin: -1px;
    min-width: calc(100% + 2px);
  }
}

.auth-form {
  display: flex;
  flex-direction: column;
  gap: var(--gap-md);
}

.auth-form .auth-form__input {
  width: 100%;
  flex-basis: auto;
}

.auth-form__additional-options {
  align-items: center;
  display: flex;
  justify-content: center;
  gap: var(--gap-md);
  flex-wrap: wrap;
}
</style>
