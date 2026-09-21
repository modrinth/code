import { ImageIcon } from '@modrinth/assets'

import { panel, toggle } from './component-builders/builders'
import { rulesAiImagesIssue } from './rules'

export const iconReviewPanel = panel({
	field: 'icon',
	title: 'Icon',
	hint: "Is this project's icon appropriate?",
	icon: ImageIcon,
}).content(
	toggle({
		issue: rulesAiImagesIssue,
		label: 'AI Images',
	}),
)
