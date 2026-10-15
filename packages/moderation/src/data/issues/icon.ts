import { ImageIcon } from '@modrinth/assets'

import flashingIconMessage from '../messages/checklist/messages/icon/flashing.md'
import inappropriateIconMessage from '../messages/checklist/messages/icon/inappropriate.md'
import iconSimilaritiesMessage from '../messages/checklist/messages/icon/similarities.md'
import { issue, panel, toggle } from './component-builders/builders'
import { issueTargets } from './component-builders/targets'
import { rulesAiImagesIssue } from './rules'

export const flashingIconIssue = issue({
	id: 'flashing-icon-issue',
	title: 'Flashing Icon',
	category: 'Icon',
	actions: [issueTargets.removeIcon()],
	suggestedStatus: 'flagged',
	message: flashingIconMessage,
})

export const inappropriateIconIssue = issue({
	id: 'icon-inappropriate',
	title: 'Inappropriate icon',
	category: 'Icon',
	actions: [issueTargets.removeIcon()],
	suggestedStatus: 'flagged',
	message: inappropriateIconMessage,
})

export const iconSimilaritiesIssue = issue({
	id: 'icon-similarities',
	title: 'Icon similarities',
	category: 'Icon',
	actions: [issueTargets.removeIcon()],
	suggestedStatus: 'flagged',
	message: iconSimilaritiesMessage,
})

export const iconReviewPanel = panel({
	title: 'Icon',
	hint: "Is this project's icon appropriate?",
	icon: ImageIcon,
	guidanceUrl:
		'https://www.notion.so/2e15ee711bf080e4a41df61bbab49892#2e35ee711bf080709084f6269835607f',
}).content(
	toggle({
		issue: rulesAiImagesIssue,
		id: 'icon-ai-images',
		label: 'AI',
		shown: ({ projectV3 }) => !!projectV3.icon_url,
	}),
	toggle({
		issue: flashingIconIssue,
		id: 'flashing-icon',
		label: 'Flashing',
		shown: ({ projectV3 }) => !!projectV3.icon_url,
	}),
	toggle({
		issue: inappropriateIconIssue,
		id: 'inappropriate-icon',
		label: 'Inappropriate',
		shown: ({ projectV3 }) => !!projectV3.icon_url,
	}),
	toggle({
		issue: iconSimilaritiesIssue,
		id: 'icon-similarities',
		label: 'Similarities',
		shown: ({ projectV3 }) => !!projectV3.icon_url,
	}),
)
