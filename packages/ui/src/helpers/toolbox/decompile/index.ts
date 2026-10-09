import type { FileContent, FileDetails, FileTypeInfo } from "#ui/helpers/toolbox/file";
import type {MessageTypes} from "#ui/helpers/toolbox/messenger";

export const ALL_DECOMPILER_TYPES = ['cfr', 'vineflower', 'procyon', 'krakatau'] as const;

export type DecompilerTypes = typeof ALL_DECOMPILER_TYPES[number];

export type DecompileInput = {
	targetFile: FileContent, resources?: FileDetails[], options?: DecompileOptions
};

export type DecompileOptions = {
	decompiler?: DecompilerTypes, options?: Record<string, any>
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
		main: {input: DecompileInput, output: DecompilerResult}
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
