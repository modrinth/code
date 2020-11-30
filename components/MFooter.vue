<template>
  <footer :class="{ centered }">
    <span>
      Modrinth is open source software. You may view the source code at
      <a target="_blank" href="https://github.com/modrinth">our GitHub page</a>.
    </span>
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
    <span> © Guavy LLC </span>
  </footer>
</template>

<script>
export default {
  props: {
    centered: {
      type: Boolean,
      default: false,
    },
  },
}
</script>

<style lang="scss" scoped>
.centered {
  align-items: center;
}

footer {
  padding: 2rem 0 5rem 0;
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
}
</style>
