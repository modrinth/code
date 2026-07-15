import { build } from 'esbuild'
import dts from 'unplugin-dts/esbuild'

await build({
	entryPoints: ['src/index.ts'],
	outdir: 'dist',
	entryNames: '[name]',
	bundle: true,
	format: 'esm',
	platform: 'neutral',
	target: 'es2020',
	minify: true,
	sourcemap: true,
	legalComments: 'none',
	external: ['ofetch', 'mitt', '@tauri-apps/plugin-http'],
	plugins: [
		dts({
			bundleTypes: true,
			outDirs: 'dist',
			processor: 'ts',
			tsconfigPath: './tsconfig.build.json',
		}),
	],
})
