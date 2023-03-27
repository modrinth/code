import { resolve } from 'path'
import svgLoader from 'vite-svg-loader'

export default {
  title: 'Omorphia',
  description: 'A components library used for Modrinth.',
  head: [['link', { rel: 'icon', type: 'image/x-icon', href: '/favicon.ico' }]],
  themeConfig: {
    sidebar: [
      {
        items: [
          { text: 'Introduction', link: '/' },
          { text: 'Setup', link: '/setup' },
        ],
      },
      {
        text: 'Components',
        items: [
          { text: 'Avatar', link: '/components/avatar' },
          { text: 'Badge', link: '/components/badge' },
          { text: 'Button', link: '/components/button' },
          { text: 'Card', link: '/components/card' },
          { text: 'Checkbox', link: '/components/checkbox' },
          { text: 'Chips', link: '/components/chips' },
          { text: 'File Input', link: '/components/file-input' },
          { text: 'Drop Area', link: '/components/drop-area' },
          { text: 'Icons', link: '/components/icons' },
          { text: 'Pagination', link: '/components/pagination' },
          { text: 'Modal', link: '/components/modal' },
          { text: 'Dropdown Select', link: '/components/dropdown-select' },
          { text: 'Project Card', link: '/components/project-card' },
          { text: 'Environment Indicator', link: '/components/environment-indicator' },
          { text: 'Categories', link: '/components/categories' },
          { text: 'Animated Logo', link: '/components/animated-logo' },
          { text: 'Text Logo', link: '/components/text-logo' },
          { text: 'Slider', link: '/components/slider' },
          { text: 'Text Inputs', link: '/components/text-inputs' },
          { text: 'Search Filter', link: '/components/search-filter' },
          { text: 'Toggle', link: '/components/toggle' },
          { text: 'Promotion', link: '/components/promotion' },
        ],
      },
    ],
    footer: {
      message:
        'Released under the <a href="https://github.com/modrinth/omoprhia/blob/main/LICENSE">GPLv3 License</a>.',
      copyright: 'Copyright © 2023-present <a href="https://modrinth.com">Rinth, Inc.</a>',
    },
  },
  vite: {
    plugins: [
      svgLoader({
        svgoConfig: {
          plugins: [
            {
              name: 'preset-default',
              params: {
                overrides: {
                  removeViewBox: false,
                },
              },
            },
          ],
        },
      }),
    ],
    resolve: {
      alias: {
        '@': resolve(__dirname, '../../lib'),
        omorphia: resolve(__dirname, '../../lib'),
      },
      dedupe: ['vue'],
    },
  },
}
