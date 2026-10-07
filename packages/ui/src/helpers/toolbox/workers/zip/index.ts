
import {
	type FileContentPromise, type FileDetails, type FileTypeInfo,
	type HashedFileContent, resolveContent
} from "#ui/helpers/toolbox/file";
import { type MessageTypes, type ServerCompoundInputHandler, setupAsCompoundServer } from "#ui/helpers/toolbox/messenger";
import { zipHandlerType } from "#ui/helpers/toolbox/workers";
import type {  ZipMessages } from "#ui/helpers/toolbox/workers/zip/api";
import type {
	LazyDataAccess,
	NodeTarget,
	RootZipTreeNode,
	Sha512,
	StringMatcher,
	ZipTreeNode } from "#ui/helpers/toolbox/zip";
import { readZipTree } from "#ui/helpers/toolbox/zip/impl";

const zipArchivesByHash = new Map<Sha512, RootZipTreeNode>();

const urlToHash = new Map<string, Sha512>();
const hashToUrls = new Map<string, string[]>();

function getNodeFrom(target: NodeTarget) {
	if (!zipArchivesByHash || zipArchivesByHash.size == 0) throw new Error(`Unable to locate node as no known root tree is within the worker!`);

	const root = zipArchivesByHash.get(target.root);
	if (root == undefined) throw new Error(`Unable to get node '${target.root}:${target.path}' as their is no active root with the hash given namespace hash!`);

	const node = (target.relative ? root.relativeFileEntries : root.fileEntries).get(target.path);
	if (node == undefined) throw new Error(`Unable to get node '${target.root}:${target.path}' as their is no active node with the given path!`);

	return node;
}

function getRootFrom(target: Sha512) {
	if (!zipArchivesByHash || zipArchivesByHash.size == 0) throw new Error(`Unable to locate node as no known root tree is within the worker!`);

	const root = zipArchivesByHash.get(target);
	if (root == undefined) throw new Error(`Unable to get root '${target}' as their is no active root with the hash given hash!`);

	return root;
}

function getZipNode(value: string) {
	if (zipArchivesByHash.has(value)) {
		return zipArchivesByHash.get(value);
	}

	if (urlToHash.has(value)) {
		const hash = urlToHash.get(value)!;
		const node = zipArchivesByHash.get(hash);

		if (node) return node;
	}

	return undefined;
}

export function getMessageHandler<P = MessageTypes<ZipMessages>>(): ServerCompoundInputHandler<ZipMessages, P> {
	return messageHandler as unknown as ServerCompoundInputHandler<ZipMessages, P>;
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

export const messageHandler: ServerCompoundInputHandler<ZipMessages> = function (messenger, msg, sender) {
	return msg.resolve({
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
			return {
				buffer: buffer
			};
		},
		node_write: async (msg) => {
			const node = getNodeFrom(msg);
			await node.setBuffer(async () => msg.buffer, msg.nodeType, msg.options);
			return {
				wasWritten: true
			};
		},
		read_zip_tree: async (msg) => {
			let tree: RootZipTreeNode | undefined = undefined;

			if (msg.hash) tree = getZipNode(msg.hash);
			if (tree == null && msg.url) tree = getZipNode(msg.url);
			if (tree == null) {
				tree = await readZipTree(messenger.logger, msg.buffer, { name: msg.name, logger: messenger.logger });

				zipArchivesByHash.set(tree.key, tree);

				if (msg.url) {
					urlToHash.set(msg.url, tree.key);

					const urls = (hashToUrls.has(tree.key))
						? hashToUrls.get(tree.key)!
						: (() => { const array = new Array<string>(); hashToUrls.set(tree.key, array); return array; })();

					urls.push(msg.url);
				}
			}

			const copiedTree = copy(tree);

			return {
				node: copiedTree
			};
		},
		get_zip_tree: async (msg) => {
			const node = getZipNode(msg.value);
			const copiedTree = node ? copy(node) : undefined;

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
			zipArchivesByHash.delete(msg.root);

			const urls = hashToUrls.get(msg.root);
			if (urls) for (const url of urls) urlToHash.delete(url);
			hashToUrls.delete(msg.root);

			return {
				err: undefined,
				wasClosed: true,
			}
		},
		close_all_zip_trees: async () => {
			zipArchivesByHash.clear();
			urlToHash.clear();
			hashToUrls.clear();

			return {
				err: undefined,
				wasClosed: true,
			}
		},
		update_node_data: async (target) => {
			if (sender != null) {
				const connectedMessengers = messenger.getActiveMessengers();

				for (const connectedMessenger of connectedMessengers) {
					messenger.sendOutputMessage(connectedMessenger, Promise.resolve(target))
				}
			}

			return target;
		}
	})
};

export function copy(node: RootZipTreeNode): RootZipTreeNode {
	const cloneNode = { children: new Map(), key: node.key } as RootZipTreeNode;
	return copyNodeData(cloneNode, node);
}

function copyNodeData<N extends ZipTreeNode>(clone: N, oldNode: N): N {
	for (const entry of oldNode.children.values()) {
		// Required to strip the functions that can not cross the message barrier
		const cloneMetaData = {
			...entry.metadata,
			getData: undefined,
			arrayBuffer: undefined,
		};

		const entryClone = { name: entry.name, path: entry.path, type: entry.type, metadata: cloneMetaData, children: new Map() } as any as ZipTreeNode

		clone.children.set(entry.name, entryClone)

		copyNodeData(entryClone, entry);
	}

	return clone;
}

setupAsCompoundServer(zipHandlerType, self as unknown as Worker, getMessageHandler());
