<template>
  <section class="universal-card experimental-styles-within">
    <h2>{{ formatMessage(messages.subscriptionTitle) }}</h2>
    <p>{{ formatMessage(messages.subscriptionDescription) }}</p>
    <div class="universal-card recessed">
      <ConfirmModal
        ref="modalCancel"
        :title="formatMessage(cancelModalMessages.title)"
        :description="formatMessage(cancelModalMessages.description)"
        :proceed-label="formatMessage(cancelModalMessages.action)"
        @proceed="cancelSubscription(cancelSubscriptionId, true)"
      />
      <div class="flex flex-wrap justify-between gap-4">
        <div class="flex flex-col gap-4">
          <template v-if="midasCharge">
            <span v-if="midasCharge.status === 'open'"> You're currently subscribed to: </span>
            <span v-else-if="midasCharge.status === 'processing'" class="text-orange">
              Your payment is being processed. Perks will activate once payment is complete.
            </span>
            <span v-else-if="midasCharge.status === 'cancelled'">
              You've cancelled your subscription. <br />
              You will retain your perks until the end of the current billing cycle.
            </span>
            <span v-else-if="midasCharge.status === 'failed'" class="text-red">
              Your subscription payment failed. Please update your payment method.
            </span>
          </template>

          <span v-else>Become a subscriber to Modrinth Plus!</span>
          <ModrinthPlusIcon class="h-8 w-min" />
          <div class="flex flex-col gap-2">
            <span class="font-bold">Benefits</span>
            <div class="flex items-center gap-2">
              <CheckCircleIcon class="h-5 w-5 text-brand" />
              <span> Ad-free browsing on modrinth.com and Modrinth App </span>
            </div>
            <div class="flex items-center gap-2">
              <CheckCircleIcon class="h-5 w-5 text-brand" />
              <span>Modrinth+ badge on your profile</span>
            </div>
            <div class="flex items-center gap-2">
              <CheckCircleIcon class="h-5 w-5 text-brand" />
              <span>Support Modrinth and creators directly</span>
            </div>
          </div>
        </div>
        <div class="flex w-full flex-wrap justify-between gap-4 xl:w-auto xl:flex-col">
          <div class="flex flex-col gap-1 xl:ml-auto xl:text-right">
            <span class="text-2xl font-bold text-dark">
              <template v-if="midasCharge">
                {{
                  formatPrice(
                    vintl.locale,
                    midasSubscriptionPrice.prices.intervals[midasCharge.subscription_interval],
                    midasSubscriptionPrice.currency_code,
                  )
                }}
                /
                {{ midasCharge.subscription_interval }}
              </template>
              <template v-else>
                {{ formatPrice(vintl.locale, price.prices.intervals.monthly, price.currency_code) }}
                / month
              </template>
            </span>
            <template v-if="midasCharge">
              <span
                v-if="
                  midasCharge.status === 'open' && midasCharge.subscription_interval === 'monthly'
                "
                class="text-sm text-purple"
              >
                Save
                {{
                  formatPrice(
                    vintl.locale,
                    midasCharge.amount * 12 - oppositePrice,
                    midasCharge.currency_code,
                  )
                }}/year by switching to yearly billing!
              </span>
              <span class="text-sm text-secondary">
                Since {{ $dayjs(midasSubscription.created).format("MMMM D, YYYY") }}
              </span>
              <span v-if="midasCharge.status === 'open'" class="text-sm text-secondary">
                Renews {{ $dayjs(midasCharge.due).format("MMMM D, YYYY") }}
              </span>
              <span v-else-if="midasCharge.status === 'cancelled'" class="text-sm text-secondary">
                Expires {{ $dayjs(midasCharge.due).format("MMMM D, YYYY") }}
              </span>
            </template>

            <span v-else class="text-sm text-secondary">
              Or
              {{ formatPrice(vintl.locale, price.prices.intervals.yearly, price.currency_code) }} /
              year (save
              {{
                calculateSavings(price.prices.intervals.monthly, price.prices.intervals.yearly)
              }}%)!
            </span>
          </div>
          <div
            v-if="midasCharge && midasCharge.status === 'failed'"
            class="ml-auto flex flex-row-reverse items-center gap-2"
          >
            <ButtonStyled v-if="midasCharge && midasCharge.status === 'failed'">
              <button
                @click="
                  () => {
                    $refs.midasPurchaseModal.show();
                  }
                "
              >
                <UpdatedIcon />
                Update method
              </button>
            </ButtonStyled>
            <ButtonStyled type="transparent" circular>
              <OverflowMenu
                :dropdown-id="`${baseId}-cancel-midas`"
                :options="[
                  {
                    id: 'cancel',
                    action: () => {
                      cancelSubscriptionId = midasSubscription.id;
                      $refs.modalCancel.show();
                    },
                  },
                ]"
              >
                <MoreVerticalIcon />
                <template #cancel><XIcon /> Cancel</template>
              </OverflowMenu>
            </ButtonStyled>
          </div>
          <div
            v-else-if="midasCharge && midasCharge.status !== 'cancelled'"
            class="ml-auto flex gap-2"
          >
            <ButtonStyled>
              <button
                :disabled="changingInterval"
                @click="
                  () => {
                    cancelSubscriptionId = midasSubscription.id;
                    $refs.modalCancel.show();
                  }
                "
              >
                <XIcon /> Cancel
              </button>
            </ButtonStyled>
            <ButtonStyled
              :color="midasCharge.subscription_interval === 'yearly' ? 'standard' : 'purple'"
              color-fill="text"
            >
              <button
                v-tooltip="
                  midasCharge.subscription_interval === 'yearly'
                    ? `Monthly billing will cost you an additional ${formatPrice(
                        vintl.locale,
                        oppositePrice * 12 - midasCharge.amount,
                        midasCharge.currency_code,
                      )} per year`
                    : undefined
                "
                :disabled="changingInterval"
                @click="switchMidasInterval(oppositeInterval)"
              >
                <SpinnerIcon v-if="changingInterval" class="animate-spin" />
                <TransferIcon v-else /> {{ changingInterval ? "Switching" : "Switch" }} to
                {{ oppositeInterval }}
              </button>
            </ButtonStyled>
          </div>
          <ButtonStyled
            v-else-if="midasCharge && midasCharge.status === 'cancelled'"
            color="purple"
          >
            <button class="ml-auto" @click="cancelSubscription(midasSubscription.id, false)">
              Resubscribe <RightArrowIcon />
            </button>
          </ButtonStyled>
          <ButtonStyled v-else color="purple" size="large">
            <button
              class="ml-auto"
              @click="
                () => {
                  $refs.midasPurchaseModal.show();
                }
              "
            >
              Subscribe <RightArrowIcon />
            </button>
          </ButtonStyled>
        </div>
      </div>
    </div>
    <div v-if="pyroSubscriptions.length > 0">
      <div
        v-for="(subscription, index) in pyroSubscriptions"
        :key="index"
        class="universal-card recessed mt-4"
      >
        <div class="flex flex-col justify-between gap-4">
          <div class="flex flex-col gap-4">
            <LazyUiServersModrinthServersIcon class="flex h-8 w-fit" />
            <div class="flex flex-col gap-2">
              <UiServersServerListing
                v-if="subscription.serverInfo"
                :server_id="subscription.serverInfo.server_id"
                :name="subscription.serverInfo.name"
                :status="subscription.serverInfo.status"
                :game="subscription.serverInfo.game"
                :loader="subscription.serverInfo.loader"
                :loader_version="subscription.serverInfo.loader_version"
                :mc_version="subscription.serverInfo.mc_version"
                :upstream="subscription.serverInfo.upstream"
                :net="subscription.serverInfo.net"
              />
              <div v-else class="w-fit">
                <p>
                  A linked server couldn't be found for this subscription. There are a few possible
                  explanations for this. If you just purchased your server, this is normal. It could
                  take up to an hour for your server to be provisioned. Otherwise, if you purchased
                  this server a while ago, it has likely since been suspended. If this is not what
                  you were expecting, please contact Modrinth Support with the following
                  information:
                </p>
                <div class="flex w-full flex-col gap-2">
                  <CopyCode
                    class="whitespace-nowrap"
                    :text="'Server ID: ' + subscription.metadata.id"
                  />
                  <CopyCode class="whitespace-nowrap" :text="'Stripe ID: ' + subscription.id" />
                </div>
              </div>
              <h3 class="m-0 mt-4 text-xl font-semibold leading-none text-contrast">
                {{ getProductSize(getPyroProduct(subscription)) }} Plan
              </h3>
              <div class="flex flex-row justify-between">
                <div class="mt-2 flex flex-col gap-2">
                  <div class="flex items-center gap-2">
                    <CheckCircleIcon class="h-5 w-5 text-brand" />
                    <span>
                      {{ getPyroProduct(subscription)?.metadata?.cpu / 2 }} Shared CPUs (Bursts up
                      to {{ getPyroProduct(subscription)?.metadata?.cpu }} CPUs)
                    </span>
                  </div>
                  <div class="flex items-center gap-2">
                    <CheckCircleIcon class="h-5 w-5 text-brand" />
                    <span>
                      {{
                        getPyroProduct(subscription)?.metadata?.ram
                          ? getPyroProduct(subscription).metadata.ram / 1024 + " GB RAM"
                          : ""
                      }}
                    </span>
                  </div>
                  <div class="flex items-center gap-2">
                    <CheckCircleIcon class="h-5 w-5 text-brand" />
                    <span>
                      {{
                        getPyroProduct(subscription)?.metadata?.swap
                          ? getPyroProduct(subscription).metadata.swap / 1024 + " GB Swap"
                          : ""
                      }}
                    </span>
                  </div>
                  <div class="flex items-center gap-2">
                    <CheckCircleIcon class="h-5 w-5 text-brand" />
                    <span>
                      {{
                        getPyroProduct(subscription)?.metadata?.storage
                          ? getPyroProduct(subscription).metadata.storage / 1024 + " GB SSD"
                          : ""
                      }}
                    </span>
                  </div>
                </div>
                <div class="flex flex-col items-end justify-between">
                  <div class="flex flex-col items-end gap-2">
                    <div class="flex text-2xl font-bold text-contrast">
                      <span class="text-contrast">
                        {{
                          formatPrice(
                            vintl.locale,
                            getProductPrice(getPyroProduct(subscription), subscription.interval)
                              .prices.intervals[subscription.interval],
                            getProductPrice(getPyroProduct(subscription), subscription.interval)
                              .currency_code,
                          )
                        }}
                      </span>
                      <span>/{{ subscription.interval.replace("ly", "") }}</span>
                    </div>
                    <div v-if="getPyroCharge(subscription)" class="mb-4 flex flex-col items-end">
                      <span class="text-sm text-secondary">
                        Since {{ $dayjs(subscription.created).format("MMMM D, YYYY") }}
                      </span>
                      <span
                        v-if="getPyroCharge(subscription).status === 'open'"
                        class="text-sm text-secondary"
                      >
                        Renews {{ $dayjs(getPyroCharge(subscription).due).format("MMMM D, YYYY") }}
                      </span>
                      <span
                        v-else-if="getPyroCharge(subscription).status === 'processing'"
                        class="text-sm text-orange"
                      >
                        Your payment is being processed. Your server will activate once payment is
                        complete.
                      </span>
                      <span
                        v-else-if="getPyroCharge(subscription).status === 'cancelled'"
                        class="text-sm text-secondary"
                      >
                        Expires {{ $dayjs(getPyroCharge(subscription).due).format("MMMM D, YYYY") }}
                      </span>
                      <span
                        v-else-if="getPyroCharge(subscription).status === 'failed'"
                        class="text-sm text-red"
                      >
                        Your subscription payment failed. Please update your payment method, then
                        resubscribe.
                      </span>
                    </div>
                  </div>
                  <div class="flex gap-2">
                    <ButtonStyled
                      v-if="
                        getPyroCharge(subscription) &&
                        getPyroCharge(subscription).status !== 'cancelled' &&
                        getPyroCharge(subscription).status !== 'failed'
                      "
                    >
                      <button @click="showCancellationSurvey(subscription)">
                        <XIcon />
                        Cancel
                      </button>
                    </ButtonStyled>
                    <ButtonStyled
                      v-if="
                        getPyroCharge(subscription) &&
                        getPyroCharge(subscription).status !== 'cancelled' &&
                        getPyroCharge(subscription).status !== 'failed'
                      "
                      color="green"
                      color-fill="text"
                    >
                      <button @click="showPyroUpgradeModal(subscription)">
                        <ArrowBigUpDashIcon />
                        Upgrade
                      </button>
                    </ButtonStyled>
                    <ButtonStyled
                      v-else-if="
                        getPyroCharge(subscription) &&
                        (getPyroCharge(subscription).status === 'cancelled' ||
                          getPyroCharge(subscription).status === 'failed')
                      "
                      color="green"
                    >
                      <button
                        @click="
                          resubscribePyro(
                            subscription.id,
                            $dayjs(getPyroCharge(subscription).due).isBefore($dayjs()),
                          )
                        "
                      >
                        Resubscribe <RightArrowIcon />
                      </button>
                    </ButtonStyled>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </section>

  <section class="universal-card experimental-styles-within">
    <ConfirmModal
      ref="modal_confirm"
      :title="formatMessage(deleteModalMessages.title)"
      :description="formatMessage(deleteModalMessages.description)"
      :proceed-label="formatMessage(deleteModalMessages.action)"
      @proceed="removePaymentMethod(removePaymentMethodIndex)"
    />
    <PurchaseModal
      ref="midasPurchaseModal"
      :product="midasProduct"
      :country="country"
      :publishable-key="config.public.stripePublishableKey"
      :send-billing-request="
        async (body) =>
          await useBaseFetch('billing/payment', { internal: true, method: 'POST', body })
      "
      :on-error="
        (err) =>
          data.$notify({
            group: 'main',
            title: 'An error occurred',
            type: 'error',
            text: err.message ?? (err.data ? err.data.description : err),
          })
      "
      :customer="customer"
      :payment-methods="paymentMethods"
      :return-url="`${config.public.siteUrl}/settings/billing`"
    />
    <PurchaseModal
      ref="pyroPurchaseModal"
      :product="upgradeProducts"
      :country="country"
      custom-server
      :existing-subscription="currentSubscription"
      :existing-plan="currentProduct"
      :publishable-key="config.public.stripePublishableKey"
      :send-billing-request="
        async (body) =>
          await useBaseFetch(`billing/subscription/${currentSubscription.id}`, {
            internal: true,
            method: `PATCH`,
            body: body,
          })
      "
      :renewal-date="currentSubRenewalDate"
      :on-error="
        (err) =>
          data.$notify({
            group: 'main',
            title: 'An error occurred',
            type: 'error',
            text: err.message ?? (err.data ? err.data.description : err),
          })
      "
      :fetch-capacity-statuses="fetchCapacityStatuses"
      :customer="customer"
      :payment-methods="paymentMethods"
      :return-url="`${config.public.siteUrl}/servers/manage`"
      :server-name="`${auth?.user?.username}'s server`"
    />
    <NewModal ref="addPaymentMethodModal">
      <template #title>
        <span class="text-lg font-extrabold text-contrast">
          {{ formatMessage(messages.paymentMethodTitle) }}
        </span>
      </template>
      <div class="min-h-[16rem] md:w-[600px]">
        <div
          v-show="loadingPaymentMethodModal !== 2"
          class="flex min-h-[16rem] items-center justify-center"
        >
          <AnimatedLogo class="w-[80px]" />
        </div>
        <div v-show="loadingPaymentMethodModal === 2" class="min-h-[16rem] p-1">
          <div id="address-element"></div>
          <div id="payment-element" class="mt-4"></div>
        </div>
        <div v-show="loadingPaymentMethodModal === 2" class="input-group mt-auto pt-4">
          <ButtonStyled color="brand">
            <button :disabled="loadingAddMethod" @click="submit">
              <PlusIcon />
              {{ formatMessage(messages.paymentMethodAdd) }}
            </button>
          </ButtonStyled>
          <ButtonStyled>
            <button @click="$refs.addPaymentMethodModal.hide()">
              <XIcon />
              {{ formatMessage(commonMessages.cancelButton) }}
            </button>
          </ButtonStyled>
        </div>
      </div>
    </NewModal>
    <div class="header__row">
      <div class="header__title">
        <h2 class="text-2xl">{{ formatMessage(messages.paymentMethodTitle) }}</h2>
      </div>
      <nuxt-link class="btn" to="/settings/billing/charges">
        <HistoryIcon /> {{ formatMessage(messages.paymentMethodHistory) }}
      </nuxt-link>
      <button class="btn" @click="addPaymentMethod">
        <PlusIcon /> {{ formatMessage(messages.paymentMethodAdd) }}
      </button>
    </div>
    <div
      v-if="!paymentMethods || paymentMethods.length === 0"
      class="universal-card recessed !mb-0"
    >
      {{ formatMessage(messages.paymentMethodNone) }}
    </div>
    <div v-else class="flex flex-col gap-4">
      <div
        v-for="(method, index) in paymentMethods"
        :key="index"
        class="universal-card recessed !mb-0 flex items-center justify-between"
      >
        <div class="flex gap-2">
          <CardIcon v-if="method.type === 'card'" class="h-8 w-8" />
          <CurrencyIcon v-else-if="method.type === 'cashapp'" class="h-8 w-8" />
          <PayPalIcon v-else-if="method.type === 'paypal'" class="h-8 w-8" />
          <div class="flex flex-col">
            <div class="flex items-center gap-2">
              <div class="font-bold text-contrast">
                <template v-if="method.type === 'card'">
                  {{
                    formatMessage(messages.paymentMethodCardDisplay, {
                      card_brand:
                        formatMessage(paymentMethodTypes[method.card.brand]) ??
                        formatMessage(paymentMethodTypes.unknown),
                      last_four: method.card.last4,
                    })
                  }}
                </template>
                <template v-else>
                  {{
                    formatMessage(paymentMethodTypes[method.type]) ??
                    formatMessage(paymentMethodTypes.unknown)
                  }}
                </template>
              </div>
              <div
                v-if="primaryPaymentMethodId === method.id"
                class="border-r-ma rounded-full bg-button-bg px-2 py-0.5 text-sm font-bold text-secondary"
              >
                {{ formatMessage(messages.paymentMethodPrimary) }}
              </div>
            </div>
            <div v-if="method.type === 'card'" class="text-secondary">
              {{
                formatMessage(messages.paymentMethodCardExpiry, {
                  month: method.card.exp_month,
                  year: method.card.exp_year,
                })
              }}
            </div>
            <div v-else-if="method.type === 'cashapp'" class="text-secondary">
              {{ method.cashapp.cashtag }}
            </div>
            <div v-else-if="method.type === 'paypal'" class="text-secondary">
              {{ method.paypal.payer_email }}
            </div>
          </div>
        </div>
        <OverflowMenu
          :dropdown-id="`${baseId}-payment-method-overflow-${index}`"
          class="btn icon-only transparent"
          :options="
            [
              {
                id: 'primary',
                action: () => editPaymentMethod(index, true),
              },
              {
                id: 'remove',
                action: () => {
                  removePaymentMethodIndex = index;
                  $refs.modal_confirm.show();
                },
                color: 'red',
                hoverOnly: true,
              },
            ].slice(primaryPaymentMethodId === method.id ? 1 : 0, 2)
          "
        >
          <MoreVerticalIcon />
          <template #primary>
            <StarIcon />
            {{ formatMessage(messages.paymentMethodMakePrimary) }}
          </template>
          <template #edit>
            <EditIcon />
            {{ formatMessage(commonMessages.editButton) }}
          </template>
          <template #remove>
            <TrashIcon />
            {{ formatMessage(commonMessages.deleteLabel) }}
          </template>
        </OverflowMenu>
      </div>
    </div>
  </section>
</template>

<script setup>
import {
  ConfirmModal,
  NewModal,
  OverflowMenu,
  AnimatedLogo,
  PurchaseModal,
  ButtonStyled,
  CopyCode,
  commonMessages,
} from "@modrinth/ui";
import {
  PlusIcon,
  TransferIcon,
  SpinnerIcon,
  ArrowBigUpDashIcon,
  XIcon,
  CardIcon,
  MoreVerticalIcon,
  TrashIcon,
  EditIcon,
  StarIcon,
  PayPalIcon,
  CurrencyIcon,
  CheckCircleIcon,
  RightArrowIcon,
  ModrinthPlusIcon,
  UpdatedIcon,
  HistoryIcon,
} from "@modrinth/assets";
import { calculateSavings, formatPrice, createStripeElements, getCurrency } from "@modrinth/utils";
import { ref, computed } from "vue";
import { products } from "~/generated/state.json";

definePageMeta({
  middleware: "auth",
});

const app = useNuxtApp();
const auth = await useAuth();
const baseId = useId();

useHead({
  script: [
    {
      src: "https://js.stripe.com/v3/",
      defer: true,
      async: true,
    },
  ],
});

const data = useNuxtApp();
const config = useRuntimeConfig();

const vintl = useVIntl();
const { formatMessage } = vintl;

const deleteModalMessages = defineMessages({
  title: {
    id: "settings.billing.modal.delete.title",
    defaultMessage: "Are you sure you want to remove this payment method?",
  },
  description: {
    id: "settings.billing.modal.delete.description",
    defaultMessage: "This will remove this payment method forever (like really forever).",
  },
  action: {
    id: "settings.billing.modal.delete.action",
    defaultMessage: "Remove this payment method",
  },
});

const cancelModalMessages = defineMessages({
  title: {
    id: "settings.billing.modal.cancel.title",
    defaultMessage: "Are you sure you want to cancel your subscription?",
  },
  description: {
    id: "settings.billing.modal.cancel.description",
    defaultMessage:
      "This will cancel your subscription. You will retain your perks until the end of the current billing cycle.",
  },
  action: {
    id: "settings.billing.modal.cancel.action",
    defaultMessage: "Cancel subscription",
  },
});

const messages = defineMessages({
  subscriptionTitle: {
    id: "settings.billing.subscription.title",
    defaultMessage: "Subscriptions",
  },
  subscriptionDescription: {
    id: "settings.billing.subscription.description",
    defaultMessage: "Manage your Modrinth subscriptions.",
  },
  paymentMethodTitle: {
    id: "settings.billing.payment_method.title",
    defaultMessage: "Payment methods",
  },
  paymentMethodNone: {
    id: "settings.billing.payment_method.none",
    defaultMessage: "You have not added any payment methods.",
  },
  paymentMethodHistory: {
    id: "settings.billing.payment_method.action.history",
    defaultMessage: "View past charges",
  },
  paymentMethodAdd: {
    id: "settings.billing.payment_method.action.add",
    defaultMessage: "Add payment method",
  },
  paymentMethodPrimary: {
    id: "settings.billing.payment_method.primary",
    defaultMessage: "Primary",
  },
  paymentMethodMakePrimary: {
    id: "settings.billing.payment_method.action.primary",
    defaultMessage: "Make primary",
  },
  paymentMethodCardDisplay: {
    id: "settings.billing.payment_method.card_display",
    defaultMessage: "{card_brand} ending in {last_four}",
  },
  paymentMethodCardExpiry: {
    id: "settings.billing.payment_method.card_expiry",
    defaultMessage: "Expires {month}/{year}",
  },
  pyroSubscriptionTitle: {
    id: "settings.billing.pyro_subscription.title",
    defaultMessage: "Modrinth Server Subscriptions",
  },
  pyroSubscriptionDescription: {
    id: "settings.billing.pyro_subscription.description",
    defaultMessage: "Manage your Modrinth Server subscriptions.",
  },
});

const paymentMethodTypes = defineMessages({
  visa: {
    id: "settings.billing.payment_method_type.visa",
    defaultMessage: "Visa",
  },
  amex: { id: "settings.billing.payment_method_type.amex", defaultMessage: "American Express" },
  diners: { id: "settings.billing.payment_method_type.diners", defaultMessage: "Diners Club" },
  discover: { id: "settings.billing.payment_method_type.discover", defaultMessage: "Discover" },
  eftpos: { id: "settings.billing.payment_method_type.eftpos", defaultMessage: "EFTPOS" },
  jcb: { id: "settings.billing.payment_method_type.jcb", defaultMessage: "JCB" },
  mastercard: {
    id: "settings.billing.payment_method_type.mastercard",
    defaultMessage: "MasterCard",
  },
  unionpay: { id: "settings.billing.payment_method_type.unionpay", defaultMessage: "UnionPay" },
  paypal: { id: "settings.billing.payment_method_type.paypal", defaultMessage: "PayPal" },
  cashapp: { id: "settings.billing.payment_method_type.cashapp", defaultMessage: "Cash App" },
  amazon_pay: {
    id: "settings.billing.payment_method_type.amazon_pay",
    defaultMessage: "Amazon Pay",
  },
  unknown: {
    id: "settings.billing.payment_method_type.unknown",
    defaultMessage: "Unknown payment method",
  },
});

let stripe = null;
let elements = null;

function loadStripe() {
  try {
    if (!stripe) {
      stripe = Stripe(config.public.stripePublishableKey);
    }
  } catch (error) {
    console.error("Error loading Stripe:", error);
  }
}

const [
  { data: paymentMethods, refresh: refreshPaymentMethods },
  { data: charges, refresh: refreshCharges },
  { data: customer, refresh: refreshCustomer },
  { data: subscriptions, refresh: refreshSubscriptions },
  { data: productsData, refresh: refreshProducts },
  { data: serversData, refresh: refreshServers },
] = await Promise.all([
  useAsyncData("billing/payment_methods", () =>
    useBaseFetch("billing/payment_methods", { internal: true }),
  ),
  useAsyncData("billing/payments", () => useBaseFetch("billing/payments", { internal: true })),
  useAsyncData("billing/customer", () => useBaseFetch("billing/customer", { internal: true })),
  useAsyncData("billing/subscriptions", () =>
    useBaseFetch("billing/subscriptions", { internal: true }),
  ),
  useAsyncData("billing/products", () => useBaseFetch("billing/products", { internal: true })),
  useAsyncData("servers", () => usePyroFetch("servers")),
]);

const midasProduct = ref(products.find((x) => x.metadata?.type === "midas"));
const midasSubscription = computed(() =>
  subscriptions.value?.find(
    (x) =>
      x.status === "provisioned" && midasProduct.value?.prices?.find((y) => y.id === x.price_id),
  ),
);
const midasSubscriptionPrice = computed(() =>
  midasSubscription.value
    ? midasProduct.value?.prices?.find((x) => x.id === midasSubscription.value.price_id)
    : null,
);
const midasCharge = computed(() =>
  midasSubscription.value
    ? charges.value?.find((x) => x.subscription_id === midasSubscription.value.id)
    : null,
);

const oppositePrice = computed(() =>
  midasSubscription.value
    ? midasProduct.value?.prices?.find((price) => price.id === midasSubscription.value.price_id)
        ?.prices?.intervals?.[oppositeInterval.value]
    : undefined,
);

const pyroSubscriptions = computed(() => {
  const pyroSubs = subscriptions.value?.filter((s) => s?.metadata?.type === "pyro") || [];
  const servers = serversData.value?.servers || [];

  return pyroSubs.map((subscription) => {
    const server = servers.find((s) => s.server_id === subscription.metadata.id);
    return {
      ...subscription,
      serverInfo: server,
    };
  });
});

const midasPurchaseModal = ref();
const country = useUserCountry();
const price = computed(() =>
  midasProduct.value?.prices?.find((x) => x.currency_code === getCurrency(country.value)),
);

const primaryPaymentMethodId = computed(() => {
  if (customer.value?.invoice_settings?.default_payment_method) {
    return customer.value.invoice_settings.default_payment_method;
  } else if (paymentMethods.value?.[0]?.id) {
    return paymentMethods.value[0].id;
  } else {
    return null;
  }
});

const addPaymentMethodModal = ref();
const loadingPaymentMethodModal = ref(0);
async function addPaymentMethod() {
  try {
    loadingPaymentMethodModal.value = 0;
    addPaymentMethodModal.value.show();

    const result = await useBaseFetch("billing/payment_method", {
      internal: true,
      method: "POST",
    });

    loadStripe();
    const {
      elements: elementsVal,
      addressElement,
      paymentElement,
    } = createStripeElements(stripe, paymentMethods.value, {
      clientSecret: result.client_secret,
    });

    elements = elementsVal;
    paymentElement.on("ready", () => {
      loadingPaymentMethodModal.value += 1;
    });
    addressElement.on("ready", () => {
      loadingPaymentMethodModal.value += 1;
    });
  } catch (err) {
    data.$notify({
      group: "main",
      title: "An error occurred",
      text: err.data ? err.data.description : err,
      type: "error",
    });
  }
}

const loadingAddMethod = ref(false);
async function submit() {
  startLoading();
  loadingAddMethod.value = true;

  loadStripe();
  const { error } = await stripe.confirmSetup({
    elements,
    confirmParams: {
      return_url: `${config.public.siteUrl}/settings/billing`,
    },
  });

  if (error && error.type !== "validation_error") {
    data.$notify({
      group: "main",
      title: "An error occurred",
      text: error.message,
      type: "error",
    });
  } else if (!error) {
    await refresh();
    addPaymentMethodModal.value.close();
  }
  loadingAddMethod.value = false;
  stopLoading();
}

const removePaymentMethodIndex = ref();

const changingInterval = ref(false);

const oppositeInterval = computed(() =>
  midasCharge.value?.subscription_interval === "yearly" ? "monthly" : "yearly",
);

async function switchMidasInterval(interval) {
  changingInterval.value = true;
  startLoading();
  try {
    await useBaseFetch(`billing/subscription/${midasSubscription.value.id}`, {
      internal: true,
      method: "PATCH",
      body: {
        interval,
      },
    });
    await refresh();
  } catch (error) {
    console.error("Error switching Modrinth+ payment interval:", error);
  }
  stopLoading();
  changingInterval.value = false;
}

async function editPaymentMethod(index, primary) {
  startLoading();
  try {
    await useBaseFetch(`billing/payment_method/${paymentMethods.value[index].id}`, {
      internal: true,
      method: "PATCH",
      data: {
        primary,
      },
    });
    await refresh();
  } catch (err) {
    data.$notify({
      group: "main",
      title: "An error occurred",
      text: err.data ? err.data.description : err,
      type: "error",
    });
  }
  stopLoading();
}

async function removePaymentMethod(index) {
  startLoading();
  try {
    await useBaseFetch(`billing/payment_method/${paymentMethods.value[index].id}`, {
      internal: true,
      method: "DELETE",
    });
    await refresh();
  } catch (err) {
    data.$notify({
      group: "main",
      title: "An error occurred",
      text: err.data ? err.data.description : err,
      type: "error",
    });
  }
  stopLoading();
}

const cancelSubscriptionId = ref(null);
async function cancelSubscription(id, cancelled) {
  startLoading();
  try {
    await useBaseFetch(`billing/subscription/${id}`, {
      internal: true,
      method: "PATCH",
      body: {
        cancelled,
      },
    });
    await refresh();
  } catch (err) {
    data.$notify({
      group: "main",
      title: "An error occurred",
      text: err.data ? err.data.description : err,
      type: "error",
    });
  }
  stopLoading();
}

const getPyroProduct = (subscription) => {
  if (!subscription || !productsData.value) return null;
  return productsData.value.find((p) => p.prices?.some((x) => x.id === subscription.price_id));
};

const getPyroCharge = (subscription) => {
  if (!subscription || !charges.value) return null;
  return charges.value.find(
    (charge) => charge.subscription_id === subscription.id && charge.status !== "succeeded",
  );
};

const getProductSize = (product) => {
  if (!product || !product.metadata) return "Unknown";
  const ramSize = product.metadata.ram;
  if (ramSize === 4096) return "Small";
  if (ramSize === 6144) return "Medium";
  if (ramSize === 8192) return "Large";
  return "Custom";
};

const getProductPrice = (product, interval) => {
  if (!product || !product.prices) return null;
  const countryValue = country.value;
  return (
    product.prices.find(
      (p) => p.currency_code === getCurrency(countryValue) && p.prices?.intervals?.[interval],
    ) ??
    product.prices.find((p) => p.currency_code === "USD" && p.prices?.intervals?.[interval]) ??
    product.prices[0]
  );
};

const modalCancel = ref(null);

const pyroPurchaseModal = ref();
const currentSubscription = ref(null);
const currentProduct = ref(null);
const upgradeProducts = ref([]);
upgradeProducts.value.metadata = { type: "pyro" };

const currentSubRenewalDate = ref();

const showPyroUpgradeModal = async (subscription) => {
  currentSubscription.value = subscription;
  currentSubRenewalDate.value = getPyroCharge(subscription).due;
  currentProduct.value = getPyroProduct(subscription);
  upgradeProducts.value = products.filter(
    (p) =>
      p.metadata.type === "pyro" &&
      (!currentProduct.value || p.metadata.ram > currentProduct.value.metadata.ram),
  );
  upgradeProducts.value.metadata = { type: "pyro" };

  await nextTick();

  if (!currentProduct.value) {
    console.error("Could not find product for current subscription");
    data.$notify({
      group: "main",
      title: "An error occurred",
      text: "Could not find product for current subscription",
      type: "error",
    });
    return;
  }

  if (!pyroPurchaseModal.value) {
    console.error("pyroPurchaseModal ref is undefined");
    return;
  }

  pyroPurchaseModal.value.show();
};

async function fetchCapacityStatuses(serverId, product) {
  if (product) {
    try {
      return {
        custom: await usePyroFetch(`servers/${serverId}/upgrade-stock`, {
          method: "POST",
          body: {
            cpu: product.metadata.cpu,
            memory_mb: product.metadata.ram,
            swap_mb: product.metadata.swap,
            storage_mb: product.metadata.storage,
          },
        }),
      };
    } catch (error) {
      console.error("Error checking server capacities:", error);
      app.$notify({
        group: "main",
        title: "Error checking server capacities",
        text: error,
        type: "error",
      });
      return {
        custom: { available: 0 },
        small: { available: 0 },
        medium: { available: 0 },
        large: { available: 0 },
      };
    }
  }
}

const resubscribePyro = async (subscriptionId, wasSuspended) => {
  try {
    await useBaseFetch(`billing/subscription/${subscriptionId}`, {
      internal: true,
      method: "PATCH",
      body: {
        cancelled: false,
      },
    });
    await refresh();
    if (wasSuspended) {
      data.$notify({
        group: "main",
        title: "Resubscription request submitted",
        text: "If the server is currently suspended, it may take up to 10 minutes for another charge attempt to be made.",
        type: "success",
      });
    } else {
      data.$notify({
        group: "main",
        title: "Success",
        text: "Server subscription resubscribed successfully",
        type: "success",
      });
    }
  } catch {
    data.$notify({
      group: "main",
      title: "Error resubscribing",
      text: "An error occurred while resubscribing to your Modrinth server.",
      type: "error",
    });
  }
};

const refresh = async () => {
  await Promise.all([
    refreshPaymentMethods(),
    refreshCharges(),
    refreshCustomer(),
    refreshSubscriptions(),
    refreshProducts(),
    refreshServers(),
  ]);
};

function showCancellationSurvey(subscription) {
  if (!subscription) {
    console.warn("No survey notice to open");
    return;
  }

  const product = getPyroProduct(subscription);
  const priceObj = product?.prices?.find((x) => x.id === subscription.price_id);
  const price = priceObj?.prices?.intervals?.[subscription.interval];
  const currency = priceObj?.currency_code;

  const popupOptions = {
    layout: "modal",
    width: 700,
    autoClose: 2000,
    hideTitle: true,
    hiddenFields: {
      username: auth.value?.user?.username,
      user_id: auth.value?.user?.id,
      user_email: auth.value?.user?.email,
      subscription_id: subscription.id,
      price_id: subscription.price_id,
      interval: subscription.interval,
      started: subscription.created,
      plan_ram: product?.metadata.ram / 1024,
      plan_cpu: product?.metadata.cpu,
      price: price ? `${price / 100}` : "unknown",
      currency: currency ?? "unknown",
    },
    onOpen: () => console.log(`Opened cancellation survey for: ${subscription.id}`),
    onClose: () => console.log(`Closed cancellation survey for: ${subscription.id}`),
    onSubmit: (payload) => {
      console.log("Form submitted, cancelling server.", payload);
      cancelSubscription(subscription.id, true);
    },
  };

  const formId = "mOr7lM";

  try {
    if (window.Tally?.openPopup) {
      console.log(
        `Opening Tally popup for servers subscription ${subscription.id} (form ID: ${formId})`,
      );
      window.Tally.openPopup(formId, popupOptions);
    } else {
      console.warn("Tally script not yet loaded");
    }
  } catch (e) {
    console.error("Error opening Tally popup:", e);
  }
}

useHead({
  script: [
    {
      src: "https://tally.so/widgets/embed.js",
      defer: true,
    },
  ],
});
</script>
