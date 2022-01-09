<template>
  <div>
    <section class="card essentials pad-maker">
      <h3>Revoke your Modrinth token</h3>
      <p>
        Revoking your Modrinth token can have unintended consequences. Please be
        aware that the following could break:
      </p>
      <ul>
        <li>Any application that uses your token to access the API.</li>
        <li>
          Gradle - if Minotaur is given a incorrect token, your Gradle builds
          could fail.
        </li>
        <li>
          GitHub - if you use a GitHub action that uses the Modrinth API, it
          will cause errors.
        </li>
      </ul>
      <p>If you are willing to continue, complete the following steps:</p>
      <ol>
        <li>
          <a
            href="https://github.com/settings/connections/applications/3acffb2e808d16d4b226"
            target="_blank"
          >
            Head to the Modrinth Application page on GitHub.
          </a>
          Make sure to be logged into the GitHub account you used for Modrinth!
        </li>
        <li>
          Press the big red "Revoke Access" button next to the "Permissions"
          header.
        </li>
      </ol>
      <p>
        Once you have completed those steps, press the continue button below.
      </p>
      <p>
        <strong>
          This will log you out of Modrinth, however, when you log back in, your
          token will be regenerated.
        </strong>
      </p>
      <button class="iconified-button brand-button-colors" @click="logout">
        <CheckIcon />
        Continue
      </button>
    </section>
  </div>
</template>

<script>
import CheckIcon from '~/assets/images/utils/right-arrow.svg?inline'

export default {
  components: {
    CheckIcon,
  },
  head: {
    title: 'Revoke Token - Modrinth',
  },
  methods: {
    async logout() {
      this.$cookies.set('auth-token-reset', true)
      await this.$router.replace(
        `auth/init?url=${process.env.domain}${this.$route.fullPath}`
      )
    },
  },
}
</script>

<style lang="scss" scoped>
section {
  a {
    color: var(--color-link);
    text-decoration: underline;
  }

  button {
    padding: 0.5rem 0.75rem;
    margin-left: auto;
  }
}
</style>
