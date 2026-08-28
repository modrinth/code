import type { Nag, ProjectValidationContext } from '../types/nags.ts'
import { getDescriptionNags } from '../validation-rules/rules/description.ts'
import { getDisclosureNags } from '../validation-rules/rules/disclosures.ts'
import { getGalleryNags } from '../validation-rules/rules/gallery.ts'
import { getIconNags } from '../validation-rules/rules/icon.ts'
import { getLicenseNags } from '../validation-rules/rules/license.ts'
import { getLinksNags } from '../validation-rules/rules/links.ts'
import { getModerationNags } from '../validation-rules/rules/moderation.ts'
import { getNameNags } from '../validation-rules/rules/name.ts'
import { getPermissionsNags } from '../validation-rules/rules/permissions.ts'
import { getServerSettingsNags } from '../validation-rules/rules/server-settings.ts'
import { getSummaryNags } from '../validation-rules/rules/summary.ts'
import { getTagsNags } from '../validation-rules/rules/tags.ts'
import { getVersionNags } from '../validation-rules/rules/versions.ts'

export function getNags(context: ProjectValidationContext): Nag[] {
	return [
		...getNameNags(context),
		...getSummaryNags(context),
		...getIconNags(context),
		...getGalleryNags(context),
		...getDescriptionNags(context),
		...getLicenseNags(context),
		...getLinksNags(context),
		...getPermissionsNags(context),
		...getServerSettingsNags(context),
		...getTagsNags(context),
		...getVersionNags(context),
		...getDisclosureNags(context),
		...getModerationNags(context),
	]
}
