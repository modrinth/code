<template>
	<div>
		<ConfirmLeaveModal ref="confirmLeaveModal" />
		<!-- Server Project Links -->
		<section v-if="isServerProject" class="universal-card">
			<h2>External links</h2>
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
				<LinkCheckMessage :check="siteCheck" />
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
				<LinkCheckMessage :check="storeCheck" />
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
				<LinkCheckMessage :check="wikiCheck" />
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
				<LinkCheckMessage :check="discordInviteCheck" />
			</div>
		</section>

		<!-- Standard Project Links -->
		<section v-if="!isServerProject" class="universal-card">
			<h2>External links</h2>
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
				<StyledInput
					id="project-issue-tracker"
					v-model="current.issues"
					type="url"
					placeholder="Enter a valid URL"
					:maxlength="2048"
					:disabled="!hasPermission"
				/>
				<LinkCheckMessage :check="issuesCheck" />
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
				<StyledInput
					id="project-source-code"
					v-model="current.source"
					type="url"
					:maxlength="2048"
					placeholder="Enter a valid URL"
					:disabled="!hasPermission"
				/>
				<LinkCheckMessage :check="sourceCheck" />
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
				<StyledInput
					id="project-wiki-page"
					v-model="current.wiki"
					type="url"
					:maxlength="2048"
					placeholder="Enter a valid URL"
					:disabled="!hasPermission"
				/>
				<LinkCheckMessage :check="wikiCheck" />
			</div>
			<div class="adjacent-input">
				<label id="project-discord-invite" title="An invitation link to your Discord server.">
					<span class="label__title">Discord invite </span>
					<span class="label__description"> An invitation link to your Discord server. </span>
				</label>
				<StyledInput
					id="project-discord-invite"
					v-model="current.discord"
					type="url"
					:maxlength="2048"
					placeholder="Enter a valid URL"
					:disabled="!hasPermission"
				/>
				<LinkCheckMessage :check="discordInviteCheck" />
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
				<StyledInput
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
				<LinkCheckMessage :check="donationCheckState(donationLink, index)" />
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

<script setup>
import { checkLink, getLinkCheckState, isLinkCheckPending, useLinkCheck } from '@modrinth/moderation'
import {
	Combobox,
	ConfirmLeaveModal,
	defineMessage,
	injectModrinthClient,
	injectNotificationManager,
	injectProjectPageContext,
	StyledInput,
	UnsavedChangesPopup,
	usePageLeaveSafety,
	useSavable,
} from '@modrinth/ui'

import LinkCheckMessage from '@/components/LinkCheckMessage.vue'

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

const isServerProject = computed(() => project.value?.minecraft_server != null)

const { saved, current, reset: resetFields } = useSavable(
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

function donationRowsFromLinks(linkUrls) {
	const rows = (tags.value.donationPlatforms ?? [])
		.filter((platform) => linkUrls?.[platform.short]?.url)
		.map((platform) => ({ id: platform.short, url: linkUrls[platform.short].url }))
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

function fieldContext(field, getUrl, extra) {
	return computed(() => ({ field, url: getUrl(), ...extra }))
}

const discordContext = fieldContext('discord', () => current.value.discord, { platformName: 'Discord' })
const issuesContext = fieldContext('issues', () => current.value.issues)
const sourceContext = fieldContext('source', () => current.value.source)
const wikiContext = fieldContext('wiki', () => current.value.wiki)
const siteContext = fieldContext('site', () => current.value.site)
const storeContext = fieldContext('store', () => current.value.store)

const discordInviteCheck = useLinkCheck(discordContext)
const issuesCheck = useLinkCheck(issuesContext)
const sourceCheck = useLinkCheck(sourceContext)
const wikiCheck = useLinkCheck(wikiContext)
const siteCheck = useLinkCheck(siteContext)
const storeCheck = useLinkCheck(storeContext)

function donationContext(row) {
	return {
		field: row.id,
		url: row.url,
		isDonation: true,
		platformName: tags.value.donationPlatforms.find((platform) => platform.short === row.id)?.name,
	}
}

function donationCheckState(row, index) {
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

	return getLinkCheckState(donationContext(row))
}

const donationCheckTimers = new Map()
watch(
	donationLinks,
	(rows) => {
		rows.forEach((row, index) => {
			if (!row.id || !row.url) return
			clearTimeout(donationCheckTimers.get(index))
			donationCheckTimers.set(
				index,
				setTimeout(() => checkLink(donationContext(row)), 500),
			)
		})
	},
	{ deep: true, immediate: true },
)

const hasPermission = computed(() => {
	const EDIT_DETAILS = 1 << 2
	return (currentMember.value?.permissions & EDIT_DETAILS) === EDIT_DETAILS
})

function donationsMapFromLinkUrls(linkUrls) {
	const donations = {}
	for (const platform of tags.value.donationPlatforms ?? []) {
		donations[platform.short] = linkUrls?.[platform.short]?.url ?? ''
	}
	return donations
}

const donationsOriginal = computed(() =>
	isServerProject.value ? {} : donationsMapFromLinkUrls(project.value?.link_urls),
)

const donationsModified = computed(() => {
	if (isServerProject.value) return {}
	const donations = {}
	for (const row of donationLinks.value) {
		if (row.id && !(row.id in donations)) donations[row.id] = row.url ?? ''
	}
	return donations
})

function serializeDonationRow(row) {
	return `${row.id ?? ''}:${row.url}`
}

function donationRowsToObject(rows) {
	const entries = {}
	rows.forEach((row, index) => {
		if (!row.url) return
		entries[`donation-row-${index}`] = serializeDonationRow(row)
	})
	return entries
}

const donationsSavedRows = computed(() => donationRowsFromLinks(project.value?.link_urls))

const original = computed(() => ({
	...saved.value,
	...(isServerProject.value ? {} : donationRowsToObject(donationsSavedRows.value)),
}))
const modified = computed(() => ({
	...current.value,
	...(isServerProject.value ? {} : donationRowsToObject(donationLinks.value)),
}))

const hasChanges = computed(() =>
	Object.keys(modified.value).some((key) => modified.value[key] !== original.value[key]),
)

const { confirmLeaveModal } = usePageLeaveSafety(hasChanges)

const patchData = computed(() => {
	const data = {}
	for (const key of Object.keys(current.value)) {
		if (current.value[key] == saved.value[key]) continue
		data[key] = current.value[key] === '' ? null : current.value[key].trim()
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
	const checks = isServerProject.value
		? [siteCheck, storeCheck, wikiCheck, discordInviteCheck]
		: [issuesCheck, sourceCheck, wikiCheck, discordInviteCheck]
	const contexts = isServerProject.value
		? [siteContext, storeContext, wikiContext, discordContext]
		: [issuesContext, sourceContext, wikiContext, discordContext]

	const fieldsInvalid = checks.some((check) => check.value?.severity === 'error')
	const fieldsPending = contexts.some((context) => isLinkCheckPending(context.value))

	const donationsInvalid =
		!isServerProject.value &&
		donationLinks.value.some((row, index) => donationCheckState(row, index)?.severity === 'error')
	const donationsPending =
		!isServerProject.value &&
		donationLinks.value.some((row) => isLinkCheckPending(donationContext(row)))

	return (
		!fieldsInvalid &&
		!fieldsPending &&
		!donationsInvalid &&
		!donationsPending &&
		Object.keys(patchData.value).length > 0
	)
})

const saving = ref(false)

async function save() {
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
	} catch (err) {
		addNotification({
			title: 'Failed to update links',
			text: err.data?.description ?? String(err),
			type: 'error',
		})
	} finally {
		saving.value = false
	}
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
