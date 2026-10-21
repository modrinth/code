<script setup lang="ts">
import {
	ArchiveIcon,
	BrainCogIcon,
	CircleDollarSignIcon,
	ExternalIcon,
	EyeIcon,
	GitForkIcon,
	MegaphoneIcon,
	MonitorCogIcon,
	RadioTowerIcon,
	SparklesIcon,
} from '@modrinth/assets'
import { AutoLink, TagItem } from '@modrinth/ui'
import { type Component, computed } from 'vue'

import DisclosureLockControls from '~/components/ui/project-settings/disclosures/DisclosureLockControls.vue'
import DisclosureUpdatedBy from '~/components/ui/project-settings/disclosures/DisclosureUpdatedBy.vue'
import { formToDisclosures } from '~/components/ui/project-settings/disclosures/form'
import ProjectReviewDisclosureCard from '~/components/ui/project-settings/disclosures/ProjectReviewDisclosureCard.vue'
import type {
	DisclosureFormState,
	DisclosureType,
} from '~/components/ui/project-settings/disclosures/types'
import { injectProjectReviewPageContext } from '~/providers/project-review'
import { injectReviewPanels } from '~/providers/project-review/review-panels'

const aiUsageLabels = {
	code: 'Code',
	assets: 'Assets',
	text: 'Text',
}
const consentLabels = {
	opt_in: 'Opt-in',
	opt_out: 'Opt-out',
	always_active: 'Always active',
}
const disclosureTitles = {
	ai_content: 'AI-generated content',
	ai_functionality: 'Generative AI functionality',
	advertisements: 'Advertisements',
	paid_features: 'Paid features',
	telemetry: 'Telemetry',
	derivative_work: 'Derivative content',
	epilepsy_triggers: 'Photosensitivity triggers',
	system_interactions: 'External system interactions',
	archived: 'Archived',
}

const props = defineProps<{ type: DisclosureType }>()
const emit = defineEmits<{ edit: [] }>()

const { disclosures } = injectProjectReviewPageContext()
const panels = injectReviewPanels()

const types = [
	{ type: 'ai_content', formKey: 'ai', icon: SparklesIcon, key: 'ai' },
	{
		type: 'ai_functionality',
		formKey: 'aiFunctionality',
		icon: BrainCogIcon,
		key: 'ai-functionality',
	},
	{
		type: 'advertisements',
		formKey: 'advertising',
		icon: MegaphoneIcon,
		key: 'ads',
	},
	{
		type: 'paid_features',
		formKey: 'paidFeatures',
		icon: CircleDollarSignIcon,
		key: 'paid-features',
	},
	{
		type: 'telemetry',
		formKey: 'telemetry',
		icon: RadioTowerIcon,
		key: 'telemetry',
	},
	{
		type: 'derivative_work',
		formKey: 'derivative',
		icon: GitForkIcon,
		key: 'derivative-content',
	},
	{
		type: 'epilepsy_triggers',
		formKey: 'photosensitivity',
		icon: EyeIcon,
		key: 'photosensitivity',
	},
	{
		type: 'system_interactions',
		formKey: 'systemInteractions',
		icon: MonitorCogIcon,
		key: 'system-interactions',
	},
	{ type: 'archived', formKey: 'archived', icon: ArchiveIcon, key: 'archive' },
] satisfies {
	type: DisclosureType
	key: string
	icon: Component
	formKey: keyof Omit<DisclosureFormState, 'lockStatuses'>
}[]
const sections = computed(() =>
	types
		.filter(({ type }) => type === props.type)
		.map(({ type, key, icon, formKey }) => ({
			icon,
			type,
			key,
			title: panels.resolve({ kind: 'disclosure', key })?.panel.title ?? disclosureTitles[type],
			data: formToDisclosures(disclosures.current.value).find((entry) => entry.type === type),
			meta: disclosures.disclosureUpdateProps(type),
			lock: disclosures.current.value.lockStatuses[type],
			model: disclosures.current.value[formKey],
		})),
)
</script>

<template>
	<div class="disclosure-details flex min-w-0 flex-col gap-4 text-sm text-secondary">
		<ProjectReviewDisclosureCard
			v-for="section in sections"
			:key="section.type"
			v-model="section.model.enabled"
			:title="section.title"
			:icon="section.icon"
			:disabled="section.meta.disabled"
			:toggle-disabled="section.meta.toggleDisabled"
			:on-toggle-editing="() => emit('edit')"
		>
			<template v-if="section.meta.showLockControls" #lock-controls>
				<DisclosureLockControls
					compact
					:lock-status="section.lock"
					:disabled="section.meta.disabled"
					@set-lock-status="disclosures.setDisclosureLockStatus(section.type, $event)"
				/>
			</template>
			<template v-if="section.meta.updatedAt" #updated-by>
				<DisclosureUpdatedBy
					:updated-at="section.meta.updatedAt"
					:updated-by="section.meta.updatedBy"
					:set-by-moderator="section.meta.setByModerator"
				/>
			</template>
			<template v-if="section.data" #expanded>
				<dl class="disclosure-fields">
					<div v-if="section.data.type === 'ai_content'">
						<dt>Content types</dt>
						<dd>
							<div v-if="section.data.uses.length" class="flex flex-wrap gap-1.5">
								<TagItem
									v-for="use in section.data.uses"
									:key="use"
									class="!border-surface-5 !bg-surface-4 !text-primary"
								>
									{{ aiUsageLabels[use] }}
								</TagItem>
							</div>
							<p v-else class="empty-field">None listed.</p>
						</dd>
					</div>
					<template v-if="section.data.type === 'telemetry'">
						<div>
							<dt>Consent model</dt>
							<dd>
								<TagItem class="!border-surface-5 !bg-surface-4 !text-primary">{{
									consentLabels[section.data.consent]
								}}</TagItem>
							</dd>
						</div>
						<div>
							<dt>Data collected</dt>
							<dd>
								<ul v-if="section.data.data_collected.length">
									<li v-for="(entry, index) in section.data.data_collected" :key="index">
										{{ entry }}
									</li>
								</ul>
								<p v-else class="empty-field">None listed.</p>
							</dd>
						</div>
					</template>
					<div v-if="section.data.type === 'paid_features'">
						<dt>Paid features</dt>
						<dd>
							<ul v-if="section.data.features.length">
								<li v-for="(feature, index) in section.data.features" :key="index">
									{{ feature }}
								</li>
							</ul>
							<p v-else class="empty-field">None listed.</p>
						</dd>
					</div>
					<div v-if="section.data.type === 'system_interactions'">
						<dt>System interactions</dt>
						<dd>
							<ul v-if="section.data.interactions.some((entry) => entry.trim())">
								<li
									v-for="(interaction, index) in section.data.interactions.filter((entry) =>
										entry.trim(),
									)"
									:key="index"
								>
									{{ interaction }}
								</li>
							</ul>
							<p v-else class="empty-field">None listed.</p>
						</dd>
					</div>
					<div v-if="section.data.type === 'derivative_work'" class="original-works">
						<dt>Original works</dt>
						<dd>
							<div v-if="section.data.sources.length" class="flex flex-col gap-4">
								<dl
									v-for="(source, index) in section.data.sources"
									:key="index"
									class="disclosure-fields border-0 border-l-2 border-solid border-surface-4 pl-3"
								>
									<div>
										<dt>Name</dt>
										<dd>
											<span v-if="source.label">{{ source.label }}</span>
											<span v-else class="empty-field">None provided.</span>
										</dd>
									</div>
									<div>
										<dt>Link</dt>
										<dd>
											<AutoLink
												v-if="source.link"
												:to="source.link"
												target="_blank"
												rel="noopener noreferrer"
												class="inline-flex max-w-full items-start gap-1 hover:text-contrast"
											>
												<span class="min-w-0 break-all underline">{{ source.link }}</span>
												<ExternalIcon class="mt-0.5 size-4 shrink-0" aria-hidden="true" />
											</AutoLink>
											<p v-else class="empty-field">None provided.</p>
										</dd>
									</div>
									<div>
										<dt>How it's used</dt>
										<dd>
											<p v-if="source.note" class="field-text">
												{{ source.note }}
											</p>
											<p v-else class="empty-field">None provided.</p>
										</dd>
									</div>
								</dl>
							</div>
							<p v-else class="empty-field">None listed.</p>
						</dd>
					</div>
					<div v-if="'note' in section.data">
						<dt>
							{{
								section.type === 'epilepsy_triggers'
									? 'Photosensitivity triggers'
									: section.type === 'archived'
										? 'Reason for archiving'
										: 'Explanation'
							}}
						</dt>
						<dd>
							<p v-if="section.data.note?.trim()" class="field-text">
								{{ section.data.note }}
							</p>
							<p v-else class="empty-field">None provided.</p>
						</dd>
					</div>
				</dl>
			</template>
		</ProjectReviewDisclosureCard>
	</div>
</template>

<style scoped>
.disclosure-details {
	container: disclosure-view / inline-size;
}
.disclosure-fields {
	@apply m-0 flex min-w-0 flex-col gap-4;
}
.disclosure-fields > div {
	@apply grid min-w-0 grid-cols-1 gap-1.5;
}
.disclosure-fields dt {
	@apply text-sm font-semibold text-secondary;
}
.disclosure-fields dd {
	@apply m-0 min-w-0 break-words leading-normal text-primary;
}
.disclosure-fields ul {
	@apply m-0 space-y-1 pl-4;
}
.disclosure-fields li,
.field-text {
	@apply whitespace-pre-wrap;
}
.field-text,
.empty-field {
	@apply m-0;
}
.empty-field {
	@apply text-secondary opacity-80;
}
@container disclosure-view (min-width: 32rem) {
	.disclosure-fields {
		@apply gap-2;
	}
	.disclosure-fields > div {
		@apply grid-cols-[142px_minmax(0,1fr)] gap-x-3;
	}
	.disclosure-fields > .original-works {
		@apply grid-cols-1;
	}
	.original-works .disclosure-fields > div {
		grid-template-columns: calc(142px - 0.75rem - 2px) minmax(0, 1fr);
	}
}
</style>
