<template>
	<NewModal ref="createModal" :header="formatMessage(messages.createHeader)">
		<div class="flex flex-col gap-4">
			<label class="contents" for="create-affiliate-title-input">
				<span class="text-lg font-semibold text-contrast">
					{{ formatMessage(messages.createTitleLabel) }}
				</span>
				<span class="text-secondary">{{ formatMessage(messages.createTitleDescription) }}</span>
			</label>
			<div class="flex items-center gap-2">
				<div class="iconified-input">
					<AutoBrandIcon :keyword="affiliateLinkTitle" aria-hidden="true">
						<AffiliateIcon />
					</AutoBrandIcon>
					<input
						id="create-affiliate-title-input"
						v-model="affiliateLinkTitle"
						class="card-shadow"
						autocomplete="off"
						spellcheck="false"
						type="text"
						:placeholder="formatMessage(messages.createTitlePlaceholder)"
					/>
					<Button v-if="affiliateLinkTitle" class="r-btn" @click="() => (affiliateLinkTitle = '')">
						<XIcon />
					</Button>
				</div>
				<ButtonStyled color="brand">
					<button :disabled="creatingLink" @click="createAffiliateLink">
						<SpinnerIcon v-if="creatingLink" class="animate-spin" />
						<PlusIcon v-else />
						{{ formatMessage(creatingLink ? messages.creatingButton : messages.createButton) }}
					</button>
				</ButtonStyled>
			</div>
		</div>
	</NewModal>
	<ConfirmModal
		ref="revokeModal"
		:title="formatMessage(messages.revokeConfirmTitle, { title: revokingTitle })"
		:description="formatMessage(messages.revokeConfirmBody, { id: revokingId })"
		:proceed-icon="XCircleIcon"
		:proceed-label="formatMessage(messages.revokeConfirmButton)"
		@proceed="confirmRevokeAffiliateLink"
	/>
	<div class="page">
		<div
			class="mb-6 flex items-center gap-6 border-0 border-b-[1px] border-solid border-divider pb-6"
		>
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
			v-for="affiliate in filteredAffiliates"
			v-else
			:key="`affiliate-${affiliate.id}`"
			class="card-shadow mb-3 flex flex-col gap-4 rounded-2xl bg-bg-raised p-4"
		>
			<div class="flex items-center gap-4">
				<div
					class="flex items-center justify-center rounded-xl border-[1px] border-solid border-button-border bg-button-bg p-2"
				>
					<AutoBrandIcon :keyword="affiliate.source_name" class="h-6 w-6">
						<AffiliateIcon />
					</AutoBrandIcon>
				</div>
				<div class="flex flex-col gap-1">
					<span class="w-fit text-lg font-bold text-contrast">
						{{ affiliate.source_name }}
					</span>
				</div>
				<div class="ml-auto flex items-center gap-2">
					<ButtonStyled color="red" color-fill="text">
						<button @click="revokeAffiliateLink(affiliate)">
							<XCircleIcon /> {{ formatMessage(messages.revokeAffiliateLink) }}
						</button>
					</ButtonStyled>
				</div>
			</div>
			<CopyCode :text="`https://modrinth.gg?afl=${affiliate.id}`" />
		</div>
	</div>
</template>
<script setup lang="ts">
import {
	AffiliateIcon,
	PlusIcon,
	SearchIcon,
	SpinnerIcon,
	XCircleIcon,
	XIcon,
} from '@modrinth/assets'
import {
	Admonition,
	AutoBrandIcon,
	Button,
	ButtonStyled,
	ConfirmModal,
	CopyCode,
	NewModal,
} from '@modrinth/ui'
import { useVIntl } from '@vintl/vintl'

const createModal = useTemplateRef<typeof NewModal>('createModal')
const revokeModal = useTemplateRef<typeof ConfirmModal>('revokeModal')

const auth: { user: any } & any = await useAuth()

const { formatMessage } = useVIntl()

type AffiliateLink = {
	id: string
	created_at: string
	created_by: string
	affiliate: string
	source_name: string
}

const {
	data: affiliateLinks,
	error,
	refresh,
} = await useAsyncData(
	'affiliateLinks',
	() => useBaseFetch('affiliate', { method: 'GET', internal: true }) as Promise<AffiliateLink[]>,
)

const filterQuery = ref('')

const filteredAffiliates = computed(() =>
	affiliateLinks
		? affiliateLinks.value?.filter((link: AffiliateLink) =>
				link.affiliate === auth.user?.id && filterQuery.value.trim()
					? link.source_name.trim().toLowerCase().includes(filterQuery.value.trim().toLowerCase())
					: true,
			)
		: [],
)

const affiliateLinkTitle = ref<string>('')

const creatingLink = ref(false)

function createAffiliateLink() {
	creatingLink.value = true

	useBaseFetch('affiliate', {
		method: 'PUT',
		body: {
			source_name: affiliateLinkTitle.value,
		},
		internal: true,
	}).then(() => {
		refresh()
		createModal.value?.hide()
		creatingLink.value = false
		affiliateLinkTitle.value = ''
	})
}

const revokingTitle = ref<string | null>(null)
const revokingId = ref<string | null>(null)

function revokeAffiliateLink(affiliate: AffiliateLink) {
	revokingTitle.value = affiliate.source_name
	revokingId.value = affiliate.id
	revokeModal.value?.show()
}

function confirmRevokeAffiliateLink() {
	useBaseFetch(`affiliate/${revokingId.value}`, {
		method: 'DELETE',
		internal: true,
	}).then(() => {
		refresh()
		revokeModal.value?.hide()
		revokingTitle.value = null
		revokingId.value = null
	})
}

const messages = defineMessages({
	createHeader: {
		id: 'dashboard.affiliate-links.create.header',
		defaultMessage: 'Creating new affiliate code',
	},
	createTitleLabel: {
		id: 'dashboard.affiliate-links.create.title.label',
		defaultMessage: 'Title of affiliate link',
	},
	createTitleDescription: {
		id: 'dashboard.affiliate-links.create.title.description',
		defaultMessage: 'Give your affiliate link a name so you know where people are coming from!',
	},
	createTitlePlaceholder: {
		id: 'dashboard.affiliate-links.create.title.placeholder',
		defaultMessage: 'e.g. YouTube',
	},
	createButton: {
		id: 'dashboard.affiliate-links.create.button',
		defaultMessage: 'Create affiliate link',
	},
	creatingButton: {
		id: 'dashboard.affiliate-links.creating.button',
		defaultMessage: 'Creating affiliate link...',
	},
	yourAffiliateLinks: {
		id: 'dashboard.affiliate-links.header',
		defaultMessage: 'Your affiliate links',
	},
	revokeAffiliateLink: {
		id: 'dashboard.affiliate-links.revoke',
		defaultMessage: 'Revoke affiliate link',
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
