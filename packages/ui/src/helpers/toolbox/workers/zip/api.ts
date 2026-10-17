import type { ZipWriterAddDataOptions } from "@zip.js/zip.js";

import type { DecompilerResult } from "#ui/helpers/toolbox/decompile";
import type { ContentHashes, FileDetails, HashedFileContent } from "#ui/helpers/toolbox/file";
import {
	type ArchiveUpdateTypes, DelegateFileZipTreeNode,
	type FileMatcher, type FileZipTreeNode, getAsTarget, type LazyDataAccess, type NodeTarget,
	reconstituteNodes,
	reconstituteRoot, type RootNodeTarget, type RootZipTreeNode, type SourceOptions,
	type UUID_V7, type ZipDataSource,
	type ZipHandler, type ZipNode, ZipTreeNode
} from "#ui/helpers/toolbox/zip";
import type {
	ClientMessenger,
	MessageTypes,
	MessengerHandlerConstructor
} from "#ui/helpers/toolbox/messenger";
import {EventStream} from "#ui/helpers/toolbox/misc";
import type {Sha512} from "#ui/helpers/toolbox/workers/zip/index.ts";

export type ZipMessages = MessageTypes<
	{
		lazy_read: {
			serverbound: { key: keyof LazyDataAccess } & NodeTarget,
			clientbound: { value: Awaited<ReturnType<LazyDataAccess[keyof LazyDataAccess]["get"]>> }
		},
		source_read: {
			serverbound: NodeTarget & { options?: SourceOptions }
			clientbound: DecompilerResult
		}
		node_read: {
			serverbound: NodeTarget & { optional?: boolean },
			clientbound: { buffer: ArrayBuffer | undefined }
		},
		node_write: {
			serverbound: NodeTarget & { options?: ZipWriterAddDataOptions } & { buffer: ArrayBuffer },
			clientbound: { wasWritten: boolean }
		},
		read_zip_tree: {
			serverbound: ZipDataSource,
			clientbound: { node: RootZipTreeNode, isFullyLoaded?: boolean}
		},
		get_zip_tree: {
			serverbound: { value: string | UUID_V7 },
			clientbound: { node: RootZipTreeNode | undefined, isFullyLoaded?: boolean }
		},
		collect_files: {
			serverbound: RootNodeTarget & { matcher?: FileMatcher },
			clientbound: { files: FileDetails[] }
		},
		collect_files_hashed: {
			serverbound: RootNodeTarget & { matcher?: FileMatcher, deduplicate?: boolean},
			clientbound: { files: HashedFileContent[] }
		},
		close_zip_tree: {
			serverbound: RootNodeTarget,
			clientbound: { err?: Error, wasClosed: boolean }
		},
		close_all_zip_trees: {
			serverbound: { },
			clientbound: { err?: Error, wasClosed: boolean }
		},
		archive_update: {
			//serverbound: {}
			clientbound: ArchiveUpdateTypes
		}
	}
>

/**
 * Adds a node read in the background (e.g. a nested archive's tree) to the client's copy of the
 * tree. A node the client already has (e.g. the nested archive's file) gets the loaded children.
 */
function mergeLoadedNode(handler: ZipHandler, root: RootZipTreeNode, loaded: ZipNode) {
	const existing = root.entries.get(loaded.path);
	if (existing != null) {
		for (const child of loaded.children.values()) {
			existing.children.set(child.name, reconstituteNodes<ZipNode>(handler, root, existing, child));
		}
		return;
	}

	const parentPath = loaded.path.split("/").slice(0, -1).join("/");
	const parent = parentPath === "" ? root : root.entries.get(parentPath);
	if (parent == null) return;

	parent.children.set(loaded.name, reconstituteNodes<ZipNode>(handler, root, parent, loaded));
}

export const handlerConstructor: MessengerHandlerConstructor<ZipMessages, ZipHandler> = (messenger) => {
    const uuidToUrls = new Map<string, string[]>();
	const hashToUUID = new Map<Sha512, UUID_V7>();
	const uuidToOnFinishedLoadingHolder = new Map<UUID_V7, PromiseWithResolvers<void>>();
	/** Roots whose loading finished, which may be reported before the tree itself is returned. */
	const finishedRoots = new Set<UUID_V7>();
	/** Hashes of roots, which may also be reported before the tree itself is returned. */
	const rootHashes = new Map<UUID_V7, ContentHashes>();

	function forgetRoot(key: UUID_V7) {
		finishedRoots.delete(key);
		uuidToOnFinishedLoadingHolder.delete(key);
		rootHashes.delete(key);
		for (const [hash, uuid] of hashToUUID) if (uuid === key) hashToUUID.delete(hash);
	}

	function trackLoading(node: RootZipTreeNode, isFullyLoaded: boolean | undefined) {
		node.hashes ??= rootHashes.get(node.key);
		if (node.sha512Hash != null) hashToUUID.set(node.sha512Hash, node.key);

		if (isFullyLoaded || finishedRoots.has(node.key)) {
			node.onFinishedLoading = Promise.resolve();
			return;
		}

		const holder = uuidToOnFinishedLoadingHolder.get(node.key) ?? Promise.withResolvers<void>();
		uuidToOnFinishedLoadingHolder.set(node.key, holder);
		node.onFinishedLoading = holder.promise;
	}

    return {
        zipByUUID: new Map(),
        urlToUUID: new Map(),
        readLazyData: async function <K extends keyof LazyDataAccess = any>(target: FileZipTreeNode | NodeTarget, key: K): Promise<Awaited<ReturnType<LazyDataAccess[K]["get"]>>> {
            return (await messenger.sendMessage({ channel: "lazy_read", ...getAsTarget(target), key: key })).value as Awaited<ReturnType<LazyDataAccess[K]["get"]>>;
        },
		readSource: async function (target: FileZipTreeNode | NodeTarget, options?: SourceOptions): Promise<DecompilerResult> {
			return (await messenger.sendMessage({ channel: 'source_read', ...getAsTarget(target), options: options}))
		},
		readNodeData: async (target: FileZipTreeNode | NodeTarget) => {
			const { buffer } = await messenger.sendMessage({ channel: "node_read", ...getAsTarget(target) });
			if (buffer == null) throw new Error(`No data was read for '${getAsTarget(target).path}'`);
			return buffer;
		},
		findNodeData: async (target: NodeTarget) => {
			return (await messenger.sendMessage({ channel: "node_read", ...target, optional: true })).buffer
		},
        getOrReadZipTree: async function (source: ZipDataSource) {
			const { url, sha512Hash } = source;
            let node: RootZipTreeNode | undefined;
			const filteredURL = url?.split("?")[0];
            if (filteredURL) node = await this.getZipTree(filteredURL, true);
            if (node == null && sha512Hash) node = await this.getZipTree(sha512Hash, false);
			if (node == null) node = await this.readZipTree(source);
            return node;
        },
        readZipTree: async function (source: ZipDataSource) {
			const { data_type, name, url, sha512Hash, buffer, rangeOptions } = source as ZipDataSource;
			const msg = (await messenger.sendMessage({
				channel: "read_zip_tree",
				data_type,
				name,
				url,
				sha512Hash,
				buffer: buffer != null ? await (typeof buffer === 'function' ? buffer() : Promise.resolve(buffer)) : undefined,
				rangeOptions: rangeOptions,
			}));
            const node = reconstituteRoot(messenger.handler, msg.node);

			trackLoading(node, msg.isFullyLoaded);

			const filteredURL = url?.split("?")[0];

            if (filteredURL) {
                this.urlToUUID.set(filteredURL, node.key);
                const urls = (uuidToUrls.has(node.key))
                    ? uuidToUrls.get(node.key)!
                    : (() => { const array = new Array<string>(); uuidToUrls.set(node.key, array); return array; })();

                urls.push(filteredURL);
            }

			if (sha512Hash != null && node.sha512Hash == null) {
				hashToUUID.set(sha512Hash, node.key)
			}

            this.zipByUUID.set(node.key, node);

            return node;
        },
		writeNodeData: async (target, buffer, options) => {
            return (await messenger.sendMessage({ channel: "node_write", ...getAsTarget(target), buffer, options})).wasWritten;
        },
        collectFiles: async (root, matcher) => {
            return (await messenger.sendMessage({ channel: "collect_files", matcher: matcher, root: (root as RootZipTreeNode).key ?? root})).files
        },
        collectHashedFiles: async (root, matcher, deduplicate)=> {
            return (await messenger.sendMessage({ channel: "collect_files_hashed", matcher: matcher, root: (root as RootZipTreeNode).key ?? root, deduplicate: deduplicate})).files
        },
        closeZipTree: async function (root){
            const hash: UUID_V7 = (root as RootZipTreeNode).key ?? root;
            const value = await messenger.sendMessage({ channel: "close_zip_tree", root: (root as RootZipTreeNode).key ?? root});

            const urls = uuidToUrls.get(hash);
            if (urls) for (const url of urls) this.urlToUUID.delete(url);
            uuidToUrls.delete(hash);
            this.zipByUUID.delete(hash);
            forgetRoot(hash);

            return value;
        },
        closeAllZipTrees: async function () {
            const value = await messenger.sendMessage({ channel: "close_all_zip_trees" });

            this.zipByUUID.clear();
            this.urlToUUID.clear();
            uuidToUrls.clear();
            finishedRoots.clear();
            uuidToOnFinishedLoadingHolder.clear();
            rootHashes.clear();
            hashToUUID.clear();

            return value;
        },
        getZipTree: async function (value, isUrl, localOnly) {
            if (this.zipByUUID.has(value)) {
                return this.zipByUUID.get(value);
            }

            if (isUrl) {
                const filteredURL = value.split("?")[0];
                if (this.urlToUUID.has(filteredURL)) {
                    const hash = this.urlToUUID.get(filteredURL)!;
                    const node = this.zipByUUID.get(hash);

                    if (node) return node;
                }
            }

            if (localOnly) return undefined;

            const response = await messenger.sendMessage({ channel: "get_zip_tree", value: value });
            const rawNode = response.node;

            if (rawNode == undefined) return undefined;

            const node = reconstituteRoot(messenger.handler, rawNode);

			trackLoading(node, response.isFullyLoaded);

            if (isUrl) {
                const filteredURL = value.split("?")[0];
                this.urlToUUID.set(filteredURL, node.key);
                if (uuidToUrls.has(node.key)) uuidToUrls.set(node.key, [])
                uuidToUrls.get(node.key)?.push(filteredURL);
            }

            this.zipByUUID.set(node.key, node);

            return node;
        },
		archiveUpdate: function (msg) {
			if (msg.type == 'updated_node_data') {
				const nodeTarget = msg.node;
				const root = messenger.handler.zipByUUID.get(nodeTarget.root);

				if (root != null) {
					const node = root.findFileEntry(nodeTarget.path, nodeTarget.relative);

					if (node) {
						for (const lazy of [node.sha1, node.sha256, node.sha512, node.murmur2, node.fileTypeInfo]) {
							lazy.reset()
						}
						(node as unknown as Partial<DelegateFileZipTreeNode>).onDataUpdate?.()
					}
				}
			} else if (msg.type == 'node_loaded'){
				const root = messenger.handler.zipByUUID.get(msg.root.root);

				if (root != null) mergeLoadedNode(this, root, msg.node);
			} else if (msg.type == 'root_loaded') {
				finishedRoots.add(msg.root.root);
				uuidToOnFinishedLoadingHolder.get(msg.root.root)?.resolve()
				uuidToOnFinishedLoadingHolder.delete(msg.root.root);
			} else if (msg.type == 'root_hashed') {
				const key = msg.root.root;
				rootHashes.set(key, msg.hashes);
				hashToUUID.set(msg.sha512Hash, key);
				if (msg.expectedHash != null && hashToUUID.get(msg.expectedHash) === key) hashToUUID.delete(msg.expectedHash);

				const root = messenger.handler.zipByUUID.get(key);
				if (root != null) {
					root.hashes = msg.hashes;
				}
			}

			this.onArchiveUpdate.stream.sink(msg);
		},
		onArchiveUpdate: new EventStream<(msg: ArchiveUpdateTypes) => void>((callbacks) => (msg) => {
			for(const callback of callbacks) {
				callback(msg);
			}
		}).source
    }
};

