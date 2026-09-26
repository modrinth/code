import { ServerIcon } from '@modrinth/assets'

import { panel } from './component-builders/builders'

export const serverReviewPanel = panel({
	title: 'Server details',
	hint: "Are there any issues with this project's server details?",
	icon: ServerIcon,
	guidanceUrl: '',
	shown: ({ ProjectV3 }) => !!ProjectV3.minecraft_server,
}).content()
