import { BookOpenIcon } from '@modrinth/assets'

import minecraftBranding from '../messages/checklist/messages/title-slug/title/minecraft-branding.md'
import similarities from '../messages/checklist/messages/title-slug/title/similarities.md'
import forkSimilarities from '../messages/checklist/messages/title-slug/title/similarities/fork.md'
import modpackSimilarities from '../messages/checklist/messages/title-slug/title/similarities/modpack.md'
import uselessInfo from '../messages/checklist/messages/title-slug/title/useless-info.md'
import { issue, panel, section, text, toggle } from './component-builders/builders'
import { issueTargets } from './component-builders/targets'

export const titleSuggestion = issue({
	id: 'title-suggestion',
	title: 'Title suggestion',
	category: 'Title',
	actions: [issueTargets.modifyTitle()],
	suggestedStatus: 'flagged',
	message: 'Consider suggesting a better title for the project.',
})

export const titleUselessInfoIssue = issue({
	id: 'title-useless-info',
	title: 'Unnecessary title information',
	category: 'Title',
	actions: [issueTargets.modifyTitle()],
	suggestedStatus: 'flagged',
	message: uselessInfo,
})

export const minecraftTitleIssue = issue({
	id: 'title-minecraft-branding',
	title: 'Minecraft branding in title',
	category: 'Title',
	actions: [issueTargets.modifyTitle()],
	suggestedStatus: 'flagged',
	message: minecraftBranding,
})

export const titleSimilaritiesIssue = issue({
	id: 'title-similarities',
	title: 'Misuse of project name',
	category: 'Title',
	actions: [issueTargets.modifyTitle()],
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
	title: 'Modpack uses another project’s name',
	category: 'Title',
	actions: [issueTargets.modifyTitle()],
	suggestedStatus: 'flagged',
	message: [similarities.trim(), modpackSimilarities.trim()].join('\n\n'),
})

export const titleReviewPanel = panel({
	title: 'Title',
	hint: "Is the project's name accurate and appropriate?",
	icon: BookOpenIcon,
	guidanceUrl:
		'https://www.notion.so/2e15ee711bf080e4a41df61bbab49892#2e15ee711bf0803c9660e90f0fead705',
}).content(
	section().content(
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
	),
	section({
		label: 'Similarities Additional Info',
		shown: (ctx) => ctx.selected.issueIds.includes('title-similarities'),
	}).content(
		toggle({
			label: 'Forked Project',
			id: 'title-similarities-fork',
			issue: titleSimilaritiesIssue,
			shown: (ctx) => !ctx.projectV3.minecraft_server,
		}),
		toggle({
			label: 'Modpack Named After Mod',
			id: 'title-similarities-modpack',
			issue: modpackTitleSimilaritiesIssue,
			shown: (ctx) => ctx.projectV3.project_types.includes('modpack'),
		}),
	),
	section({
		label: 'Name Suggestions',
		shown: (ctx) => ctx.selected.issueIds.includes('title-useless-info'),
	}).content(
		text({
			label: 'Suggest a Better Name',
			id: 'title-name-suggestion',
			issue: titleSuggestion,
		}),
	),
)
