export default {
  /*
   ** Nuxt rendering mode
   ** See https://nuxtjs.org/api/configuration-mode
   */
  mode: 'universal',
  /*
   ** Nuxt target
   ** See https://nuxtjs.org/api/configuration-target
   */
  target: 'server',
  /*
   ** Headers of the page
   ** See https://nuxtjs.org/api/configuration-head
   */
  head: {
    title: 'Modrinth',
    meta: [
      { charset: 'utf-8' },
      { name: 'viewport', content: 'width=device-width, initial-scale=1' },
      {
        hid: 'description',
        name: 'description',
        content:
          'Modrinth is a mod distribution platform. Modrinth is modern, easy to use, and built for modders. Modrinth currently supports Minecraft, including the forge and fabric mod loaders.',
      },
      {
        hid: 'og-title',
        property: 'og:title',
        content: 'Open source modding platform',
      },
      {
        hid: 'og-site-name',
        property: 'og:site_name',
        content: 'Modrinth',
      },
      {
        hid: 'og-image',
        property: 'og:image',
        content: 'https://modrinth.com/_nuxt/img/logo.e3136b7.svg',
      },
      {
        hid: 'og-image-url',
        property: 'og:image:url',
        content: 'https://modrinth.com/_nuxt/img/logo.e3136b7.svg',
      },
      { hid: 'og-image-width', property: 'og:image:width', content: 1280 },
      { hid: 'og-image-height', property: 'og:image:height', content: 720 },
      {
        hid: 'og-image-type',
        property: 'og:image:type',
        content: 'image/jpeg',
      },
      { hid: 'twitter-card', property: 'twitter:card' },
      {
        hid: 'twitter-site',
        property: 'twitter:site',
        content: 'modrinth.com',
      },
      {
        hid: 'twitter-creator',
        property: 'twitter:creator',
        content: 'modrinth',
      },
      {
        hid: 'twitter-img-src',
        property: 'twitter:image',
        content: 'https://modrinth.com/_nuxt/img/logo.e3136b7.svg',
      },
      {
        hid: 'twitter-title',
        property: 'twitter:title',
        content: 'Modrinth',
      },
    ],
    link: [
      { rel: 'icon', type: 'image/x-icon', href: '/favicon.ico' },
      {
        rel: 'stylesheet',
        href:
          'https://fonts.googleapis.com/css2?family=Inter:wght@400;700&display=swap',
      },
      {
        rel: 'stylesheet',
        href:
          'https://fonts.googleapis.com/css2?family=Montserrat+Alternates:wght@600&display=swap',
      },
    ],
    script: [
      {
        src: 'https://pagead2.googlesyndication.com/pagead/js/adsbygoogle.js',
        'data-ad-client': 'ca-pub-4800120742989028',
        async: true,
      },
    ],
  },

  vue: {
    config: {
      productionTip: false,
      devtools: true,
    },
  },

  router: {
    middleware: ['auth'],
  },
  /*
   ** Global CSS
   */
  css: ['~assets/styles/global.scss'],
  /*
   ** Plugins to load before mounting the App
   ** https://nuxtjs.org/guide/plugins
   */
  plugins: [],
  /*
   ** Auto import components
   ** See https://nuxtjs.org/api/configuration-components
   */
  components: true,
  /*
   ** Nuxt.js dev-modules
   */
  buildModules: [
    // Doc: https://github.com/nuxt-community/eslint-module
    '@nuxtjs/eslint-module',
  ],
  /*
   ** Nuxt.js modules
   */
  modules: [
    // Doc: https://axios.nuxtjs.org/usage
    '@nuxtjs/axios',
    '@nuxtjs/auth',
  ],
  auth: {
    strategies: {
      local: {
        endpoints: {
          user: {
            url: 'https://api.modrinth.com/api/v1/user',
            method: 'get',
            propertyName: false,
          },
        },
        tokenType: false,
      },
    },
  },
  /*
   ** Axios module configuration
   ** See https://axios.nuxtjs.org/options
   */
  axios: {},
  /*
   ** Build configuration
   ** See https://nuxtjs.org/api/configuration-build/
   */
  build: {},
  loading: {
    color: 'green',
    height: '5px',
  },
}
