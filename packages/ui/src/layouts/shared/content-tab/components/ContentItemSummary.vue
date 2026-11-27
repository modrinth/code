<script setup lang="ts">
import { Link2Icon, SpinnerIcon, TriangleAlertIcon, UploadIcon } from '@modrinth/assets'
import { computed, ref } from 'vue'
import type { RouteLocationRaw } from 'vue-router'

import AutoLink from '#ui/components/base/AutoLink.vue'
import Avatar from '#ui/components/base/Avatar.vue'
import BulletDivider from '#ui/components/base/BulletDivider.vue'
import ProgressSpinner from '#ui/components/base/ProgressSpinner.vue'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { PROJECT_TYPE_ICONS } from '#ui/utils/auto-icons'
import { commonMessages, getProjectTypeTitleMessage } from '#ui/utils/common-messages'
import { truncatedTooltip } from '#ui/utils/truncate'

import type {
	ClientWarningType,
	ContentCardEmbeddedIcon,
	ContentCardProject,
	ContentCardVersion,
	ContentOwner,
	ContentSource,
} from '../types'
import ContentCardItemIcon from './ContentCardItemIcon.vue'

const props = withDefaults(
	defineProps<{
		project: ContentCardProject
		projectType?: string
		projectLink?: string | RouteLocationRaw
		version?: ContentCardVersion
		versionLink?: string | RouteLocationRaw
		showVersion?: boolean
		owner?: ContentOwner
		source?: ContentSource
		external?: boolean
		externalFile?: boolean
		overrideFile?: boolean
		installing?: boolean
		installProgress?: number | null
		clientWarning?: ClientWarningType | null
		syncedBadge?: boolean
		syncUpdatePending?: boolean
		embeddedIcon?: ContentCardEmbeddedIcon
		dimmed?: boolean
	}>(),
	{
		projectType: undefined,
		projectLink: undefined,
		version: undefined,
		versionLink: undefined,
		showVersion: true,
		owner: undefined,
		source: undefined,
		external: false,
		externalFile: false,
		overrideFile: false,
		installing: false,
		installProgress: undefined,
		clientWarning: null,
		syncedBadge: false,
		syncUpdatePending: false,
		embeddedIcon: undefined,
		dimmed: false,
	},
)

const { formatMessage } = useVIntl()

const messages = defineMessages({
	uploaded: {
		id: 'content.card.uploaded-file',
		defaultMessage: 'Uploaded file',
	},
	externalFile: {
		id: 'content.card.external-file',
		defaultMessage: 'External file',
	},
	overrideFile: {
		id: 'content.card.override-file',
		defaultMessage: 'Override file',
	},
	synced: {
		id: 'content.card.synced',
		defaultMessage: 'Synced across instances',
	},
	syncUpdatePending: {
		id: 'content.card.sync-update-pending',
		defaultMessage:
			'Some synced copies are waiting for changes. An instance may be running, have a frozen or incompatible version, or already contain its own copy.',
	},
	installingProgress: {
		id: 'content.card.installing-progress',
		defaultMessage: 'Installing... ({progress, number}%)',
	},
})

const compactVersionNumberRef = ref<HTMLElement | null>(null)
const titleRef = ref<HTMLElement | null>(null)
const sourceTitleRef = ref<HTMLElement | null>(null)
const ownerNameRef = ref<HTMLElement | null>(null)

const projectTitle = computed(() => props.project.title.replace(/§[0-9a-fk-orx]/gi, ''))
const projectTypeIcon = computed(() => {
	const type = props.projectType === 'shaderpack' ? 'shader' : props.projectType
	return type && type in PROJECT_TYPE_ICONS
		? PROJECT_TYPE_ICONS[type as keyof typeof PROJECT_TYPE_ICONS]
		: undefined
})
const projectTypeLabel = computed(() =>
	formatMessage(getProjectTypeTitleMessage(props.projectType), { count: 1 }),
)
const syncStatusLabel = computed(() =>
	formatMessage(props.syncUpdatePending ? messages.syncUpdatePending : messages.synced),
)
const clientWarningMessage = computed(() => {
	switch (props.clientWarning) {
		case 'retained':
			return commonMessages.clientRetainedWarning
		case 'depends':
			return commonMessages.clientDependsWarning
		default:
			return commonMessages.clientOnlyWarning
	}
})
const installTooltip = computed(() => {
	if (!props.installing) return undefined
	if (props.installProgress == null) return formatMessage(commonMessages.installingLabel)
	return formatMessage(messages.installingProgress, {
		progress: Math.round(props.installProgress),
	})
})

function linkTarget(link: unknown) {
	return typeof link === 'string' && link.startsWith('http') ? '_blank' : undefined
}
</script>

<template>
	<div
		class="flex min-w-0 items-center gap-3 transition-[filter,opacity] duration-200"
		:class="{ 'grayscale opacity-50': dimmed }"
	>
		<div v-tooltip="installTooltip" class="relative flex shrink-0 items-center">
			<ContentCardItemIcon
				:src="project.icon_url"
				:alt="projectTitle"
				:tint-by="project.id"
				:embedded-icon="embeddedIcon"
			/>
			<div
				v-if="installing"
				class="absolute inset-0 flex items-center justify-center rounded-2xl bg-black/20"
			>
				<ProgressSpinner
					v-if="installProgress != null && installProgress > 0"
					:progress="installProgress"
					:max="100"
					class="size-5 text-white"
				/>
				<SpinnerIcon v-else class="size-5 animate-spin text-white" />
			</div>
		</div>
		<div class="flex min-w-0 flex-col gap-0.5">
			<div class="flex min-w-0 items-center gap-1">
				<AutoLink
					:target="linkTarget(projectLink)"
					:to="projectLink"
					class="inline-flex min-w-0 items-center gap-1 font-semibold leading-6 text-contrast !decoration-contrast"
					:class="{ 'hover:underline': projectLink }"
				>
					<component
						:is="projectTypeIcon"
						v-if="projectTypeIcon"
						v-tooltip="projectTypeLabel"
						class="mx-0.5 size-5 shrink-0 text-icon"
						aria-hidden="true"
					/>
					<span v-if="projectTypeIcon" class="sr-only">{{ projectTypeLabel }}: </span>
					<span
						ref="titleRef"
						v-tooltip="truncatedTooltip(titleRef, projectTitle)"
						class="min-w-0 truncate"
						>{{ projectTitle }}</span
					>
				</AutoLink>
				<slot name="title-badges" />
				<span
					v-if="syncedBadge"
					v-tooltip="syncStatusLabel"
					role="img"
					:aria-label="syncStatusLabel"
					class="inline-flex shrink-0 cursor-help items-center justify-center rounded-full border border-solid border-brand-blue bg-highlight-blue px-2.5 py-1 text-brand-blue"
					tabindex="0"
				>
					<Link2Icon class="size-5" aria-hidden="true" />
				</span>
				<span
					v-if="clientWarning"
					v-tooltip="formatMessage(clientWarningMessage)"
					role="img"
					:aria-label="formatMessage(clientWarningMessage)"
					class="inline-flex size-5 shrink-0 cursor-help items-center justify-center"
					tabindex="0"
				>
					<TriangleAlertIcon class="pointer-events-none size-4 text-orange" aria-hidden="true" />
				</span>
			</div>

			<div class="flex min-w-0 items-center gap-1">
				<template v-if="source">
					<span v-if="source.label" class="shrink-0 text-sm leading-5 text-secondary">
						{{ source.label }}
					</span>
					<AutoLink
						:target="linkTarget(source.link)"
						:to="source.link"
						class="flex min-w-0 items-center gap-1 !decoration-secondary"
						:class="{ 'hover:underline': source.link }"
					>
						<Avatar
							:src="source.project.icon_url"
							:alt="source.project.title"
							:tint-by="source.project.id"
							size="1.25rem"
							no-shadow
							class="mx-0.5 shrink-0 rounded-md"
						/>
						<span
							ref="sourceTitleRef"
							v-tooltip="truncatedTooltip(sourceTitleRef, source.project.title)"
							class="truncate text-sm leading-5 text-secondary"
						>
							{{ source.project.title }}
						</span>
					</AutoLink>
				</template>
				<AutoLink
					v-else-if="owner"
					:target="linkTarget(owner.link)"
					:to="owner.link"
					class="flex min-w-0 items-center gap-1 !decoration-secondary"
					:class="{ 'hover:underline': owner.link }"
				>
					<Avatar
						:src="owner.avatar_url"
						:alt="owner.name"
						size="1.5rem"
						:circle="owner.type === 'user'"
						no-shadow
						class="shrink-0"
					/>
					<span
						ref="ownerNameRef"
						v-tooltip="truncatedTooltip(ownerNameRef, owner.name)"
						class="truncate text-sm leading-5 text-secondary"
						>{{ owner.name }}</span
					>
				</AutoLink>
				<span v-else-if="external" class="flex items-center gap-1 text-secondary">
					<UploadIcon class="mx-1 size-4 shrink-0" />
					<span class="text-sm leading-5">{{
						formatMessage(
							overrideFile
								? messages.overrideFile
								: externalFile
									? messages.externalFile
									: messages.uploaded,
						)
					}}</span>
				</span>
				<template v-if="showVersion && version?.version_number && !external">
					<BulletDivider class="shrink-0" />
					<AutoLink
						:target="linkTarget(versionLink)"
						:to="versionLink"
						class="min-w-0 text-sm leading-5 text-secondary !decoration-secondary"
						:class="{ 'hover:underline': versionLink }"
					>
						<span
							ref="compactVersionNumberRef"
							v-tooltip="truncatedTooltip(compactVersionNumberRef, version.version_number)"
							class="block truncate"
						>
							{{ version.version_number }}
						</span>
					</AutoLink>
				</template>
			</div>
		</div>
	</div>
</template>
