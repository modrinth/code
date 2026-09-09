<template>
	<div>
		<ConfirmLeaveModal ref="confirmLeaveModal" />
		<!-- Server Project Links -->
		<section v-if="isServerProject" class="universal-card">
			<h2>External links</h2>
			<ValidationMessage
				:check="externalLinksValidation"
				:project-field="JSON.stringify(saved)"
				:current-field="JSON.stringify(current)"
				class="mb-4"
			/>
			<div class="adjacent-input">
				<label id="server-website" title="Your server's website.">
					<span class="label__title">Website</span>
					<span class="label__description">Your server's official website.</span>
				</label>
				<input
					id="server-website"
					v-model="current.site"
					type="url"
					placeholder="Enter a valid URL"
					maxlength="2048"
					:disabled="!hasPermission"
				/>
				<ValidationMessage
					:check="siteCheck"
					:project-field="saved.site"
					:current-field="current.site"
				/>
			</div>
			<div class="adjacent-input">
				<label id="server-store" title="Your server's store page.">
					<span class="label__title">Store</span>
					<span class="label__description">A link to your server's store or shop.</span>
				</label>
				<input
					id="server-store"
					v-model="current.store"
					type="url"
					placeholder="Enter a valid URL"
					maxlength="2048"
					:disabled="!hasPermission"
				/>
				<ValidationMessage
					:check="storeCheck"
					:project-field="saved.store"
					:current-field="current.store"
				/>
			</div>
			<div class="adjacent-input">
				<label
					id="server-wiki"
					title="A page containing information, documentation, and help for the server."
				>
					<span class="label__title">Wiki page</span>
					<span class="label__description"
						>A page containing information, documentation, and help for the server.</span
					>
				</label>
				<input
					id="server-wiki"
					v-model="current.wiki"
					type="url"
					placeholder="Enter a valid URL"
					maxlength="2048"
					:disabled="!hasPermission"
				/>
				<ValidationMessage
					:check="wikiCheck"
					:project-field="saved.wiki"
					:current-field="current.wiki"
				/>
			</div>
			<div class="adjacent-input">
				<label id="server-discord" title="An invitation link to your Discord server.">
					<span class="label__title">Discord</span>
					<span class="label__description">An invitation link to your Discord server.</span>
				</label>
				<input
					id="server-discord"
					v-model="current.discord"
					type="url"
					placeholder="Enter a valid URL"
					maxlength="2048"
					:disabled="!hasPermission"
				/>
				<ValidationMessage
					:check="discordInviteCheck"
					:project-field="saved.discord"
					:current-field="current.discord"
				/>
			</div>
		</section>

		<!-- Standard Project Links -->
		<section v-if="!isServerProject" class="universal-card">
			<h2>External links</h2>
			<ValidationMessage
				:check="externalLinksValidation"
				:project-field="JSON.stringify(saved)"
				:current-field="JSON.stringify(current)"
				class="mb-4"
			/>
			<div class="adjacent-input">
				<label
					id="project-issue-tracker"
					title="A place for users to report bugs, issues, and concerns about your project."
				>
					<span class="label__title">Issue tracker </span>
					<span class="label__description">
						A place for users to report bugs, issues, and concerns about your project.
					</span>
				</label>
				<Input
					id="project-issue-tracker"
					v-model="current.issues"
					type="url"
					placeholder="Enter a valid URL"
					:maxlength="2048"
					:disabled="!hasPermission"
				/>
				<ValidationMessage
					:check="issuesCheck"
					:project-field="saved.issues"
					:current-field="current.issues"
				/>
			</div>
			<div class="adjacent-input">
				<label
					id="project-source-code"
					title="A page/repository containing the source code for your project"
				>
					<span class="label__title">Source code </span>
					<span class="label__description">
						A page/repository containing the source code for your project
					</span>
				</label>
				<Input
					id="project-source-code"
					v-model="current.source"
					type="url"
					:maxlength="2048"
					placeholder="Enter a valid URL"
					:disabled="!hasPermission"
				/>
				<ValidationMessage
					:check="sourceCheck"
					:project-field="saved.source"
					:current-field="current.source"
				/>
			</div>
			<div class="adjacent-input">
				<label
					id="project-wiki-page"
					title="A page containing information, documentation, and help for the project."
				>
					<span class="label__title">Wiki page</span>
					<span class="label__description">
						A page containing information, documentation, and help for the project.
					</span>
				</label>
				<Input
					id="project-wiki-page"
					v-model="current.wiki"
					type="url"
					:maxlength="2048"
					placeholder="Enter a valid URL"
					:disabled="!hasPermission"
				/>
				<ValidationMessage
					:check="wikiCheck"
					:project-field="saved.wiki"
					:current-field="current.wiki"
				/>
			</div>
			<div class="adjacent-input">
				<label id="project-discord-invite" title="An invitation link to your Discord server.">
					<span class="label__title">Discord invite </span>
					<span class="label__description"> An invitation link to your Discord server. </span>
				</label>
				<Input
					id="project-discord-invite"
					v-model="current.discord"
					type="url"
					:maxlength="2048"
					placeholder="Enter a valid URL"
					:disabled="!hasPermission"
				/>
				<ValidationMessage
					:check="discordInviteCheck"
					:project-field="saved.discord"
					:current-field="current.discord"
				/>
			</div>
			<span class="label">
				<span class="label__title">Donation links</span>
				<span class="label__description">
					Add donation links for users to support you directly.
				</span>
			</span>

			<div
				v-for="(donationLink, index) in donationLinks"
				:key="`donation-link-${index}`"
				class="input-group donation-link-group"
			>
				<Input
					v-model="donationLink.url"
					type="url"
					:maxlength="2048"
					placeholder="Enter a valid URL"
					:disabled="!hasPermission"
					@update:model-value="updateDonationLinks"
				/>
				<Combobox
					v-model="donationLink.id"
					:options="donationPlatformOptions"
					placeholder="Select platform"
					:disabled="!hasPermission"
					force-direction="up"
					trigger-type="base"
					class="platform-selector !w-80"
					@update:model-value="updateDonationLinks"
				/>
				<ValidationMessage :check="donationCheckState(donationLink, index)" />
			</div>
		</section>
		<UnsavedChangesPopup
			:original="original"
			:modified="modified"
			:saving="saving"
			:can-save="canSave"
			@reset="reset"
			@save="save"
		/>
	</div>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import {
	Combobox,
	commonProjectSettingsMessages,
	ConfirmLeaveModal,
	defineMessage,
	injectModrinthClient,
	injectNotificationManager,
	injectProjectPageContext,
	Input,
	UnsavedChangesPopup,
	usePageLeaveSafety,
	useSavable,
} from '@modrinth/ui'
import { isAdmin } from '@modrinth/utils'

import ValidationMessage from '@/components/ValidationMessage.vue'
import { useProjectNagMessages } from '~/composables/project-nag-validation'

type EditableLinkField = 'discord' | 'issues' | 'site' | 'source' | 'store' | 'wiki'
type EditableLinks = Partial<Record<EditableLinkField, string>>
type ProjectLinkUrls = Labrinth.Projects.v3.Project['link_urls']

interface DonationRow {
	id?: string
	url?: string
}

const tags = useGeneratedState()

const donationPlatformOptions = computed(() =>
	tags.value.donationPlatforms.map((platform) => ({
		value: platform.short,
		label: platform.name,
	})),
)

const { projectV3: project, currentMember, invalidate } = injectProjectPageContext()
const { labrinth } = injectModrinthClient()
const { addNotification } = injectNotificationManager()

useProjectSettingsHeadTitle(commonProjectSettingsMessages.links)

const isServerProject = computed(() => project.value?.minecraft_server != null)

const {
	saved,
	current,
	reset: resetFields,
} = useSavable<EditableLinks>(
	() => {
		if (isServerProject.value) {
			return {
				site: project.value.link_urls?.site?.url ?? '',
				store: project.value.link_urls?.store?.url ?? '',
				wiki: project.value.link_urls?.wiki?.url ?? '',
				discord: project.value.link_urls?.discord?.url ?? '',
			}
		}
		return {
			issues: project.value.link_urls?.issues?.url ?? '',
			source: project.value.link_urls?.source?.url ?? '',
			wiki: project.value.link_urls?.wiki?.url ?? '',
			discord: project.value.link_urls?.discord?.url ?? '',
		}
	},
	() => {},
)

function donationRowsFromLinks(linkUrls?: ProjectLinkUrls): DonationRow[] {
	const rows: DonationRow[] = (tags.value.donationPlatforms ?? []).flatMap((platform) => {
		const url = linkUrls?.[platform.short]?.url
		return url ? [{ id: platform.short, url }] : []
	})
	rows.push({ id: undefined, url: undefined })
	return rows
}

const donationLinks = ref(donationRowsFromLinks(project.value?.link_urls))

function resetDonations() {
	donationLinks.value = donationRowsFromLinks(project.value?.link_urls)
}

function reset() {
	resetFields()
	resetDonations()
}

const externalLinksValidation = useProjectNagMessages('external-links')

function useLinkFieldMessages(field: EditableLinkField, includeSourceRequirement = false) {
	const verification = useProjectNagMessages('source-issues-discord-links', field)
	const discordMisuse = useProjectNagMessages('non-discord-link-fields', field)
	const sourceRequirement = useProjectNagMessages('source-availability', field)
	return computed(() => [
		...verification.value,
		...(field === 'discord' ? [] : discordMisuse.value),
		...(includeSourceRequirement ? sourceRequirement.value : []),
	])
}

const discordInviteCheck = useLinkFieldMessages('discord')
const issuesCheck = useLinkFieldMessages('issues')
const sourceCheck = useLinkFieldMessages('source', true)
const wikiCheck = useLinkFieldMessages('wiki')
const siteCheck = useLinkFieldMessages('site')
const storeCheck = useLinkFieldMessages('store')

function donationCheckState(row: DonationRow, index: number) {
	if (row.url && !row.id) {
		return {
			severity: 'error',
			message: defineMessage({
				id: 'project.settings.links.donation.no-type',
				defaultMessage: 'Please select a platform for this Donation link.',
			}),
		}
	}

	if (row.id) {
		const firstIndex = donationLinks.value.findIndex((other) => other.id === row.id)
		if (firstIndex !== index) {
			return {
				severity: 'error',
				message: defineMessage({
					id: 'project.settings.links.donation.duplicate-type',
					defaultMessage: 'You already have another {platform} link.',
				}),
				values: {
					platform:
						tags.value.donationPlatforms.find((platform) => platform.short === row.id)?.name ??
						row.id,
				},
			}
		}
	}

	return undefined
}

const isAdminUser = computed(() => isAdmin(currentMember.value?.user))

const hasPermission = computed(() => {
	const EDIT_DETAILS = 1 << 2
	return isAdminUser.value || (currentMember.value?.permissions & EDIT_DETAILS) === EDIT_DETAILS
})

function donationsMapFromLinkUrls(linkUrls?: ProjectLinkUrls): Record<string, string> {
	const donations: Record<string, string> = {}
	for (const platform of tags.value.donationPlatforms ?? []) {
		donations[platform.short] = linkUrls?.[platform.short]?.url ?? ''
	}
	return donations
}

const donationsOriginal = computed<Record<string, string>>(() =>
	isServerProject.value ? {} : donationsMapFromLinkUrls(project.value?.link_urls),
)

const donationsModified = computed<Record<string, string>>(() => {
	if (isServerProject.value) return {}
	const donations: Record<string, string> = {}
	for (const row of donationLinks.value) {
		if (row.id && !(row.id in donations)) donations[row.id] = row.url ?? ''
	}
	return donations
})

function serializeDonationRow(row: DonationRow): string {
	return `${row.id ?? ''}:${row.url}`
}

function donationRowsToObject(rows: DonationRow[]): Record<string, string> {
	const entries: Record<string, string> = {}
	rows.forEach((row, index) => {
		if (!row.url) return
		entries[`donation-row-${index}`] = serializeDonationRow(row)
	})
	return entries
}

const donationsSavedRows = computed(() => donationRowsFromLinks(project.value?.link_urls))

const originalDonationRows = computed<Record<string, string>>(() =>
	isServerProject.value ? {} : donationRowsToObject(donationsSavedRows.value),
)
const modifiedDonationRows = computed<Record<string, string>>(() =>
	isServerProject.value ? {} : donationRowsToObject(donationLinks.value),
)

const original = computed<Record<string, string | undefined>>(() => ({
	...saved.value,
	...originalDonationRows.value,
}))
const modified = computed<Record<string, string | undefined>>(() => {
	const donations: Record<string, string | undefined> = { ...modifiedDonationRows.value }
	for (const key of Object.keys(originalDonationRows.value)) {
		if (!(key in donations)) donations[key] = undefined
	}
	return { ...current.value, ...donations }
})

const hasChanges = computed(() =>
	Object.keys(modified.value).some((key) => modified.value[key] !== original.value[key]),
)

const { confirmLeaveModal } = usePageLeaveSafety(hasChanges)

const patchData = computed<Record<string, string | null>>(() => {
	const data: Record<string, string | null> = {}
	for (const key of Object.keys(current.value) as EditableLinkField[]) {
		const value = current.value[key]
		if (value == null || value === saved.value[key]) continue
		data[key] = value === '' ? null : value.trim()
	}
	if (!isServerProject.value) {
		for (const platform of tags.value.donationPlatforms ?? []) {
			const newUrl = donationsModified.value[platform.short] ?? ''
			const oldUrl = donationsOriginal.value[platform.short] ?? ''
			if (newUrl === oldUrl) continue
			data[platform.short] = newUrl === '' ? null : newUrl.trim()
		}
	}
	return data
})

const canSave = computed(() => {
	if (!hasPermission.value || Object.keys(patchData.value).length === 0) return false

	const donationsInvalid =
		!isServerProject.value &&
		donationLinks.value.some((row, index) => donationCheckState(row, index)?.severity === 'error')

	return !donationsInvalid
})

const saving = ref(false)

async function save() {
	if (!canSave.value) return
	const data = patchData.value
	if (Object.keys(data).length === 0) return

	saving.value = true
	try {
		await labrinth.projects_v3.edit(project.value.id, {
			link_urls: data,
		})
		await invalidate()
		reset()
		addNotification({
			title: 'Links updated',
			text: isServerProject.value
				? 'Your server links have been updated.'
				: 'Your links have been updated.',
			type: 'success',
		})
	} catch (err: unknown) {
		addNotification({
			title: 'Failed to update links',
			text: getErrorDescription(err),
			type: 'error',
		})
	} finally {
		saving.value = false
	}
}

function getErrorDescription(error: unknown): string {
	if (typeof error === 'object' && error !== null && 'data' in error) {
		const data = (error as { data?: { description?: string } }).data
		if (data?.description) return data.description
	}

	return error instanceof Error ? error.message : String(error)
}

function updateDonationLinks() {
	const links = donationLinks.value
	links.forEach((link) => {
		if (link.url) {
			const url = link.url.toLowerCase()
			if (url.includes('patreon.com')) {
				link.id = 'patreon'
			} else if (url.includes('ko-fi.com')) {
				link.id = 'ko-fi'
			} else if (url.includes('paypal.com') || url.includes('paypal.me')) {
				link.id = 'paypal'
			} else if (url.includes('buymeacoffee.com') || url.includes('buymeacoff.ee')) {
				link.id = 'bmac'
			} else if (url.includes('github.com/sponsors')) {
				link.id = 'github'
			}
		}
	})
	if (!links.find((link) => !(link.url && link.id))) {
		links.push({
			id: undefined,
			url: undefined,
		})
	}
	donationLinks.value = links
}
</script>
<style lang="scss" scoped>
.donation-link-group {
	input {
		flex-grow: 2;
		max-width: 26rem;
	}
}
</style>
