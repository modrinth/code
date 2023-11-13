import { localeDefinitions } from '@modrinth/omorphia-dev/locales/index.js'
import { createPlugin } from '@vintl/vintl/plugin'
import { plugin as Omorphia } from 'omorphia'
import DefaultTheme from 'vitepress/theme'
import { createVNode } from 'vue'
import DemoContainer from './DemoContainer.vue'
import LanguageSwitcher from './LanguageSwitcher.vue'

import './compat.scss'
import './style.scss'

/** @type {import('vitepress').Theme} */
export default {
  ...DefaultTheme,
  enhanceApp(ctx) {
    ctx.app.use(Omorphia)
    ctx.app.component('DemoContainer', DemoContainer)
    ctx.app.use(
      createPlugin({
        controllerOpts: {
          locales: Object.keys(localeDefinitions).map((tag) => ({ tag })),
          listen: {
            async localeload(event) {
              const locale = event.locale.tag
              if (!Object.hasOwn(localeDefinitions, locale)) {
                throw new Error(`Unknown locale: ${locale}`)
              }

              try {
                const { messages } = await localeDefinitions[locale].importFunction()
                event.addMessages(messages)
              } catch (err) {
                console.error(`Failed to load locale: ${locale}`, err)
              }
            },
          },
          defaultMessageOrder: ['locale', 'descriptor'],
        },
        globalMixin: false,
      })
    )
  },
  Layout() {
    return createVNode(DefaultTheme.Layout, null, {
      'sidebar-nav-before'() {
        return createVNode(LanguageSwitcher)
      },
    })
  },
}
