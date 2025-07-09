<template>
  <div class="markdown-body">
    <h1>Rewards Program Information</h1>
    <p><em>Last modified: Feb 20, 2025</em></p>
    <p>
      This page was created for transparency for how the rewards program works on Modrinth. Feel
      free to join our Discord or email
      <a href="mailto:support@modrinth.com">support@modrinth.com</a> if you have any questions!
    </p>
    <p>
      This document is provided for informational purposes only and does not constitute a legal
      agreement. Modrinth makes no representations or warranties as to the accuracy, completeness,
      or reliability of the information contained herein.
    </p>
    <h2>Rewards Distribution</h2>
    <p>
      We collect ad revenue on our website and app through our ad network
      <a href="https://adrinth.com">Adrinth</a>, which is powered by
      <a href="https://aditude.io">Aditude</a>. We then distribute this ad revenue to creators.
    </p>
    <p>
      The advertising revenue of the entire website and app is split 75% to creators and 25% to
      Modrinth.
    </p>
    <p>
      The creator allotment to the pool is decided by how many page views and in-app downloads your
      project receives (user profiles are not used in this calculation). Each page view and in-app
      download counts as a "point". Then, the money is distributed based on each author's point
      earnings daily.
    </p>
    <p>For example, consider this test scenario (all numbers are fake):</p>
    <ul>
      <li>The site earns $100 on a day.</li>
      <li>User A has the projects: NoobCraft and Minesweeper</li>
      <li>NoobCraft receives 10 page views and 30 in-app downloads (40 points)</li>
      <li>Minesweeper receives 100 page views and 10 in-app downloads (110 points)</li>
      <li>
        User B and C both co-own these projects: Bloxcraft and Craftnite. They split their payouts
        40/60.
      </li>
      <li>Bloxcraft receives 50 page views and 20 in-app downloads (70 points)</li>
      <li>Craftnite receives 10 page views and 0 in-app downloads (10 points)</li>
    </ul>
    <p>In this scenario, the earnings for each creator and Modrinth would be as follows:</p>
    <ul>
      <li>Modrinth: $25 (25% of $100, the site's earnings for the day)</li>
      <li>User A: $48.91 ($75 * (10 + 30 + 100 + 10)/230)</li>
      <li>User B: $10.43 (0.4 * $75 * (50 + 20 + 10 + 0)/230)</li>
      <li>User C: $15.65 (0.6 * $75 * (50 + 20 + 10 + 0)/230)</li>
      <li>Note: 230 is the sum of all page views and in-app downloads from above</li>
    </ul>
    <p>
      Page views are counted when a legitimate browser views a project page. In-app downloads when a
      user logged into the launcher downloads a project. Project downloads alongside modpack
      downloads are counted equally. In each category, Modrinth actively removes botted downloads
      and page views at our own discretion. If users are caught botting, they will be permanently
      banned from using Modrinth's services.
    </p>
    <p>
      You can view your page views and project downloads in your
      <a href="https://modrinth.com/dashboard/analytics">analytics dashboard</a>.
    </p>
    <h2>Frequently Asked Questions</h2>
    <p>
      This section covers some common concerns people have about our monetization program. If you
      have more, feel free to join our Discord or contact support.
    </p>
    <h3>Do you have to enroll in the monetization program to get money?</h3>
    <p>
      No. All creators who upload to Modrinth automatically will receive funds as according to the
      above algorithm. However, if you would like to withdraw money from your account, you must
      enroll by adding your payment information.
    </p>
    <h3>What methods can I use withdraw money from my account? Are there any fees?</h3>
    <p>
      Right now, you can use PayPal or Venmo to withdraw money from your Modrinth account. Gift card
      withdrawal is also available. We are working on more methods to withdraw money from your
      account. There are fees to withdraw money from your Modrinth account—see the revenue page in
      your dashboard for more information.
    </p>
    <h3 id="pending">What does "pending" revenue mean in my dashboard?</h3>
    <p>
      Modrinth receives ad revenue from our ad providers on a NET 60 day basis. Due to this, not all
      revenue is immediately available to withdraw. We pay creators as soon as we receive the money
      from our ad providers, which is 60 days after the last day of each month.
    </p>

    <p>
      To understand when revenue becomes available, you can use this calculator to estimate when
      revenue earned on a specific date will be available for withdrawal. Please be advised that all
      dates within this calculator are represented at 00:00 UTC.
    </p>

    <table>
      <thead>
        <tr>
          <th>Timeline</th>
          <th>Date</th>
        </tr>
      </thead>
      <tbody>
        <tr>
          <td>Revenue earned on</td>
          <td>
            <input id="revenue-date-picker" v-model="rawSelectedDate" type="date" />
            <noscript
              >(JavaScript must be enabled for the date picker to function, example date:
              2024-07-15)
            </noscript>
          </td>
        </tr>
        <tr>
          <td>End of the month</td>
          <td>{{ formatDate(endOfMonthDate) }}</td>
        </tr>
        <tr>
          <td>NET 60 policy applied</td>
          <td>+ 60 days</td>
        </tr>
        <tr class="final-result">
          <td>Available for withdrawal</td>
          <td>{{ formatDate(withdrawalDate) }}</td>
        </tr>
      </tbody>
    </table>

    <h3>How do I know Modrinth is being transparent about revenue?</h3>
    <p>
      We aim to be as transparent as possible with creator revenue. All of our code is open source,
      including our
      <a href="https://github.com/modrinth/code/blob/main/apps/labrinth/src/queue/payouts.rs#L598">
        revenue distribution system</a
      >. We also have an
      <a href="https://api.modrinth.com/v3/payout/platform_revenue">API route</a>
      to query the exact daily advertising revenue for the site - so far, creators on Modrinth have
      earned a total of <strong>{{ formatMoney(platformRevenue) }}</strong> in ad revenue.
    </p>
    <table>
      <thead>
        <tr>
          <th>Date</th>
          <th>Revenue</th>
          <th>Creator Revenue (75%)</th>
          <th>Modrinth's Cut (25%)</th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="item in platformRevenueData" :key="item.time">
          <td>{{ formatDate(dayjs.unix(item.time)) }}</td>
          <td>{{ formatMoney(Number(item.revenue) + Number(item.creator_revenue)) }}</td>
          <td>{{ formatMoney(Number(item.creator_revenue)) }}</td>
          <td>{{ formatMoney(Number(item.revenue)) }}</td>
        </tr>
      </tbody>
    </table>
    <small
      >Modrinth's total ad revenue in the previous 5 days, for the entire dataset, use the
      aforementioned
      <a href="https://api.modrinth.com/v3/payout/platform_revenue">API route</a>.</small
    >
  </div>
</template>

<script lang="ts" setup>
import dayjs from "dayjs";
import { computed, ref } from "vue";
import { formatDate, formatMoney } from "@modrinth/utils";

const description =
  "Information about the Rewards Program of Modrinth, an open source modding platform focused on Minecraft.";

useSeoMeta({
  title: "Rewards Program Information - Modrinth",
  description,
  ogTitle: "Rewards Program Information",
  ogDescription: description,
});

const rawSelectedDate = ref(dayjs().format("YYYY-MM-DD"));
const selectedDate = computed(() => dayjs(rawSelectedDate.value));
const endOfMonthDate = computed(() => selectedDate.value.endOf("month"));
const withdrawalDate = computed(() => endOfMonthDate.value.add(60, "days"));

const { data: transparencyInformation } = await useAsyncData("payout/platform_revenue", () =>
  useBaseFetch("payout/platform_revenue", {
    apiVersion: 3,
  }),
);

const platformRevenue = (transparencyInformation.value as any)?.all_time;
const platformRevenueData = (transparencyInformation.value as any)?.data?.slice(0, 5) ?? [];
</script>
