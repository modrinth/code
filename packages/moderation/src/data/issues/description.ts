import { LibraryIcon } from '@modrinth/assets'

import clarity from '../messages/checklist/messages/description/clarity.md'
import headersAsBody from '../messages/checklist/messages/description/headers-as-body.md'
import imageOnly from '../messages/checklist/messages/description/image-only.md'
import insufficientPacks from '../messages/checklist/messages/description/insufficient/default/packs.md'
import insufficientProjects from '../messages/checklist/messages/description/insufficient/default/projects.md'
import insufficientServers from '../messages/checklist/messages/description/insufficient/default/servers.md'
import insufficientHeader from '../messages/checklist/messages/description/insufficient/header.md'
import fork from '../messages/checklist/messages/description/insufficient/piece/fork.md'
import spoilerGuide from '../messages/checklist/messages/description/insufficient/piece/spoiler-guide.md'
import spoilers from '../messages/checklist/messages/description/insufficient/piece/spoilers.md'
import unfinished from '../messages/checklist/messages/description/insufficient/piece/unfinished.md'
import nonEnglish from '../messages/checklist/messages/description/non-english.md'
import nonEnglishServer from '../messages/checklist/messages/description/non-english-server.md'
import nonStandardText from '../messages/checklist/messages/description/non-standard-text.md'
import { issue, markdown, panel, section, toggle } from './component-builders/builders'
import { rulesAiImagesIssue } from './rules'

export const insufficientDescriptionIssue = issue({
	id: 'description-insufficient',
	suggestedStatus: 'flagged',
	message: ({ ProjectV3, selected, getMarkdownValue }) => {
		const { toggleIds } = selected
		const parts: string[] = []

		if (toggleIds.includes('description-custom')) {
			const custom = getMarkdownValue('description-explainer').trim()
			if (custom) parts.push(custom)
		}
		if (toggleIds.includes('description-fork')) parts.push(fork.trim())
		if (toggleIds.includes('description-unfinished')) parts.push(unfinished.trim())
		if (toggleIds.includes('description-spoilers')) parts.push(spoilers.trim())

		if (!parts.length) {
			if (ProjectV3.minecraft_java_server) {
				parts.push(insufficientServers.trim())
			} else if (ProjectV3.project_types.includes('modpack')) {
				parts.push(insufficientPacks.trim())
			} else {
				parts.push(insufficientProjects.trim())
			}
		}

		if (toggleIds.includes('description-spoilers')) parts.push(spoilerGuide.trim())

		return [insufficientHeader.trim(), ...parts].join('\n\n')
	},
})

export const nonEnglishDescriptionIssue = issue({
	id: 'description-non-english',
	suggestedStatus: 'flagged',
	message: ({ ProjectV3 }) => (ProjectV3.minecraft_java_server ? nonEnglishServer : nonEnglish),
})

export const descriptionHeadersAsBodyIssue = issue({
	id: 'description-headers-as-body',
	suggestedStatus: 'flagged',
	message: headersAsBody,
})

export const imageOnlyDescriptionIssue = issue({
	id: 'description-image-only',
	suggestedStatus: 'flagged',
	message: imageOnly,
})

export const nonStandardDescriptionTextIssue = issue({
	id: 'description-non-standard-text',
	suggestedStatus: 'flagged',
	message: nonStandardText,
})

export const unclearDescriptionIssue = issue({
	id: 'description-clarity',
	suggestedStatus: 'rejected',
	message: clarity,
})

export const descriptionReviewPanel = panel({
	field: 'description',
	title: 'Description',
	hint: 'Is the description sufficient, accurate, and accessible?',
	icon: LibraryIcon,
}).content(
	toggle({
		label: 'Insufficient',
		issue: insufficientDescriptionIssue,
	}),
	toggle({
		label: 'Non-English',
		issue: nonEnglishDescriptionIssue,
		shown: ({ ProjectV3 }) =>
			!ProjectV3.minecraft_java_server || !!ProjectV3.minecraft_server?.languages?.includes('en'),
	}),
	toggle({
		label: 'Headers as body text',
		issue: descriptionHeadersAsBodyIssue,
	}),
	toggle({
		label: 'Image-only',
		issue: imageOnlyDescriptionIssue,
	}),
	toggle({
		label: 'Non-standard text',
		issue: nonStandardDescriptionTextIssue,
	}),
	toggle({
		label: 'Unclear / Misleading',
		issue: unclearDescriptionIssue,
	}),
	toggle({
		label: 'AI Images',
		issue: rulesAiImagesIssue,
	}),
	section({
		label: 'Why is this Description Insufficient?',
		shown: ({ selected }) => selected.issueIds.includes(insufficientDescriptionIssue.id),
	}).content(
		toggle({
			label: 'Custom',
			id: 'description-custom',
			issue: insufficientDescriptionIssue,
		}),
		toggle({
			label: 'Fork',
			id: 'description-fork',
			issue: insufficientDescriptionIssue,
		}),
		toggle({
			label: 'Unfinished',
			id: 'description-unfinished',
			issue: insufficientDescriptionIssue,
		}),
		toggle({
			label: 'Spoilers',
			id: 'description-spoilers',
			issue: insufficientDescriptionIssue,
		}),
	),
	section({
		shown: ({ selected }) =>
			selected.issueIds.includes(insufficientDescriptionIssue.id) &&
			selected.toggleIds.includes('description-custom'),
	}).content(
		markdown({
			label: 'How can the author improve their description?',
			issue: insufficientDescriptionIssue,
			id: 'description-explainer',
			required: true,
		}),
	),
)
