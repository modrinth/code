import { BlobReader, type EntryMetaData, type FileEntry, Uint8ArrayReader, Uint8ArrayWriter, ZipReader, ZipWriter, type ZipWriterAddDataOptions } from "@zip.js/zip.js";

import {
	type DecompilerResult, DEFAULT_DECOMPILE_OPTIONS
} from "#ui/helpers/toolbox/decompile";
import { type FileTypeInfo, getFileTypeInfo, hashBuffer } from "#ui/helpers/toolbox/file";
import {
type LazyMutableValue, lazyMutableValue,
	type LazyMutableValueImpl,
	type LazyValue} from '#ui/helpers/toolbox/misc';
import type { Logger } from '#ui/helpers/toolbox/misc/logging';
import {decompileMessengerType, messengerRegistry} from "#ui/helpers/toolbox/workers";
import {
	type ArrayBufferSource,
	type EntryData,
	fileOrNull,
	type FileZipTreeNode,
	RootZipTreeNode,
	type Sha512, type SourceOptions,
	type TypedArrayBuffer,
	type TypedBlob,
	type ZipFileContent,
	type ZipNodeTypes,
	ZipTreeNode
} from "#ui/helpers/toolbox/zip";

/**
 * Method used to generate the zip tree of the given blob returning the root node of the entire zip file
 */
export async function readZipTree(logger: Logger, buffer: ArrayBuffer, options: ReadExtraOptions = {}): Promise<RootZipTreeNodeImpl> {
    const root = new RootZipTreeNodeImpl(logger, await hashBuffer(buffer, "SHA-512"));
    return readZipDataToNode(root, root, buffer, options);
}

export async function writeZipTree(node: ZipTreeNodeImpl): Promise<ArrayBuffer|undefined> {
    const zip = new ZipWriter(new Uint8ArrayWriter());

    if (node.type == "directory") {
        const children = node.children
        for (const node of Object.values(children)) {
            const blob = await node.blob();
            const reader = blob != null ? new BlobReader(blob) : null;
            if (reader) await zip.add(node.path, reader, node.metadata);
        }
    }

    return (await zip.close()).buffer;
}

const zipArchiveExtensionRegex = /\.(ear|war|jar|zip|mrpack)$/;

export type ReadExtraOptions = { name?: string, logger?: Logger };

/**
 * Reads a given buffer and apply its content to the baseNode passed with the primary
 */
async function readZipDataToNode<N extends ZipTreeNode>(rootNode: RootZipTreeNodeImpl, baseNode: N, buffer: ArrayBuffer, {name, logger}: ReadExtraOptions): Promise<N> {
    const zip = new ZipReader(new Uint8ArrayReader(new Uint8Array(buffer)));

    logger?.debug(`Setting up to read ${name} archive.`)

    const handleEntry = async (entry: EntryData) => {
        const segments = entry.filename.split('/').filter(Boolean); // Remove empty strings from trailing slashes

        let parentNode: ZipTreeNode = baseNode;

        let path = "";

        for (const [index, segment] of segments.entries()) {
            path = path.length == 0 ? segment : `${path}/${segment}`;

            let segmentNode = parentNode.children.get(segment);

            if (!segmentNode) {
                const fileEntry = (index === segments.length - 1 && !entry.directory)
                    ? fileOrNull(entry)
                    : null;

                if (fileEntry && fileEntry.filename == path) {
                    segmentNode = parentNode.addFile(
                        segment,
                        undefined,
                        zipArchiveExtensionRegex.test(segment) ? "zip" : "file",
                        fileEntry
                    );
                } else {
                    segmentNode = parentNode.addDirectory(segment, entry);
                }
            }

            if (segmentNode.type === 'directory') {
                // Set next parent node to add the given next segment node
                parentNode = segmentNode;
            } else if(segmentNode.type === 'zip' && !segmentNode.metadata.encrypted) {
                // FYI: This technically will not work when using onprogress as technically the
                // given arraybuffer method is not ready yet meaning this is here just for
                // legacy or needing to go back to waiting for all entries is better.
                const arrayBuffer = await fileOrNull(segmentNode)?.arrayBuffer();
                if (arrayBuffer) readZipDataToNode(rootNode, segmentNode, arrayBuffer, { name: segmentNode.name, logger: logger });
            }
        }
    }

    logger?.debug(`Reading entries ${name} archive.`)

    const entries = await zip.getEntries({
        onprogress: (_progress, _total, entry) => handleEntry(entry as EntryData)
    })

    const zipFiles: Promise<FileZipTreeNode>[] = [];

    for (const entry of entries) {
        const fileNode = rootNode.fileEntries.get(
            (baseNode.type != "root" ? baseNode.path + "/" : "") + entry.filename.split('/').filter(Boolean).join('/')
        );

        if (fileNode) {
            fileNode.setBuffer(
                fileOrNull(entry)!.arrayBuffer.bind(entry),
                fileNode.type,
                entry as ZipWriterAddDataOptions
            );

            if (!fileNode.isSetup) fileNode.isSetup = true;

            if (fileNode.type == "zip") zipFiles.push(readZipDataToNode(rootNode, fileNode, await fileNode.getBuffer(), {name: fileNode.name, logger: logger}))
        }
    }

    logger?.debug(`Entries from ${name} archive have been read${zipFiles.length > 0 ? ', waiting for given inner Zip Files' : '.'}`)

    // TODO: MAKE INNER JARS ASYNC I.E. UPDATE LATER TYPE DEAL AFTER THEY ARE DONE LOADING....
    if (zipFiles.length > 0) await Promise.all(zipFiles);

    return baseNode;
}

class ZipTreeNodeImpl extends ZipTreeNode {
    protected override createFileNode(root: RootZipTreeNodeImpl, parent: ZipTreeNode, name: string, path: string, buffer: ArrayBufferSource | undefined, type: "zip" | "file", metadata: FileEntry): FileZipTreeNode {
        return new FileZipTreeNodeImpl(root, parent, name, path, buffer, type, metadata);
    }

    protected override createNode(root: RootZipTreeNodeImpl, parent: ZipTreeNode, name: string, path: string, metadata?: EntryMetaData): ZipTreeNode {
        return new ZipTreeNodeImpl(root, parent, name, path, "directory", metadata)
    }
}

/**
 * Base Node type for a given Archive Tree acting as the holder for the base children
 * combined with extra info like maps for all entries and all file entries within the archive
 */
class RootZipTreeNodeImpl extends RootZipTreeNode {
    constructor(logger: Logger, key: Sha512) {
        super(key);
        this.logger = logger;
    }

    protected override createFileNode(root: RootZipTreeNode, parent: ZipTreeNode, name: string, path: string, buffer: ArrayBufferSource | undefined, type: "zip" | "file", metadata: FileEntry): FileZipTreeNode {
        return new FileZipTreeNodeImpl(root as RootZipTreeNodeImpl, parent as ZipTreeNodeImpl, name, path, buffer, type, metadata);
    }

    protected override createNode(root: RootZipTreeNode, parent: ZipTreeNode, name: string, path: string, metadata?: EntryMetaData): ZipTreeNode {
        return new ZipTreeNodeImpl(root, parent, name, path, "directory", metadata)
    }
}

class FileZipTreeNodeImpl extends ZipTreeNodeImpl implements FileZipTreeNode {

    isSetup: boolean;
    private arrayBuffer: LazyMutableValue<ArrayBuffer>;

    declare type: "zip" | "file";
    declare metadata: FileEntry;
    declare root: RootZipTreeNode;

    fileTypeInfo: LazyValue<FileTypeInfo | null>;

    asText: (options?: SourceOptions) => Promise<DecompilerResult>;

    sha1: LazyValue<string>;

    sha256: LazyValue<string>;

    sha512: LazyValue<string>;

    constructor(root: RootZipTreeNode, parent: ZipTreeNode, name: string, path: string, buffer: ArrayBufferSource | undefined, type: "zip" | "file", metadata?: FileEntry) {
        super(root, parent, name, path, type, metadata);

        if (buffer) {
            this.isSetup = true;
            this.arrayBuffer = lazyMutableValue(() => typeof buffer === 'function' ? buffer() : Promise.resolve(buffer));
        } else {
            this.isSetup = false;
            this.arrayBuffer = lazyMutableValue(
                () => {
                    return new Promise<ArrayBuffer>((resolve, reject) => {
                        const id = setInterval(async () => {
                            if (this.isSetup) {
                                clearInterval(id);
                                return this.getBuffer().then(resolve, reject)
                            } else {
                                root.logger?.debug(`Waiting for '${this.path}' buffer to be set`)
                            }
                        }, 100)
                    })
                }
            );
        }

        this.fileTypeInfo = this.arrayBuffer.lazyMap(async (buffer) => await getFileTypeInfo(this.path, buffer));

		const map = new Map<Partial<SourceOptions>, LazyValue<DecompilerResult>>();

		(this.arrayBuffer as LazyMutableValueImpl<unknown>)._onResetEvent.subscribe(() => map.clear());

        this.asText = async (options?: SourceOptions) => {
			let finalOptions: SourceOptions | undefined = undefined;
			if (options == null || !map.has(options)) {
				const type = await this.fileTypeInfo.get();
				let isValid = false;

				if (type != null) {
					if (this.name.endsWith(".class") || type.mime == 'application/java-vm' || type.extension == 'class' || options?.forcedType == 'class') {
						finalOptions = options ?? DEFAULT_DECOMPILE_OPTIONS;
						if (!map.has(finalOptions)) {
							const lazyValue = this.arrayBuffer.lazyMap(async (buffer) => {
								let error: string | undefined = undefined;

								const decompiler = (await messengerRegistry?.getMessenger(decompileMessengerType))!;

								if (buffer != undefined) {
									return await decompiler.decompile({
										targetFile: { fullPath: this.path, name: this.name, content: buffer, rootKey: this.root.key } as ZipFileContent,
										resources: this.parent?.children.values().map(entry => { return { fullPath: entry.path, name: entry.name } }).toArray(),
										options: options
									});
								}

								error = "Unable to decompile file as Blob was found to be undefined!";

								return { error, wasDecompiled: false, asString: error } as DecompilerResult
							});
							map.set(options ?? DEFAULT_DECOMPILE_OPTIONS, lazyValue);
							return lazyValue.get();
						}
						isValid = true;
					} else if (type.mime.startsWith('text') || options?.forcedType == 'text') {
						finalOptions = options ?? { forcedType: 'text' }
						if (!map.has(finalOptions)) {
							const lazyValue = this.arrayBuffer.lazyMap(async (buffer) => {
								const str = new TextDecoder().decode(buffer);
								return { source: str, wasDecompiled: true, asString: str } as DecompilerResult
							});
							map.set({ forcedType: 'text' }, lazyValue);
							return lazyValue.get();
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

        this.sha1 = this.arrayBuffer.lazyMap(async (buffer) => hashBuffer(buffer, "SHA-1"))

        this.sha256 = this.arrayBuffer.lazyMap(async (buffer) => hashBuffer(buffer, "SHA-256"))

        this.sha512 = this.arrayBuffer.lazyMap(async (buffer) => hashBuffer(buffer, "SHA-512"))
    }

    async setBuffer(buffer: ArrayBufferSource, type?: "zip" | "file", options?: ZipWriterAddDataOptions) {
        this.arrayBuffer.updateGetter(() => typeof buffer === 'function' ? buffer() : Promise.resolve(buffer));
        this.type = type ?? "file";
        this.metadata = {
            ...options,
            ...this.metadata
        } as FileEntry;

        return this;
    }

    getBuffer(): Promise<ArrayBuffer> {
        return this.arrayBuffer.get();
    }

    async getFileType(): Promise<FileTypeInfo | null> {
        return this.fileTypeInfo.get();
    }

    async getTypedBlob(): Promise<TypedBlob>  {
        const buffer = await this.getBuffer();
        const fileType = await this.fileTypeInfo.get();
        const typedBlob = new Blob([buffer], { type: fileType?.mime }) as TypedBlob;
        typedBlob.fileType = fileType;
        return typedBlob;
    }

    async getTypedBuffer(): Promise<TypedArrayBuffer> {
        const buffer = await this.getBuffer();
        const typedBuffer = buffer as TypedArrayBuffer;
        typedBuffer.fileType = await this.fileTypeInfo.get();
        return typedBuffer;
    }
}
