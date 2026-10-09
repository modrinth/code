import type { ZipWriterAddDataOptions } from "@zip.js/zip.js";

import type { DecompilerResult } from "#ui/helpers/toolbox/decompile";
import type { FileDetails, HashedFileContent } from "#ui/helpers/toolbox/file";
import type { ZipHandlerMessenger } from "#ui/helpers/toolbox/workers";
import {
	type FileMatcher, type FileZipTreeNode, getAsTarget, type LazyDataAccess, type NodeTarget,
	reconstituteRoot, type RootNodeTarget, type RootZipTreeNode, type Sha512, type SourceOptions,
	type ZipHandler
} from "#ui/helpers/toolbox/zip";
import type {MessageTypes} from "#ui/helpers/toolbox/messenger";

export interface DataInput {
    buffer: ArrayBuffer
}

export type ZipMessages = MessageTypes<
	{
		lazy_read: {
			input: { key: keyof LazyDataAccess } & NodeTarget,
			output: { value: Awaited<ReturnType<LazyDataAccess[keyof LazyDataAccess]["get"]>> }
		},
		source_read: {
			input: NodeTarget & { options?: SourceOptions }
			output: DecompilerResult
		}
		node_read: {
			input: NodeTarget,
			output: { buffer: ArrayBuffer }
		},
		node_write: {
			input: NodeTarget & { nodeType?: "zip" | "file", options?: ZipWriterAddDataOptions } & DataInput,
			output: { wasWritten: boolean }
		},
		read_zip_tree: {
			input: { name?: string, url?: string, hash?: string } & DataInput,
			output: { node: RootZipTreeNode }
		},
		get_zip_tree: {
			input: { value: string | Sha512 },
			output: { node: RootZipTreeNode | undefined }
		},
		collect_files: {
			input: RootNodeTarget & { matcher?: FileMatcher },
			output: { files: FileDetails[] }
		},
		collect_files_hashed: {
			input: RootNodeTarget & { matcher?: FileMatcher, deduplicate?: boolean},
			output: { files: HashedFileContent[] }
		},
		close_zip_tree: {
			input: RootNodeTarget,
			output: { err?: Error, wasClosed: boolean }
		},
		close_all_zip_trees: {
			input: { },
			output: { err?: Error, wasClosed: boolean }
		},
		update_node_data: {
			input: NodeTarget,
			output: NodeTarget
		}
	}
>

export const handlerConstructor: (messenger: ZipHandlerMessenger) => ZipHandler = (messenger: ZipHandlerMessenger) => {
    const hashToUrls = new Map<string, string[]>();
    return {
        zipArchivesByHash: new Map(),
        urlToHash: new Map(),
        readLazyData: async function <K extends keyof LazyDataAccess = any>(target: FileZipTreeNode | NodeTarget, key: K): Promise<Awaited<ReturnType<LazyDataAccess[K]["get"]>>> {
            return (await messenger.sendMessage({ channel: "lazy_read", ...getAsTarget(target), key: key })).value as Awaited<ReturnType<LazyDataAccess[K]["get"]>>;
        },
		readSource: async function (target: FileZipTreeNode | NodeTarget, options?: SourceOptions): Promise<DecompilerResult> {
			return (await messenger.sendMessage({ channel: 'source_read', ...getAsTarget(target), options: options}))
		},
		readNodeData: async (target: FileZipTreeNode | NodeTarget) => {
            return (await messenger.sendMessage({ channel: "node_read", ...getAsTarget(target)})).buffer
        },
        getOrReadZipTree: async function (buffer, {name, url, hash} = {}) {
            let node: RootZipTreeNode | undefined;
            if (url) node = await this.getZipTree(url, true);
            if (node == null && hash) node = await this.getZipTree(hash, false);
            const data = await buffer();
            return data ? this.readZipTree(data, {name: name, url: url, hash: hash}) : undefined;
        },
        readZipTree: async function (buffer, {name, url, hash} = {}) {
            const filteredURL = url?.split("?")[0];

            if (filteredURL) {
                const node = await this.getZipTree(filteredURL, true, true);
                if (node) return node;
            }

            if (hash) {
                const node = await this.getZipTree(hash, false, true);
                if (node) return node;
            }

            const node = reconstituteRoot(messenger.handler, (await messenger.sendMessage({ channel: "read_zip_tree", buffer, name: name, url: url, hash: hash })).node);

            if (filteredURL) {
                this.urlToHash.set(filteredURL, node.key);
                const urls = (hashToUrls.has(node.key))
                    ? hashToUrls.get(node.key)!
                    : (() => { const array = new Array<string>(); hashToUrls.set(node.key, array); return array; })();

                urls.push(filteredURL);
            }

            this.zipArchivesByHash.set(node.key, node);

            return node;
        },
        writeNodeData: async (target, buffer, type, options) => {
            return (await messenger.sendMessage({ channel: "node_write", ...getAsTarget(target), buffer, nodeType: type, options})).wasWritten;
        },
        collectFiles: async (root, matcher) => {
            return (await messenger.sendMessage({ channel: "collect_files", matcher: matcher, root: (root as RootZipTreeNode).key ?? root})).files
        },
        collectHashedFiles: async (root, matcher, deduplicate)=> {
            return (await messenger.sendMessage({ channel: "collect_files_hashed", matcher: matcher, root: (root as RootZipTreeNode).key ?? root, deduplicate: deduplicate})).files
        },
        closeZipTree: async function (root){
            const hash: Sha512 = (root as RootZipTreeNode).key ?? root;
            const value = await messenger.sendMessage({ channel: "close_zip_tree", root: (root as RootZipTreeNode).key ?? root});

            const urls = hashToUrls.get(hash);
            if (urls) for (const url of urls) this.urlToHash.delete(url);
            hashToUrls.delete(hash);

            return value;
        },
        closeAllZipTrees: async function () {
            const value = await messenger.sendMessage({ channel: "close_all_zip_trees" });

            this.zipArchivesByHash.clear();
            this.urlToHash.clear();
            hashToUrls.clear();

            return value;
        },
        getZipTree: async function (value, isUrl, localOnly) {
            if (this.zipArchivesByHash.has(value)) {
                return this.zipArchivesByHash.get(value);
            }

            if (isUrl) {
                const filteredURL = value.split("?")[0];
                if (this.urlToHash.has(filteredURL)) {
                    const hash = this.urlToHash.get(value)!;
                    const node = this.zipArchivesByHash.get(hash);

                    if (node) return node;
                }
            }

            if (localOnly) return undefined;

            const rawNode = (await messenger.sendMessage({ channel: "get_zip_tree", value: value })).node;

            if (rawNode == undefined) return undefined;

            const node = reconstituteRoot(messenger.handler, rawNode);

            if (isUrl) {
                const filteredURL = value.split("?")[0];
                this.urlToHash.set(filteredURL, node.key);
                if (hashToUrls.has(node.key)) hashToUrls.set(node.key, [])
                hashToUrls.get(node.key)?.push(filteredURL);
            }

            this.zipArchivesByHash.set(node.key, node);

            return node;
        }
    }
};

