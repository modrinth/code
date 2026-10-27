<template>
	<EditModal
		ref="modal"
		section="Links"
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
						label="Add link"
						:disabled="saving || donationPlatformOptions.length === 0"
						:icon-only="false"
						type="outlined"
						placement="bottom-end"
					>
						<PlusIcon aria-hidden="true" />
						Add link
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
						<EmptyState
							description="Add donation links for users to support you directly."
							class="my-4"
						>
							<template #actions>
								<TeleportOverflowMenu
									:options="donationPlatformOptions"
									label="Add link"
									:disabled="saving || donationPlatformOptions.length === 0"
									:icon-only="false"
									type="outlined"
									placement="bottom-start"
									class="-mt-4"
								>
									<PlusIcon aria-hidden="true" />
									Add link
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
									row.donation ? donationPlaceholder(row.donation) : 'Enter a valid URL'
								"
								:maxlength="2048"
								:disabled="saving"
								:aria-describedby="`${row.inputId}-help`"
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
								v-tooltip="visitUrl(row.url) ? `Visit ${visitUrl(row.url)}` : null"
								type="base"
								class="shrink-0"
								:label="`Visit ${visitUrl(row.url)}`"
								:disabled="!visitUrl(row.url)"
								@mousedown.stop
								@click.stop="visitLink(row.url)"
							>
								<ExternalIcon />
							</IconButton>
							<IconButton
								v-if="row.donation"
								v-tooltip="'Remove donation link'"
								type="base"
								class="shrink-0"
								label="Remove donation link"
								:disabled="saving"
								@click.stop="removeDonation(row.donation)"
							>
								<XIcon />
							</IconButton>
						</div>
						<span :id="`${row.inputId}-help`" class="sr-only">{{ row.description }}</span>
					</template>
				</Table>
			</section>
			<p v-if="!urlsValid" class="m-0 text-sm text-red" role="alert">
				Enter valid HTTP or HTTPS URLs before saving.
			</p>
		</div>
	</EditModal>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { ExternalIcon, PlusIcon, XIcon } from '@modrinth/assets'
import {
	EmptyState,
	IconButton,
	Input,
	Table,
	type TableColumn,
	Tabs,
	type TabsValue,
	TeleportOverflowMenu,
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
	issues: {
		title: 'Issue tracker',
		description: 'A place for users to report bugs, issues, and concerns about your project.',
	},
	source: {
		title: 'Source code',
		description: 'A page/repository containing the source code for your project',
	},
	wiki: {
		title: 'Wiki page',
		description: 'A page containing information, documentation, and help for the project.',
	},
	discord: {
		title: 'Discord invite',
		description: 'An invitation link to your Discord server.',
	},
	site: { title: 'Website', description: "Your server's official website." },
	store: {
		title: 'Store',
		description: "A link to your server's store or shop.",
	},
}))
const donationModeTabs = computed(() => [
	{ value: 'username', label: 'Username' },
	{ value: 'url', label: 'URL' },
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
		label: 'Link type',
		width: '12rem',
		cellClass: 'pr-3 py-3',
	},
	{
		key: 'url',
		label: 'URL',
		cellClass: 'py-3 pl-1',
	},
])
const linkRows = computed<LinkTableRow[]>(() => [
	...visibleFields.value.map((field) => ({
		id: field,
		inputId: `project-review-link-${field}`,
		name: fieldMessages.value[field].title,
		description: fieldMessages.value[field].description,
		value: currentLinks[field],
		url: currentLinks[field],
		field,
	})),
	...(isServerProject.value
		? []
		: donationLinks.value.map((donation) => ({
				id: `donation-${donation.key}`,
				inputId: `project-review-donation-${donation.key}`,
				name: donationPlatformTitle(donation),
				description: 'Add donation links for users to support you directly.',
				value: donation.input,
				url: donation.url,
				donation,
			}))),
])
const linkSections = computed(() => [
	{
		id: 'links',
		title: 'Links',
		columns: linkColumns.value,
		rows: linkRows.value.filter((row) => row.field),
	},
	...(isServerProject.value
		? []
		: [
				{
					id: 'donations',
					title: 'Donation links',
					columns: linkColumns.value.map((column) => ({
						...column,
						label: column.key === 'name' ? 'Platform' : 'Link',
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
	if (row.mode !== 'username') return 'Enter a valid URL'
	return `Enter your ${donationPlatformTitle(row)} username`
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
