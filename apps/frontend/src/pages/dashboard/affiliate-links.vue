<template>
	<AffiliateLinkCreateModal
		ref="createModal"
		:creating-link="creatingLink"
		@create="createAffiliateCode"
	/>
	<ConfirmModal
		ref="revokeModal"
		:title="formatMessage(messages.revokeConfirmTitle, { title: revokingTitle })"
		:description="formatMessage(messages.revokeConfirmBody, { id: revokingId })"
		:proceed-icon="XCircleIcon"
		:proceed-label="formatMessage(messages.revokeConfirmButton)"
		@proceed="confirmRevokeAffiliateLink"
	/>
	<div class="page">
		<div class="mb-6 flex items-center gap-6">
			<h1 class="m-0 grow text-2xl font-extrabold">
				{{ formatMessage(messages.yourAffiliateLinks) }}
			</h1>
			<div class="flex items-center gap-2">
				<div class="iconified-input">
					<SearchIcon aria-hidden="true" />
					<input
						v-model="filterQuery"
						class="card-shadow"
						autocomplete="off"
						spellcheck="false"
						type="text"
						:placeholder="formatMessage(messages.searchAffiliateLinks)"
					/>
					<Button v-if="filterQuery" class="r-btn" @click="() => (filterQuery = '')">
						<XIcon />
					</Button>
				</div>
				<ButtonStyled color="brand">
					<button @click="createModal?.show">
						<PlusIcon />
						{{ formatMessage(messages.createButton) }}
					</button>
				</ButtonStyled>
			</div>
		</div>
		<Admonition v-if="error" type="critical">
			<template #header>
				{{ formatMessage(messages.errorTitle) }}
			</template>
			{{ error }}
		</Admonition>
		<div
			v-else-if="!filteredAffiliates || filteredAffiliates.length === 0"
			class="py-8 text-center"
		>
			<p class="text-secondary">No affiliate codes found.</p>
		</div>
		<div v-else class="space-y-3">
			<AffiliateLinkCard
				v-for="affiliate in filteredAffiliates"
				:key="`affiliate-${affiliate.id}`"
				:affiliate="affiliate"
				@revoke="revokeAffiliateLink"
			/>
		</div>
	</div>
</template>
<script setup lang="ts">
import { PlusIcon, SearchIcon, XCircleIcon, XIcon } from '@modrinth/assets'
import {
	Admonition,
	AffiliateLinkCard,
	AffiliateLinkCreateModal,
	Button,
	ButtonStyled,
	ConfirmModal,
	injectNotificationManager,
} from '@modrinth/ui'
import type { AffiliateLink } from '@modrinth/utils'
import { useVIntl } from '@vintl/vintl'

const createModal = useTemplateRef<typeof AffiliateLinkCreateModal>('createModal')
const revokeModal = useTemplateRef<typeof ConfirmModal>('revokeModal')

const auth = await useAuth()

const { handleError } = injectNotificationManager()

const { formatMessage } = useVIntl()

const {
	data: affiliateLinks,
	error,
	refresh,
} = await useAsyncData(
	'affiliateLinks',
	() => useBaseFetch('affiliate', { method: 'GET', internal: true }) as Promise<AffiliateLink[]>,
)

const filterQuery = ref('')
const creatingLink = ref(false)

const filteredAffiliates = computed(() =>
	affiliateLinks
		? affiliateLinks.value?.filter(
				(link: AffiliateLink) =>
					link.affiliate === auth.value?.user?.id &&
					(filterQuery.value.trim()
						? link.source_name.trim().toLowerCase().includes(filterQuery.value.trim().toLowerCase())
						: true),
			)
		: [],
)

async function createAffiliateCode(data: { sourceName: string }) {
	creatingLink.value = true

	try {
		await useBaseFetch('affiliate', {
			method: 'PUT',
			body: {
				source_name: data.sourceName,
			},
			internal: true,
		})

		await refresh()
		createModal.value?.close()
	} catch (err) {
		handleError(err)
	} finally {
		creatingLink.value = false
	}
}

const revokingTitle = ref<string | null>(null)
const revokingId = ref<string | null>(null)

function revokeAffiliateLink(affiliate: AffiliateLink) {
	revokingTitle.value = affiliate.source_name
	revokingId.value = affiliate.id
	revokeModal.value?.show()
}

async function confirmRevokeAffiliateLink() {
	if (!revokingId.value) {
		return
	}

	try {
		await useBaseFetch(`affiliate/${revokingId.value}`, {
			method: 'DELETE',
			internal: true,
		})

		await refresh()
		revokeModal.value?.hide()
		revokingTitle.value = null
		revokingId.value = null
	} catch (err) {
		console.error('Failed to revoke affiliate code:', err)
	}
}

const messages = defineMessages({
	createButton: {
		id: 'dashboard.affiliate-links.create.button',
		defaultMessage: 'Create affiliate link',
	},
	yourAffiliateLinks: {
		id: 'dashboard.affiliate-links.header',
		defaultMessage: 'Your affiliate links',
	},
	searchAffiliateLinks: {
		id: 'dashboard.affiliate-links.search',
		defaultMessage: 'Search affiliate links...',
	},
	errorTitle: {
		id: 'dashboard.affiliate-links.error.title',
		defaultMessage: 'Error loading affiliate links',
	},
	revokeConfirmButton: {
		id: 'dashboard.affiliate-links.revoke-confirm.button',
		defaultMessage: 'Revoke',
	},
	revokeConfirmTitle: {
		id: 'dashboard.affiliate-links.revoke-confirm.title',
		defaultMessage: "Are you sure you want to revoke your ''{title}'' affiliate link?",
	},
	revokeConfirmBody: {
		id: 'dashboard.affiliate-links.revoke-confirm.body',
		defaultMessage:
			'This will permanently revoke the affiliate code `{id}` and any existing links with this code that have been shared will no longer be valid.',
	},
})
</script>

<style lang="scss" scoped>
.page {
	padding: 1rem;
	margin-left: auto;
	margin-right: auto;
	max-width: 78.5rem;
}
</style>
