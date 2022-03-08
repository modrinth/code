# theseus_gui

## Developing

First, make sure [NodeJS](https://nodejs.org/en/download/package-manager/) & [pnpm](https://pnpm.io/installation#nodejs-is-preinstalled) are installed, then run:

```zsh
pnpm install # Install dependencies
pnpm dev # Start dev server
```

> If after quitting the dev process, you find that the SvelteKit process is still running (or preventing you from restarting the dev command), run `pnpm kill:dev`

## Building

```bash
pnpm build
```