/**
 * The WASM runtimes of the run-slicer decompilers, which ship without types. `load` takes the
 * module's URL or its bytes.
 */
interface RunSlicerWasmRuntime {
	load(
		module: string | BufferSource,
		options?: { noAutoImports?: boolean },
	): Promise<{
		exports: {
			decompile: (
				names: string | string[],
				config?: object,
			) => Promise<string | Record<string, string>>
		}
	}>
}

declare module '@run-slicer/cfr/cfr.wasm-runtime.js' {
	export const load: RunSlicerWasmRuntime['load']
}

declare module '@run-slicer/vf/vf.wasm-runtime.js' {
	export const load: RunSlicerWasmRuntime['load']
}

declare module '@run-slicer/procyon/procyon.wasm-runtime.js' {
	export const load: RunSlicerWasmRuntime['load']
}

declare module '@run-slicer/cfr/cfr.runtime.js' {
	export const decompile: (names: string[], config?: object) => Promise<Record<string, string>>
}

declare module '@run-slicer/vf/vf.runtime.js' {
	export const decompile: (names: string[], config?: object) => Promise<Record<string, string>>
}
