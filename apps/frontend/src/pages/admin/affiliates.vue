<template>
	<NewModal ref="createModal" header="Creating new affiliate code">
		<div class="flex flex-col gap-4">
			<span class="text-lg font-semibold text-contrast">Modrinth username of affiliate</span>
			<div class="flex items-center gap-2">
				<div class="iconified-input">
					<UserIcon aria-hidden="true" />
					<input
						v-model="affiliateUsername"
						class="card-shadow"
						autocomplete="off"
						spellcheck="false"
						type="text"
						:placeholder="`Username`"
					/>
					<Button v-if="affiliateUsername" class="r-btn" @click="() => (affiliateUsername = '')">
						<XIcon />
					</Button>
				</div>
				<ButtonStyled color="brand">
					<button @click="createAffiliateCode">
						<PlusIcon />
						Create affiliate code
					</button>
				</ButtonStyled>
			</div>
		</div>
	</NewModal>
	<ConfirmModal
		ref="revokeModal"
		:title="`Are you sure you want to revoke ${revokingAffiliateUsername}'s affiliate code?`"
		:description="`This will permanently revoke the affiliate code \`${revokingAffiliateId}\` and make any links that this user has shared invalid.`"
		:proceed-icon="XCircleIcon"
		:proceed-label="`Revoke`"
		@proceed="confirmRevokeAffiliateCode"
	/>
	<div class="page">
		<div
			class="mb-6 flex items-center gap-6 border-0 border-b-[1px] border-solid border-divider pb-6"
		>
			<h1 class="m-0 grow text-2xl font-extrabold">Manage affiliates</h1>
			<div class="flex items-center gap-2">
				<div class="iconified-input">
					<SearchIcon aria-hidden="true" />
					<input
						v-model="filterQuery"
						class="card-shadow"
						autocomplete="off"
						spellcheck="false"
						type="text"
						:placeholder="`Search affiliates...`"
					/>
					<Button v-if="filterQuery" class="r-btn" @click="() => (filterQuery = '')">
						<XIcon />
					</Button>
				</div>
				<ButtonStyled color="brand">
					<button @click="createModal?.show">
						<PlusIcon />
						Create affiliate
					</button>
				</ButtonStyled>
			</div>
		</div>
		<div
			v-for="affiliate in filteredAffiliates"
			:key="`affiliate-${affiliate.id}`"
			class="card-shadow mb-3 flex items-center gap-4 rounded-2xl bg-bg-raised p-4"
		>
			<nuxt-link
				:to="`/user/${affiliate.username}`"
				tabindex="-1"
				class="hover:brightness-[--hover-brightness]"
			>
				<Avatar :src="affiliate.avatar_url" circle size="48px" />
			</nuxt-link>
			<div class="flex flex-col gap-1">
				<nuxt-link
					:to="`/user/${affiliate.username}`"
					class="w-fit text-lg font-bold text-contrast hover:underline"
				>
					{{ affiliate.username }}
				</nuxt-link>
				<div class="flex items-center gap-2">
					<CopyCode :text="`https://modrinth.gg?afl=${affiliate.affiliate_codes[0]}`" />
					<CopyCode :text="affiliate.affiliate_codes[0]" />
				</div>
			</div>
			<div class="ml-auto flex items-center gap-2">
				<ButtonStyled>
					<nuxt-link>
						<ChartIcon />
						View analytics
					</nuxt-link>
				</ButtonStyled>
				<ButtonStyled color="red" color-fill="text">
					<button @click="revokeAffiliateCode(affiliate.username, affiliate.id)">
						<XCircleIcon /> Revoke affiliate code
					</button>
				</ButtonStyled>
			</div>
		</div>
	</div>
</template>
<script setup lang="ts">
import { ChartIcon, PlusIcon, SearchIcon, UserIcon, XCircleIcon, XIcon } from '@modrinth/assets'
import { Avatar, Button, ButtonStyled, ConfirmModal, CopyCode, NewModal } from '@modrinth/ui'

const createModal = useTemplateRef<typeof NewModal>('createModal')
const revokeModal = useTemplateRef<typeof ConfirmModal>('revokeModal')

const filterQuery = ref('')

const filteredAffiliates = computed(() =>
	affiliateUsers.value.filter((affiliate) =>
		filterQuery.value.trim()
			? affiliate.username.trim().toLowerCase().includes(filterQuery.value.trim().toLowerCase())
			: true,
	),
)

// placeholder affiliate data
const affiliateUsers = computed(() => {
	return [
		{
			id: 'Dc7EYhxG',
			username: 'Prospector',
			avatar_url:
				'https://cdn.modrinth.com/user/Dc7EYhxG/32e8b1f7d18288262d1ed92cbdf43272d21b4fcd.png',
			bio: 'Software Engineer at Modrinth.\n\nFounder of ModFest, co-founder of TerraformersMC. Creator of Traverse and Mod Menu.',
			created: '2020-11-06T04:56:05.014379Z',
			role: 'admin',
			badges: 1,
			auth_providers: null,
			email: null,
			email_verified: null,
			has_password: null,
			has_totp: null,
			payout_data: null,
			stripe_customer_id: null,
			allow_friend_requests: null,
			github_id: null,
			affiliate_codes: ['cD2EThlR'],
		},
		{
			id: 'MpxzqsyW',
			username: 'Geometrically',
			avatar_url:
				'https://cdn.modrinth.com/data/MpxzqsyW/eb0038489a55e7e7a188a5b50462f0b10dfc1613_96.webp',
			bio: 'I make stuff',
			created: '2020-10-19T02:30:03.202550Z',
			role: 'admin',
			badges: 1,
			auth_providers: null,
			email: null,
			email_verified: null,
			has_password: null,
			has_totp: null,
			payout_data: null,
			stripe_customer_id: null,
			allow_friend_requests: null,
			github_id: null,
			affiliate_codes: ['p4DcS4lQ'],
		},
	]
})

const affiliateUsername = ref<string | null>(null)

function createAffiliateCode() {
	createModal.value?.hide()
	affiliateUsername.value = null
}

const revokingAffiliateUsername = ref<string | null>(null)
const revokingAffiliateId = ref<string | null>(null)

function revokeAffiliateCode(username: string, affiliateId: string) {
	revokingAffiliateUsername.value = username
	revokingAffiliateId.value = affiliateId
	revokeModal.value?.show()
}

function confirmRevokeAffiliateCode() {
	revokeModal.value?.hide()
	revokingAffiliateUsername.value = null
	revokingAffiliateId.value = null
}
</script>

<style lang="scss" scoped>
.page {
	padding: 1rem;
	margin-left: auto;
	margin-right: auto;
	max-width: 78.5rem;
}
</style>
