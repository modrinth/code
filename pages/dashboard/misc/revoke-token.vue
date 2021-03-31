<template>
  <div>
    <div class="section-header columns">
      <h3 class="column-grow-1">Revoke your Modrinth token</h3>
    </div>
    <section class="essentials pad-maker">
      <p>
        Revoking your Modrinth token can have unintended consequences. Please be
        wary, the following could break:
      </p>
      <ul>
        <li>Any application that uses your token to access the API.</li>
        <li>
          Gradle - if Minotaur is given a incorrect token, your gradle builds
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
      <p>Once you have completed the steps, press the continue button below.</p>
      <p>
        <strong>
          This will log you out of Modrinth, however when you log back in your
          token will be regenerated.
        </strong>
      </p>
      <button @click="logout">Continue</button>
    </section>
  </div>
</template>

<script>
export default {
  components: {},
  methods: {
    async logout() {
      this.$cookies.set('auth-token-reset', true)
      await this.$router.replace(
        'https://api.modrinth.com/api/v1/auth/init?url=https://modrinth.com/'
      )
    },
  },
}
</script>

<style lang="scss" scoped>
.pad-rem {
  margin-top: 0;
}

.pad-maker {
  margin-top: var(--spacing-card-md);
}

.save-btn-div {
  overflow: hidden;
  clear: both;
}

.save-btn {
  float: right;
}

a {
  color: var(--color-link);
  text-decoration: underline;
}

section {
  @extend %card;
  padding: var(--spacing-card-md) var(--spacing-card-lg);
}

label {
  display: flex;

  span {
    flex: 2;
    padding-right: var(--spacing-card-lg);
  }

  input {
    flex: 3;
    height: fit-content;
  }

  button {
    :hover {
      cursor: pointer;
    }

    height: fit-content;
    flex: 1;
  }
}
</style>
