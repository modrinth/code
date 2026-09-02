<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { CheckCircleIcon, ScaleIcon, UserRoundIcon, XCircleIcon } from '@modrinth/assets'
import { builtinLicenses } from '@modrinth/utils'
import { computed } from 'vue'

import { IntlFormatted } from '#ui/components'
import { AutoLink, Avatar } from '#ui/components/base'

import { useFormatDateTime } from '../../composables/format-date-time'
import { defineMessage, defineMessages, useVIntl } from '../../composables/i18n'
import type { ProjectPermissionField } from './external-project-utils'
import {
	attributionLinkToWork,
	isAutomaticNoPermissionAttribution,
	isCustomAttributionLicense,
	isHttpUrl,
	PERMISSION_REASONS,
} from './external-project-utils'

const props = withDefaults(
	defineProps<{
		attribution: Labrinth.Attribution.Internal.AttributionResolution
		attributedAt?: string | null
		attributedBy?: string | null
		attributorHref: string | null
		attributorLabel: string
		attributorAvatarUrl?: string | null
		moderator?: boolean
	}>(),
	{
		moderator: false,
	},
)

const { formatMessage } = useVIntl()
const formatDate = useFormatDateTime({ dateStyle: 'long' })

const messages = defineMessages({
	linkLabel: {
		id: 'external-files.permissions-card.link-label',
		defaultMessage: 'Link to work:',
	},
	notesLabel: {
		id: 'external-files.permissions-card.notes-label',
		defaultMessage: 'Notes:',
	},
	licensedAs: {
		id: 'external-files.permissions-card.licensed-as',
		defaultMessage: 'Licensed:',
	},
	lastUpdated: {
		id: 'external-files.permissions-card.last-updated',
		defaultMessage: 'Last updated on {date} by {user}',
	},
	proofImagesLabel: {
		id: 'external-files.permissions-card.proof-images-label',
		defaultMessage: 'Proof images:',
	},
	proofImageThumbnailAlt: {
		id: 'external-files.permissions-card.proof-image-alt',
		defaultMessage: 'Proof screenshot {n}',
	},
	updatedByModerator: {
		id: 'external-files.permissions-card.updated-by-moderator',
		defaultMessage: 'Moderator',
	},
})

const unknownLicenseMessage = defineMessage({
	id: 'external-files.permissions-card.license.unknown',
	defaultMessage: 'Unknown',
})

const notesNoneMessage = defineMessage({
	id: 'external-files.permissions-card.notes-none',
	defaultMessage: 'None',
})

const automaticNoPermission = computed(() =>
	isAutomaticNoPermissionAttribution(props.attribution, props.attributedBy),
)

const readViewFields = computed(() => {
	if (automaticNoPermission.value) {
		return ['link_to_work'] as ProjectPermissionField[]
	}
	return PERMISSION_REASONS[props.attribution.kind]?.fields ?? []
})

const automaticAttributionDescription = computed(() => {
	if (props.attribution.kind === 'globally_allowed') {
		return PERMISSION_REASONS.globally_allowed.description
	}
	if (automaticNoPermission.value) {
		return PERMISSION_REASONS.no_permission.automaticDescription ?? null
	}
	return null
})

const licenseReadDisplay = computed(() => {
	const attr = props.attribution
	if (attr.kind !== 'license' && attr.kind !== 'my_project') {
		return null
	}
	if (isCustomAttributionLicense(attr.license)) {
		return { kind: 'custom' as const, value: attr.license.name }
	}
	const licenseId = attr.license
	if (licenseId) {
		const friendly =
			builtinLicenses.find((license) => license.short === licenseId)?.friendly ?? licenseId
		return { kind: 'standard' as const, value: friendly }
	}
	return { kind: 'unknown' as const, value: formatMessage(unknownLicenseMessage) }
})

const linkToWork = computed(() => attributionLinkToWork(props.attribution))
</script>

<template>
	<div>
		<div
			class="flex flex-col gap-4 rounded-t-2xl p-4 mt-2 bg-surface-3 border border-solid border-surface-4"
			:class="{ 'rounded-b-2xl': !$slots.footer, 'border-b-0': $slots.footer }"
		>
			<div class="flex gap-4">
				<div class="flex flex-col gap-3 w-full">
					<div class="flex items-start justify-between gap-3">
						<span class="text-contrast font-semibold flex items-center gap-2">
							<CheckCircleIcon
								v-if="attribution.kind === 'globally_allowed'"
								class="text-green size-5"
							/>
							<XCircleIcon v-else-if="automaticNoPermission" class="text-red size-5" />
							{{ formatMessage(PERMISSION_REASONS[attribution.kind].label) }}
						</span>
					</div>
					<template v-if="automaticAttributionDescription">
						<p class="m-0">
							{{ formatMessage(automaticAttributionDescription) }}
						</p>
					</template>
					<div
						v-if="!(readViewFields.includes('link_to_work') && !linkToWork)"
						class="flex flex-col gap-3"
					>
						<div class="grid grid-cols-[max-content_1fr] gap-x-4 gap-y-2 items-baseline">
							<template v-if="attribution.kind === 'license' || attribution.kind === 'my_project'">
								<span class="text-secondary font-medium">
									{{ formatMessage(messages.licensedAs) }}
								</span>
								<a
									v-if="
										licenseReadDisplay?.kind === 'custom' && isHttpUrl(licenseReadDisplay.value)
									"
									:href="licenseReadDisplay.value"
									target="_blank"
									rel="noopener"
									class="text-link truncate"
								>
									{{ licenseReadDisplay.value }}
								</a>
								<span v-else class="text-primary whitespace-pre-wrap break-words">
									{{ licenseReadDisplay?.value }}
								</span>
							</template>
							<template v-if="readViewFields.includes('link_to_work') && linkToWork">
								<span class="text-secondary font-medium">
									{{ formatMessage(messages.linkLabel) }}
								</span>
								<a :href="linkToWork" target="_blank" rel="noopener" class="text-link truncate">{{
									linkToWork
								}}</a>
							</template>
							<template v-if="readViewFields.includes('notes')">
								<span class="text-secondary font-medium">
									{{ formatMessage(messages.notesLabel) }}
								</span>
								<span class="text-primary whitespace-pre-wrap break-words">
									{{
										attribution.notes?.trim() ? attribution.notes : formatMessage(notesNoneMessage)
									}}
								</span>
							</template>
						</div>
						<div v-if="attribution.image_urls?.length" class="flex flex-col gap-2">
							<span class="text-secondary font-medium">
								{{ formatMessage(messages.proofImagesLabel) }}
							</span>
							<div class="flex flex-wrap gap-2">
								<a
									v-for="(src, idx) in attribution.image_urls"
									:key="`${src}-${idx}`"
									:href="src"
									target="_blank"
									rel="noopener"
									class="block rounded-xl border-[1px] border-solid border-surface-5 overflow-hidden shrink-0"
								>
									<img
										:src="src"
										:alt="formatMessage(messages.proofImageThumbnailAlt, { n: idx + 1 })"
										class="max-h-40 max-w-full object-contain"
									/>
								</a>
							</div>
						</div>
					</div>
				</div>
				<div v-if="$slots.actions">
					<slot name="actions" />
				</div>
			</div>

			<div
				v-if="attributedAt"
				class="inline-flex items-center flex-wrap gap-x-2 gap-y-1 pt-3 mt-1 border-0 border-t border-solid border-surface-5"
			>
				<IntlFormatted
					:message-id="messages.lastUpdated"
					:values="{ date: formatDate(attributedAt) }"
				>
					<template #user>
						<span
							v-if="!moderator && attribution.updated_by_moderator"
							class="text-orange flex items-center gap-1"
						>
							<ScaleIcon class="size-4 shrink-0" />
							{{ formatMessage(messages.updatedByModerator) }}
						</span>
						<AutoLink
							v-if="attributedBy && moderator"
							:to="attributorHref"
							class="inline-flex items-center gap-1.5 text-primary font-medium hover:underline max-w-full min-w-0"
						>
							<Avatar
								v-if="attributorAvatarUrl"
								:src="attributorAvatarUrl"
								:alt="attributorLabel"
								size="18px"
								class="shrink-0"
								circle
							/>
							<UserRoundIcon v-else class="size-4 shrink-0" />
							<span class="truncate">{{ attributorLabel }}</span>
						</AutoLink>
					</template>
				</IntlFormatted>
			</div>
		</div>
		<div
			v-if="$slots.footer"
			class="p-4 border-surface-4 border bg-surface-2 border-solid rounded-b-2xl"
		>
			<slot name="footer" />
		</div>
	</div>
</template>
