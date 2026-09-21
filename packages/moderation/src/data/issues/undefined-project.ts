import { XIcon } from '@modrinth/assets'

import noVersionsMessage from '../messages/checklist/messages/undefined-project/no-versions.md'
import { issue, panel, toggle } from './component-builders/builders'

export const undefinedProjectNoVersionsIssue = issue({
	id: 'undefined-project-no-versions',
	message: noVersionsMessage,
	suggestedStatus: 'rejected',
})

export const undefinedProjectReviewPanel = panel({
	field: 'undefined-project',
	title: 'Undefined Project',
	hint: 'This project is undefined!',
	icon: XIcon,
	shown: ({ ProjectV3 }) => !ProjectV3.minecraft_server && ProjectV3.versions.length === 0,
}).content(
	toggle({
		issue: undefinedProjectNoVersionsIssue,
		label: 'No Versions',
	}),
)
