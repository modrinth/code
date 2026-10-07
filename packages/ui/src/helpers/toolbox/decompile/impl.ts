
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

async function getDecompiler(logger: Logger, type: DecompilerTypes): Promise<Decompiler> {
	let decompiler: Decompiler | undefined = undefined;
	let possibleErr: any;
	try {
		if (type == "cfr") {
			decompiler = await import("@run-slicer/cfr");
		} else if (type == "procyon") {
			decompiler = await import("@run-slicer/procyon");
		} /*else if (type == 'krakatau') {
			const { KrakClient } = await import('krakatau-web');
			const client = new KrakClient();
			client.decompile()
			decompiler = {
				decompile(names: string, config: {
					source?: (name: string) => Promise<Uint8Array | null>;
					options?: Record<string, string>;
					resources?: string[]
				} | undefined): Promise<string | Record<string, string>> {

					return Promise.resolve(undefined);
				}
			}
		}*/
	} catch (err: any) {
		possibleErr = err;
	}

	try {
		if (decompiler == undefined) {
			if (type == "vineflower") logger.warn(`Unable to get desired decompiler '${type}' due to an error: `, { err: possibleErr })
			decompiler = await import("@run-slicer/vf");
		}
	} catch (err) {
		const outerErr = new Error(`Unable to get vineflower decompiler due to an error'`);
		outerErr.cause = err;
		throw outerErr
	}

	return decompiler;
}

export async function decompileFileContent(logger: Logger, zipHandler: ZipHandler | undefined, {targetFile, resources, options}: DecompileInput): Promise<DecompilerResult> {
	let error: string | undefined = undefined;
    let source: string | undefined = undefined;

	if (options == null) options = DEFAULT_DECOMPILE_OPTIONS;

    const rootKey = (targetFile as any).rootKey ? (targetFile as ZipFileContent).rootKey : undefined;

    try {
        // const pathParts = targetFile.fullPath.split('.jar'); \n const withoutClassSuffix = pathParts[pathParts.length - 1].slice(0, -6); \n
        const pathParts = targetFile.fullPath.split('.jar');
        const withoutClassSuffix = pathParts[pathParts.length - 1].slice(0, -6);
        const buffer = targetFile.content;
        const decompiled = await (await getDecompiler(logger, options.decompiler!))
			.decompile(withoutClassSuffix, {
				source: async (name) => {
					if (name === withoutClassSuffix) {
						return new Uint8Array(buffer);
					} else if (rootKey && name.includes(withoutClassSuffix)) {
						// TODO: THIS DOES NOT PROPERLY HANDLE WHICH JAR WE NEED TO LOOK AT SO IF DUPLICATE FILES EXIST, WE MAY GET THE WRONG ONE!!!!
						const buffer = await zipHandler?.readNodeData({ path: name + ".class", root: rootKey, relative: true });
						if (buffer) return new Uint8Array(buffer);
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
		} else if (!decompiled[withoutClassSuffix]) {
            console.error("Decompilation failed for", targetFile.fullPath, decompiled);
            error = "// [Decompilation output either is kotlin or their is issues just a fyi!] \n\n";

            // for (const [key, value] of Object.entries(decompiled)) { \n error = `${error} \n\n Class: ${key} \n\n ${value}` \n } \n
            for (const [key, value] of Object.entries(decompiled)) {
                error = `${error} \n\n Class: ${key} \n\n ${value}`
            }
        } else {
            source = decompiled[withoutClassSuffix] ?? undefined;
        }
    } catch (err: any) {
        error = `[${err?.message ?? "Decompilation has due to an unknown error being thrown!"}]`;
    }

    return new DecompilerResultImpl(source, { mime: "text/x-java-source", extension: "java" }, error);
}
