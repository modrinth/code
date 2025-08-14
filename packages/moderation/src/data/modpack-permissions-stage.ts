import { PackageOpenIcon } from '@modrinth/assets'
import type { ModerationModpackPermissionApprovalType, Project } from '@modrinth/utils'

import type { Stage } from '../types/stage'

export default {
	id: 'modpack-permissions',
	title: 'Modpack Permissions',
	icon: PackageOpenIcon,
	// Replace me please.
	guidance_url:
		'https://www.notion.so/Content-Moderation-Cheat-Sheets-22d5ee711bf081a4920ef08879fe6bf5?source=copy_link#22d5ee711bf08116bd8bc1186f357062',
	shouldShow: (project: Project) => project.project_type === 'modpack',
	actions: [
		{
			id: 'button',
			type: 'button',
			label: 'This dummy button must be present or the stage will not appear.',
		},
	],
} as Stage

export const finalPermissionMessages: Record<
	ModerationModpackPermissionApprovalType['id'],
	string | undefined
> = {
	yes: undefined,
	'with-attribution-and-source': undefined,
	'with-attribution': `The following content has attribution requirements, meaning that you must link back to the page where you originally found this content in your Modpack's description or version changelog (e.g. linking a mod's CurseForge page if you got it from CurseForge):`,
	no: 'The following content is not allowed in Modrinth modpacks due to licensing restrictions. Please contact the author(s) directly for permission or remove the content from your modpack:',
	'permanent-no': `The following content is not allowed in Modrinth modpacks, regardless of permission obtained. This may be because it breaks Modrinth's content rules or because the authors, upon being contacted for permission, have declined. Please remove the content from your modpack:`,
	unidentified: `The following content could not be identified. Please provide proof of its origin along with proof that you have permission to include it:`,
}
