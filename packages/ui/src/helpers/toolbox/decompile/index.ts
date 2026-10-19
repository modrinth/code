import type { FileContent, FileDetails, FileTypeInfo } from "#ui/helpers/toolbox/file";
import type {MessageTypes} from "#ui/helpers/toolbox/messenger";

export const ALL_DECOMPILER_TYPES = ['cfr', 'vineflower', 'procyon', 'krakatau'] as const;

export type DecompilerTypes = typeof ALL_DECOMPILER_TYPES[number];

/**
 * The decompilers the toolbox can load. Krakatau isn't bundled yet, so asking for it falls back to
 * Vineflower; it isn't offered until it is.
 */
export const AVAILABLE_DECOMPILER_TYPES = ['vineflower', 'cfr', 'procyon'] as const satisfies readonly DecompilerTypes[];

export interface DecompileInput {
	targetFile: FileContent,
	transferBuffer?: boolean,
	resources?: FileDetails[],
	options?: DecompileOptions
}

export interface DecompileOptions {
	decompiler?: DecompilerTypes,
	options?: Record<string, any>
}

export const DEFAULT_DECOMPILE_OPTIONS = {
	decompiler: 'vineflower',
	options: {
		'remove-synthetic': '0',
		'remove-bridge': '0',
		'kt-enable': '0' // TODO: MAYBE ADJUST STUFF IDK
	}
} as const;

export type DecompilerMessages = MessageTypes<
	{
		main: { serverbound: DecompileInput, clientbound: DecompilerResult }
	}
>

export type DecompileHandler = {
	decompile(input: DecompileInput): Promise<DecompilerResult>;
};

export interface DecompilerResult {
	wasDecompiled: boolean,
	error?: string,
	source?: string,
	fileType?: FileTypeInfo
	asString: string,
}

export interface Decompiler {
	// TODO: SOME DECOMPILERS DO NOT TAKE IN AN ARRAY AT ALL
	decompile(names: string/* | string[]*/, config?: {
		source?: (name: string) => Promise<Uint8Array | null>,
		options?: Record<string, string>,
		resources?: string[]
	}): Promise<string | Record<string, string>>
}
