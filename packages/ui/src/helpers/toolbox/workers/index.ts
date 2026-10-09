import type {
	DecompileHandler,
	DecompilerMessages,
} from "#ui/helpers/toolbox/decompile";
import {
	type ClientMessenger,
	createRegistry,
	type InputResolverMap,
	type MessengerType,
	resolveInputTransfers,
} from '#ui/helpers/toolbox/messenger';
// import { AtlasMessenger, handlerConstructor as atlasHandlerConstructor  } from "./atlas/common";
import { handlerConstructor as zipHandlerConstructor,type ZipMessages } from "#ui/helpers/toolbox/workers/zip/api";
import type { DelegateFileZipTreeNode, ZipHandler } from "#ui/helpers/toolbox/zip";

import DecompileWorker from "./decompile?worker";
import ZipWorker from "./zip?worker";

export const messengerRegistry = createRegistry();

export type ZipHandlerMessenger = ClientMessenger<ZipMessages, ZipHandler>;

export const zipHandlerType: MessengerType<ZipHandlerMessenger> = messengerRegistry.of<ZipMessages, ZipHandler>("zip-handler", {
    handlerConstructor: zipHandlerConstructor,
    inputTransferHandler: (input) => {
        return resolveInputTransfers(input, {
            node_write: (msg) => [msg.buffer],
            read_zip_tree: (msg) => [msg.buffer],
        } as InputResolverMap<ZipMessages>);
    },
    independentOutputHandler: (messenger, msg) => {
        if (msg.channel == "update_node_data") {
            const data = msg as ZipMessages['update_node_data']['output'];

            const root = messenger.handler.zipArchivesByHash.get(data.root);

            if (root != null) {
                const node = (data.relative ? root.relativeFileEntries : root.fileEntries).get(data.path);

				if (node) {
					for (const lazy of [node.sha1, node.sha256, node.sha512, node.fileTypeInfo]) {
						lazy.reset()
					}
					(node as unknown as Partial<DelegateFileZipTreeNode>).onDataUpdate?.()
				}
            }
        }
        return false;
    },
    closeCallback: (messenger) => {
        return messenger.handler.closeAllZipTrees();
    },
	shared: true,
	count: 1,
});

// export const atlasMessengerType: MessengerType<AtlasMessenger> = messengerRegistry.simpleMessenger("atlas-builder", {
//     handlerConstructor: atlasHandlerConstructor,
//     inputBypassHandler: import.meta.env.DEV ? (await import('./atlas/implementation')).convertArchiveFilesToAtlas : undefined,
//     instanceType: "temporary-exclusive"
// });
//

type DecompileMessenger = ClientMessenger<DecompilerMessages, DecompileHandler>;

export const decompileMessengerType: MessengerType<DecompileMessenger> = messengerRegistry.of<DecompilerMessages, DecompileHandler>("decompiler-vf", {
	handlerConstructor: (messenger) => {
		return {
			decompile: async (input) => {
				return await messenger.sendMessage({
					...input,
					channel: 'main'
				})
			}
		};
	},
    inputTransferHandler: (input) => {
		return resolveInputTransfers(input, {
			main: (msg) => [msg.targetFile.content],
		} as InputResolverMap<DecompilerMessages>)
	},
	shared: true,
	timeout: 30000,
	count: 1
});

messengerRegistry.setWorkerFactoryFor(decompileMessengerType, () => new DecompileWorker());
messengerRegistry.setWorkerFactoryFor(zipHandlerType, () => new ZipWorker());
