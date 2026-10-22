import {
	BlobReader,
	type EntryMetaData,
	type FileEntry,
	Uint8ArrayReader,
	Uint8ArrayWriter,
	ZipReader,
	ZipWriter,
	type ZipWriterAddDataOptions,
} from '@zip.js/zip.js'
import { v7 as uuidV7 } from 'uuid'

import { type DecompilerResult, DEFAULT_DECOMPILE_OPTIONS } from '#ui/helpers/toolbox/decompile'
import {
	computeHash,
	type FileTypeInfo,
	getFileTypeInfo,
	type HashType,
} from '#ui/helpers/toolbox/file'
import {
	type LazyMutableValue,
	lazyMutableValue,
	type LazyMutableValueImpl,
	type LazyValue,
} from '#ui/helpers/toolbox/misc'
import type { Logger } from '#ui/helpers/toolbox/misc/logging'
import { decompileMessengerType, messengerRegistry } from '#ui/helpers/toolbox/workers/registry'
import {
	arrayBufferFromSource,
	type ArrayBufferSource,
	type EntryData,
	fileOrNull,
	type FileZipTreeNode,
	RootZipTreeNode,
	type SourceOptions,
	type TypedArrayBuffer,
	type TypedBlob,
	type UUID_V7,
	type ZipFileContent,
	type ZipNode,
	ZipTreeNode,
} from '#ui/helpers/toolbox/zip'
import {
	RangeCacheReader,
	type RangeCacheReaderOptions,
} from '#ui/helpers/toolbox/zip/range-cache-reader.ts'

/**
 * Method used to generate the zip tree of the given blob returning the root node of the entire zip file
 */
export async function readZipTreeFromBuffer(
	logger: Logger,
	buffer: ArrayBuffer,
	options: ReadExtraOptions = {},
): Promise<RootZipTreeNodeImpl> {
	const root = new RootZipTreeNodeImpl(
		logger,
		options.key ?? uuidV7(),
		Promise.withResolvers<void>(),
	)
	root.setContent(buffer)
	const result = await readZipDataToNode(root, root, zipReaderFromBuffer(buffer), options)
	root.markEntriesRead()
	return result
}

export async function readZipTreeFromURL(
	logger: Logger,
	url: string,
	options: ReadExtraOptions & { rangeOptions?: RangeCacheReaderOptions } = {},
): Promise<RootZipTreeNodeImpl> {
	const root = new RootZipTreeNodeImpl(
		logger,
		options.key ?? uuidV7(),
		Promise.withResolvers<void>(),
	)
	const rangeReader = new RangeCacheReader(url, options.rangeOptions)
	root.closeCallbacks.push(() => rangeReader.abort())

	const result = await readZipDataToNode(root, root, new ZipReader(rangeReader), options)
	root.markEntriesRead()

	// Files are fetched on demand ahead of this, but caching the rest makes later reads instant
	rangeReader
		.toBlob()
		.then(async (blob) => {
			if (root.isClosed) return
			root.setContent(await blob.arrayBuffer())
			const sha512Hash = await root.hash('sha512')
			if (!root.isClosed) options.archiveHashedCallback?.(root, sha512Hash)
		})
		.catch((err) => {
			if (!root.isClosed)
				logger.warn(`Unable to fetch the rest of '${url}' in the background`, { err: err })
		})

	return result
}

export async function writeZipTree(node: ZipTreeNodeImpl): Promise<ArrayBuffer | undefined> {
	const zip = new ZipWriter(new Uint8ArrayWriter())

	if (node.type == 'directory') {
		const children = node.children
		for (const node of Object.values(children)) {
			const blob = await node.blob()
			const reader = blob != null ? new BlobReader(blob) : null
			if (reader) await zip.add(node.path, reader, node.metadata)
		}
	}

	return (await zip.close()).buffer
}

export const zipArchiveExtensionRegex = /\.(ear|war|jar|zip|mrpack)$/

export type ReadExtraOptions = {
	name?: string
	logger?: Logger
	/** Key for the root, for callers that need to refer to it before the read finishes. */
	key?: UUID_V7
	/**
	 * Called once a nested archive's tree has been read. When given, nested archives are read in
	 * the background instead of being waited on, with `onFinishedLoading` resolving once all are.
	 */
	innerArchiveReadCallback?: (root: RootZipTreeNode, node: FileZipTreeNode) => void
	/** Called with the SHA-512 of an archive read from a URL, once all of it has been downloaded. */
	archiveHashedCallback?: (root: RootZipTreeNode, sha512Hash: string) => void
}

function zipReaderFromBuffer(buffer: ArrayBuffer): ZipReader<Uint8Array<ArrayBuffer>> {
	return new ZipReader(new Uint8ArrayReader(new Uint8Array(buffer)))
}

/**
 * Reads a given buffer and apply its content to the baseNode passed with the primary
 */
async function readZipDataToNode<N extends ZipNode>(
	rootNode: RootZipTreeNodeImpl,
	baseNode: N,
	reader: ZipReader<unknown>,
	options: ReadExtraOptions,
): Promise<N> {
	const { name, logger, innerArchiveReadCallback } = options
	logger?.debug(`Setting up to read ${name} archive.`)

	const zipFiles: Promise<FileZipTreeNode>[] = []

	async function handleEntry(entry: EntryData) {
		const segments = entry.filename.split('/').filter(Boolean) // Remove empty strings from trailing slashes

		let parentNode: ZipNode = baseNode

		let path = ''

		for (const [index, segment] of segments.entries()) {
			path = path.length == 0 ? segment : `${path}/${segment}`

			let segmentNode = parentNode.children.get(segment)

			if (!segmentNode) {
				const fileEntry =
					index === segments.length - 1 && !entry.directory ? fileOrNull(entry) : null

				if (fileEntry && fileEntry.filename == path) {
					segmentNode = parentNode.addFile(segment, undefined, fileEntry)
				} else {
					segmentNode = parentNode.addDirectory(segment, entry)
				}
			}

			// Entry data isn't readable until all entries are, so buffers are set up afterwards
			if (segmentNode.type === 'directory') parentNode = segmentNode
		}
	}

	/**
	 * Files are read lazily, which with a ranged reader only fetches their bytes once asked for.
	 * Nested archives are read right away to build their trees.
	 */
	function handleFileNode(
		fileEntry: FileEntry,
		fileNode: FileZipTreeNode,
	): Promise<FileZipTreeNode> | undefined {
		if (fileEntry.encrypted || !zipArchiveExtensionRegex.test(fileNode.name)) {
			fileNode.setBuffer(() => fileEntry.arrayBuffer())
			return undefined
		}

		const archivePromise = fileEntry
			.arrayBuffer()
			.then(async (buffer) => {
				await fileNode.setBuffer(buffer)
				await readZipDataToNode(rootNode, fileNode, zipReaderFromBuffer(buffer), options)
				innerArchiveReadCallback?.(rootNode, fileNode)
				return fileNode
			})
			.catch((err) => {
				logger?.warn(`Unable to read inner archive '${fileNode.path}'`, { err: err })
				return fileNode
			})

		rootNode.trackInnerArchive(archivePromise)

		return archivePromise
	}

	logger?.debug(`Reading entries ${name} archive.`)

	const entries = await reader.getEntries({
		onprogress: (_progress, _total, entry) => handleEntry(entry as EntryData),
	})

	for (const entry of entries) {
		const fileNode = rootNode.fileEntries.get(
			[baseNode.path].concat(entry.filename.split('/')).join('/'),
		)
		if (fileNode == null) continue

		// TODO: SOMETHING HAS GONE WRONG IF WE HAVE A FILE NODE BUT NO FILE ENTRY
		const fileEntry = fileOrNull(entry)
		if (fileEntry == null) continue

		const bufferSetPromise = handleFileNode(fileEntry, fileNode)
		if (bufferSetPromise != null) zipFiles.push(bufferSetPromise)
	}

	logger?.debug(
		`Entries from ${name} archive have been read${zipFiles.length > 0 ? ', waiting for given inner Zip Files' : '.'}`,
	)

	// TODO: MAKE INNER JARS ASYNC I.E. UPDATE LATER TYPE DEAL AFTER THEY ARE DONE LOADING....
	if (zipFiles.length > 0 && innerArchiveReadCallback == null) await Promise.all(zipFiles)

	return baseNode
}

class ZipTreeNodeImpl extends ZipTreeNode {
	protected override createFileNode(
		root: RootZipTreeNodeImpl,
		parent: ZipNode,
		name: string,
		path: string,
		buffer: ArrayBufferSource | undefined,
		metadata: FileEntry,
	): FileZipTreeNode {
		return new FileZipTreeNodeImpl(root, parent, name, path, buffer, 'file', metadata)
	}

	protected override createNode(
		root: RootZipTreeNodeImpl,
		parent: ZipNode,
		name: string,
		path: string,
		metadata?: EntryMetaData,
	): ZipNode {
		return new ZipTreeNodeImpl(root, parent, name, path, 'directory', metadata) as ZipNode
	}
}

/**
 * Base Node type for a given Archive Tree acting as the holder for the base children
 * combined with extra info like maps for all entries and all file entries within the archive
 */
class RootZipTreeNodeImpl extends RootZipTreeNode {
	loadingInnerZipCount: number = 0
	onFinishedLoadingHolder: PromiseWithResolvers<void>
	isFullyLoaded = false
	isClosed = false
	/** Stops work still running for this archive (e.g. background downloads) once it is closed. */
	closeCallbacks: (() => void)[] = []
	private entriesRead = false
	/** The whole archive's bytes, once all of it is at hand (for a URL, once downloaded). */
	private content = Promise.withResolvers<ArrayBuffer>()
	private hashResults = new Map<HashType, Promise<string>>()

	setContent(data: ArrayBuffer) {
		this.content.resolve(data)
	}

	/** The whole archive's hash of the given type, once all of it is at hand; kept in `hashes` too. */
	hash(type: HashType): Promise<string> {
		let result = this.hashResults.get(type)
		if (!result) {
			result = this.content.promise.then(async (data) => {
				const value = await computeHash(data, type)
				this.hashes = { ...this.hashes, [type]: value }
				return value
			})
			this.hashResults.set(type, result)
		}
		return result
	}

	constructor(logger: Logger, key: UUID_V7, onFinishedLoadingHolder: PromiseWithResolvers<void>) {
		super(key, onFinishedLoadingHolder.promise)
		this.logger = logger
		this.onFinishedLoadingHolder = onFinishedLoadingHolder
	}

	/** Counts a nested archive (at any depth) as loading until the given read settles. */
	trackInnerArchive(read: Promise<unknown>) {
		this.loadingInnerZipCount += 1
		read.finally(() => {
			this.loadingInnerZipCount -= 1
			this.resolveIfLoaded()
		})
	}

	close() {
		if (this.isClosed) return
		this.isClosed = true
		for (const callback of this.closeCallbacks) callback()
	}

	/** Marks the top level entries as read, after which the root finishes once no nested archive is loading. */
	markEntriesRead() {
		this.entriesRead = true
		this.resolveIfLoaded()
	}

	private resolveIfLoaded() {
		if (this.isFullyLoaded || !this.entriesRead || this.loadingInnerZipCount > 0) return
		this.isFullyLoaded = true
		this.onFinishedLoadingHolder.resolve()
	}

	protected override createFileNode(
		root: RootZipTreeNode,
		parent: ZipNode,
		name: string,
		path: string,
		buffer: ArrayBufferSource | undefined,
		metadata: FileEntry,
	): FileZipTreeNode {
		return new FileZipTreeNodeImpl(
			root as RootZipTreeNodeImpl,
			parent as ZipNode,
			name,
			path,
			buffer,
			'file',
			metadata,
		)
	}

	protected override createNode(
		root: RootZipTreeNode,
		parent: ZipNode,
		name: string,
		path: string,
		metadata?: EntryMetaData,
	): ZipNode {
		return new ZipTreeNodeImpl(root, parent, name, path, 'directory', metadata) as ZipNode
	}
}

class FileZipTreeNodeImpl extends ZipTreeNodeImpl implements FileZipTreeNode {
	isSetup: boolean
	private arrayBuffer: LazyMutableValue<ArrayBuffer>

	declare type: 'file'
	declare metadata: FileEntry
	declare root: RootZipTreeNode

	fileTypeInfo: LazyValue<FileTypeInfo | null>

	asText: (options?: SourceOptions) => Promise<DecompilerResult>

	private readonly hashes = new Map<HashType, LazyValue<string>>()

	hash(type: HashType): Promise<string> {
		let value = this.hashes.get(type)
		if (!value) {
			value = this.arrayBuffer.lazyMap(async (buffer) => computeHash(buffer, type))
			this.hashes.set(type, value)
		}
		return value.get()
	}

	constructor(
		root: RootZipTreeNode,
		parent: ZipNode,
		name: string,
		path: string,
		buffer: ArrayBufferSource | undefined,
		type: 'file',
		metadata?: FileEntry,
	) {
		super(root, parent, name, path, type, metadata)

		if (buffer) {
			this.isSetup = true
			this.arrayBuffer = lazyMutableValue(() => arrayBufferFromSource(buffer))
		} else {
			this.isSetup = false
			this.arrayBuffer = lazyMutableValue(() => {
				return new Promise<ArrayBuffer>((resolve, reject) => {
					const id = setInterval(async () => {
						if (this.isSetup) {
							clearInterval(id)
							return this.getBuffer().then(resolve, reject)
						} else {
							root.logger?.debug(`Waiting for '${this.path}' buffer to be set`)
						}
					}, 100)
				})
			})
		}

		this.fileTypeInfo = this.arrayBuffer.lazyMap(
			async (buffer) => await getFileTypeInfo(this.path, buffer),
		)

		const map = new Map<Partial<SourceOptions>, LazyValue<DecompilerResult>>()

		;(this.arrayBuffer as LazyMutableValueImpl<unknown>)._onResetEvent.subscribe(() => map.clear())

		this.asText = async (options?: SourceOptions) => {
			let finalOptions: SourceOptions | undefined = undefined
			if (options == null || !map.has(options)) {
				const type = await this.fileTypeInfo.get()
				let isValid = false

				if (type != null) {
					if (
						this.name.endsWith('.class') ||
						type.mime == 'application/java-vm' ||
						type.extension == 'class' ||
						options?.forcedType == 'class'
					) {
						finalOptions = options ?? DEFAULT_DECOMPILE_OPTIONS
						if (!map.has(finalOptions)) {
							const lazyValue = this.arrayBuffer.lazyMap(async (buffer) => {
								let error: string | undefined = undefined

								const decompiler = (await messengerRegistry?.getMessenger(decompileMessengerType))!

								if (buffer != undefined) {
									return await decompiler.decompile({
										targetFile: {
											fullPath: this.path,
											name: this.name,
											content: buffer,
											root: this.root.key,
										} as ZipFileContent,
										resources: this.parent?.children
											.values()
											.map((entry) => {
												return { fullPath: entry.path, name: entry.name }
											})
											.toArray(),
										options: options,
									})
								}

								error = 'Unable to decompile file as Blob was found to be undefined!'

								return { error, wasDecompiled: false, asString: error } as DecompilerResult
							})
							map.set(options ?? DEFAULT_DECOMPILE_OPTIONS, lazyValue)
							return lazyValue.get()
						}
						isValid = true
					} else if (type.mime.startsWith('text') || options?.forcedType == 'text') {
						finalOptions = options ?? { forcedType: 'text' }
						if (!map.has(finalOptions)) {
							const lazyValue = this.arrayBuffer.lazyMap(async (buffer) => {
								const str = new TextDecoder().decode(buffer)
								return { source: str, wasDecompiled: true, asString: str } as DecompilerResult
							})
							map.set({ forcedType: 'text' }, lazyValue)
							return lazyValue.get()
						}
						isValid = true
					}
				}

				if (!isValid) {
					const msg =
						'Current file was found not to be either a class or text, no sources could be created'
					return Promise.resolve({
						source: msg,
						wasDecompiled: false,
						asString: msg,
					} as DecompilerResult)
				}
			} else {
				finalOptions = options
			}

			return map.get(finalOptions!)!.get()
		}
	}

	async setBuffer(buffer: ArrayBufferSource, options?: ZipWriterAddDataOptions) {
		this.arrayBuffer.updateGetter(() => arrayBufferFromSource(buffer))
		this.type = 'file'
		this.metadata = {
			...options,
			...this.metadata,
		} as FileEntry

		this.isSetup = true

		return this
	}

	getBuffer(): Promise<ArrayBuffer> {
		return this.arrayBuffer.get()
	}

	async getFileType(): Promise<FileTypeInfo | null> {
		return this.fileTypeInfo.get()
	}

	async getTypedBlob(): Promise<TypedBlob> {
		const buffer = await this.getBuffer()
		const fileType = await this.fileTypeInfo.get()
		const typedBlob = new Blob([buffer], { type: fileType?.mime }) as TypedBlob
		typedBlob.fileType = fileType
		return typedBlob
	}

	async getTypedBuffer(): Promise<TypedArrayBuffer> {
		const buffer = await this.getBuffer()
		const typedBuffer = buffer as TypedArrayBuffer
		typedBuffer.fileType = await this.fileTypeInfo.get()
		return typedBuffer
	}
}
