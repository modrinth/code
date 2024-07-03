<template>
  <div class="markdown-body">
    <h1>Rewards Program Information</h1>
    <p><em>Last modified: May 13, 2024</em></p>
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
      <a href="https://adrinth.com">Adrinth</a>. We then distribute this ad revenue to creators.
    </p>
    <p>
      The advertising revenue of the entire website and app is split 90% to creators and 10% to
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
      <li>Modrinth: $10 (10% of $100, the site's earnings for the day)</li>
      <li>User A: $58.69 ($90 * (10 + 30 + 100 + 10)/230)</li>
      <li>User B: $12.52 (0.4 * $90 * (50 + 20 + 10 + 0)/230)</li>
      <li>User C: $18.78 (0.6 * $90 * (50 + 20 + 10 + 0)/230)</li>
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
      Right now, you can use PayPal or Venmo to withdraw money from your Modrinth account. We are
      working on more methods to withdraw money from your account. There are fees to withdraw money
      from your Modrinth account—see the revenue page in your dashboard for more information.
    </p>
    <h3>Modrinth used to give 100% of project page revenue to creators. What changed?</h3>
    <p>
      While this is true, our new system (as of 08/05/23) gives more of the site's revenue to
      creators, so creators will earn more. In the old system, we would earn revenue through
      advertisements in search and user profile pages. This amounted on average each month to about
      15-20% of the site's total advertising revenue (so a 80-85% split to creators). The new system
      gives creators more revenue and a more favorable split towards creators (90%).
    </p>
    <h3></h3>
  </div>
</template>

<script setup>
const description =
  'Information about the Rewards Program of Modrinth, an open source modding platform focused on Minecraft.'

useSeoMeta({
  title: 'Rewards Program Information - Modrinth',
  description,
  ogTitle: 'Rewards Program Information',
  ogDescription: description,
})
</script>
