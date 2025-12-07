<script setup lang="ts">
import type { Archon, Labrinth } from '@modrinth/api-client'
import {
	CheckCircleIcon,
	ChevronRightIcon,
	LeftArrowIcon,
	RightArrowIcon,
	SpinnerIcon,
	XIcon,
} from '@modrinth/assets'
import { defineMessage, type MessageDescriptor, useVIntl } from '@vintl/vintl'
import type Stripe from 'stripe'
import { computed, nextTick, ref, useTemplateRef, watch } from 'vue'

import { useStripe } from '../../composables/stripe'
import { commonMessages } from '../../utils'
import { ButtonStyled } from '../index'
import ModalLoadingIndicator from '../modal/ModalLoadingIndicator.vue'
import NewModal from '../modal/NewModal.vue'
import PlanSelector from './ServersPurchase0Plan.vue'
import RegionSelector from './ServersPurchase1Region.vue'
import PaymentMethodSelector from './ServersPurchase2PaymentMethod.vue'
import ConfirmPurchase from './ServersPurchase3Review.vue'

const { formatMessage } = useVIntl()

export type RegionPing = {
	region: string
	ping: number
}

// Type alias for billing interval that matches both the local and api-client types
export type ServerBillingInterval = 'monthly' | 'quarterly' | 'yearly'

const props = defineProps<{
	publishableKey: string
	returnUrl: string
	paymentMethods: Stripe.PaymentMethod[]
	customer: Stripe.Customer
	currency: string
	pings: RegionPing[]
	regions: Archon.Servers.v1.Region[]
	availableProducts: Labrinth.Billing.Internal.Product[]
	planStage?: boolean
	existingPlan?: Labrinth.Billing.Internal.Product
	existingSubscription?: Labrinth.Billing.Internal.UserSubscription
	refreshPaymentMethods: () => Promise<void>
	fetchStock: (
		region: Archon.Servers.v1.Region,
		request: Archon.Servers.v0.StockRequest,
	) => Promise<number>
	initiatePayment: (
		body: Labrinth.Billing.Internal.InitiatePaymentRequest,
	) => Promise<
		| Labrinth.Billing.Internal.InitiatePaymentResponse
		| Labrinth.Billing.Internal.EditSubscriptionResponse
		| null
	>
	onError: (err: Error) => void
	onFinalizeNoPaymentChange?: () => Promise<void>
	affiliateCode?: string | null
}>()

const modal = useTemplateRef<InstanceType<typeof NewModal>>('modal')
const selectedPlan = ref<Labrinth.Billing.Internal.Product>()
const selectedInterval = ref<ServerBillingInterval>('quarterly')
const loading = ref(false)
const selectedRegion = ref<string>()
const projectId = ref<string>()
const affiliateCode = ref(props.affiliateCode ?? null)

const {
	initializeStripe,
	selectPaymentMethod,
	primaryPaymentMethodId,
	loadStripeElements,
	selectedPaymentMethod,
	inputtedPaymentMethod,
	createNewPaymentMethod,
	loadingElements,
	loadingElementsFailed,
	tax,
	total,
	paymentMethodLoading,
	reloadPaymentIntent,
	hasPaymentMethod,
	submitPayment,
	completingPurchase,
	noPaymentRequired,
} = useStripe(
	props.publishableKey,
	props.customer,
	props.paymentMethods,
	props.currency,
	selectedPlan,
	selectedInterval,
	selectedRegion,
	projectId,
	props.initiatePayment,
	props.onError,
	affiliateCode,
)

const customServer = ref<boolean>(false)
const acceptedEula = ref<boolean>(false)
const skipPaymentMethods = ref<boolean>(true)

type Step = 'plan' | 'region' | 'payment' | 'review'

const steps: Step[] = props.planStage
	? (['plan', 'region', 'payment', 'review'] as Step[])
	: (['region', 'payment', 'review'] as Step[])

const titles: Record<Step, MessageDescriptor> = {
	plan: defineMessage({ id: 'servers.purchase.step.plan.title', defaultMessage: 'Plan' }),
	region: defineMessage({ id: 'servers.purchase.step.region.title', defaultMessage: 'Region' }),
	payment: defineMessage({
		id: 'servers.purchase.step.payment.title',
		defaultMessage: 'Payment method',
	}),
	review: defineMessage({ id: 'servers.purchase.step.review.title', defaultMessage: 'Review' }),
}

const currentRegion = computed(() => {
	return props.regions.find((region) => region.shortcode === selectedRegion.value)
})

const currentPing = computed(() => {
	return props.pings.find((ping) => ping.region === currentRegion.value?.shortcode)?.ping
})

const currentStep = ref<Step>()

const currentStepIndex = computed(() => (currentStep.value ? steps.indexOf(currentStep.value) : -1))
const previousStep = computed(() => {
	const step = currentStep.value ? steps[steps.indexOf(currentStep.value) - 1] : undefined
	if (step === 'payment' && skipPaymentMethods.value && primaryPaymentMethodId.value) {
		return 'region'
	}
	return step
})
const nextStep = computed(() =>
	currentStep.value ? steps[steps.indexOf(currentStep.value) + 1] : undefined,
)

const canProceed = computed(() => {
	switch (currentStep.value) {
		case 'plan':
			console.log('Plan step:', {
				customServer: customServer.value,
				selectedPlan: selectedPlan.value,
				existingPlan: props.existingPlan,
			})
			return (
				customServer.value ||
				(!!selectedPlan.value &&
					(!props.existingPlan || selectedPlan.value.id !== props.existingPlan.id))
			)
		case 'region':
			return selectedRegion.value && selectedPlan.value && selectedInterval.value
		case 'payment':
			return selectedPaymentMethod.value || !loadingElements.value
		case 'review':
			return (
				(noPaymentRequired.value || (acceptedEula.value && hasPaymentMethod.value)) &&
				!completingPurchase.value
			)
		default:
			return false
	}
})

async function beforeProceed(step: string) {
	switch (step) {
		case 'plan':
			return true
		case 'region':
			return true
		case 'payment':
			await initializeStripe()

			if (primaryPaymentMethodId.value && skipPaymentMethods.value) {
				const paymentMethod = await props.paymentMethods.find(
					(x) => x.id === primaryPaymentMethodId.value,
				)
				await selectPaymentMethod(paymentMethod)
				await setStep('review', true)
				return false
			}
			return true
		case 'review':
			if (noPaymentRequired.value) {
				return true
			}
			if (selectedPaymentMethod.value) {
				return true
			} else {
				const token = await createNewPaymentMethod()
				return !!token
			}
	}
}

async function afterProceed(step: string) {
	switch (step) {
		case 'region':
			break
		case 'payment':
			await loadStripeElements()
			break
		case 'review':
			break
	}
}

async function setStep(step: Step | undefined, skipValidation = false) {
	if (!step) {
		await submitPayment(props.returnUrl)
		return
	}

	if (!skipValidation && !canProceed.value) {
		return
	}

	if (await beforeProceed(step)) {
		currentStep.value = step
		await nextTick()

		await afterProceed(step)
	}
}

watch(selectedPlan, () => {
	if (currentStep.value === 'plan') {
		customServer.value = !selectedPlan.value
	}
})

const defaultPlan = computed<Labrinth.Billing.Internal.Product | undefined>(() => {
	return (
		props.availableProducts.find((p) => p?.metadata?.type === 'pyro' && p.metadata.ram === 6144) ??
		props.availableProducts.find((p) => p?.metadata?.type === 'pyro') ??
		props.availableProducts[0]
	)
})

function begin(
	interval: ServerBillingInterval,
	plan?: Labrinth.Billing.Internal.Product | null,
	project?: string,
) {
	loading.value = false

	if (plan === null) {
		// Explicitly open in custom mode
		selectedPlan.value = undefined
		customServer.value = true
	} else {
		selectedPlan.value = plan ?? defaultPlan.value
		customServer.value = !selectedPlan.value
	}

	selectedInterval.value = interval
	customServer.value = !selectedPlan.value
	selectedPaymentMethod.value = undefined
	currentStep.value = steps[0]
	skipPaymentMethods.value = true
	projectId.value = project
	modal.value?.show()
}

defineExpose({
	show: begin,
})

defineEmits<{
	(e: 'hide'): void
}>()

function handleChooseCustom() {
	customServer.value = true
	selectedPlan.value = undefined
}

// When the user explicitly wants to change or add a payment method from Review
// we must disable the auto-skip behavior, clear any selected method, and
// navigate to the Payment step so Stripe Elements can mount.
async function changePaymentMethod() {
	skipPaymentMethods.value = false
	selectedPaymentMethod.value = undefined
	await setStep('payment', true)
}

function goToBreadcrumbStep(id: string) {
	if (id === 'payment') {
		return changePaymentMethod()
	}

	return setStep(id as Step, true)
}
</script>
<template>
	<NewModal ref="modal" @hide="$emit('hide')">
		<template #title>
			<div class="flex items-center gap-1 font-bold text-secondary">
				<template v-for="(step, index) in steps" :key="step">
					<button
						v-if="index < currentStepIndex"
						class="bg-transparent active:scale-95 font-bold text-secondary p-0"
						@click="goToBreadcrumbStep(step)"
					>
						{{ formatMessage(titles[step]) }}
					</button>
					<span
						v-else
						:class="{
							'text-contrast': index === currentStepIndex,
						}"
					>
						{{ formatMessage(titles[step]) }}
					</span>
					<ChevronRightIcon
						v-if="index < steps.length - 1"
						class="h-5 w-5 text-secondary"
						stroke-width="3"
					/>
				</template>
			</div>
		</template>
		<div class="w-[40rem] max-w-full">
			<PlanSelector
				v-if="currentStep === 'plan'"
				v-model:plan="selectedPlan"
				v-model:interval="selectedInterval"
				:existing-plan="existingPlan"
				:available-products="availableProducts"
				:currency="currency"
				@choose-custom="handleChooseCustom"
			/>
			<RegionSelector
				v-else-if="currentStep === 'region'"
				v-model:region="selectedRegion"
				v-model:plan="selectedPlan"
				:regions="regions"
				:pings="pings"
				:custom="customServer"
				:available-products="availableProducts"
				:currency="currency"
				:interval="selectedInterval"
				:fetch-stock="fetchStock"
			/>
			<PaymentMethodSelector
				v-else-if="currentStep === 'payment' && selectedPlan && selectedInterval"
				:payment-methods="paymentMethods"
				:selected="selectedPaymentMethod"
				:loading-elements="loadingElements"
				:loading-elements-failed="loadingElementsFailed"
				@select="selectPaymentMethod"
			/>
			<ConfirmPurchase
				v-else-if="
					currentStep === 'review' &&
					(hasPaymentMethod || noPaymentRequired) &&
					currentRegion &&
					selectedInterval &&
					selectedPlan
				"
				v-model:interval="selectedInterval"
				v-model:accepted-eula="acceptedEula"
				:currency="currency"
				:plan="selectedPlan"
				:region="currentRegion"
				:ping="currentPing"
				:loading="paymentMethodLoading"
				:selected-payment-method="selectedPaymentMethod || inputtedPaymentMethod"
				:has-payment-method="hasPaymentMethod"
				:tax="tax"
				:total="total"
				:no-payment-required="noPaymentRequired"
				:existing-plan="existingPlan"
				:existing-subscription="existingSubscription"
				@change-payment-method="changePaymentMethod"
				@reload-payment-intent="reloadPaymentIntent"
			/>
			<div v-else>Something went wrong</div>
			<div
				v-show="
					selectedPaymentMethod === undefined &&
					currentStep === 'payment' &&
					selectedPlan &&
					selectedInterval
				"
				class="min-h-[16rem] flex flex-col gap-2 mt-2 p-4 bg-table-alternateRow rounded-xl justify-center items-center"
			>
				<div v-show="loadingElements">
					<ModalLoadingIndicator :error="loadingElementsFailed">
						Loading...
						<template #error> Error loading Stripe payment UI. </template>
					</ModalLoadingIndicator>
				</div>
				<div class="w-full">
					<div id="address-element"></div>
					<div id="payment-element" class="mt-4"></div>
				</div>
			</div>
		</div>
		<div class="flex gap-2 justify-between mt-4">
			<ButtonStyled>
				<button v-if="previousStep" @click="previousStep && setStep(previousStep, true)">
					<LeftArrowIcon /> {{ formatMessage(commonMessages.backButton) }}
				</button>
				<button v-else @click="modal?.hide()">
					<XIcon />
					{{ formatMessage(commonMessages.cancelButton) }}
				</button>
			</ButtonStyled>
			<ButtonStyled color="brand">
				<button
					v-tooltip="
						currentStep === 'review' && !acceptedEula && !noPaymentRequired
							? 'You must accept the Minecraft EULA to proceed.'
							: undefined
					"
					:disabled="!canProceed"
					@click="
						noPaymentRequired && currentStep === 'review'
							? (async () => {
									if (props.onFinalizeNoPaymentChange) {
										try {
											await props.onFinalizeNoPaymentChange()
										} catch (e) {
											return
										}
									}
									modal?.hide()
								})()
							: setStep(nextStep)
					"
				>
					<template v-if="currentStep === 'review'">
						<template v-if="noPaymentRequired"><CheckCircleIcon /> Confirm Change</template>
						<template v-else>
							<SpinnerIcon v-if="completingPurchase" class="animate-spin" />
							<CheckCircleIcon v-else />
							Subscribe
						</template>
					</template>
					<template v-else>
						{{ formatMessage(commonMessages.nextButton) }} <RightArrowIcon />
					</template>
				</button>
			</ButtonStyled>
		</div>
	</NewModal>
</template>
