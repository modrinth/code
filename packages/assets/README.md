# `@modrinth/assets`

This package contains various assets used across the Modrinth platform, including icons, images, and branding materials.

Modrinth uses the [Lucide icon set](https://lucide.dev/) for its icons, which are automatically imported and exported in the `index.ts` file. This file is generated through the `pnpm run fix` command, which also ensures that all icons are consistent and correctly formatted.

The "Mr Rinth"/"Rinthbot" branding assets were created and given to Modrinth by [Devin (integrav)](https://github.com/intergrav) and are used across the platform. These assets are also included in this package.

## Adding New Assets

If you're adding a new icon from the [Lucide icon set](https://lucide.dev/), download the icon as an SVG file and place it in the `icons` directory. The icon should be named in kebab-case (e.g., `example-icon.svg`). Then run the `pnpm run fix` command to automatically generate the necessary imports and exports.

If you're adding anything else, you should manually add the import statement to `index.ts` and ensure it is exported correctly.
