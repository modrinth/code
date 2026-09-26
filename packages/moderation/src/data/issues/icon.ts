import { ImageIcon } from '@modrinth/assets'

import { panel, toggle } from './component-builders/builders'
import { rulesAiImagesIssue } from './rules'

export const iconReviewPanel = panel({
	title: 'Icon',
	hint: "Is this project's icon appropriate?",
	icon: ImageIcon,
	guidanceUrl:
		'https://www.notion.so/2e15ee711bf080e4a41df61bbab49892#2e35ee711bf080709084f6269835607f',
}).content(
	toggle({
		issue: rulesAiImagesIssue,
		label: 'AI Images',
	}),
)
