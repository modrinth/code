<template>
	<div>
		<!-- Server Project Links -->
		<section v-if="isServerProject" class="universal-card">
			<h2>External links</h2>
			<div class="adjacent-input">
				<label id="server-website" title="Your server's website.">
					<span class="label__title">Website</span>
					<span class="label__description">Your server's official website.</span>
				</label>
				<LinkWarningIcon :check="siteCheck" />
				<input
					id="server-website"
					v-model="siteUrl"
					type="url"
					placeholder="Enter a valid URL"
					maxlength="2048"
					:disabled="!hasPermission"
				/>
			</div>
			<div class="adjacent-input">
				<label id="server-store" title="Your server's store page.">
					<span class="label__title">Store</span>
					<span class="label__description">A link to your server's store or shop.</span>
				</label>
				<LinkWarningIcon :check="storeCheck" />
				<input
					id="server-store"
					v-model="storeUrl"
					type="url"
					placeholder="Enter a valid URL"
					maxlength="2048"
					:disabled="!hasPermission"
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
				<LinkWarningIcon :check="wikiCheck" />
				<input
					id="server-wiki"
					v-model="wikiUrl"
					type="url"
					placeholder="Enter a valid URL"
					maxlength="2048"
					:disabled="!hasPermission"
				/>
			</div>
			<div class="adjacent-input">
				<label id="server-discord" title="An invitation link to your Discord server.">
					<span class="label__title">Discord</span>
					<span class="label__description">An invitation link to your Discord server.</span>
				</label>
				<LinkWarningIcon :check="discordInviteCheck" />
				<input
					id="server-discord"
					v-model="discordUrl"
					type="url"
					placeholder="Enter a valid URL"
					maxlength="2048"
					:disabled="!hasPermission"
				/>
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
				<LinkWarningIcon :check="issuesCheck" />
				<StyledInput
					id="project-issue-tracker"
					v-model="issuesUrl"
					type="url"
					placeholder="Enter a valid URL"
					:maxlength="2048"
					:disabled="!hasPermission"
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
				<LinkWarningIcon :check="sourceCheck" />
				<StyledInput
					id="project-source-code"
					v-model="sourceUrl"
					type="url"
					:maxlength="2048"
					placeholder="Enter a valid URL"
					:disabled="!hasPermission"
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
				<LinkWarningIcon :check="wikiCheck" />
				<StyledInput
					id="project-wiki-page"
					v-model="wikiUrl"
					type="url"
					:maxlength="2048"
					placeholder="Enter a valid URL"
					:disabled="!hasPermission"
				/>
			</div>
			<div class="adjacent-input">
				<label id="project-discord-invite" title="An invitation link to your Discord server.">
					<span class="label__title">Discord invite </span>
					<span class="label__description"> An invitation link to your Discord server. </span>
				</label>
				<LinkWarningIcon :check="discordInviteCheck" />
				<StyledInput
					id="project-discord-invite"
					v-model="discordUrl"
					type="url"
					:maxlength="2048"
					placeholder="Enter a valid URL"
					:disabled="!hasPermission"
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
				<StyledInput
					v-model="donationLink.url"
					type="url"
					:maxlength="2048"
					placeholder="Enter a valid URL"
					:disabled="!hasPermission"
					@update:model-value="updateDonationLinks"
				/>
				<DropdownSelect
					v-model="donationLink.id"
					name="Donation platform selector"
					:options="tags.donationPlatforms.map((x) => x.short)"
					:display-name="
						(option) => tags.donationPlatforms.find((platform) => platform.short === option)?.name
					"
					placeholder="Select platform"
					render-up
					class="platform-selector"
					@update:model-value="updateDonationLinks"
				/>
				<LinkWarningIcon :check="getLinkCheckState(donationLink.id, donationLink.url)" />
			</div>
		</section>
		<UnsavedChangesPopup
			:original="original"
			:modified="modified"
			:saving="saving"
			:can-save="!hasConfirmedInvalidLinks"
			@reset="reset"
			@save="save"
		/>
	</div>
</template>

<script setup>
import { checkLink, getLinkCheckState, useLinkCheck } from '@modrinth/moderation'
import {
	DropdownSelect,
	injectModrinthClient,
	injectNotificationManager,
	injectProjectPageContext,
	StyledInput,
	UnsavedChangesPopup,
} from '@modrinth/ui'

import LinkWarningIcon from '@/components/LinkWarningIcon.vue'

const tags = useGeneratedState()

const { projectV3: project, currentMember, invalidate } = injectProjectPageContext()
const { labrinth } = injectModrinthClient()
const { addNotification } = injectNotificationManager()

const isServerProject = computed(() => project.value?.minecraft_server != null)

const linkRefs = {}
function linkUrl(key) {
	const urlRef = ref(project.value?.link_urls?.[key]?.url ?? '')
	linkRefs[key] = urlRef
	return urlRef
}

const issuesUrl = linkUrl('issues')
const sourceUrl = linkUrl('source')
const wikiUrl = linkUrl('wiki')
const discordUrl = linkUrl('discord')
const siteUrl = linkUrl('site')
const storeUrl = linkUrl('store')

function donationRowsFromLinks(linkUrls) {
	const rows = (tags.value.donationPlatforms ?? [])
		.filter((platform) => linkUrls?.[platform.short]?.url)
		.map((platform) => ({ id: platform.short, url: linkUrls[platform.short].url }))
	rows.push({ id: null, url: null })
	return rows
}

const donationLinks = ref(donationRowsFromLinks(project.value?.link_urls))

function resetLinks(newVal) {
	if (!newVal) return
	for (const [key, urlRef] of Object.entries(linkRefs)) {
		urlRef.value = newVal.link_urls?.[key]?.url ?? ''
	}
	donationLinks.value = donationRowsFromLinks(newVal.link_urls)
}

watch(project, resetLinks, { immediate: true })

const discordInviteCheck = useLinkCheck('discord', discordUrl)
const issuesCheck = useLinkCheck('issues', issuesUrl)
const sourceCheck = useLinkCheck('source', sourceUrl)
const wikiCheck = useLinkCheck('wiki', wikiUrl)
const siteCheck = useLinkCheck('site', siteUrl)
const storeCheck = useLinkCheck('store', storeUrl)

const donationCheckTimers = new Map()
watch(
	donationLinks,
	(rows) => {
		rows.forEach((row, index) => {
			if (!row.id || !row.url) return
			clearTimeout(donationCheckTimers.get(index))
			donationCheckTimers.set(
				index,
				setTimeout(() => checkLink(row.id, row.url), 500),
			)
		})
	},
	{ deep: true, immediate: true },
)

const hasPermission = computed(() => {
	const EDIT_DETAILS = 1 << 2
	return (currentMember.value?.permissions & EDIT_DETAILS) === EDIT_DETAILS
})

const original = computed(() => {
	const linkUrls = project.value?.link_urls
	const of = (key) => linkUrls?.[key]?.url ?? ''

	if (isServerProject.value) {
		return { site: of('site'), store: of('store'), wiki: of('wiki'), discord: of('discord') }
	}

	const donations = {}
	for (const platform of tags.value.donationPlatforms ?? []) {
		donations[platform.short] = of(platform.short)
	}
	return {
		issues: of('issues'),
		source: of('source'),
		wiki: of('wiki'),
		discord: of('discord'),
		...donations,
	}
})

const modified = computed(() => {
	if (isServerProject.value) {
		return { site: siteUrl.value, store: storeUrl.value, wiki: wikiUrl.value, discord: discordUrl.value }
	}

	const donations = {}
	for (const row of donationLinks.value) {
		if (row.id) donations[row.id] = row.url ?? ''
	}
	return {
		issues: issuesUrl.value,
		source: sourceUrl.value,
		wiki: wikiUrl.value,
		discord: discordUrl.value,
		...donations,
	}
})

const patchData = computed(() => {
	const data = {}
	for (const key of Object.keys(modified.value)) {
		if (modified.value[key] == original.value[key]) continue
		data[key] = modified.value[key] === '' ? null : modified.value[key].trim()
	}
	return data
})

const hasConfirmedInvalidLinks = computed(() => {
	const checks = isServerProject.value
		? [siteCheck, storeCheck, wikiCheck, discordInviteCheck]
		: [issuesCheck, sourceCheck, wikiCheck, discordInviteCheck]

	const fieldsInvalid = checks.some((check) => check.value?.severity === 'error')
	const donationsInvalid =
		!isServerProject.value &&
		donationLinks.value.some((row) => getLinkCheckState(row.id, row.url)?.severity === 'error')

	return fieldsInvalid || donationsInvalid
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

function reset() {
	resetLinks(project.value)
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
			id: null,
			url: null,
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

	:deep(.animated-dropdown .selected) {
		height: 40px;
	}
}
</style>
