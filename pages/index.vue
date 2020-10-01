<template>
  <div>
    <header class="columns">
      <img class="logo" src="~/assets/images/logo.svg" alt="logo" />
      <div class="links">
        <nuxt-link to="/" no-prefetch>Home</nuxt-link>
        <nuxt-link to="/mods" no-prefetch>Mods</nuxt-link>
        <nuxt-link to="/modpacks" no-prefetch>Packs</nuxt-link>
        <nuxt-link to="/about" no-prefetch>About</nuxt-link>
      </div>
    </header>
    <div class="main-hero columns">
      <div class="main-left">
        <h1 class="typewriter">{{ currentText }}</h1>
        <h1>modding platform</h1>
      </div>
      <div class="main-right columns">
        <img class="char" src="~/assets/images/logo.svg" alt="logo" />
      </div>
    </div>
    <div class="slanted-hero"></div>
  </div>
</template>

<script>
export default {
  layout: 'none',
  data() {
    return {
      currentText: 'Open source',
      texts: ['Open source', 'Easy to use', 'Developer focused', 'API Based'],
    }
  },
  beforeMount() {
    document.documentElement.setAttribute('data-theme', 'light')

    this.startNext(0)
  },
  methods: {
    startNext(i) {
      const startIndex = i % this.texts.length

      this.typeWriter(this.texts[startIndex], 0, () => {
        this.startNext(startIndex + 1)
      })
    },
    typeWriter(text, i, callback) {
      if (!text || i >= text.length) {
        setTimeout(callback, 1000 + Math.random() * 500)
        return
      }

      this.currentText = text.substring(0, i + 1)
      setTimeout(
        () => this.typeWriter(text, i + 1, callback),
        150 + Math.random() * 50
      )
    },
  },
}
</script>

<style lang="scss">
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

      &.nuxt-link-active {
        border-bottom: 3px var(--color-brand) solid;
      }
    }
  }
}

.main-hero {
  height: 600px;

  .main-left {
    width: 50%;
    padding-top: 75px;
    padding-left: 100px;

    .typewriter {
      display: inline-block;
      color: var(--color-brand);
      border-right: 0.15em solid var(--color-brand);
      animation: caret 1s steps(1) infinite;

      @keyframes caret {
        50% {
          border-color: transparent;
        }
      }
    }

    h1 {
      margin: 0;
      font-size: 4em;
    }
  }
  .main-right {
    width: 50%;
    padding-left: 20%;

    .char {
      image-rendering: pixelated;
      height: 400px;
    }
  }
}

.slanted-hero {
  background: var(--color-brand);
  height: 500px;
  position: relative;
  z-index: 1;

  &:before,
  &:after {
    background: inherit;
    content: '';
    display: block;
    height: 50%;
    left: 0;
    position: absolute;
    right: 0;
    z-index: -1;
    -webkit-backface-visibility: hidden;
  }

  &:before {
    top: 0;
    transform: skewY(5deg);
    transform-origin: 100% 0;
  }

  &:after {
    bottom: 0;
    transform: skewY(-5deg);
    transform-origin: 100%;
  }
}
</style>
