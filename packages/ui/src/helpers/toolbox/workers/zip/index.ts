
import {
	type FileContentPromise, type FileDetails, type FileTypeInfo, hashBuffer,
	type HashedFileContent, resolveContent
} from "#ui/helpers/toolbox/file";
import {
	isWorker,
	type ServerMessageHandler,
	setupAsServer
} from "#ui/helpers/toolbox/messenger";
import { zipHandlerType } from "#ui/helpers/toolbox/workers";
import type {  ZipMessages } from "#ui/helpers/toolbox/workers/zip/api";
import {
	arrayBufferFromSource,
	type LazyDataAccess,
	type NodeTarget, type RootNodeTarget,
	type RootZipTreeNode,
	type StringMatcher,
	type UUID_V7,
	type ZipNode,
} from "#ui/helpers/toolbox/zip";
import {readZipTreeFromBuffer, readZipTreeFromURL} from "#ui/helpers/toolbox/zip/impl";

export type Sha512 = string;

const zipByUUID = new Map<UUID_V7, RootZipTreeNode>();

const hashToUUID = new Map<Sha512, UUID_V7>();

const urlToUUID = new Map<string, UUID_V7>();
const UUIDToUrls = new Map<UUID_V7, string[]>();

function getNodeFrom(target: NodeTarget) {
	if (!zipByUUID || zipByUUID.size == 0) throw new Error(`Unable to locate node as no known root tree is within the worker!`);

	const root = zipByUUID.get(target.root);
	if (root == undefined) throw new Error(`Unable to get node '${target.root}:${target.path}' as their is no active root with the hash given namespace hash!`);

	const node = (target.relative ? root.relativeFileEntries : root.fileEntries).get(target.path);
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

export const messageHandler: ServerMessageHandler<ZipMessages> = (messenger, msg, resolver, sender)=>  {
	return resolver(msg, {
		lazy_read: async (msg) => {
			const node = getNodeFrom(msg);

			return {
				key: msg.key,
				value: await (node as LazyDataAccess)[msg.key].get()
			};
		},
		source_read: (msg) => {
			return getNodeFrom(msg).asText(msg.options);
		},
		node_read: async (msg) => {
			const node = getNodeFrom(msg);

			const buffer = await node.getBuffer();
			console.log(buffer);
			return {
				buffer: buffer
			};
		},
		node_write: async (msg) => {
			const node = getNodeFrom(msg);
			await node.setBuffer(async () => msg.buffer, msg.options);
			messenger.forActiveMessengersUnless(sender, (connection) => {
				messenger.sendClientMessage(connection, async () => {
					return {
						channel: "archive_update",
						type: "updated_node_data",
						node: {
							root: node.root?.key!,
							path: node.path,
						}
					}
				})
			})
			return {
				wasWritten: true
			};
		},
		read_zip_tree: async (msg) => {
			let tree: RootZipTreeNode | undefined = undefined;
			let isFullyLoaded = sender != null;

			if (msg.sha512Hash) tree = getZipNode(msg.sha512Hash);
			if (tree == null && msg.url) tree = getZipNode(msg.url);
			if (tree == null) {
				if (msg.data_type == 'buffer') {
					const buffer = await arrayBufferFromSource(msg.buffer);
					const hash = await hashBuffer(buffer)

					tree = await readZipTreeFromBuffer(messenger.logger, buffer, {
						name: msg.name,
						logger: messenger.logger,
						innerArchiveReadCallback: sender != null ? (root, node) => {
							messenger.forActiveMessengersUnless(sender, (connection) => {
								messenger.sendClientMessage(connection, async () => {
									return {
										channel: 'archive_update',
										type: 'node_loaded',
										root: { root: root.key } as RootNodeTarget,
										node: copyChildNode(node)
									}
								})
							})
						} : undefined
					});

					hashToUUID.set(hash, tree.key)
				} else {
					tree = await readZipTreeFromURL(messenger.logger, msg.url, {
						name: msg.name,
						logger: messenger.logger,
						innerArchiveReadCallback: (root, node) => {
							messenger.forActiveMessengersUnless(sender, (connection) => {
								messenger.sendClientMessage(connection, async () => {
									return {
										channel: 'archive_update',
										type: 'node_loaded',
										root: { root: root.key } as RootNodeTarget,
										node: copyChildNode(node)
									}
								})
							})
						},
						rangeOptions: {
							...msg.rangeOptions,
							onProgress(current, total) {
								messenger.forActiveMessengersUnless(sender, (connection) => {
									messenger.sendClientMessage(connection, async () => {
										return {
											channel: 'archive_update',
											type: 'archive_download',
											root: { root: tree!.key } as RootNodeTarget,
											current: current,
											total: total
										}
									})
								})
							}
						},
					});

					if (msg.sha512Hash) zipByUUID.set(msg.sha512Hash, tree);
				}

				tree!.onFinishedLoading.then(() => {
					isFullyLoaded = true;
					messenger.forActiveMessengersUnless(sender, (connection) => {
						messenger.sendClientMessage(connection, async () => {
							return {
								channel: 'archive_update',
								type: 'root_loaded',
								root: {root: tree!.key} as RootNodeTarget
							}
						})
					})
				})

				zipByUUID.set(tree.key, tree);

				if (msg.url) {
					urlToUUID.set(msg.url, tree.key);

					const urls = (UUIDToUrls.has(tree.key))
						? UUIDToUrls.get(tree.key)!
						: (() => { const array = new Array<string>(); UUIDToUrls.set(tree.key, array); return array; })();

					urls.push(msg.url);
				}
			}

			return { node: copyRoot(tree), isFullyLoaded: isFullyLoaded };
		},
		get_zip_tree: async (msg) => {
			const node = getZipNode(msg.value);
			const copiedTree = node ? copyRoot(node) : undefined;

			return {
				node: copiedTree
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
			zipByUUID.delete(msg.root);

			const urls = UUIDToUrls.get(msg.root);
			if (urls) for (const url of urls) urlToUUID.delete(url);
			UUIDToUrls.delete(msg.root);

			return {
				err: undefined,
				wasClosed: true,
			}
		},
		close_all_zip_trees: async () => {
			zipByUUID.clear();
			urlToUUID.clear();
			UUIDToUrls.clear();

			return {
				err: undefined,
				wasClosed: true,
			}
		},
	})
};

export function copyRoot(node: RootZipTreeNode): RootZipTreeNode {
	const cloneNode = { children: new Map(), key: node.key } as RootZipTreeNode;

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
