
import {
	type DecompileInput, type Decompiler,
	type DecompilerResult,
	type DecompilerTypes, DEFAULT_DECOMPILE_OPTIONS
} from "#ui/helpers/toolbox/decompile/index.ts";
import type { FileTypeInfo } from "#ui/helpers/toolbox/file";
import type { Logger } from "#ui/helpers/toolbox/misc/logging.ts";
import type { ZipFileContent, ZipHandler } from "#ui/helpers/toolbox/zip";


export class DecompilerResultImpl implements DecompilerResult {
    wasDecompiled: boolean;
    error?: string | undefined;
    source?: string | undefined;
    fileType?: FileTypeInfo | undefined;
    asString: string;

    constructor(source?: string, fileType?: FileTypeInfo, error?: string,) {
        this.error = error;
        this.source = source;
        this.fileType = fileType;
        this.wasDecompiled = source != null;
        this.asString = this.source ?? this.error ?? "UNKNOWN DUE TO AN ODD ERROR?";
    }
}

type DecompileFunction = Decompiler["decompile"];

/**
 * Loads a run-slicer decompiler's WASM build. Their own entry points take any defined `process`
 * (which Nuxt shims, even in workers) to mean Node and then fail to locate the module, so it's
 * fetched here and handed over as bytes instead. Its URL is imported only when a decompiler is
 * first loaded (on the client), so the server build never deals with the module.
 */
async function loadWasmDecompiler(
	runtime: () => Promise<RunSlicerWasmRuntime>,
	wasmUrlModule: () => Promise<{ default: string }>,
	options?: { noAutoImports?: boolean, takesArray?: boolean },
): Promise<DecompileFunction> {
	const [{ load }, bytes] = await Promise.all([
		runtime(),
		wasmUrlModule().then(({ default: wasmUrl }) => fetch(wasmUrl)).then((response) => {
			if (!response.ok) throw new Error(`Unable to fetch '${response.url}' (${response.status})`);
			return response.arrayBuffer();
		}),
	]);
	const { exports } = await load(bytes, { noAutoImports: options?.noAutoImports });
	return options?.takesArray
		? (name, config) => exports.decompile([name], config)
		: (name, config) => exports.decompile(name, config);
}

/** Falls back to the decompiler's JavaScript build when its WASM one can't be used. */
async function withJsFallback(logger: Logger, type: DecompilerTypes, wasm: () => Promise<DecompileFunction>, js: () => Promise<DecompileFunction>) {
	try {
		return await wasm();
	} catch (err) {
		logger.warn(`Unable to load the WASM build of '${type}', falling back to its JavaScript build`, { err });
		return js();
	}
}

const DECOMPILER_LOADERS: Partial<Record<DecompilerTypes, (logger: Logger) => Promise<DecompileFunction>>> = {
	cfr: (logger) => withJsFallback(
		logger,
		"cfr",
		() => loadWasmDecompiler(
			() => import("@run-slicer/cfr/cfr.wasm-runtime.js"),
			() => import("@run-slicer/cfr/cfr.wasm?url"),
			{ takesArray: true },
		),
		async () => {
			const { decompile } = await import("@run-slicer/cfr/cfr.runtime.js");
			return (name, config) => decompile([name], config);
		},
	),
	vineflower: (logger) => withJsFallback(
		logger,
		"vineflower",
		() => loadWasmDecompiler(
			() => import("@run-slicer/vf/vf.wasm-runtime.js"),
			() => import("@run-slicer/vf/vf.wasm?url"),
			{ takesArray: true },
		),
		async () => {
			const { decompile } = await import("@run-slicer/vf/vf.runtime.js");
			return (name, config) => decompile([name], config);
		},
	),
	procyon: () => loadWasmDecompiler(
		() => import("@run-slicer/procyon/procyon.wasm-runtime.js"),
		() => import("@run-slicer/procyon/procyon.wasm?url"),
		{ noAutoImports: true },
	),
};

const loadedDecompilers = new Map<DecompilerTypes, Promise<DecompileFunction>>();

function loadDecompiler(logger: Logger, type: DecompilerTypes) {
	let loaded = loadedDecompilers.get(type);
	if (loaded == null) {
		const loader = DECOMPILER_LOADERS[type];
		if (loader == null) return Promise.reject(new Error(`The '${type}' decompiler is not available`));
		loaded = loader(logger);
		loaded.catch(() => loadedDecompilers.delete(type));
		loadedDecompilers.set(type, loaded);
	}
	return loaded;
}

/** The requested decompiler, or Vineflower when that one can't be loaded. */
async function getDecompiler(logger: Logger, type: DecompilerTypes): Promise<Decompiler> {
	try {
		return { decompile: await loadDecompiler(logger, type) };
	} catch (err) {
		if (type === "vineflower") throw new Error(`Unable to get vineflower decompiler due to an error`, { cause: err });
		logger.warn(`Unable to get desired decompiler '${type}' due to an error, using vineflower instead`, { err });
	}

	try {
		return { decompile: await loadDecompiler(logger, "vineflower") };
	} catch (err) {
		throw new Error(`Unable to get vineflower decompiler due to an error`, { cause: err });
	}
}

export async function decompileFileContent(logger: Logger, zipHandler: ZipHandler | undefined, {targetFile, resources, options}: DecompileInput): Promise<DecompilerResult> {
	let error: string | undefined = undefined;
    let source: string | undefined = undefined;

	if (options == null) options = DEFAULT_DECOMPILE_OPTIONS;

    const rootKey = (targetFile as Partial<ZipFileContent>).root ?? undefined;

    try {
        // const pathParts = targetFile.fullPath.split('.jar'); \n const withoutClassSuffix = pathParts[pathParts.length - 1].slice(0, -6); \n
        const pathParts = targetFile.fullPath.split('.jar');
        const withoutClassSuffix = pathParts[pathParts.length - 1].slice(0, -6);
		const unnormalizedPath = withoutClassSuffix.slice(1);

        const buffer = targetFile.content;
        const decompiled = await (await getDecompiler(logger, options.decompiler!))
			.decompile(unnormalizedPath, {
				source: async (name) => {
					if (name === unnormalizedPath) {
						return new Uint8Array(buffer);
					} else if (rootKey != null && zipHandler != null && name.includes(unnormalizedPath)) {
						// TODO: THIS DOES NOT PROPERLY HANDLE WHICH JAR WE NEED TO LOOK AT SO IF DUPLICATE FILES EXIST, WE MAY GET THE WRONG ONE!!!!
						try {
							const buffer = await zipHandler.findNodeData({ path: "/" + name + ".class", root: rootKey, relative: true });
							if (buffer) return new Uint8Array(buffer);
						} catch (err) {
							logger.warn(`Unable to read '${name}' for decompiling '${targetFile.fullPath}'`, { err });
						}
					}

					return null;
				},
				resources: [
					...(resources?.map(details => details.fullPath.slice(0, -6)) ?? []),
				],
				options: options.options
			});

		if (typeof decompiled === 'string') {
			source = decompiled ?? undefined;
		} else if (!decompiled[unnormalizedPath]) {
            console.error("Decompilation failed for: ", targetFile.fullPath, unnormalizedPath, decompiled);
            error = "// [Decompilation output either is kotlin or their is issues just a fyi!] \n\n";

            // for (const [key, value] of Object.entries(decompiled)) { \n error = `${error} \n\n Class: ${key} \n\n ${value}` \n } \n
            for (const [key, value] of Object.entries(decompiled)) {
                error = `${error} \n\n Class: ${key} \n\n ${value}`
            }
        } else {
            source = decompiled[unnormalizedPath] ?? undefined;
        }
    } catch (err: any) {
        error = `[${err?.message ?? "Decompilation has due to an unknown error being thrown!"}]`;
    }

    return new DecompilerResultImpl(source, { mime: "text/x-java-source", extension: "java" }, error);
}
