<template>
  <div>
    <section class="universal-card payout-history">
      <Breadcrumbs
        current-title="Transfer history"
        :link-stack="[{ href: '/dashboard/revenue', label: 'Revenue' }]"
      />
      <h2>Transfer history</h2>
      <p>
        All of your transfers from your Modrinth balance to your PayPal or Venmo accounts will be
        listed here:
      </p>
      <div class="grid-table">
        <div class="grid-table__row grid-table__header">
          <div class="desktop">Date</div>
          <div class="desktop">Status</div>
          <div class="desktop">Amount</div>
          <div class="mobile">Transaction</div>
        </div>
        <div
          v-for="(payout, index) in payouts.payouts.filter((x) => x.status === 'success')"
          :key="`payout-${index}`"
          class="grid-table__row"
        >
          <div>{{ $dayjs(payout.created).format('MMMM D, YYYY [at] h:mm A') }}</div>
          <div><Badge :type="payout.status" /></div>
          <div class="amount">{{ $formatMoney(payout.amount) }}</div>
        </div>
      </div>
    </section>
  </div>
</template>
<script setup>
import Badge from '~/components/ui/Badge.vue'
import Breadcrumbs from '~/components/ui/Breadcrumbs.vue'

useHead({
  title: 'Transfer history - Modrinth',
})

const auth = await useAuth()
const app = useNuxtApp()

const { data: payouts } = await useAsyncData(`user/${auth.value.user.id}/payouts`, () =>
  useBaseFetch(`user/${auth.value.user.id}/payouts`, app.$defaultHeaders())
)
</script>
<style lang="scss" scoped>
.grid-table {
  display: grid;
  grid-template-columns: auto auto auto;
  border-radius: var(--size-rounded-sm);
  overflow: hidden;
  margin-top: var(--spacing-card-md);

  .grid-table__header {
    .mobile {
      display: none;
    }
  }

  .grid-table__row {
    display: contents;

    > div {
      display: flex;
      flex-direction: column;
      justify-content: center;
      padding: var(--spacing-card-sm);

      // Left edge of table
      &:first-child,
      &.mobile {
        padding-left: var(--spacing-card-bg);
      }

      // Right edge of table
      &:last-child {
        padding-right: var(--spacing-card-bg);
      }
    }

    &:nth-child(2n + 1) > div {
      background-color: var(--color-table-alternate-row);
    }

    > div {
      padding-top: var(--spacing-card-bg);
      padding-bottom: var(--spacing-card-bg);
    }

    &.grid-table__header > div {
      background-color: var(--color-bg);
      font-weight: bold;
      color: var(--color-text-dark);
    }
  }

  @media screen and (max-width: 560px) {
    display: flex;
    flex-direction: column;
    .grid-table__row {
      display: flex;
      flex-direction: column;

      > div {
        padding: var(--spacing-card-xs) var(--spacing-card-bg);

        &:first-child,
        &.mobile {
          padding-top: var(--spacing-card-bg);
        }

        &:last-child,
        &.mobile {
          padding-bottom: var(--spacing-card-bg);
        }
      }
    }

    .grid-table__header {
      .mobile {
        display: flex;
      }
      .desktop {
        display: none;
      }
    }
  }

  .amount {
    color: var(--color-heading);
    font-weight: 500;
  }
}
</style>
