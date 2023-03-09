<template>
  <div>
    <section class="universal-card">
      <h2>Overview</h2>
      <div class="grid-display">
        <div class="grid-display__item">
          <div class="label">Total downloads</div>
          <div class="value">
            {{ $formatNumber(user.projects.reduce((agg, x) => agg + x.downloads, 0)) }}
          </div>
          <span
            >from
            {{ downloadsProjectCount }}
            project{{ downloadsProjectCount === 1 ? '' : 's' }}</span
          >
          <!--          <NuxtLink class="goto-link" to="/dashboard/analytics"-->
          <!--            >View breakdown-->
          <!--            <ChevronRightIcon-->
          <!--              class="featured-header-chevron"-->
          <!--              aria-hidden="true"-->
          <!--          /></NuxtLink>-->
        </div>
        <div class="grid-display__item">
          <div class="label">Total followers</div>
          <div class="value">
            {{ $formatNumber(user.projects.reduce((agg, x) => agg + x.followers, 0)) }}
          </div>
          <span>
            <span
              >from {{ followersProjectCount }} project{{
                followersProjectCount === 1 ? '' : 's'
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
        <div class="grid-display__item">
          <div class="label">Total revenue</div>
          <div class="value">
            {{ $formatMoney(payouts.all_time) }}
          </div>
          <span>{{ $formatMoney(payouts.last_month) }} this month</span>
          <!--          <NuxtLink class="goto-link" to="/dashboard/analytics"-->
          <!--            >View breakdown-->
          <!--            <ChevronRightIcon-->
          <!--              class="featured-header-chevron"-->
          <!--              aria-hidden="true"-->
          <!--          /></NuxtLink>-->
        </div>
        <div class="grid-display__item">
          <div class="label">Current balance</div>
          <div class="value">
            {{ $formatMoney(auth.user.payout_data.balance) }}
          </div>
          <NuxtLink
            v-if="auth.user.payout_data.balance >= minWithdraw"
            class="goto-link"
            to="/dashboard/revenue"
          >
            Withdraw earnings
            <ChevronRightIcon class="featured-header-chevron" aria-hidden="true" />
          </NuxtLink>
          <span v-else>${{ minWithdraw }} is the withdraw minimum</span>
        </div>
      </div>
    </section>
    <section class="universal-card more-soon">
      <h2>More coming soon!</h2>
      <p>
        Stay tuned for more metrics and analytics (pretty graphs, anyone? 👀) coming to the creators
        dashboard soon!
      </p>
    </section>
  </div>
</template>
<script setup>
import ChevronRightIcon from '~/assets/images/utils/chevron-right.svg'

useHead({
  title: 'Creator dashboard - Modrinth',
})

const auth = await useAuth()
const app = useNuxtApp()

const [raw] = await Promise.all([
  useBaseFetch(`user/${auth.value.user.id}/payouts`, app.$defaultHeaders()),
])
const user = await useUser()

raw.all_time = Math.floor(raw.all_time * 100) / 100
raw.last_month = Math.floor(raw.last_month * 100) / 100

const payouts = ref(raw)
const minWithdraw = ref(0.26)

const downloadsProjectCount = computed(
  () => user.value.projects.filter((project) => project.downloads > 0).length
)
const followersProjectCount = computed(
  () => user.value.projects.filter((project) => project.followers > 0).length
)
</script>
