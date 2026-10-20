import type { DecompileHandler, DecompilerMessages } from '#ui/helpers/toolbox/decompile'
import { createRegistry, type MessengerType } from '#ui/helpers/toolbox/messenger'
// import { AtlasMessenger, handlerConstructor as atlasHandlerConstructor  } from "./atlas/common";
import {
	handlerConstructor as zipHandlerConstructor,
	type ZipMessages,
} from '#ui/helpers/toolbox/workers/zip/api'
import type { ZipHandler } from '#ui/helpers/toolbox/zip'

import DecompileWorker from './decompile?worker'
import ZipWorker from './zip?worker'

export const messengerRegistry = createRegistry()

export const zipHandlerType = messengerRegistry.of<ZipMessages, ZipHandler>('zip-handler', {
	handlerConstructor: zipHandlerConstructor,
	serverMessageTransferHandler(msg, resolver) {
		return resolver(msg, {
			node_write: (msg) => [msg.buffer],
			read_zip_tree: (msg) => (msg.data_type == 'buffer' ? [msg.buffer] : []),
		})
	},
	independentClientMessageHandler(messenger, msg) {
		if (msg.channel === 'archive_update') {
			messenger.handler.archiveUpdate(msg as ZipMessages['archive_update']['clientbound'])
			return true
		}
		return false
	},
	closeCallback(messenger) {
		return messenger.handler.closeAllZipTrees()
	},
	shared: true,
	count: 1,
})

// export const atlasMessengerType: MessengerType<AtlasMessenger> = messengerRegistry.simpleMessenger("atlas-builder", {
//     handlerConstructor: atlasHandlerConstructor,
//     inputBypassHandler: import.meta.env.DEV ? (await import('./atlas/implementation')).convertArchiveFilesToAtlas : undefined,
//     instanceType: "temporary-exclusive"
// });
//

export const decompileMessengerType: MessengerType<DecompilerMessages, DecompileHandler> =
	messengerRegistry.of<DecompilerMessages, DecompileHandler>('decompiler-vf', {
		serverMessageTransferHandler(msg, resolver) {
			return resolver(msg, {
				main: (msg) => (msg.transferBuffer ? [msg.targetFile.content] : []),
			})
		},
		handlerConstructor(messenger) {
			return {
				decompile: async (input) => {
					return await messenger.sendMessage({
						...input,
						channel: 'main',
						transferBuffer: input.transferBuffer ?? false,
					})
				},
			} as DecompileHandler
		},
		shared: true,
		timeout: 30000,
		count: 1,
	})

messengerRegistry.setWorkerFactoryFor(decompileMessengerType, () => new DecompileWorker())
messengerRegistry.setWorkerFactoryFor(zipHandlerType, () => new ZipWorker())
