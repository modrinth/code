
import {
	type ContentHashes, type FileContentPromise, type FileDetails, type FileTypeInfo,
	type HashedFileContent, hashContent, resolveContent
} from "#ui/helpers/toolbox/file";
import { v7 as uuidV7 } from 'uuid';

import {
	isWorker,
	type Messenger,
	type ServerMessageHandler,
	type ServerMessenger,
	setupAsServer
} from "#ui/helpers/toolbox/messenger";
import { zipHandlerType } from "#ui/helpers/toolbox/workers";
import type {  ZipMessages } from "#ui/helpers/toolbox/workers/zip/api";
import {
	type ArchiveUpdateTypes,
	arrayBufferFromSource,
	type LazyDataAccess,
	type NodeTarget, type RootNodeTarget,
	type RootZipTreeNode,
	type StringMatcher,
	type UUID_V7,
	type ZipNode,
} from "#ui/helpers/toolbox/zip";
import {
	type ReadExtraOptions,
	readZipTreeFromBuffer,
	readZipTreeFromURL
} from "#ui/helpers/toolbox/zip/impl";

export type Sha512 = string;

const zipByUUID = new Map<UUID_V7, RootZipTreeNode>();

const hashToUUID = new Map<Sha512, UUID_V7>();

const urlToUUID = new Map<string, UUID_V7>();
const UUIDToUrls = new Map<UUID_V7, string[]>();

/**
 * Finds the targeted file. Nested archives are read in the background, so a file not known yet is
 * looked for again once the archive finishes loading, before it's deemed missing.
 */
async function findNodeFrom(target: NodeTarget) {
	if (!zipByUUID || zipByUUID.size == 0) throw new Error(`Unable to locate node as no known root tree is within the worker!`);

	const root = zipByUUID.get(target.root);
	if (root == undefined) throw new Error(`Unable to get node '${target.root}:${target.path}' as their is no active root with the hash given namespace hash!`);

	const node = root.findFileEntry(target.path, target.relative);
	if (node != undefined || (root as Partial<{ isFullyLoaded: boolean }>).isFullyLoaded !== false) return node;

	await root.onFinishedLoading;
	return root.findFileEntry(target.path, target.relative);
}

async function getNodeFrom(target: NodeTarget) {
	const node = await findNodeFrom(target);
	if (node == undefined) throw new Error(`Unable to get node '${target.root}:${target.path}' as their is no active node with the given path!`);

	return node;
}

function getRootFrom(target: UUID_V7) {
	if (!zipByUUID || zipByUUID.size == 0) throw new Error(`Unable to locate node as no known root tree is within the worker!`);

	const root = zipByUUID.get(target);
	if (root == undefined) throw new Error(`Unable to get root '${target}' as their is no active root with the hash given hash!`);

	return root;
}

function getZipNode(value: string) {
	if (zipByUUID.has(value)) {
		return zipByUUID.get(value);
	}

	const hashedUUID = hashToUUID.get(value);
	if (hashedUUID != null && zipByUUID.has(hashedUUID)) return zipByUUID.get(hashedUUID);

	if (urlToUUID.has(value)) {
		const hash = urlToUUID.get(value)!;
		const node = zipByUUID.get(hash);

		if (node) return node;
	}

	return undefined;
}

async function isValidFor(path: string, fileType: () => Promise<FileTypeInfo | null>, pathMatcher?: StringMatcher, mimeMatcher?: StringMatcher) {
	if (pathMatcher) {
		const negate = pathMatcher.negate;
		if (pathMatcher.regex && (negate ? path.match(pathMatcher.regex) : !path.match(pathMatcher.regex))) return false;
		if (pathMatcher.startsWith && (negate ? path.startsWith(pathMatcher.startsWith) : !path.startsWith(pathMatcher.startsWith))) return false;
		if (pathMatcher.includes && (negate ? path.includes(pathMatcher.includes) : !path.startsWith(pathMatcher.includes))) return false;
		if (pathMatcher.endsWith && (negate ? path.endsWith(pathMatcher.endsWith) : !path.endsWith(pathMatcher.endsWith))) return false;
	}

	if (mimeMatcher) {
		const info = await fileType();
		if (info == null) return false;
		const mime = info.mime;
		const negate = mimeMatcher.negate;
		if (mimeMatcher.regex && (negate ? mime.match(mimeMatcher.regex) : !mime.match(mimeMatcher.regex))) return false;
		if (mimeMatcher.startsWith && (negate ? mime.startsWith(mimeMatcher.startsWith) : !mime.startsWith(mimeMatcher.startsWith))) return false;
		if (mimeMatcher.includes && (negate ? mime.includes(mimeMatcher.includes) : !mime.includes(mimeMatcher.includes))) return false;
		if (mimeMatcher.endsWith && (negate ? mime.endsWith(mimeMatcher.endsWith) : !mime.endsWith(mimeMatcher.endsWith))) return false;
	}

	return true;
}

/** Stops anything still running for the archive, such as fetching the rest of it. */
function closeRoot(root: RootZipTreeNode | undefined) {
	(root as Partial<{ close: () => void }> | undefined)?.close?.();
}

/**
 * Maps the archive's actual hash to it, dropping the hash it was opened with when that turns out to
 * be wrong, and lets every connection know.
 */
function onArchiveHashed(messenger: ServerMessenger<ZipMessages>, root: RootZipTreeNode, hashes: ContentHashes, expectedHash?: Sha512) {
	const sha512Hash = hashes.sha512;
	root.hashes = hashes;
	hashToUUID.set(hashes.sha512, root.key);

	const mismatched = expectedHash != null && expectedHash !== sha512Hash;
	if (mismatched) {
		messenger.logger.warn(`Archive '${root.key}' was opened with SHA-512 '${expectedHash}' but its contents hash to '${sha512Hash}'`);
		if (hashToUUID.get(expectedHash) === root.key) hashToUUID.delete(expectedHash);
	}

	broadcastArchiveUpdate(messenger, {
		type: 'root_hashed',
		root: { root: root.key },
		sha512Hash: sha512Hash,
		hashes: hashes,
		expectedHash: mismatched ? expectedHash : undefined,
	});
}

/**
 * Sends an archive update to every connection, i.e. the main thread (the server itself) and any
 * peer worker, except the one given.
 */
function broadcastArchiveUpdate(messenger: ServerMessenger<ZipMessages>, update: ArchiveUpdateTypes, except?: Messenger<ZipMessages>) {
	messenger.forConnections((connection) => {
		messenger.sendClientMessage(connection, async () => ({ channel: 'archive_update', ...update }))
	}, except);
}

export const messageHandler: ServerMessageHandler<ZipMessages> = (messenger, msg, resolver, sender)=>  {
	return resolver(msg, {
		lazy_read: async (msg) => {
			const node = await getNodeFrom(msg);

			return {
				key: msg.key,
				value: await (node as LazyDataAccess)[msg.key].get()
			};
		},
		source_read: async (msg) => {
			return (await getNodeFrom(msg)).asText(msg.options);
		},
		node_read: async (msg) => {
			const node = msg.optional ? await findNodeFrom(msg) : await getNodeFrom(msg);

			return {
				buffer: node ? await node.getBuffer() : undefined
			};
		},
		node_write: async (msg) => {
			const node = await getNodeFrom(msg);
			await node.setBuffer(async () => msg.buffer, msg.options);
			broadcastArchiveUpdate(messenger, {
				type: "updated_node_data",
				node: {
					root: node.root?.key!,
					path: node.path,
				}
			}, sender)
			return {
				wasWritten: true
			};
		},
		read_zip_tree: async (msg) => {
			let tree: RootZipTreeNode | undefined = undefined;

			if (msg.sha512Hash) tree = getZipNode(msg.sha512Hash);
			if (tree == null && msg.url) tree = getZipNode(msg.url);
			if (tree == null) {
				const key = uuidV7();
				const readOptions: ReadExtraOptions = {
					key: key,
					name: msg.name,
					logger: messenger.logger,
					innerArchiveReadCallback: (_root: RootZipTreeNode, node: ZipNode) => broadcastArchiveUpdate(messenger, {
						type: 'node_loaded',
						root: { root: key },
						node: copyChildNode(node)
					}),
				};

				if (msg.data_type == 'buffer') {
					const buffer = await arrayBufferFromSource(msg.buffer);
					const hashes = await hashContent(buffer)

					tree = await readZipTreeFromBuffer(messenger.logger, buffer, readOptions);

					onArchiveHashed(messenger, tree, hashes, msg.sha512Hash);
				} else {
					const url = msg.url;
					tree = await readZipTreeFromURL(messenger.logger, url, {
						...readOptions,
						archiveHashedCallback: (root, hashes) => onArchiveHashed(messenger, root, hashes, msg.sha512Hash),
						rangeOptions: {
							...msg.rangeOptions,
							onProgress(current, total) {
								broadcastArchiveUpdate(messenger, {
									type: 'archive_download',
									root: key,
									url: url,
									current: current,
									total: total
								})
							}
						},
					});

					// Trusted until the download finishes and the archive is hashed, unless that already happened
					if (msg.sha512Hash && tree.sha512Hash == null) hashToUUID.set(msg.sha512Hash, tree.key);
				}

				tree.onFinishedLoading.then(() => broadcastArchiveUpdate(messenger, { type: 'root_loaded', root: { root: key } }))

				zipByUUID.set(tree.key, tree);

				if (msg.url) {
					urlToUUID.set(msg.url, tree.key);

					const urls = (UUIDToUrls.has(tree.key))
						? UUIDToUrls.get(tree.key)!
						: (() => { const array = new Array<string>(); UUIDToUrls.set(tree.key, array); return array; })();

					urls.push(msg.url);
				}
			}

			return { node: copyRoot(tree), isFullyLoaded: (tree as Partial<{ isFullyLoaded: boolean }>).isFullyLoaded ?? true };
		},
		get_zip_tree: async (msg) => {
			const node = getZipNode(msg.value);
			const copiedTree = node ? copyRoot(node) : undefined;

			return {
				node: copiedTree,
				isFullyLoaded: (node as Partial<{ isFullyLoaded: boolean }> | undefined)?.isFullyLoaded ?? true
			};
		},
		collect_files: async (msg) => {
			const root = getRootFrom(msg.root);

			const mimeMatcher = msg.matcher?.mime;
			const pathMatcher = msg.matcher?.path;

			const entries = root.fileEntries;

			if (mimeMatcher || pathMatcher) {
				const files: FileDetails[] = [];
				let index = 0;
				const size = entries.size;

				const promises: Promise<void>[] = [];

				for (const file of entries.values()) {
					// TODO: ADD ABILITY TO TELL WORKERS TO NOT DEBUG LOG THOUGH A MESSAGE?
					messenger.logger.debug(`[File Collector]: Looking at ${index}/${size} '${file.path}'`);

					index += 1;

					promises.push(
						isValidFor(file.path, async () => file.fileTypeInfo.get(), pathMatcher, mimeMatcher)
							.then((value) => { if (value) files.push({ fullPath: file.path, name: file.name }); })
					)
				}

				await Promise.all(promises);

				return {
					files: files,
				};
			} else {
				return {
					files: entries.values().map(entry => { return { fullPath: entry.path, name: entry.name }; }).toArray(),
				};
			}
		},
		collect_files_hashed: async (msg) => {
			const root = getRootFrom(msg.root);

			const mimeMatcher = msg.matcher?.mime;
			const pathMatcher = msg.matcher?.path;

			const entries = root.fileEntries;

			const hashes = new Set<string>();

			let files: HashedFileContent[];

			if (mimeMatcher || pathMatcher || msg.deduplicate) {
				files = [];
				let index = 0;
				const baseFiles: (FileContentPromise & { sha512: () => Promise<string> }) [] = [];

				const filterPromises: Promise<void>[] = [];

				for (const file of entries.values()) {
					// TODO: ADD ABILITY TO TELL WORKERS TO NOT DEBUG LOG THOUGH A MESSAGE?
					//this.logger().debug(`[File Collector]: Looking at ${index}/${size} '${file.path}'`);

					index += 1;

					filterPromises.push(
						isValidFor(file.path, async () => file.fileTypeInfo.get(), pathMatcher, mimeMatcher)
							.then((value) => {
								if (value) baseFiles.push({ fullPath: file.path, name: file.name, content: () => file.getBuffer(), sha512: () => file.sha512.get()});
							})
					)
				}

				await Promise.all(filterPromises);

				const baseFileContent = await Promise.all(baseFiles.map((filePromise) => resolveContent(filePromise)));

				const duplicateCheckPromises: Promise<void>[] = [];

				async function isDuplicateFile(file: { sha512: () => Promise<string>}) {
					const sha512 = await file.sha512();

					if (msg.deduplicate) {
						if (hashes.has(sha512)) return { result: true, sha512: sha512 };

						hashes.add(sha512)
					}

					return {result: false, sha512: sha512 };
				}

				for (const file of baseFileContent) {
					duplicateCheckPromises.push(
						isDuplicateFile(file)
							.then((value) => {if (!value.result) files.push({ ...file, sha512: value.sha512 });})
					)
				}

				if (duplicateCheckPromises.length > 0) await Promise.all(duplicateCheckPromises);
			} else {
				files = await Promise.all(
					entries.values().map(async file => {
						const buffer = await file.getBuffer();
						const sha512 = await file.sha512.get();

						return { fullPath: file.path, name: file.name, content: buffer, sha512: sha512};
					}).toArray()
				);
			}

			return { files: files };
		},
		close_zip_tree: async (msg) => {
			closeRoot(zipByUUID.get(msg.root));
			zipByUUID.delete(msg.root);

			const urls = UUIDToUrls.get(msg.root);
			if (urls) for (const url of urls) urlToUUID.delete(url);
			UUIDToUrls.delete(msg.root);

			for (const [hash, key] of hashToUUID) if (key === msg.root) hashToUUID.delete(hash);

			return {
				err: undefined,
				wasClosed: true,
			}
		},
		close_all_zip_trees: async () => {
			for (const root of zipByUUID.values()) closeRoot(root);
			zipByUUID.clear();
			urlToUUID.clear();
			UUIDToUrls.clear();
			hashToUUID.clear();

			return {
				err: undefined,
				wasClosed: true,
			}
		},
	})
};

export function copyRoot(node: RootZipTreeNode): RootZipTreeNode {
	const cloneNode = {
		children: new Map(),
		key: node.key,
		name: node.name,
		path: node.path,
		type: node.type,
		metadata: { directory: true },
		sha512Hash: node.sha512Hash,
		hashes: node.hashes,
	} as unknown as RootZipTreeNode;

	return copyNodeData(cloneNode, node);
}

export function copyChildNode(node: ZipNode): ZipNode {
	// Required to strip the functions that can not cross the message barrier
	const cloneMetaData = {
		...node.metadata,
		getData: undefined,
		arrayBuffer: undefined,
	};

	const cloneNode = { name: node.name, path: node.path, type: node.type, metadata: cloneMetaData, children: new Map() } as any as ZipNode

	return copyNodeData(cloneNode, node);
}

function copyNodeData<N extends ZipNode>(clone: N, oldNode: N): N {
	for (const entry of oldNode.children.values()) {
		clone.children.set(entry.name, copyChildNode(entry))
	}

	return clone;
}

/** In development the handler is also loaded on the main thread as a bypass, where no server is set up. */
if (isWorker()) setupAsServer<ZipMessages>(zipHandlerType, self as unknown as Worker, messageHandler);
