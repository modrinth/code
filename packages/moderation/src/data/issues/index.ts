import { categoriesReviewPanel } from './categories'
import type { Panel } from './component-builders/types'
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
import { galleryReviewPanel } from './gallery'
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
import { slugReviewPanel } from './slug'
import { statusAlertsReviewPanel } from './status-alerts'
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
	icon: iconReviewPanel,
	license: licenseReviewPanel,
	metadata: metadataReviewPanel,
	permissions: permissionsReviewPanel,
	'post-approval': postApprovalReviewPanel,
	're-review': reReviewReviewPanel,
	reupload: reuploadReviewPanel,
	rules: rulesReviewPanel,
	'status-alerts': statusAlertsReviewPanel,
	'undefined-project': undefinedProjectReviewPanel,
	versions: versionsReviewPanel,
} satisfies Record<string, Panel>

export type ReviewPanelKey = keyof typeof reviewPanels
