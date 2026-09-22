import { BookOpenIcon } from '@modrinth/assets'

import { generateUrlSlug } from '../../utils'
import minecraftBranding from '../messages/checklist/messages/title-slug/title/minecraft-branding.md'
import similarities from '../messages/checklist/messages/title-slug/title/similarities.md'
import forkSimilarities from '../messages/checklist/messages/title-slug/title/similarities/fork.md'
import modpackSimilarities from '../messages/checklist/messages/title-slug/title/similarities/modpack.md'
import uselessInfo from '../messages/checklist/messages/title-slug/title/useless-info.md'
import { issue, panel, section, toggle } from './component-builders/builders'
import { misusedSlugIssue } from './slug'

export const titleUselessInfoIssue = issue({
	id: 'title-useless-info',
	suggestedStatus: 'flagged',
	message: uselessInfo,
})

export const minecraftTitleIssue = issue({
	id: 'title-minecraft-branding',
	suggestedStatus: 'flagged',
	message: minecraftBranding,
})

export const titleSimilaritiesIssue = issue({
	id: 'title-similarities',
	suggestedStatus: 'flagged',
	message: ({ selected }) => {
		if (selected.toggleIds.includes('title-similarities-fork')) {
			return [similarities.trim(), forkSimilarities.trim()].join('\n\n')
		} else {
			return similarities
		}
	},
})

export const modpackTitleSimilaritiesIssue = issue({
	id: 'title-similarities-modpack',
	suggestedStatus: 'flagged',
	message: [similarities.trim(), modpackSimilarities.trim()].join('\n\n'),
})

export const titleReviewPanel = panel({
	field: 'title',
	title: 'Title and slug',
	hint: "Are the project's name and URL accurate and appropriate?",
	icon: BookOpenIcon,
}).content(
	section().content(
		toggle({
			label: 'Misused slug',
			issue: misusedSlugIssue,
			shown: ({ ProjectV3 }) => generateUrlSlug(ProjectV3.name) !== ProjectV3.slug,
		}),
		toggle({
			label: 'Contains Useless Info',
			issue: titleUselessInfoIssue,
		}),
		toggle({
			label: 'Minecraft Title',
			issue: minecraftTitleIssue,
		}),
		toggle({
			label: 'Title Similarities',
			issue: titleSimilaritiesIssue,
		}),
		toggle({
			label: 'Modpack Named After Mod',
			issue: modpackTitleSimilaritiesIssue,
			shown: (ctx) => ctx.ProjectV3.project_types.includes('modpack'),
		}),
	),
	section({
		label: 'Similarities Additional Info',
		shown: (ctx) =>
			!ctx.ProjectV3.minecraft_server && ctx.selected.issueIds.includes('title-similarities'),
	}).content(
		toggle({
			label: 'Forked Project',
			id: 'title-similarities-fork',
			issue: titleSimilaritiesIssue,
		}),
	),
)
