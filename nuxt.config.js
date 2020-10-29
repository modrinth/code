export default {
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
    htmlAttrs: {
      lang: 'en',
    },
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

      { hid: 'publisher', name: 'publisher', content: 'Guavy LLC' },
      {
        hid: 'apple-mobile-web-app-title',
        name: 'apple-mobile-web-app-title',
        content: 'Modrinth',
      },
      { hid: 'theme-color', name: 'theme-color', content: '#4d9227' },
      { hid: 'color-scheme', name: 'color-scheme', content: 'light dark' },

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
        content: 'https://cdn.modrinth.com/modrinth.png',
      },
      { hid: 'twitter:card', name: 'twitter:card', content: 'summary' },
      { hid: 'twitter:site', name: 'twitter:site', content: '@modrinth' },
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
        src: 'https://analytics.modrinth.com/umami.js',
        'data-website-id': 'c37613de-245d-4767-90e7-ba7980a4f1a2',
        async: true,
        defer: true,
      },
      {
        src: 'https://media.ethicalads.io/media/client/ethicalads.min.js',
        async: true,
      },
    ],
  },

  vue: {
    config: {
      productionTip: false,
      devtools: false,
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
  ],
  /*
   ** Nuxt.js modules
   */
  modules: [
    // Doc: https://axios.nuxtjs.org/usage
    '@nuxtjs/dayjs',
    '@nuxtjs/axios',
    '@nuxtjs/auth',
    '@nuxtjs/robots',
    '@nuxtjs/sitemap',
  ],
  robots: {
    Sitemap: 'https://modrinth.com/sitemap.xml',
  },
  sitemap: {
    exclude: ['/dashboard/**', '/dashboard', '/mod/create'],
  },
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
  dayjs: {
    locales: ['en'],
    defaultLocale: 'en',
    plugins: ['relativeTime'],
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
