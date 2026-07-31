<template>
	<NewModal ref="refundModal">
		<template #title>
			<span class="text-lg font-extrabold text-contrast">Refund charge</span>
		</template>
		<div class="flex flex-col gap-3">
			<div class="flex flex-col gap-2">
				<label for="visibility" class="flex flex-col gap-1">
					<span class="text-lg font-semibold text-contrast">
						Refund type
						<span class="text-brand-red">*</span>
					</span>
					<span> The type of refund to issue. </span>
				</label>
				<DropdownSelect
					id="refund-type"
					v-model="refundType"
					:options="refundTypes"
					name="Refund type"
				/>
			</div>
			<div v-if="refundType === 'partial'" class="flex flex-col gap-2">
				<label for="amount" class="flex flex-col gap-1">
					<span class="text-lg font-semibold text-contrast">
						Amount
						<span class="text-brand-red">*</span>
					</span>
					<span>
						Enter the amount in cents of USD. For example for $2, enter 200. (net
						{{ selectedCharge.net }})
					</span>
				</label>
				<StyledInput id="amount" v-model="refundAmount" type="number" autocomplete="off" />
			</div>
			<div class="flex flex-col gap-2">
				<label for="unprovision" class="flex flex-col gap-1">
					<span class="text-lg font-semibold text-contrast">
						Unprovision
						<span class="text-brand-red">*</span>
					</span>
					<span> Whether or not the subscription should be unprovisioned on refund. </span>
				</label>
				<Toggle id="unprovision" v-model="unprovision" />
			</div>
			<div class="flex gap-2">
				<ButtonStyled color="brand">
					<button :disabled="refunding" @click="refundCharge">
						<CheckIcon aria-hidden="true" />
						Refund charge
					</button>
				</ButtonStyled>
				<ButtonStyled>
					<button @click="refundModal.hide()">
						<XIcon aria-hidden="true" />
						Cancel
					</button>
				</ButtonStyled>
			</div>
		</div>
	</NewModal>
	<NewModal ref="modifyModal">
		<template #title>
			<span class="text-lg font-extrabold text-contrast">Modify charge</span>
		</template>
		<div class="flex flex-col gap-3">
			<div class="flex flex-col gap-2">
				<label for="cancel" class="flex flex-col gap-1">
					<span class="text-lg font-semibold text-contrast">
						Cancel server
						<span class="text-brand-red">*</span>
					</span>
					<span>
						Whether or not the subscription should be cancelled. Submitting this as "true" will
						cancel the subscription, while submitting it as "false" will
						{{
							selectedCharge.status === 'open'
								? 'keep it as-is'
								: 'force another charge attempt to be made'
						}}.
					</span>
				</label>
				<Toggle id="cancel" v-model="cancel" />
			</div>
			<div class="flex gap-2">
				<ButtonStyled color="brand">
					<button :disabled="modifying" @click="modifyCharge">
						<CheckIcon aria-hidden="true" />
						Modify charge
					</button>
				</ButtonStyled>
				<ButtonStyled>
					<button @click="modifyModal.hide()">
						<XIcon aria-hidden="true" />
						Cancel
					</button>
				</ButtonStyled>
			</div>
		</div>
	</NewModal>
	<NewModal ref="creditModal">
		<template #title>
			<span class="text-lg font-extrabold text-contrast">Credit subscription</span>
		</template>
		<div class="flex flex-col gap-3">
			<div class="flex flex-col gap-2">
				<label for="days" class="flex flex-col gap-1">
					<span class="text-lg font-semibold text-contrast">Days to credit</span>
					<span>Enter the number of days to add to the next due date.</span>
				</label>
				<StyledInput id="days" v-model="creditDays" type="number" :min="1" autocomplete="off" />
			</div>
			<div class="flex flex-col gap-2">
				<label for="sendEmail" class="flex flex-col gap-1">
					<span class="text-lg font-semibold text-contrast">Send email to user</span>
					<span>Notify the user about the credited days.</span>
				</label>
				<Toggle id="sendEmail" v-model="creditSendEmail" />
			</div>
			<div class="flex gap-2">
				<ButtonStyled color="brand">
					<button :disabled="crediting" @click="applyCredit">
						<CheckIcon aria-hidden="true" />
						Apply credit
					</button>
				</ButtonStyled>
				<ButtonStyled>
					<button @click="creditModal.hide()">
						<XIcon aria-hidden="true" />
						Cancel
					</button>
				</ButtonStyled>
			</div>
		</div>
	</NewModal>
	<div class="page">
		<div
			class="mb-4 flex items-center justify-between border-0 border-b border-solid border-divider pb-4"
		>
			<div class="flex items-center gap-2">
				<Avatar :src="user?.avatar_url" :alt="user?.username" size="32px" circle />
				<h1 class="m-0 text-2xl font-extrabold">{{ user?.username }}'s subscriptions</h1>
			</div>
			<div class="flex items-center gap-2">
				<ButtonStyled>
					<nuxt-link :to="`/user/${user?.id}`">
						<UserIcon aria-hidden="true" />
						User profile
						<ExternalIcon class="h-4 w-4" />
					</nuxt-link>
				</ButtonStyled>
			</div>
		</div>
		<div>
			<div v-for="subscription in subscriptionCharges" :key="subscription.id" class="card">
				<div class="mb-4 grid grid-cols-[1fr_auto]">
					<div>
						<span class="flex items-center gap-2 font-semibold text-contrast">
							<!-- TODO(backend): provide proper metadata for midas (MR+) subscriptions -->
							<template v-if="subscription.price_id === 'a6eRm92L'">
								<ModrinthPlusIcon class="h-7 w-min" />
							</template>
							<template v-else-if="subscription.metadata?.type === 'pyro'">
								<ModrinthServersIcon class="h-7 w-min" />
							</template>
							<template v-else-if="subscription.metadata?.type === 'medal'">
								<span>Medal Trial Server</span>
							</template>
							<template v-else> Unknown product </template>
						</span>
						<div class="mb-4 mt-2 flex w-full items-center gap-1 text-sm text-secondary">
							{{ capitalizeString(subscription.interval) }} ⋅ {{ subscription.status }} ⋅
							{{ formatDateTime(subscription.created) }} ({{
								formatRelativeTime(subscription.created)
							}})
						</div>
					</div>
					<div v-if="subscription.metadata?.id" class="flex flex-col items-end gap-2">
						<CopyCode :text="subscription.metadata.id" />
						<ButtonStyled
							v-if="
								subscription.metadata?.type === 'pyro' || subscription.metadata?.type === 'medal'
							"
						>
							<nuxt-link
								:to="`/hosting/manage/${subscription.metadata.id}`"
								target="_blank"
								class="w-fit"
							>
								<ServerIcon /> Server panel <ExternalIcon class="h-4 w-4" />
							</nuxt-link>
						</ButtonStyled>
						<ButtonStyled>
							<button @click="showCreditModal(subscription)">
								<CurrencyIcon />
								Credit
							</button>
						</ButtonStyled>
					</div>
				</div>
				<div class="flex flex-col gap-2">
					<AdminBillingChargeCard
						v-for="(charge, index) in subscription.charges"
						:key="charge.id"
						:charge="charge"
						:subscription="subscription"
						:all-charges="charges"
						:charge-index="index"
						:charge-count="subscription.charges.length"
						@refund="showRefundModal"
						@modify="showModifyModal"
					/>
				</div>
			</div>
		</div>
	</div>
</template>
<script setup>
import {
	CheckIcon,
	CurrencyIcon,
	ExternalIcon,
	ModrinthPlusIcon,
	ServerIcon,
	UserIcon,
	XIcon,
} from '@modrinth/assets'
import {
	Avatar,
	ButtonStyled,
	CopyCode,
	defineMessages,
	DropdownSelect,
	injectModrinthClient,
	injectNotificationManager,
	NewModal,
	StyledInput,
	Toggle,
	useFormatDateTime,
	useRelativeTime,
	useVIntl,
} from '@modrinth/ui'
import { capitalizeString } from '@modrinth/utils'
import { DEFAULT_CREDIT_EMAIL_MESSAGE } from '@modrinth/utils/utils.ts'
import { useQuery } from '@tanstack/vue-query'
import dayjs from 'dayjs'

import ModrinthServersIcon from '~/components/brand/ModrinthServersIcon.vue'
import AdminBillingChargeCard from '~/components/ui/admin/AdminBillingChargeCard.vue'

const { addNotification } = injectNotificationManager()
const { labrinth } = injectModrinthClient()
const formatDateTime = useFormatDateTime({
	timeStyle: 'short',
	dateStyle: 'long',
})

const vintl = useVIntl()

const { formatMessage } = vintl
const formatRelativeTime = useRelativeTime()

const messages = defineMessages({
	userNotFoundError: {
		id: 'admin.billing.error.not-found',
		defaultMessage: 'User not found',
	},
})

const userId = useRouteId('user')

const {
	data: user,
	error: userError,
	suspense: userSuspense,
} = useQuery({
	queryKey: ['user', userId],
	queryFn: () => labrinth.users_v2.get(userId),
})

onServerPrefetch(userSuspense)

watch(userError, (error) => {
	if (error) {
		showError({
			fatal: true,
			statusCode: error.statusCode ?? error.status ?? 404,
			message: formatMessage(messages.userNotFoundError),
		})
	}
})

const { data: subscriptions } = useQuery({
	queryKey: computed(() => ['billing', 'subscriptions', user.value?.id]),
	queryFn: () => labrinth.billing_internal.getSubscriptions(user.value?.id),
	enabled: computed(() => !!user.value?.id),
	placeholderData: [],
})

const { data: charges, refetch: refreshCharges } = useQuery({
	queryKey: computed(() => ['billing', 'payments', user.value?.id]),
	queryFn: () => labrinth.billing_internal.getPayments(user.value?.id),
	enabled: computed(() => !!user.value?.id),
	placeholderData: [],
})

const subscriptionCharges = computed(() => {
	return subscriptions.value.map((subscription) => {
		return {
			...subscription,
			charges: charges.value
				.filter((charge) => charge.subscription_id === subscription.id)
				.slice()
				.sort((a, b) => dayjs(b.due).diff(dayjs(a.due))),
		}
	})
})

const refunding = ref(false)
const refundModal = ref()
const selectedCharge = ref(null)
const selectedSubscription = ref(null)
const refundType = ref('full')
const refundTypes = ref(['full', 'partial', 'none'])
const refundAmount = ref(0)
const unprovision = ref(true)

const modifying = ref(false)
const modifyModal = ref()
const cancel = ref(false)

const crediting = ref(false)
const creditModal = ref()
const creditDays = ref(7)
const creditSendEmail = ref(true)

function showRefundModal(charge) {
	selectedCharge.value = charge
	refundType.value = 'full'
	refundAmount.value = 0
	unprovision.value = true
	refundModal.value.show()
}

function showModifyModal(charge, subscription) {
	selectedCharge.value = charge
	selectedSubscription.value = subscription
	cancel.value = false
	modifyModal.value.show()
}

function showCreditModal(subscription) {
	selectedSubscription.value = subscription
	creditDays.value = 1
	creditSendEmail.value = true
	creditModal.value.show()
}

async function applyCredit() {
	crediting.value = true
	try {
		const daysParsed = Math.max(1, Math.floor(Number(creditDays.value) || 1))
		await labrinth.billing_internal.credit({
			subscription_ids: [selectedSubscription.value.id],
			days: daysParsed,
			send_email: creditSendEmail.value,
			message: DEFAULT_CREDIT_EMAIL_MESSAGE,
		})
		addNotification({
			title: 'Credit applied',
			text: 'The subscription due date has been updated.',
			type: 'success',
		})
		await refreshCharges()
		creditModal.value.hide()
	} catch (err) {
		addNotification({
			title: 'Error applying credit',
			text: err.data?.description ?? String(err),
			type: 'error',
		})
	}
	crediting.value = false
}

async function refundCharge() {
	refunding.value = true
	try {
		const amountParsed = Math.max(0, Math.floor(Number(refundAmount.value) || 0))
		const payload =
			refundType.value === 'partial'
				? { type: 'partial', amount: amountParsed, unprovision: unprovision.value }
				: refundType.value === 'none'
					? { type: 'none', unprovision: unprovision.value }
					: { type: 'full', unprovision: unprovision.value }

		await labrinth.billing_internal.refundCharge(selectedCharge.value.id, payload)
		await refreshCharges()
		refundModal.value.hide()
	} catch (err) {
		addNotification({
			title: 'Error refunding',
			text: err.data?.description ?? err,
			type: 'error',
		})
	}
	refunding.value = false
}

async function modifyCharge() {
	modifying.value = true
	try {
		await labrinth.billing_internal.editSubscription(selectedSubscription.value.id, {
			cancelled: cancel.value,
		})
		addNotification({
			title: 'Modifications made',
			text: 'If the server is currently cancelled, it may take up to 10 minutes for another charge attempt to be made.',
			type: 'success',
		})
		await refreshCharges()
	} catch (err) {
		addNotification({
			title: 'Error while sending request',
			text: err.data?.description ?? err,
			type: 'error',
		})
	}
	modifying.value = false
}
</script>
<style scoped>
.page {
	padding: 1rem;
	margin-left: auto;
	margin-right: auto;
	max-width: 56rem;
}
</style>
