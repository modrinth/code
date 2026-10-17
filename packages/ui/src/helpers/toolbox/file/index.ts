import { fileTypeFromBuffer } from "file-type";
import yaml from "js-yaml"
import mime from "mime";
import * as toml from 'smol-toml'

export interface FileDetails {
    fullPath: string,
    name: string,
    url?: string
}

export interface URLFileDetails extends FileDetails  {
    url: string
}

export interface FileContent extends FileDetails {
    content: ArrayBuffer;
}

export interface HashedFileContent extends FileContent {
    sha512: string,
}

export interface FileContentPromise extends FileDetails {
    content(): Promise<ArrayBuffer>;
}

export async function resolveContent<T extends FileContentPromise>(content: T): Promise<FileContent & Omit<T, keyof FileContentPromise>> {
	return {
		...content,
		fullPath: content.fullPath,
		name: content.name,
		content: await content.content(),
		url: content.url
	}
}

export function fileFrom<T extends ArrayBuffer | (() => Promise<ArrayBuffer>), P extends {fullPath?: string, name: string, content: T, url?: string}>(params: Partial<P> & {fullPath?: string, name: string, content: T, url?: string}): T extends Promise<ArrayBuffer> ? FileContentPromise & P : FileContent & P {
    return {
        ...params,
        fullPath: params.fullPath ?? params.name,
        name: params.name,
        content: typeof params.content === "function" ? params.content : params.content
    } as any;
}


export function prettyifyJson(data: string, indentSize?: number) {
    return JSON.stringify(JSON.parse(data), null, indentSize ?? 2);
}

export interface FileTypeInfo {
    mime: string;
    extension?: string;
    method?: 'content-analysis' | 'extension-lookup';
}

export type AsyncMaybe<T, R, D = undefined> = T extends D ? R : Promise<R>

export function getFileTypeInfo<T extends ArrayBuffer | Blob | undefined = undefined>(path: string, data?: T): AsyncMaybe<T, FileTypeInfo | null> {
	const promiseLike = data ? getFileTypeFromData(data) : undefined;

	return ((promiseLike != null)
		? promiseLike.then((result) => result ?? getFileTypeFromExt(path))
		: getFileTypeFromExt(path)
	) as AsyncMaybe<T, FileTypeInfo | null>
}

function getFileTypeFromExt(path: string): FileTypeInfo | null {
	const typeFromPath = mime.getType(path);

	if (typeFromPath != null) {
		return {
			mime: typeFromPath,
			extension: path.split('.').pop()?.split(/\#|\?/)[0],
			method: 'extension-lookup'
		};
	}

	return null;
}

async function getFileTypeFromData(data: ArrayBuffer | Blob): Promise<FileTypeInfo | null> {
	const buffer = (data as Partial<Blob>).arrayBuffer
		? await (data as Blob).arrayBuffer()
		: data as ArrayBuffer;

	if (buffer != null) {
		const typeFromContent = await fileTypeFromBuffer(buffer.slice(0, 4100));

		if (typeFromContent != null) {
			return {
				mime: typeFromContent.mime,
				extension: typeFromContent.ext,
				method: 'content-analysis'
			};
		}
	}
	return null;
}

export async function hashString(str: string, algorithm: AlgorithmIdentifier = "SHA-512") {
    return hashBuffer(new TextEncoder().encode(str).buffer, algorithm);
}

export async function hashBlob(blob: Blob, algorithm: AlgorithmIdentifier = "SHA-512") {
    return hashBuffer(await blob.arrayBuffer(), algorithm);
}


export async function hashArrays(bytes: Uint8Array[], errorNameGetter?: () => string): Promise<string | null> {
    try {
        // Concatenate all byte arrays into a single Buffer/Uint8Array
        const totalLength = bytes.reduce((acc, curr) => acc + curr.length, 0);
        const combinedBytes = new Uint8Array(totalLength);

        let offset = 0;
        for (const buffer of bytes) {
            combinedBytes.set(buffer, offset);
            offset += bytes.length;
        }

        // Digest via Web Crypto API (returns ArrayBuffer)
        return await hashBuffer(combinedBytes.buffer, "SHA-512");
    } catch (e) {
        console.warn(`Unable to hash the following '${errorNameGetter?.() ?? "UNKNOWN"}' due the following exception: `, e);
        return null;
    }
}

export async function hashBuffer(data: ArrayBuffer, algorithm: AlgorithmIdentifier = "SHA-512") {
    const hashBuffer = await crypto.subtle.digest(algorithm, data);

    return Array.from(new Uint8Array(hashBuffer))
        .map(b => b.toString(16).padStart(2, '0'))
        .join('');
}

const MIME_TO_LANGUAGE: Record<string, string> = {
    // JavaScript & TypeScript
    'text/javascript': 'javascript',
    'application/javascript': 'javascript',
    'text/typescript': 'typescript',
    'application/typescript': 'typescript',
    'text/jsx': 'javascript',
    'text/tsx': 'typescript',

    // Web Formats
    'text/html': 'xml',
    'text/css': 'css',
    'application/json': 'json',
    'text/xml': 'xml',
    'application/xml': 'xml',

    // Scripting & Languages
    'text/x-python': 'python',
    'text/x-script.python': 'python',
    'text/x-sh': 'bash',
    'application/x-sh': 'bash',
    'text/x-c': 'c',
    'text/x-c++': 'cpp',
    'text/x-java-source': 'java',
    'text/x-rust': 'rust',
    'text/x-go': 'go',
    'text/x-kotlin': 'kotlin',
    'text/markdown': 'markdown',
    'text/x-yaml': 'yaml',
    'application/x-yaml': 'yaml',

    // Shell & Batch
    'application/x-bat': 'dos',
    'application/bat': 'dos',
    'text/x-bat': 'dos',
    'text/x-shellscript': 'bash',

    // Configs & Markup
    'application/toml': 'toml',
    'text/x-ini': 'ini',
    'text/x-properties': 'properties',
};

const EXTENSION_OVERRIDES: Record<string, string> = {
    // Minecraft Modding Specifics
    'mcfunction': 'mcfunction', // Uses custom grammar if registered, otherwise falls to fallback
    'mcmeta': 'json',
    'bbmodel': 'json',         // Blockbench 3D model files are JSON inside
    'accesswidener': 'properties',
    'accesstransformer': 'properties',
    'lang': 'properties',      // Legacy translation files
    'nbt': 'plaintext',        // Binary format

    // Build & Config Scripts
    'gradle': 'gradle',
    'kts': 'kotlin',
    'bat': 'dos',
    'cmd': 'dos',
    'sh': 'bash',
    'toml': 'toml',
    'cfg': 'ini',
    'conf': 'ini',
    'properties': 'properties',
    'env': 'ini',
    'log': 'plaintext',

    // Special files without extensions (e.g. "gradlew", "Dockerfile")
    'gradlew': 'bash',
    'dockerfile': 'dockerfile',
    'gitignore': 'plaintext'
};

export function getLanguageFromFile(info: FileTypeInfo): string | undefined {
    return MIME_TO_LANGUAGE[info.mime] ?? (info.extension ? EXTENSION_OVERRIDES[info.extension] : undefined);
}

const javaManifestParser =  {
    parse(text: string) {
        const manifest = {} as Record<string, string>;
        let currentKey: string | null = null;

        for (const line of text.replace(/\r\n/g, '\n').split('\n')) {
            if (line.startsWith(' ') && currentKey) {
                // Continuation line (Java Manifests wrap at 72 chars)
                manifest[currentKey] += line.substring(1);
            } else if (line.includes(':')) {
                const [key, ...valueParts] = line.split(':');
                currentKey = key.trim();
                manifest[currentKey] = valueParts.join(':').trim();
            }
        }
        return manifest;
    }
}

export function parseInfoFile(fileName: string, ext: string, text: string) {
    try {
        if (ext === 'json' || ext === 'mcmeta' || fileName === 'mcmod.info') {
            return JSON.parse(text);
        } else if (ext === 'yaml' || ext === 'yml') {
            return yaml.load(text);
        } else if (ext === 'toml') {
            return toml.parse(text);
        } else if (fileName.toUpperCase().endsWith('MANIFEST.MF')) {
            return javaManifestParser.parse(text);
        } else {
            throw new Error(`Unrecognized file type: ${fileName}`);
        }
    } catch (e: any) {
        (window as any)?.["toolbox"]?.app?.error("Metadata File Parser", `Error processing ${fileName}: ${e.message}`, { err: e })
    }

    return null;
}
