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

      { hid: 'author', name: 'author', content: 'Modrinth' },
      { hid: 'publisher', name: 'publisher', content: 'Guavy LLC' },
      {
        hid: 'apple-mobile-web-app-title',
        name: 'apple-mobile-web-app-title',
        content: 'Modrinth',
      },
      { hid: 'theme-color', name: 'theme-color', content: '#4d9227' },

      { hid: 'og:site_name', name: 'og:site_name', content: 'Modrinth' },
      {
        hid: 'og:description',
        name: 'og:description',
        content: 'An open source modding platform',
      },
      { hid: 'og:type', name: 'og:type', content: 'website' },
      { hid: 'og:url', name: 'og:url', content: 'https://www.modrinth.com' },
      {
        hid: 'og:image',
        name: 'og:image',
        content: 'https://cdn.modrinth.com/file/modrinth/modrinth.png',
      },

      // Twitter
      {
        hid: 'twitter:card',
        name: 'twitter:card',
        content: 'summary',
      },
      { hid: 'twitter:site', name: 'twitter:site', content: '@modrinth' },
      { hid: 'twitter:creator', name: 'twitter:creator', content: '@modrinth' },
      {
        hid: 'twitter:title',
        name: 'twitter:title',
        content: 'Modrinth',
      },
      {
        hid: 'twitter:description',
        name: 'twitter:description',
        content: 'An open source modding platform',
      },
      {
        hid: 'twitter:image',
        name: 'twitter:image',
        content: 'https://cdn.modrinth.com/file/modrinth/modrinth.png',
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
    '@nuxtjs/svg',
    '@nuxtjs/color-mode',
    '@nuxtjs/moment',
  ],
  /*
   ** Nuxt.js modules
   */
  modules: [
    // Doc: https://axios.nuxtjs.org/usage
    '@nuxtjs/axios',
    '@nuxtjs/auth',
    '@nuxtjs/markdownit',
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
  axios: {
    headers: {
      common: {
        Accept: 'application/json',
      },
    },
  },
  moment: {
    defaultTimezone: 'America/Los_Angeles',
    timezone: true,
    startYear: 2010,
    endYear: 2030,
  },
  markdownit: {
    injected: true,
  },
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
