<template>
  <div>
    <header class="columns">
      <nuxt-link to="/">
        <img class="logo" src="~/assets/images/logo.svg" alt="logo" />
      </nuxt-link>
      <div class="links">
        <nuxt-link to="/">Home</nuxt-link>
        <nuxt-link to="/mods">Mods</nuxt-link>
        <nuxt-link to="/modpacks">Packs</nuxt-link>
        <nuxt-link to="/about">About</nuxt-link>
        <a href="https://discord.gg/gFRbNQ2">Discord</a>
      </div>
    </header>
    <nuxt />
    <footer>
      <div class="logo-wrapper columns">
        <img class="logo" src="~/assets/images/logo.svg" alt="logo" />
        <p class="name">modrinth</p>
      </div>
      <p class="copyright">© Guavy LLC</p>
      <div class="column">
        <h4 class="pages">Pages</h4>
        <nuxt-link to="/">Home</nuxt-link>
        <nuxt-link to="/mods">Mods</nuxt-link>
        <nuxt-link to="/modpacks">Packs</nuxt-link>
        <nuxt-link to="/about">About</nuxt-link>
        <nuxt-link to="/guides">Guides</nuxt-link>
      </div>
      <div class="column">
        <h4 class="developers">Developers</h4>
        <nuxt-link to="/documentation">Documentation</nuxt-link>
        <a href="https://github.com/modrinth">Open Source</a>
        <a href="https://github.com/modrinth/knossos/issues">Issues</a>
      </div>
      <div class="column legal-wrapper">
        <h4 class="legal">Legal</h4>
        <nuxt-link to="/tos">TOS</nuxt-link>
        <nuxt-link to="/privacy">Privacy Policy</nuxt-link>
      </div>
    </footer>
  </div>
</template>

<script>
export default {}
</script>

<style lang="scss" scoped>
header {
  width: 100%;

  .logo {
    margin: 25px 50px;
    height: 100px;
  }

  .links {
    margin: auto 0;

    a {
      text-transform: uppercase;
      font-weight: bold;
      margin: 0 25px;

      &:hover,
      &:focus {
        background-color: var(--color-grey-1);
        color: var(--color-text);
      }

      &.nuxt-link-exact-active {
        border-bottom: 3px var(--color-brand) solid;
      }
    }
  }
}

footer {
  justify-content: center;
  margin-top: 250px;
  margin-bottom: 100px;
  display: grid;
  column-gap: 50px;
  grid-template-columns: 175px 120px 120px 175px;
  grid-template-rows: 20px 20px 20px 20px 20px 20px 20px 20px 20px 20px 20px 20px;

  .logo-wrapper {
    grid-column-start: 1;
    grid-row-start: 1;

    .logo {
      width: 30px;
    }
    .name {
      place-self: center;
      margin: 0;
      font-size: 20px;
      font-family: 'Montserrat Alternates', serif;
    }
  }
  .copyright {
    grid-column-start: 1;
    grid-row-start: 10;
    color: var(--color-grey-6);
  }
  h4 {
    font-weight: 600;
    margin: 0 0 20px;
    grid-row-start: 1;
  }
  .column {
    display: grid;
    row-gap: 5px;
    grid-template-rows: 40px 20px 20px 20px 20px 20px 20px 20px;
    a {
      &:hover,
      &:focus {
        color: var(--color-grey-5);
      }
    }
  }
}

@media screen and (max-width: 550px) {
  footer {
    column-gap: 10px;
    margin-left: 300px !important;
  }
  .legal-wrapper {
    display: none !important;
  }
}

@media screen and (max-width: 800px) {
  footer {
    margin-left: 200px;
  }
  .copyright,
  .logo-wrapper {
    display: none;
  }
}
</style>
