import { useRouter } from "vue-router";

import type {DecompileHandler, DecompileInput, DecompilerResult} from "#ui/helpers/toolbox/decompile";
import {
	type ClientCompoundMessenger, type ClientMessenger,
	createRegistry,
	isWorker,
	type MessengerType
} from '#ui/helpers/toolbox/messenger';
// import { AtlasMessenger, handlerConstructor as atlasHandlerConstructor  } from "./atlas/common";
import { handlerConstructor as zipHandlerConstructor,type ZipMessages } from "#ui/helpers/toolbox/workers/zip/api";
import type { DelegateFileZipTreeNode, ZipHandler } from "#ui/helpers/toolbox/zip";

import DecompileWorker from "./decompile?worker";
import ZipWorker from "./zip?worker";

export const messengerRegistry = createRegistry();

export type ZipHandlerMessenger = ClientCompoundMessenger<ZipMessages, ZipHandler>;
export const zipHandlerType: MessengerType<ZipHandlerMessenger> = messengerRegistry.compoundMessenger<ZipMessages, ZipHandler>("zip-handler", {
    handlerConstructor: zipHandlerConstructor,
    inputBypassHandler: import.meta.env.DEV ? (await import('./zip')).getMessageHandler() : undefined,
    inputTransferHandler: ((input) => {
        return input.resolve({
            node_write: (msg) => [msg.buffer],
            read_zip_tree: (msg) => [msg.buffer],
        });
    }),
    forcedOutputHandler: (messenger, msg) => {
        if (msg.type == "update_node_data") {
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

type DecompileMessenger = ClientMessenger<DecompileInput, DecompilerResult, DecompileHandler>;
export const decompileMessengerType: MessengerType<DecompileMessenger> = messengerRegistry.simpleMessenger("decompiler-vf", {
    inputBypassHandler: import.meta.env.DEV ? async (messenger, input) => {
		return (await import('../decompile/impl.ts')).decompileFileContent(messenger.logger, (await messengerRegistry.getMessenger(zipHandlerType)), input)
	} : undefined,
	handlerConstructor: (messenger) => messenger.sendMessage,
    inputTransferHandler: (file) => [file.targetFile.content],
	shared: true,
	timeout: 30000,
	count: 1
});
messengerRegistry.setWorkerFactoryFor(decompileMessengerType, () => new DecompileWorker());
messengerRegistry.setWorkerFactoryFor(zipHandlerType, () => new ZipWorker());

if (!isWorker()) {
	// TODO: IMPLEMENT CORRECT CLEANUP I.E. WHEN PROPERLY CHANGING ROOT PAGES?
	useRouter().beforeEach((to, from, next) => {
		if (to.params.project != from.params.project) messengerRegistry.closeAll(true);
	})
}
