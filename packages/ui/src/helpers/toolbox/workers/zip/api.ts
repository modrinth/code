import type { ZipWriterAddDataOptions } from "@zip.js/zip.js";

import type { DecompilerResult } from "#ui/helpers/toolbox/decompile";
import type { FileDetails, HashedFileContent } from "#ui/helpers/toolbox/file";
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
			serverbound: NodeTarget,
			clientbound: { buffer: ArrayBuffer }
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
			clientbound: { node: RootZipTreeNode | undefined }
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

export const handlerConstructor: MessengerHandlerConstructor<ZipMessages, ZipHandler> = (messenger) => {
    const uuidToUrls = new Map<string, string[]>();
	const hashToUUID = new Map<Sha512, UUID_V7>();
	const uuidToOnFinishedLoadingHolder = new Map<UUID_V7, PromiseWithResolvers<void>>();
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
            return (await messenger.sendMessage({ channel: "node_read", ...getAsTarget(target)})).buffer
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

			const holder = Promise.withResolvers<void>();

			node.onFinishedLoading = msg.isFullyLoaded ? Promise.resolve() : holder.promise;

			const filteredURL = url?.split("?")[0];

            if (filteredURL) {
                this.urlToUUID.set(filteredURL, node.key);
                const urls = (uuidToUrls.has(node.key))
                    ? uuidToUrls.get(node.key)!
                    : (() => { const array = new Array<string>(); uuidToUrls.set(node.key, array); return array; })();

                urls.push(filteredURL);
            }

			if (sha512Hash != null) {
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

            return value;
        },
        closeAllZipTrees: async function () {
            const value = await messenger.sendMessage({ channel: "close_all_zip_trees" });

            this.zipByUUID.clear();
            this.urlToUUID.clear();
            uuidToUrls.clear();

            return value;
        },
        getZipTree: async function (value, isUrl, localOnly) {
            if (this.zipByUUID.has(value)) {
                return this.zipByUUID.get(value);
            }

            if (isUrl) {
                const filteredURL = value.split("?")[0];
                if (this.urlToUUID.has(filteredURL)) {
                    const hash = this.urlToUUID.get(value)!;
                    const node = this.zipByUUID.get(hash);

                    if (node) return node;
                }
            }

            if (localOnly) return undefined;

            const rawNode = (await messenger.sendMessage({ channel: "get_zip_tree", value: value })).node;

            if (rawNode == undefined) return undefined;

            const node = reconstituteRoot(messenger.handler, rawNode);

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
					const node = (nodeTarget.relative ? root.relativeFileEntries : root.fileEntries).get(nodeTarget.path);

					if (node) {
						for (const lazy of [node.sha1, node.sha256, node.sha512, node.fileTypeInfo]) {
							lazy.reset()
						}
						(node as unknown as Partial<DelegateFileZipTreeNode>).onDataUpdate?.()
					}
				}
			} else if (msg.type == 'node_loaded'){
				const root = messenger.handler.zipByUUID.get(msg.root.root);

				if (root != null) {
					const parentPath = msg.node.path.split("/").slice(0, -1).join("/");
					const parent = root.entries.get(parentPath)

					if (parent != null) {
						reconstituteNodes(this, root, parent, msg.node)
					}
				}
			} else if (msg.type == 'root_loaded') {
				const root = messenger.handler.zipByUUID.get(msg.root.root);

				if (root != null) {
					uuidToOnFinishedLoadingHolder.get(root.key)?.resolve()
					uuidToOnFinishedLoadingHolder.delete(root.key);
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

