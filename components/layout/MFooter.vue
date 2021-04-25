<template>
  <footer :class="{ centered, hideBig, hideSmall }">
    <span>
      Modrinth is open source software. You may view the source code at
      <a target="_blank" href="https://github.com/modrinth">our GitHub page</a>.
    </span>
    <ul>
      <li>
        <a @click="changeTheme">
          <span v-if="$colorMode.value === 'light'">Switch to Dark Mode</span>
          <span v-else>Switch to Light Mode</span>
        </a>
      </li>
    </ul>
    <ul>
      <li>
        <nuxt-link to="/legal/terms">Terms</nuxt-link>
      </li>
      <li>
        <nuxt-link to="/legal/privacy">Privacy</nuxt-link>
      </li>
      <li>
        <nuxt-link to="/about">About</nuxt-link>
      </li>
    </ul>
    <ul>
      <li>
        <a target="_blank" href="https://blog.modrinth.com">Blog</a>
      </li>
      <li>
        <a target="_blank" href="https://discord.gg/gFRbNQ2">Discord</a>
      </li>
      <li>
        <a target="_blank" href="https://twitter.com/modrinth">Twitter</a>
      </li>
    </ul>
    <ul>
      <li>
        <nuxt-link to="/dashboard/privacy">Set privacy preferences</nuxt-link>
      </li>
    </ul>
    <span> © Guavy LLC </span><br />
    <span v-if="version !== 'unknown'">Version: {{ version }}</span>
  </footer>
</template>

<script>
export default {
  props: {
    version: {
      type: String,
      default: process.env.version || 'unknown',
    },
    centered: {
      type: Boolean,
      default: false,
    },
    hideSmall: {
      type: Boolean,
      default: false,
    },
    hideBig: {
      type: Boolean,
      default: false,
    },
  },
  methods: {
    changeTheme() {
      this.$colorMode.preference =
        this.$colorMode.value === 'dark' ? 'light' : 'dark'
    },
  },
}
</script>

<style lang="scss" scoped>
.centered {
  align-items: center;
}

footer {
  padding: 2rem 0 2rem 0;
  display: flex;
  flex-direction: column;

  ul {
    list-style: none;
    padding: 0;
    margin: 0;
    display: flex;
    flex-wrap: wrap;

    li {
      margin-bottom: var(--spacing-card-sm);
      &:not(:last-child)::after {
        content: '•';
        padding: 0;
        margin: 0 var(--spacing-card-sm);
      }
    }
  }

  & > *:not(:last-child) {
    margin-bottom: var(--spacing-card-sm);
  }
}

a {
  text-decoration: underline;
  cursor: pointer;
}

.hideSmall {
  @media screen and (max-width: 1024px) {
    display: none;
  }
}

.hideBig {
  @media screen and (min-width: 1024px) {
    display: none;
  }
}
</style>
