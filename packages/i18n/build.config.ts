import { globSync } from 'glob'
import { defineBuildConfig } from 'unbuild'
import unimport from 'unimport/unplugin'
import macro from 'unplugin-macros/rollup'

export default defineBuildConfig({
  entries: globSync(['./src/index.ts', './src/utils/index.ts', './src/common-messages/**/*.ts'], {
    posix: true,
  }).map((input) => ({
    input,
  })),
  hooks: {
    'rollup:options'(_ctx, options) {
      if (Array.isArray(options.output)) {
        options.output.forEach((o) => (o.preserveModules = true))
      }
      options.plugins ??= []

      const customPlugins = [
        unimport.rollup({
          imports: [
            {
              from: './src/macros/define-message.ts',
              name: 'defineMessage',
              with: { type: 'macro' },
            },
          ],
        }),
        macro(),
      ]

      options.plugins = Array.isArray(options.plugins)
        ? [...customPlugins, ...options.plugins]
        : [...customPlugins, macro(), options.plugins]
    },
  },
  declaration: 'compatible',
  // Disabling cleaning of dist may be not the brightest idea, but it helps
  // avoiding a faulty state where files don't exist (HMR/live-reload concern).
  //
  // Hey, now it's your responsibility as a dev to clean the dist if you do
  // major refactors which involves addition or deletion of files:
  //
  // `pnpm turbo --filter @modrinth/i18n clean` :)
  clean: false,
})
