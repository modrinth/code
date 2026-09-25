<template>
	<EditModal
		ref="modal"
		:section="formatMessage(reviewMessages.links)"
		:saving="saving"
		:can-save="canSave"
		width="52rem"
		@cancel="reset"
		@save="save"
	>
		<div class="flex min-w-0 flex-col gap-8">
			<section v-for="section in linkSections" :key="section.id" class="min-w-0">
				<div class="mb-2.5 flex flex-wrap items-center justify-between gap-3">
					<h3 class="m-0 text-xl font-semibold text-contrast">
						{{ section.title }}
					</h3>
					<TeleportOverflowMenu
						v-if="section.id === 'donations' && section.rows.length > 0"
						:options="donationPlatformOptions"
						:label="formatMessage(messages.addLink)"
						:disabled="saving || donationPlatformOptions.length === 0"
						:icon-only="false"
						type="outlined"
						placement="bottom-end"
					>
						<PlusIcon aria-hidden="true" />
						{{ formatMessage(messages.addLink) }}
					</TeleportOverflowMenu>
				</div>
				<Table
					:columns="section.columns"
					:data="section.rows"
					row-key="id"
					:table-min-width="section.rows.length ? '36rem' : undefined"
				>
					<template #header-url="{ column }">
						<span class="ml-3.5">{{ column.label }}</span>
					</template>
					<template #empty-state>
						<EmptyState :description="formatMessage(messages.donationsDescription)" class="my-4">
							<template #actions>
								<TeleportOverflowMenu
									:options="donationPlatformOptions"
									:label="formatMessage(messages.addLink)"
									:disabled="saving || donationPlatformOptions.length === 0"
									:icon-only="false"
									type="outlined"
									placement="bottom-start"
									class="-mt-4"
								>
									<PlusIcon aria-hidden="true" />
									{{ formatMessage(messages.addLink) }}
								</TeleportOverflowMenu>
							</template>
						</EmptyState>
					</template>
					<template #cell-name="{ row }">
						<label v-tooltip="row.description" :for="row.inputId" class="font-medium">
							{{ row.name }}
						</label>
					</template>
					<template #cell-url="{ row }">
						<div class="flex min-w-0 items-center gap-2">
							<Input
								:id="row.inputId"
								:model-value="row.value"
								:type="row.donation?.mode === 'username' ? 'text' : 'url'"
								:placeholder="
									row.donation
										? donationPlaceholder(row.donation)
										: formatMessage(messages.urlPlaceholder)
								"
								:maxlength="2048"
								:disabled="saving"
								:aria-describedby="row.inputId + '-help'"
								autocapitalize="none"
								autocomplete="off"
								:spellcheck="false"
								wrapper-class="min-w-0 flex-1 !bg-surface-3 !border-surface-4 !text-contrast"
								input-class="!text-contrast"
								@update:model-value="changeLink(row, $event)"
							/>
							<Tabs
								v-if="row.donation?.id && usernamePrefixes[row.donation.id]"
								:value="row.donation.mode"
								:tabs="donationModeTabs"
								:disabled="saving"
								color="gray"
								class="shrink-0"
								@update:value="changeDonationMode(row.donation, $event)"
							/>
							<IconButton
								v-tooltip="
									visitUrl(row.url)
										? formatMessage(messages.visitLink, {
												url: visitUrl(row.url),
											})
										: null
								"
								type="base"
								class="shrink-0"
								:label="formatMessage(messages.visitLink, { url: visitUrl(row.url) })"
								:disabled="!visitUrl(row.url)"
								@mousedown.stop
								@click.stop="visitLink(row.url)"
							>
								<ExternalIcon />
							</IconButton>
							<IconButton
								v-if="row.donation"
								v-tooltip="formatMessage(messages.removeDonation)"
								type="base"
								class="shrink-0"
								:label="formatMessage(messages.removeDonation)"
								:disabled="saving"
								@click.stop="removeDonation(row.donation)"
							>
								<XIcon />
							</IconButton>
						</div>
						<span :id="row.inputId + '-help'" class="sr-only">{{ row.description }}</span>
					</template>
				</Table>
			</section>
			<p v-if="!urlsValid" class="m-0 text-sm text-red" role="alert">
				{{ formatMessage(messages.invalidUrl) }}
			</p>
		</div>
	</EditModal>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { ExternalIcon, PlusIcon, XIcon } from '@modrinth/assets'
import {
	defineMessages,
	EmptyState,
	IconButton,
	Input,
	Table,
	type TableColumn,
	Tabs,
	type TabsValue,
	TeleportOverflowMenu,
	useVIntl,
} from '@modrinth/ui'
import { computed, nextTick, reactive, ref, useTemplateRef } from 'vue'

import {
	type DonationInput,
	donationInput,
	donationUsernamePrefixes as usernamePrefixes,
	setDonationInput,
	toggleDonationInput,
} from '~/helpers/donation-links'
import { normalizeProjectUrl } from '~/helpers/project-url'

import { projectReviewMessages as reviewMessages } from '../../messages'
import EditModal from './edit-modal.vue'
import { useProjectInfoEdit } from './use-project-edit'

type EditableLinkField = 'discord' | 'issues' | 'site' | 'source' | 'store' | 'wiki'
type ProjectLinkUrls = Labrinth.Projects.v3.Project['link_urls']

interface DonationRow extends DonationInput {
	key: number
}

type LinkTableRow = {
	id: string
	inputId: string
	name: string
	description: string
	value: string
	url: string
	field?: EditableLinkField
	donation?: DonationRow
}

const messages = defineMessages({
	issues: {
		id: 'project.settings.links.issues',
		defaultMessage: 'Issue tracker',
	},
	issuesDescription: {
		id: 'project.settings.links.issues-description',
		defaultMessage: 'A place for users to report bugs, issues, and concerns about your project.',
	},
	source: {
		id: 'project.settings.links.source',
		defaultMessage: 'Source code',
	},
	sourceDescription: {
		id: 'project.settings.links.source-description',
		defaultMessage: 'A page/repository containing the source code for your project',
	},
	wiki: { id: 'project.settings.links.wiki', defaultMessage: 'Wiki page' },
	wikiDescription: {
		id: 'project.settings.links.wiki-description',
		defaultMessage: 'A page containing information, documentation, and help for the project.',
	},
	discord: {
		id: 'project.settings.links.discord',
		defaultMessage: 'Discord invite',
	},
	discordDescription: {
		id: 'project.settings.links.discord-description',
		defaultMessage: 'An invitation link to your Discord server.',
	},
	site: { id: 'project.settings.links.site', defaultMessage: 'Website' },
	siteDescription: {
		id: 'project.settings.links.site-description',
		defaultMessage: "Your server's official website.",
	},
	store: { id: 'project.settings.links.store', defaultMessage: 'Store' },
	storeDescription: {
		id: 'project.settings.links.store-description',
		defaultMessage: "A link to your server's store or shop.",
	},
	linkType: {
		id: 'project.settings.links.link-type',
		defaultMessage: 'Link type',
	},
	donationPlatform: {
		id: 'project.settings.links.donation-platform',
		defaultMessage: 'Platform',
	},
	donationLink: {
		id: 'project.settings.links.donation-link',
		defaultMessage: 'Link',
	},
	donations: {
		id: 'project.settings.links.donations',
		defaultMessage: 'Donation links',
	},
	addLink: {
		id: 'project.settings.links.add-link',
		defaultMessage: 'Add link',
	},
	removeDonation: {
		id: 'project.settings.links.remove-donation-link',
		defaultMessage: 'Remove donation link',
	},
	donationsDescription: {
		id: 'project.settings.links.donations-description',
		defaultMessage: 'Add donation links for users to support you directly.',
	},
	urlPlaceholder: {
		id: 'project.settings.links.url-placeholder',
		defaultMessage: 'Enter a valid URL',
	},
	donationUsernamePlaceholder: {
		id: 'project.settings.links.donation-username-placeholder',
		defaultMessage: 'Enter your {platform} username',
	},
	donationUrl: {
		id: 'project.settings.links.donation-url',
		defaultMessage: 'URL',
	},
	donationUsername: {
		id: 'project.settings.links.donation-username',
		defaultMessage: 'Username',
	},
	visitLink: {
		id: 'project.settings.links.visit-link',
		defaultMessage: 'Visit {url}',
	},
	invalidUrl: {
		id: 'project.settings.links.invalid-url',
		defaultMessage: 'Enter valid HTTP or HTTPS URLs before saving.',
	},
})

const { formatMessage } = useVIntl()
const tags = useGeneratedState()
const { project, saving, beginEditing, saveProject } = useProjectInfoEdit()
const modal = useTemplateRef<InstanceType<typeof EditModal>>('modal')
const currentLinks = reactive<Record<EditableLinkField, string>>({
	discord: '',
	issues: '',
	site: '',
	source: '',
	store: '',
	wiki: '',
})
const donationLinks = ref<DonationRow[]>([])
let nextRowKey = 0

const isServerProject = computed(() => project.value?.minecraft_server != null)
const visibleFields = computed<EditableLinkField[]>(() =>
	isServerProject.value
		? ['site', 'store', 'wiki', 'discord']
		: ['issues', 'source', 'wiki', 'discord'],
)
const fieldMessages = computed(() => ({
	issues: { title: messages.issues, description: messages.issuesDescription },
	source: { title: messages.source, description: messages.sourceDescription },
	wiki: { title: messages.wiki, description: messages.wikiDescription },
	discord: {
		title: messages.discord,
		description: messages.discordDescription,
	},
	site: { title: messages.site, description: messages.siteDescription },
	store: { title: messages.store, description: messages.storeDescription },
}))
const donationModeTabs = computed(() => [
	{ value: 'username', label: formatMessage(messages.donationUsername) },
	{ value: 'url', label: formatMessage(messages.donationUrl) },
])
const donationPlatformOptions = computed(() =>
	tags.value.donationPlatforms
		.filter((platform) => !donationLinks.value.some((row) => row.id === platform.short))
		.map((platform) => ({
			id: platform.short,
			label: platform.name,
			action: () => addDonation(platform.short),
		})),
)
const linkColumns = computed<TableColumn<'name' | 'url'>[]>(() => [
	{
		key: 'name',
		label: formatMessage(messages.linkType),
		width: '12rem',
		cellClass: 'pr-3 py-3',
	},
	{
		key: 'url',
		label: formatMessage(messages.donationUrl),
		cellClass: 'py-3 pl-1',
	},
])
const linkRows = computed<LinkTableRow[]>(() => [
	...visibleFields.value.map((field) => ({
		id: field,
		inputId: 'project-review-link-' + field,
		name: formatMessage(fieldMessages.value[field].title),
		description: formatMessage(fieldMessages.value[field].description),
		value: currentLinks[field],
		url: currentLinks[field],
		field,
	})),
	...(isServerProject.value
		? []
		: donationLinks.value.map((donation) => ({
				id: 'donation-' + donation.key,
				inputId: 'project-review-donation-' + donation.key,
				name: donationPlatformTitle(donation),
				description: formatMessage(messages.donationsDescription),
				value: donation.input,
				url: donation.url,
				donation,
			}))),
])
const linkSections = computed(() => [
	{
		id: 'links',
		title: formatMessage(reviewMessages.links),
		columns: linkColumns.value,
		rows: linkRows.value.filter((row) => row.field),
	},
	...(isServerProject.value
		? []
		: [
				{
					id: 'donations',
					title: formatMessage(messages.donations),
					columns: linkColumns.value.map((column) => ({
						...column,
						label: formatMessage(
							column.key === 'name' ? messages.donationPlatform : messages.donationLink,
						),
					})),
					rows: linkRows.value.filter((row) => row.donation),
				},
			]),
])

function makeRow(id?: string, url = ''): DonationRow {
	return { key: nextRowKey++, ...donationInput(id, url) }
}

function donationRowsFromLinks(links?: ProjectLinkUrls) {
	return tags.value.donationPlatforms.flatMap((platform) => {
		const url = links?.[platform.short]?.url
		return url ? [makeRow(platform.short, url)] : []
	})
}

function reset() {
	for (const field of visibleFields.value) {
		currentLinks[field] = project.value?.link_urls[field]?.url ?? ''
	}
	donationLinks.value = donationRowsFromLinks(project.value?.link_urls)
}

function show() {
	beginEditing()
	reset()
	modal.value?.show()
}

async function addDonation(platform: string) {
	donationLinks.value.push(makeRow(platform))
	await nextTick()
	document.getElementById(`project-review-donation-${donationLinks.value.at(-1)?.key}`)?.focus()
}

function removeDonation(row: DonationRow) {
	donationLinks.value = donationLinks.value.filter((link) => link.key !== row.key)
}

function changeLink(row: LinkTableRow, value: string | number | undefined) {
	if (row.donation) setDonationInput(row.donation, value ?? '')
	else if (row.field) currentLinks[row.field] = String(value ?? '')
}

function changeDonationMode(row: DonationRow, mode: TabsValue) {
	if ((mode === 'username' || mode === 'url') && row.mode !== mode) toggleDonationInput(row)
}

function donationPlatformTitle(row: DonationRow) {
	return (
		tags.value.donationPlatforms.find((platform) => platform.short === row.id)?.name ?? row.id ?? ''
	)
}

function donationPlaceholder(row: DonationRow) {
	if (row.mode !== 'username') return formatMessage(messages.urlPlaceholder)
	return formatMessage(messages.donationUsernamePlaceholder, {
		platform: donationPlatformTitle(row),
	})
}

function visitUrl(value: string) {
	try {
		const url = new URL(normalizeProjectUrl(value))
		return ['http:', 'https:'].includes(url.protocol) ? url.href : ''
	} catch {
		return ''
	}
}

function visitLink(value: string) {
	const url = visitUrl(value)
	if (url) window.open(url, '_blank', 'noopener,noreferrer')
}

function isValidUrl(value: string) {
	if (!value.trim()) return true
	try {
		return ['http:', 'https:'].includes(new URL(normalizeProjectUrl(value)).protocol)
	} catch {
		return false
	}
}

const urlsValid = computed(
	() =>
		visibleFields.value.every((field) => isValidUrl(currentLinks[field])) &&
		donationLinks.value.every((row) => isValidUrl(row.url)),
)
const patchData = computed<Record<string, string | null>>(() => {
	const patch: Record<string, string | null> = {}
	for (const field of visibleFields.value) {
		const value = normalizeProjectUrl(currentLinks[field]) || null
		if (value !== (project.value?.link_urls[field]?.url ?? null)) patch[field] = value
	}
	if (!isServerProject.value) {
		for (const platform of tags.value.donationPlatforms) {
			const value =
				normalizeProjectUrl(
					donationLinks.value.find((row) => row.id === platform.short)?.url ?? '',
				) || null
			if (value !== (project.value?.link_urls[platform.short]?.url ?? null)) {
				patch[platform.short] = value
			}
		}
	}
	return patch
})
const canSave = computed(() => urlsValid.value && Object.keys(patchData.value).length > 0)

async function save() {
	if (!canSave.value) return
	if (await saveProject({ link_urls: patchData.value })) modal.value?.hide()
}

defineExpose({ show })
</script>
