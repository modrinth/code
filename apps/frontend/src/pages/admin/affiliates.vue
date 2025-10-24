<template>
	<AffiliateLinkCreateModal
		ref="createModal"
		:show-user-field="true"
		:creating-link="creatingLink"
		@create="createAffiliateCode"
	/>
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
			<h1 class="m-0 grow text-2xl font-extrabold">Manage affiliate links</h1>
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
						Create affiliate code
					</button>
				</ButtonStyled>
			</div>
		</div>
		<Admonition v-if="error" type="critical">
			<template #header> Error loading affiliate links </template>
			{{ error }}
		</Admonition>
		<div v-else-if="groupedAffiliates.length === 0" class="py-8 text-center">
			<p class="text-secondary">No affiliate codes found.</p>
		</div>
		<div v-else class="space-y-4">
			<Accordion
				v-for="(userGroup, index) in filteredGroupedAffiliates"
				:key="userGroup.user.id"
				open-by-default
				:class="{
					'border-0 border-b-[1px] border-solid border-divider pb-4':
						index < filteredGroupedAffiliates.length - 1,
				}"
				:button-class="`flex flex-col w-full gap-2 bg-transparent m-0 p-0 border-none`"
			>
				<template #title>
					<div class="flex items-center gap-4">
						<Avatar :src="userGroup.user.avatar_url" circle size="48px" />
						<div class="flex flex-col items-start">
							<span class="text-lg font-bold text-contrast">
								{{ userGroup.user.username }}
							</span>
							<span class="text-sm text-secondary">
								{{ userGroup.affiliates.length }} affiliate code{{
									userGroup.affiliates.length === 1 ? '' : 's'
								}}
							</span>
						</div>
					</div>
				</template>
				<div class="mt-4 space-y-3">
					<AffiliateLinkCard
						v-for="affiliate in userGroup.affiliates"
						:key="affiliate.id"
						:affiliate="affiliate"
						:created-by="getCreatedByUsername(affiliate.created_by)"
						@revoke="revokeAffiliateCode"
					/>
				</div>
			</Accordion>
		</div>
	</div>
</template>
<script setup lang="ts">
import { PlusIcon, SearchIcon, XCircleIcon, XIcon } from '@modrinth/assets'
import {
	Accordion,
	Admonition,
	AffiliateLinkCard,
	AffiliateLinkCreateModal,
	Avatar,
	Button,
	ButtonStyled,
	ConfirmModal,
	injectNotificationManager,
} from '@modrinth/ui'
import type { AffiliateLink, User } from '@modrinth/utils'

const { handleError } = injectNotificationManager()

type UserGroup = {
	user: User
	affiliates: AffiliateLink[]
}

const createModal = useTemplateRef<typeof AffiliateLinkCreateModal>('createModal')
const revokeModal = useTemplateRef<typeof ConfirmModal>('revokeModal')

const {
	data: affiliateCodes,
	error,
	refresh,
} = await useAsyncData(
	'AffiliateLinks',
	() => useBaseFetch('affiliate', { method: 'GET', internal: true }) as Promise<AffiliateLink[]>,
)

const filterQuery = ref('')
const creatingLink = ref(false)

const userIds = computed(() => {
	if (!affiliateCodes.value) {
		return []
	}
	const ids = new Set<string>()
	affiliateCodes.value.forEach((code) => {
		ids.add(code.affiliate)
		ids.add(code.created_by)
	})
	return Array.from(ids)
})

const { data: users } = await useAsyncData(
	'admin-affiliates-bulk-users',
	() => {
		if (userIds.value.length === 0) return Promise.resolve([])
		return useBaseFetch(`users?ids=${JSON.stringify(userIds.value)}`) as Promise<User[]>
	},
	{
		watch: [userIds],
	},
)

const userMap = computed(() => {
	if (!users.value) {
		return new Map()
	}
	return new Map(users.value.map((user) => [user.id, user]))
})

const groupedAffiliates = computed((): UserGroup[] => {
	if (!affiliateCodes.value || !users.value) {
		return []
	}

	const groups = new Map<string, UserGroup>()

	affiliateCodes.value.forEach((code) => {
		const user = userMap.value.get(code.affiliate)
		if (!user) {
			return
		}

		if (!groups.has(user.id)) {
			groups.set(user.id, {
				user,
				affiliates: [],
			})
		}

		groups.get(user.id)!.affiliates.push(code)
	})

	return Array.from(groups.values()).sort((a, b) => a.user.username.localeCompare(b.user.username))
})

const filteredGroupedAffiliates = computed(() => {
	if (!filterQuery.value.trim()) {
		return groupedAffiliates.value
	}

	const query = filterQuery.value.trim().toLowerCase()
	return groupedAffiliates.value.filter(
		(group) =>
			group.user.username.toLowerCase().includes(query) ||
			group.affiliates.some((affiliate) => affiliate.source_name.toLowerCase().includes(query)),
	)
})

function getCreatedByUsername(createdBy: string): string {
	const user = userMap.value.get(createdBy)
	return user?.username || 'Unknown'
}

async function createAffiliateCode(data: { sourceName: string; username?: string }) {
	creatingLink.value = true

	try {
		if (!data.username) {
			// noinspection ExceptionCaughtLocallyJS
			throw new Error('Username is required')
		}

		let user = users.value?.find((u) => u.username === data.username)

		if (!user) {
			try {
				user = (await useBaseFetch(`user/${data.username}`)) as User

				if (users.value) {
					users.value.push(user)
				}
			} catch {
				// noinspection ExceptionCaughtLocallyJS
				throw new Error('User not found')
			}
		}

		await useBaseFetch('affiliate', {
			method: 'PUT',
			body: {
				affiliate: user.id,
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

const revokingAffiliateUsername = ref<string | null>(null)
const revokingAffiliateId = ref<string | null>(null)

function revokeAffiliateCode(affiliate: AffiliateLink) {
	const user = userMap.value.get(affiliate.affiliate)
	revokingAffiliateUsername.value = user?.username || 'Unknown'
	revokingAffiliateId.value = affiliate.id
	revokeModal.value?.show()
}

async function confirmRevokeAffiliateCode() {
	if (!revokingAffiliateId.value) {
		return
	}

	try {
		await useBaseFetch(`affiliate/${revokingAffiliateId.value}`, {
			method: 'DELETE',
			internal: true,
		})

		await refresh()
		revokeModal.value?.hide()
		revokingAffiliateUsername.value = null
		revokingAffiliateId.value = null
	} catch (err) {
		console.error('Failed to revoke affiliate code:', err)
	}
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
