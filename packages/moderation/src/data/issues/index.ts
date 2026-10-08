import { categoriesReviewPanel } from './categories'
import { each } from './component-builders/builders'
import type { PanelRegistration } from './component-builders/types'
import { descriptionReviewPanel } from './description'
import {
	adsDisclosureReviewPanel,
	aiDisclosureReviewPanel,
	aiFunctionalityDisclosureReviewPanel,
	archiveDisclosureReviewPanel,
	derivativeContentDisclosureReviewPanel,
	disclosuresReviewPanel,
	paidFeaturesDisclosureReviewPanel,
	photosensitivityDisclosureReviewPanel,
	systemInteractionsDisclosureReviewPanel,
	telemetryDisclosureReviewPanel,
} from './disclosures'
import { galleryImageReviewPanel, galleryReviewPanel } from './gallery'
import { iconReviewPanel } from './icon'
import { licenseReviewPanel } from './license'
import {
	bmacReviewPanel,
	discordReviewPanel,
	githubReviewPanel,
	issuesReviewPanel,
	koFiReviewPanel,
	otherReviewPanel,
	patreonReviewPanel,
	paypalReviewPanel,
	siteReviewPanel,
	sourceReviewPanel,
	storeReviewPanel,
	wikiReviewPanel,
} from './links'
import { metadataReviewPanel } from './metadata'
import { permissionsReviewPanel } from './permissions'
import { postApprovalReviewPanel } from './post-approval'
import { reReviewReviewPanel } from './re-review'
import { reuploadReviewPanel } from './reupload'
import { rulesReviewPanel } from './rules'
import { serverReviewPanel } from './server'
import { slugReviewPanel } from './slug'
import { summaryReviewPanel } from './summary'
import { titleReviewPanel } from './title'
import { undefinedProjectReviewPanel } from './undefined-project'
import { versionsReviewPanel } from './versions'

export const reviewPanels = {
	title: titleReviewPanel,
	slug: slugReviewPanel,
	summary: summaryReviewPanel,
	description: descriptionReviewPanel,
	'issues-link': issuesReviewPanel,
	'source-link': sourceReviewPanel,
	'wiki-link': wikiReviewPanel,
	'discord-link': discordReviewPanel,
	'site-link': siteReviewPanel,
	'store-link': storeReviewPanel,
	'patreon-link': patreonReviewPanel,
	'bmac-link': bmacReviewPanel,
	'paypal-link': paypalReviewPanel,
	'github-link': githubReviewPanel,
	'ko-fi-link': koFiReviewPanel,
	'other-link': otherReviewPanel,
	categories: categoriesReviewPanel,
	disclosures: disclosuresReviewPanel,
	'ai-disclosure': aiDisclosureReviewPanel,
	'ai-functionality-disclosure': aiFunctionalityDisclosureReviewPanel,
	'ads-disclosure': adsDisclosureReviewPanel,
	'paid-features-disclosure': paidFeaturesDisclosureReviewPanel,
	'telemetry-disclosure': telemetryDisclosureReviewPanel,
	'derivative-content-disclosure': derivativeContentDisclosureReviewPanel,
	'photosensitivity-disclosure': photosensitivityDisclosureReviewPanel,
	'system-interactions-disclosure': systemInteractionsDisclosureReviewPanel,
	'archive-disclosure': archiveDisclosureReviewPanel,
	gallery: galleryReviewPanel,
	'gallery-image': each(({ projectV3 }) =>
		projectV3.gallery
			.filter((image) => image.id !== undefined)
			.map((image, index) => ({
				key: String(image.id),
				panel: galleryImageReviewPanel(image, index + 1),
			})),
	),
	icon: iconReviewPanel,
	license: licenseReviewPanel,
	metadata: metadataReviewPanel,
	server: serverReviewPanel,
	permissions: permissionsReviewPanel,
	're-review': reReviewReviewPanel,
	misc: [
		{
			key: 'reupload',
			label: 'Reupload',
			panel: reuploadReviewPanel,
		},
		{
			key: 'rules',
			label: 'Rule Following',
			panel: rulesReviewPanel,
		},
		{
			key: 'post-approval',
			label: 'Post-Approval',
			panel: postApprovalReviewPanel,
		},
	],
	'undefined-project': undefinedProjectReviewPanel,
	versions: versionsReviewPanel,
} satisfies Record<string, PanelRegistration>

export { IssuePriority } from './component-builders/priority'

export type MiscReviewPanelKey = (typeof reviewPanels.misc)[number]['key']

export type ReviewPanelKey = Exclude<keyof typeof reviewPanels, 'misc'> | MiscReviewPanelKey
