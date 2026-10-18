import type { DirectoryEntry, EntryMetaData, FileEntry, ZipWriterAddDataOptions } from "@zip.js/zip.js";

import {
	type DecompileOptions,
	type DecompilerResult,
	DEFAULT_DECOMPILE_OPTIONS
} from "#ui/helpers/toolbox/decompile";
import type {
	AsyncMaybe,
	ContentHashes,
	FileDetails,
	FileTypeInfo,
	HashedFileContent,
	HashType
} from "#ui/helpers/toolbox/file";
import {
	type LazyValue,
	lazyValue,
	TreeNode
} from "#ui/helpers/toolbox/misc";
import type { Logger } from "#ui/helpers/toolbox/misc/logging";
import type { EventSource } from "#ui/helpers/toolbox/misc"
import type {RangeCacheReaderOptions} from "#ui/helpers/toolbox/zip/range-cache-reader.ts";

// TODO: SWITCH TO ZipWriterAddDataOptions instead of EntryMetaData ???
export type EntryData = EntryMetaData & { directory: boolean };

export type ZipNodeType = {
	'file': FileZipTreeNode,
	'directory': ZipTreeNode & { type: 'directory'},
}

export type ZipNodeTypes = keyof ZipNodeType;
export type ZipNode = ZipNodeType[ZipNodeTypes];

export const nodeTypeToOrdinal = {
    "directory": 1,
    "file": 0
} as Record<ZipNodeTypes, number>

export const zipExtensionRegex = /\.(ear|war|jar|zip|mrpack)$/;
export const zipMimeTypes = new Set([
	'application/x-zip-compressed',
	'application/zip',
	'application/zip-compressed',
	'application/java-archive'
])

export function isZipArchive<BL extends boolean | undefined = undefined>(node: ZipNode, verifyWithData?: BL): AsyncMaybe<BL, boolean> {
	return (node.type == 'file'
		? (verifyWithData
			? node.getFileType()
					.then((type) => type != null ? zipMimeTypes.has(type.mime) : zipExtensionRegex.test(node.name))
			:  zipExtensionRegex.test(node.name)
		)
		: false
	) as AsyncMaybe<BL, boolean>
}

function nodeToOrdinal(node: ZipNode) {
	return isZipArchive(node) ? 2 : (node.type == 'directory' ? 1 : 0);
}

export function sortZipNodes(node1: ZipNode, node2: ZipNode)  {
    const order = nodeToOrdinal(node1) - nodeToOrdinal(node2);

    return order == 0 ? node1.name.localeCompare(node2.name, "en") : -order
}

export type ArrayBufferSource = (() => Promise<ArrayBuffer>) | Promise<ArrayBuffer> | ArrayBuffer;

export function arrayBufferFromSource(source: ArrayBufferSource): Promise<ArrayBuffer> {
	return typeof source === 'function' ? source() : Promise.resolve(source);
}

export interface ZipFileContent extends FileDetails {
	root: UUID_V7;
	content: ArrayBuffer;
}

export class ZipTreeNode extends TreeNode<ZipNode> {
    root?: RootZipTreeNode;
    type: ZipNodeTypes;
    metadata: EntryData

    constructor(root: RootZipTreeNode | undefined, parent: ZipNode | undefined, name: string, path: string, type: ZipNodeTypes, metadata?: EntryMetaData) {
        super(parent, name, path);
        this.root = root;
        this.type = type;
        this.metadata = {
            ...(metadata ?? {}),
            directory: type == "directory"
        } as EntryData;
    }

    remove() {
        this.children.delete(this.name);

        this.root?.removeNodeFromRoot(this as ZipNode)
    };

    addFile(name: string, buffer: ArrayBufferSource | undefined, metadata: FileEntry): FileZipTreeNode {
        return this.addNode(name, (root, parent, name, path) => this.createFileNode(root, parent, name, path, buffer, metadata));
    };

    protected createFileNode(root: RootZipTreeNode, parent: ZipNode, name: string, path: string, buffer: ArrayBufferSource | undefined, metadata: FileEntry): FileZipTreeNode {
        const node = new DelegateFileZipTreeNode(root, parent, name, path, metadata);
        if (buffer != null) node.setBuffer(buffer);
        return node;
    }

    addDirectory(name: string, metadata?: EntryMetaData): ZipNode {
        return this.addNode(name, (root, parent, name, path) => this.createNode(root, parent, name, path, metadata));
    };

    protected createNode(root: RootZipTreeNode, parent: ZipNode, name: string, path: string, metadata?: EntryMetaData): ZipNode {
        return new ZipTreeNode(root, parent, name, path, "directory", metadata) as ZipNode
    }

    addNode<N extends ZipNode>(name: string, nodeBuilder: (root: RootZipTreeNode, parent: ZipNode, name: string, path: string) => N): N {
        const root = this.root ?? this as unknown as RootZipTreeNode;
        this.children.get(name)?.remove();
        const node = nodeBuilder(root, this as ZipNode, name, this.path + "/" + name);
        this.children.set(name, node);
        root.addNodeToRoot(node);
        return node;
    };

    asFile(): FileZipTreeNode | undefined{
        return this.type == "file" ? this as unknown as FileZipTreeNode : undefined;
    }

	/** The archive this node is stored in: the root, or the nested archive (e.g. jar-in-jar) holding it. */
	getArchive(): RootZipTreeNode | FileZipTreeNode {
		for (let parent = this.parent; parent != null; parent = parent.parent) {
			if (isZipArchive(parent)) return parent as FileZipTreeNode;
		}
		return this.root ?? this as unknown as RootZipTreeNode;
	}
}

export type UUID_V7 = string;

/**
 * Root tree node — extends TreeNode with a flat entries list.
 */
export class RootZipTreeNode extends ZipTreeNode {
    key: UUID_V7;
    entries = new Map<string, ZipNode>();
    fileEntries = new Map<string, FileZipTreeNode>();
    relativeFileEntries = new Map<string, FileZipTreeNode>();

    logger?: Logger;

	onFinishedLoading: Promise<void>

	/** SHA-512 of the whole archive, once known (for a URL, only after all of it is downloaded). */
	get sha512Hash(){
		return this.hashes?.sha512;
	};

	/**
	 * Hashes of the whole archive computed so far. The SHA-512 is computed once all of it is
	 * downloaded; the others only once asked for (see `ZipHandler.readArchiveHash`).
	 */
	hashes?: ContentHashes;

    declare type: "directory";

    declare root: undefined;
    declare parent: undefined;

    constructor(key: UUID_V7, onFinishedLoading: Promise<void>) {
        super(undefined, undefined, "", "", "directory");
        this.key = key;
		this.onFinishedLoading = onFinishedLoading;
    }

    /**
     * @internal Only used to add a node to the given reference maps
     */
    addNodeToRoot(node: ZipNode) {
        this.entries.set(node.path, node)
        const fileNode = node.asFile();

        if (fileNode) {
            this.fileEntries.set(node.path, fileNode);
            this.relativeFileEntries.set(RootZipTreeNode.relativePathOf(node.path), fileNode)
        }

        return node;
    }

	/**
	 * The path of a file within its innermost archive, with a leading slash, e.g. `/a/B.class` for
	 * both `/a/B.class` and `/META-INF/jars/c.jar/a/B.class`.
	 */
	static relativePathOf(path: string) {
		const parts = path.split(".jar/");
		const relative = parts[parts.length - 1];
		return relative.startsWith("/") ? relative : "/" + relative;
	}

	/** Finds a file by its full path, or by its path within its innermost archive when `relative`. */
	findFileEntry(path: string, relative?: boolean) {
		return relative
			? this.relativeFileEntries.get(RootZipTreeNode.relativePathOf(path))
			: this.fileEntries.get(path);
	}

    /**
     * @internal Only used to remove a node from the given reference maps
     */
    removeNodeFromRoot(node: ZipNode) {
        this.entries.delete(node.path)
        this.fileEntries.delete(node.path)

        const relativePath = RootZipTreeNode.relativePathOf(node.path);
        if (this.relativeFileEntries.get(relativePath) === node) this.relativeFileEntries.delete(relativePath)

        return node;
    }
}

export type SourceOptions = DecompileOptions & { forcedType?: "class" | "text" | undefined; };

export interface SourceHandler {
	asText: (options?: SourceOptions) => Promise<DecompilerResult>,
}

export interface LazyDataAccess {
    fileTypeInfo: LazyValue<FileTypeInfo | null>,
}

export interface FileZipTreeNode extends ZipTreeNode, LazyDataAccess, SourceHandler {
    type: "file";

	/** The file's hash of the given type, computed the first time it's asked for and then kept. */
	hash(type: HashType): Promise<string>;

    metadata: FileEntry
    isSetup: boolean;

    getBuffer(): Promise<ArrayBuffer>
    setBuffer(buffer: ArrayBufferSource, options?: ZipWriterAddDataOptions): Promise<this>;

    getFileType(): Promise<FileTypeInfo | null>

    getTypedBlob(): Promise<TypedBlob>

    getTypedBuffer(): Promise<TypedArrayBuffer>
}


export interface TypedArrayBuffer extends ArrayBuffer {
    fileType: FileTypeInfo | null;
}

export interface TypedBlob extends Blob {
    fileType: FileTypeInfo | null;
}

export function fileOrNull(data: ZipNode|EntryData): FileEntry | null {
    const castedData = (((data as Partial<ZipNode>).metadata ?? data) as EntryData);
    return !castedData.directory ? castedData as FileEntry : null;
}

export function dirOrNull(data: ZipNode|EntryData): DirectoryEntry | null {
    const castedData = (((data as Partial<ZipNode>).metadata ?? data) as EntryData);
    return castedData.directory ? castedData as DirectoryEntry : null;
}

export type StringMatcher = { startsWith?: string, includes?: string, endsWith?: string, regex?: RegExp, negate?: boolean};
export type FileMatcher = {
    mime?: StringMatcher,
    path?: StringMatcher
}

export interface RootNodeTarget {
    root: UUID_V7
}

export interface NodeTarget extends RootNodeTarget {
    path: string;
    relative?: boolean
}

export function getAsTarget(target: FileZipTreeNode | NodeTarget) {
    return (target as FileZipTreeNode).type != undefined ? {path: target.path, root: (target.root as RootZipTreeNode).key} : target as NodeTarget;
}

export type ZipBufferSource = { data_type: 'buffer', name?: string, url?: string, sha512Hash?: string, buffer: ArrayBufferSource, rangeOptions?: undefined };
export type ZipURLSource = { data_type: 'url', name?: string, url: string, sha512Hash?: string, buffer?: undefined, rangeOptions?: RangeCacheReaderOptions,}
export type ZipDataSource = ZipBufferSource | ZipURLSource;

type Typed<K extends string, T> = { type: K } & T;

export type ArchiveUpdateTypes =
	Typed<'node_loaded', { root: RootNodeTarget, node: ZipNode, }> |
	Typed<'updated_node_data', { node: NodeTarget }> |
	Typed<'root_loaded', { root: RootNodeTarget }> |
	/**
	 * The archive's own hashes were computed from its downloaded bytes. `expectedHash` is the
	 * SHA-512 it was opened with when that didn't match, which is no longer mapped to the archive.
	 */
	Typed<'root_hashed', { root: RootNodeTarget, sha512Hash: string, expectedHash?: string }> |
	/** Bytes of the archive cached so far, keyed by `url` too as it starts before the root is returned. */
	Typed<'archive_download', { root?: UUID_V7, url?: string, sha512Hash?: string, current: number, total: number }>;

export interface ZipHandler {
    zipByUUID: Map<UUID_V7, RootZipTreeNode>;
    urlToUUID: Map<string, UUID_V7>;
    readLazyData<K extends keyof LazyDataAccess>(node: FileZipTreeNode | NodeTarget, key: K): Promise<Awaited<ReturnType<LazyDataAccess[K]['get']>>>;
	readSource(node: FileZipTreeNode | NodeTarget, options?: SourceOptions): Promise<DecompilerResult>;
    readNodeData(node: FileZipTreeNode | NodeTarget): Promise<ArrayBuffer>;
	/** The file's hash of the given type, computed by the worker. */
	readNodeHash(node: FileZipTreeNode | NodeTarget, type: HashType): Promise<string>;
	/**
	 * The whole archive's hash of the given type, which waits until all of it has been downloaded.
	 * The result is also kept in the root's `hashes`.
	 */
	readArchiveHash(root: RootZipTreeNode | UUID_V7, type: HashType): Promise<string>;
	/**
	 * Reads the node's data, waiting for the archive to finish loading when the node isn't known
	 * yet, and resolves to `undefined` when the archive turns out not to hold it.
	 */
	findNodeData(node: NodeTarget): Promise<ArrayBuffer | undefined>;
    writeNodeData(node: FileZipTreeNode | NodeTarget, buffer: ArrayBuffer, options?: ZipWriterAddDataOptions): Promise<boolean>;
    collectFiles(root: RootZipTreeNode | UUID_V7, matcher?: FileMatcher): Promise<FileDetails[]>;
    collectHashedFiles(root: RootZipTreeNode | UUID_V7, matcher?: FileMatcher, deduplicate?: boolean): Promise<HashedFileContent[]>;
    getOrReadZipTree(source: ZipDataSource): Promise<RootZipTreeNode>;
    readZipTree(source: ZipDataSource): Promise<RootZipTreeNode>;
    getZipTree(value: string | UUID_V7, isUrl: boolean, localOnly?: boolean): Promise<RootZipTreeNode | undefined>;
    closeZipTree(root: RootZipTreeNode | UUID_V7, name?: string): Promise<{ err?: Error, wasClosed: boolean }>;
    closeAllZipTrees(): Promise<{ err?: Error, wasClosed: boolean }>;
	archiveUpdate(msg: ArchiveUpdateTypes): void;
	onArchiveUpdate: EventSource<(msg: ArchiveUpdateTypes) => void>;
}

export class DelegateFileZipTreeNode extends ZipTreeNode implements FileZipTreeNode {
    declare type: "file";
    declare metadata: FileEntry;
    declare handler: ZipHandler | undefined;

    isSetup: boolean;

    constructor(root: RootZipTreeNode | undefined, parent: ZipNode | undefined, name: string, path: string, metadata: FileEntry) {
        super(root, parent, name, path, "file", metadata);

        this.handler = undefined;

        this.fileTypeInfo = lazyValue(() => { throw new Error("Unable to get value 'fileTypeInfo' as it was not setup yet") });
        this.asText = () => { throw new Error("Unable to get value 'asText' as it was not setup yet") };

        this.isSetup = false;
    }

	onDataUpdate?: () => void;

    finishNodeSetup(handler: ZipHandler) {
        this.handler = handler;

        this.fileTypeInfo = lazyValue(() => this.getHandler().readLazyData(this, "fileTypeInfo"));
		const map = new Map<Partial<SourceOptions>, LazyValue<DecompilerResult>>();

		this.onDataUpdate = () => {
			map.clear();
			this.hashes?.clear();
		}

		this.asText = async (options?: SourceOptions) => {
			let finalOptions: SourceOptions | undefined = undefined;
			if (options == null || !map.has(options)) {
				const type = await this.fileTypeInfo.get();
				let isValid = false;
				if (type != null) {
					if (this.name.endsWith(".class") || type.mime == 'application/java-vm' || type.extension == 'class' || options?.forcedType == 'class') {
						finalOptions = options ?? DEFAULT_DECOMPILE_OPTIONS;
						if (!map.has(finalOptions)) {
							const value = lazyValue(() => this.getHandler().readSource(this, finalOptions))
							map.set(options ?? DEFAULT_DECOMPILE_OPTIONS, value);
							return value.get();
						}
						isValid = true;
					} else if (type.mime.startsWith('text') || options?.forcedType == 'text') {
						finalOptions = options ?? { forcedType: 'text' }
						if (!map.has(finalOptions)) {
							const value = lazyValue(() => this.getHandler().readSource(this, finalOptions))
							map.set(options ?? DEFAULT_DECOMPILE_OPTIONS, value);
							return value.get();
						}
						isValid = true;
					}
				}

				if (!isValid) {
					const msg = "Current file was found not to be either a class or text, no sources could be created";
					return Promise.resolve(
						{ source: msg, wasDecompiled: false, asString: msg } as DecompilerResult
					)
				}
			} else {
				finalOptions = options;
			}

			return map.get(finalOptions!)!.get();
		}

        this.isSetup = true;
    }

    getHandler() {
        return this.handler!;
    }

    fileTypeInfo: LazyValue<FileTypeInfo | null>;
    asText: (options?: SourceOptions) => Promise<DecompilerResult>;

	/**
	 * Created on first use: nodes sent back from the worker are given this class's prototype
	 * rather than constructed, so field initializers never run for them.
	 */
	private hashes?: Map<HashType, LazyValue<string>>;

	hash(type: HashType): Promise<string> {
		this.hashes ??= new Map();
		let value = this.hashes.get(type);
		if (!value) {
			value = lazyValue(() => this.getHandler().readNodeHash(this, type));
			this.hashes.set(type, value);
		}
		return value.get();
	}

    getBuffer(): Promise<ArrayBuffer> {
        return this.getHandler().readNodeData(this);
    }

    async setBuffer(buffer: ArrayBufferSource, options?: ZipWriterAddDataOptions): Promise<this> {
        await this.getHandler().writeNodeData(this, await arrayBufferFromSource(buffer), options);
        return this;
    }

    getFileType(): Promise<FileTypeInfo | null> {
        return this.fileTypeInfo.get();
    }

    async getTypedBlob(): Promise<TypedBlob> {
        const buffer = await this.getBuffer();
        const fileType = await this.getFileType();
        const typedBlob = new Blob([buffer], { type: fileType?.mime }) as TypedBlob;
        typedBlob.fileType = fileType;
        return typedBlob;
    }

    async getTypedBuffer(): Promise<TypedArrayBuffer> {
        const buffer = await this.getBuffer();
        const typedBuffer = buffer as TypedArrayBuffer;
        typedBuffer.fileType = await this.getFileType();
        return typedBuffer;
    }
}

export function reconstituteRoot(handler: ZipHandler, root: RootZipTreeNode): RootZipTreeNode {
    Object.setPrototypeOf(root, RootZipTreeNode.prototype);
    root.entries = new Map();
    root.fileEntries = new Map();
    root.relativeFileEntries = new Map();

    return reconstituteNodes(handler, root, root, root)
}

export function reconstituteNodes<N extends ZipNode>(handler: ZipHandler, root: RootZipTreeNode, parent: N, node: N): N {
    let reconstitutedNode: ZipNode;
    if (node instanceof RootZipTreeNode) {
        reconstitutedNode = Object.setPrototypeOf(node, RootZipTreeNode.prototype);
    } else if (node.type == "directory") {
        reconstitutedNode = Object.setPrototypeOf(node, ZipTreeNode.prototype);
    } else {
        reconstitutedNode = Object.setPrototypeOf(node, DelegateFileZipTreeNode.prototype);
    }

    node.children.forEach((entry) => reconstituteNodes(handler, root, node, entry))

    if (!(reconstitutedNode instanceof RootZipTreeNode)) {
        root.addNodeToRoot(reconstitutedNode);

        reconstitutedNode.root = root;
        reconstitutedNode.parent = parent;
    }

    if (node.type == "file") (node as unknown as DelegateFileZipTreeNode).finishNodeSetup(handler)

    return reconstitutedNode as N;
}
