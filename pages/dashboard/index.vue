<template>
  <div>
    <section class="universal-card">
      <h2>Overview</h2>
      <div class="metrics">
        <div class="metric">
          <div class="label">Total downloads</div>
          <div class="value">
            {{
              $formatNumber(
                $user.projects.reduce((agg, x) => agg + x.downloads, 0)
              )
            }}
          </div>
          <span
            >from {{ $user.projects.length }} project{{
              $user.projects.length === 1 ? '' : 's'
            }}</span
          >
          <!--          <NuxtLink class="goto-link" to="/dashboard/analytics"-->
          <!--            >View breakdown-->
          <!--            <ChevronRightIcon-->
          <!--              class="featured-header-chevron"-->
          <!--              aria-hidden="true"-->
          <!--          /></NuxtLink>-->
        </div>
        <div class="metric">
          <div class="label">Total followers</div>
          <div class="value">
            {{
              $formatNumber(
                $user.projects.reduce((agg, x) => agg + x.followers, 0)
              )
            }}
          </div>
          <span>
            <span
              >from {{ $user.projects.length }} project{{
                $user.projects.length === 1 ? '' : 's'
              }}</span
            ></span
          >
          <!--          <NuxtLink class="goto-link" to="/dashboard/analytics"-->
          <!--            >View breakdown-->
          <!--            <ChevronRightIcon-->
          <!--              class="featured-header-chevron"-->
          <!--              aria-hidden="true"-->
          <!--          /></NuxtLink>-->
        </div>
        <div class="metric">
          <div class="label">Total revenue</div>
          <div class="value">{{ $formatMoney(payouts.all_time) }}</div>
          <span>{{ $formatMoney(payouts.last_month) }} this month</span>
          <!--          <NuxtLink class="goto-link" to="/dashboard/analytics"-->
          <!--            >View breakdown-->
          <!--            <ChevronRightIcon-->
          <!--              class="featured-header-chevron"-->
          <!--              aria-hidden="true"-->
          <!--          /></NuxtLink>-->
        </div>
        <div class="metric">
          <div class="label">Current balance</div>
          <div class="value">
            {{ $formatMoney($auth.user.payout_data.balance) }}
          </div>
          <NuxtLink
            v-if="$auth.user.payout_data.balance >= minWithdraw"
            class="goto-link"
            to="/dashboard/revenue"
            >Withdraw earnings
            <ChevronRightIcon
              class="featured-header-chevron"
              aria-hidden="true"
          /></NuxtLink>
          <span v-else>${{ minWithdraw }} is the withdraw minimum</span>
        </div>
      </div>
    </section>
    <section class="universal-card more-soon">
      <h2>More coming soon!</h2>
      <p>
        Stay tuned for more metrics and analytics (pretty graphs, anyone? 👀)
        coming to the creators dashboard soon!
      </p>
    </section>
  </div>
</template>

<script>
import ChevronRightIcon from '~/assets/images/utils/chevron-right.svg?inline'

export default {
  components: { ChevronRightIcon },
  async asyncData(data) {
    const [payouts] = (
      await Promise.all([
        data.$axios.get(
          `user/${data.$auth.user.id}/payouts`,
          data.$defaultHeaders()
        ),
      ])
    ).map((it) => it.data)

    payouts.all_time = Math.floor(payouts.all_time * 100) / 100
    payouts.last_month = Math.floor(payouts.last_month * 100) / 100

    return {
      payouts,
    }
  },
  data() {
    return {
      minWithdraw: 0.26,
    }
  },
  fetch() {},
  head: {
    title: 'Creator dashboard - Modrinth',
  },
  methods: {},
}
</script>
<style lang="scss" scoped>
.metrics {
  display: grid;
  grid-gap: var(--spacing-card-md);
  grid-template-columns: repeat(auto-fit, minmax(12rem, 1fr));

  .metric {
    display: flex;
    flex-grow: 1;
    flex-direction: column;
    justify-content: center;
    background-color: var(--color-bg);
    border-radius: var(--size-rounded-card);
    padding: var(--spacing-card-lg);
    gap: var(--spacing-card-md);

    .label {
      color: var(--color-heading);
      font-weight: bold;
      font-size: 1rem;
    }

    .value {
      color: var(--color-text-dark);
      font-weight: bold;
      font-size: 2rem;
    }
  }
}
</style>
