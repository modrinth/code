import type { DecompilerMessages } from '#ui/helpers/toolbox/decompile'
import { decompileFileContent } from '#ui/helpers/toolbox/decompile/impl'
import { setupAsServer } from '#ui/helpers/toolbox/messenger'
import { decompileMessengerType, zipHandlerType } from '#ui/helpers/toolbox/workers/registry'

setupAsServer<DecompilerMessages>(
	decompileMessengerType,
	self as unknown as Worker,
	async (messenger, msg, resolver) => {
		return resolver(msg, {
			main: async (msg) => {
				return decompileFileContent(
					messenger.logger,
					(await messenger.getMessenger(zipHandlerType))?.handler,
					msg,
				)
			},
		})
	},
)
