import starlight from '@astrojs/starlight'
import { defineConfig } from 'astro/config'
import starlightOpenAPI, { openAPISidebarGroups } from 'starlight-openapi'

// https://astro.build/config
export default defineConfig({
  site: 'https://docs.modrinth.com',
  integrations: [
    starlight({
      title: 'Modrinth Documentation',
      favicon: '/favicon.ico',
      editLink: {
        baseUrl: 'https://github.com/modrinth/code/edit/main/apps/docs/',
      },
      social: {
        github: 'https://github.com/modrinth/code',
        discord: 'https://discord.modrinth.com',
        'x.com': 'https://x.com/modrinth',
        mastodon: 'https://floss.social/@modrinth',
        threads: 'https://threads.net/@modrinth',
      },
      logo: {
        light: './src/assets/light-logo.svg',
        dark: './src/assets/dark-logo.svg',
        replacesTitle: true,
      },
      customCss: [
        '@modrinth/assets/styles/variables.scss',
        '@modrinth/assets/styles/inter.scss',
        './src/styles/modrinth.css',
      ],
      plugins: [
        // Generate the OpenAPI documentation pages.
        starlightOpenAPI([
          {
            base: 'api',
            label: 'Modrinth API',
            schema: './public/openapi.yaml',
          },
        ]),
      ],
      sidebar: [
        {
          label: 'Contributing to Modrinth',
          autogenerate: { directory: 'contributing' },
        },
        {
          label: 'Guides',
          autogenerate: { directory: 'guide' },
        },
        // Add the generated sidebar group to the sidebar.
        ...openAPISidebarGroups,
      ],
    }),
  ],
})
