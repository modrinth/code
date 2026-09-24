import { XIcon } from '@modrinth/assets'

import noVersionsMessage from '../messages/checklist/messages/undefined-project/no-versions.md'
import { issue, panel, toggle } from './component-builders/builders'

export const undefinedProjectNoVersionsIssue = issue({
	id: 'undefined-project-no-versions',
	title: 'No project versions',
	category: 'Project wide',
	message: noVersionsMessage,
	suggestedStatus: 'rejected',
})

export const undefinedProjectReviewPanel = panel({
	title: 'Undefined Project',
	hint: 'This project is undefined!',
	icon: XIcon,
	guidanceUrl:
		'https://www.notion.so/2e15ee711bf080e4a41df61bbab49892#3475ee711bf080018bf3d822a2f51a35',
	shown: ({ ProjectV3 }) => !ProjectV3.minecraft_server && ProjectV3.versions.length === 0,
}).content(
	toggle({
		issue: undefinedProjectNoVersionsIssue,
		label: 'No Versions',
	}),
)
