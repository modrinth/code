import type { DirectoryEntry, EntryMetaData, FileEntry, ZipWriterAddDataOptions } from "@zip.js/zip.js";

import {
	type DecompileOptions,
	type DecompilerResult,
	DEFAULT_DECOMPILE_OPTIONS
} from "#ui/helpers/toolbox/decompile";
import type { FileDetails, FileTypeInfo, HashedFileContent } from "#ui/helpers/toolbox/file";
import {
	type LazyValue,
	lazyValue,
	TreeNode
} from "#ui/helpers/toolbox/misc";
import type { Logger } from "#ui/helpers/toolbox/misc/logging";

// TODO: SWITCH TO ZipWriterAddDataOptions instead of EntryMetaData ???
export type EntryData = EntryMetaData & { directory: boolean };

export type ZipNodeTypes = "zip" | "file" | "directory" | "root";

export const nodeTypeToOrdinal = {
    "zip": 2,
    "directory": 1,
    "file": 0
} as Record<ZipNodeTypes, number>

export function sortZipNodes(node1: ZipTreeNode, node2: ZipTreeNode)  {
    const order = nodeTypeToOrdinal[node1.type] - nodeTypeToOrdinal[node2.type];

    return order == 0 ? node1.name.localeCompare(node2.name, "en") : -order
}

export type ArrayBufferSource = (() => Promise<ArrayBuffer>) | Promise<ArrayBuffer> | ArrayBuffer;

export interface ZipFileContent extends FileDetails {
	rootKey: Sha512;
	content: ArrayBuffer;
}

export class ZipTreeNode extends TreeNode<ZipTreeNode> {
    root?: RootZipTreeNode;
    type: ZipNodeTypes;
    metadata: EntryData

    constructor(root: RootZipTreeNode | undefined, parent: ZipTreeNode | undefined, name: string, path: string, type: ZipNodeTypes, metadata?: EntryMetaData) {
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

        this.root?.removeNodeFromRoot(this)
    };

    addFile(name: string, buffer: ArrayBufferSource | undefined, type: "zip" | "file", metadata: FileEntry): FileZipTreeNode {
        return this.addNode(name, (root, parent, name, path) => this.createFileNode(root, parent, name, path, buffer, type, metadata));
    };

    protected createFileNode(root: RootZipTreeNode, parent: ZipTreeNode, name: string, path: string, buffer: ArrayBufferSource | undefined, type: "zip" | "file", metadata: FileEntry): FileZipTreeNode {
        const node = new DelegateFileZipTreeNode(root, parent, name, path, type, metadata);
        if (buffer != null) node.setBuffer(buffer);
        return node;
    }

    addDirectory(name: string, metadata?: EntryMetaData): ZipTreeNode {
        return this.addNode(name, (root, parent, name, path) => this.createNode(root, parent, name, path, metadata));
    };

    protected createNode(root: RootZipTreeNode, parent: ZipTreeNode, name: string, path: string, metadata?: EntryMetaData): ZipTreeNode {
        return new ZipTreeNode(root, parent, name, path, "directory", metadata)
    }

    addNode<N extends ZipTreeNode>(name: string, nodeBuilder: (root: RootZipTreeNode, parent: ZipTreeNode, name: string, path: string) => N): N {
        const root = this.root ?? this as unknown as RootZipTreeNode;
        this.children.get(name)?.remove();
        const node = nodeBuilder(root, this, name, (this.type != "root" ? this.path + "/" : "") + name);
        this.children.set(name, node);
        root.addNodeToRoot(node);
        return node;
    };

    asFile(): FileZipTreeNode | undefined{
        return (this.type == "zip" || this.type == "file") ? this as unknown as FileZipTreeNode : undefined;
    }
}

export type Sha512 = string;

/**
 * Root tree node — extends TreeNode with a flat entries list.
 */
export class RootZipTreeNode extends ZipTreeNode {
    key: Sha512;
    entries = new Map<string, ZipTreeNode>();
    fileEntries = new Map<string, FileZipTreeNode>();
    relativeFileEntries = new Map<string, FileZipTreeNode>();

    logger?: Logger;

    declare type: "root";

    declare root: undefined;
    declare parent: undefined;

    constructor(key: Sha512) {
        super(undefined, undefined, "", "", "root");
        this.key = key;
    }

    /**
     * @internal Only used to add a node to the given reference maps
     */
    addNodeToRoot(node: ZipTreeNode) {
        this.entries.set(node.path, node)
        const fileNode = node.asFile();

        if (fileNode) {
            this.fileEntries.set(node.path, fileNode);

            const parts = node.path.split(".jar/");
            this.relativeFileEntries.set((parts.length > 1) ? parts[parts.length - 1] : node.path, fileNode)
        }

        return node;
    }

    /**
     * @internal Only used to remove a node from the given reference maps
     */
    removeNodeFromRoot(node: ZipTreeNode) {
        this.entries.delete(node.path)
        this.fileEntries.delete(node.path)

        const parts = node.path.split(".jar/");
        if (parts.length > 1) this.relativeFileEntries.delete(parts[parts.length - 1])

        return node;
    }
}

export type SourceOptions = DecompileOptions & { forcedType?: "class" | "text" | undefined; };

export interface SourceHandler {
	asText: (options?: SourceOptions) => Promise<DecompilerResult>,
}

export interface LazyDataAccess {
    fileTypeInfo: LazyValue<FileTypeInfo | null>,
    sha1: LazyValue<string>,
    sha256: LazyValue<string>,
    sha512: LazyValue<string>
}

export interface FileZipTreeNode extends ZipTreeNode, LazyDataAccess, SourceHandler {
    type: "zip" | "file";

    metadata: FileEntry
    isSetup: boolean;

    getBuffer(): Promise<ArrayBuffer>
    setBuffer(buffer: ArrayBufferSource, type?: "zip" | "file", options?: ZipWriterAddDataOptions): Promise<this>;

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

export function fileOrNull(data: ZipTreeNode|EntryData): FileEntry | null {
    const castedData = (((data as Partial<ZipTreeNode>).metadata ?? data) as EntryData);
    return !castedData.directory ? castedData as FileEntry : null;
}

export function dirOrNull(data: ZipTreeNode|EntryData): DirectoryEntry | null {
    const castedData = (((data as Partial<ZipTreeNode>).metadata ?? data) as EntryData);
    return castedData.directory ? castedData as DirectoryEntry : null;
}

export type StringMatcher = { startsWith?: string, includes?: string, endsWith?: string, regex?: RegExp, negate?: boolean};
export type FileMatcher = {
    mime?: StringMatcher,
    path?: StringMatcher
}

export interface RootNodeTarget {
    root: Sha512
}

export interface NodeTarget extends RootNodeTarget {
    path: string;
    relative?: boolean
}

export function getAsTarget(target: FileZipTreeNode | NodeTarget) {
    return (target as FileZipTreeNode).type != undefined ? {path: target.path, root: (target.root as RootZipTreeNode).key} : target as NodeTarget;
}

export interface ZipHandler {
    zipArchivesByHash: Map<Sha512, RootZipTreeNode>;
    urlToHash: Map<string, Sha512>;
    readLazyData<K extends keyof LazyDataAccess>(node: FileZipTreeNode | NodeTarget, key: K): Promise<Awaited<ReturnType<LazyDataAccess[K]['get']>>>;
	readSource(node: FileZipTreeNode | NodeTarget, options?: SourceOptions): Promise<DecompilerResult>;
    readNodeData(node: FileZipTreeNode | NodeTarget): Promise<ArrayBuffer>;
    writeNodeData(node: FileZipTreeNode | NodeTarget, buffer: ArrayBuffer, type?: "zip" | "file", options?: ZipWriterAddDataOptions): Promise<boolean>;
    collectFiles(root: RootZipTreeNode | Sha512, matcher?: FileMatcher): Promise<FileDetails[]>;
    collectHashedFiles(root: RootZipTreeNode | Sha512, matcher?: FileMatcher, deduplicate?: boolean): Promise<HashedFileContent[]>;
    getOrReadZipTree(buffer: () => Promise<ArrayBuffer | undefined>, details?: { name?: string, url?: string, hash?: string }): Promise<RootZipTreeNode | undefined>;
    readZipTree(buffer: ArrayBuffer, details?: { name?: string, url?: string, hash?: string }): Promise<RootZipTreeNode>;
    getZipTree(value: string | Sha512, isUrl: boolean, localOnly?: boolean): Promise<RootZipTreeNode | undefined>;
    closeZipTree(root: RootZipTreeNode | Sha512, name?: string): Promise<{ err?: Error, wasClosed: boolean }>;
    closeAllZipTrees(): Promise<{ err?: Error, wasClosed: boolean }>;
}

export class DelegateFileZipTreeNode extends ZipTreeNode implements FileZipTreeNode {
    declare type: "zip" | "file";
    declare metadata: FileEntry;
    declare handler: ZipHandler | undefined;

    isSetup: boolean;

    constructor(root: RootZipTreeNode | undefined, parent: ZipTreeNode | undefined, name: string, path: string, type: "zip" | "file", metadata: FileEntry) {
        super(root, parent, name, path, type, metadata);

        this.handler = undefined;

        this.fileTypeInfo = lazyValue(() => { throw new Error("Unable to get value 'fileTypeInfo' as it was not setup yet") });
        this.asText = () => { throw new Error("Unable to get value 'asText' as it was not setup yet") };
        this.sha1 = lazyValue(() => { throw new Error("Unable to get value 'sha1' as it was not setup yet") });
        this.sha256 = lazyValue(() => { throw new Error("Unable to get value 'sha256' as it was not setup yet") });
        this.sha512 = lazyValue(() => { throw new Error("Unable to get value 'sha512' as it was not setup yet") });

        this.isSetup = false;
    }

	onDataUpdate?: () => void;

    finishNodeSetup(handler: ZipHandler) {
        this.handler = handler;

        this.fileTypeInfo = lazyValue(() => this.getHandler().readLazyData(this, "fileTypeInfo"));
        this.sha1 = lazyValue(() => this.getHandler().readLazyData(this, "sha1"));
        this.sha256 = lazyValue(() => this.getHandler().readLazyData(this, "sha256"));
        this.sha512 = lazyValue(() => this.getHandler().readLazyData(this, "sha512"));

		const map = new Map<Partial<SourceOptions>, LazyValue<DecompilerResult>>();

		this.onDataUpdate = () => map.clear()

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
    sha1: LazyValue<string>;
    sha256: LazyValue<string>;
    sha512: LazyValue<string>;

    getBuffer(): Promise<ArrayBuffer> {
        return this.getHandler().readNodeData(this);
    }

    async setBuffer(buffer: ArrayBufferSource, type?: "zip" | "file", options?: ZipWriterAddDataOptions): Promise<this> {
        const bufferPromise = typeof buffer === 'function' ? buffer() : Promise.resolve(buffer);
        await this.getHandler().writeNodeData(this, await bufferPromise, type, options);
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
    root.entries = new Map();
    root.fileEntries = new Map();

    return reconstituteNodes(handler, root, root, root)
}

function reconstituteNodes<N extends ZipTreeNode>(handler: ZipHandler, root: RootZipTreeNode, parent: N, node: N): N {
    let reconstitutedNode: ZipTreeNode;
    if (node.type == "root") {
        reconstitutedNode = Object.setPrototypeOf(node, RootZipTreeNode.prototype);
    } else if (node.type == "directory") {
        reconstitutedNode = Object.setPrototypeOf(node, ZipTreeNode.prototype);
    } else {
        reconstitutedNode = Object.setPrototypeOf(node, DelegateFileZipTreeNode.prototype);
        root.fileEntries.set(reconstitutedNode.path, reconstitutedNode as FileZipTreeNode);
    }

    node.children.forEach((entry) => reconstituteNodes(handler, root, node, entry))

    if (reconstitutedNode.type != "root") {
        root.entries.set(reconstitutedNode.path, reconstitutedNode);

        reconstitutedNode.root = root;
        reconstitutedNode.parent = parent;
    }

    if (node.type == "file") (node as unknown as DelegateFileZipTreeNode).finishNodeSetup(handler)

    return reconstitutedNode as N;
}
