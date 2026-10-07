import type {DecompileInput, DecompilerResult} from "#ui/helpers/toolbox/decompile";
import { decompileFileContent } from "#ui/helpers/toolbox/decompile/impl";
import { setupAsServer } from "#ui/helpers/toolbox/messenger";
import {decompileMessengerType, zipHandlerType} from "#ui/helpers/toolbox/workers";

setupAsServer<DecompileInput, DecompilerResult>(decompileMessengerType, self as unknown as Worker, async (messenger, msg) => {
	return decompileFileContent(messenger.logger, (await messenger.getMessenger(zipHandlerType))?.handler, msg);
})
