<template>
  <div>
    <section class="universal-card">
      <h2>External links</h2>
      <div class="adjacent-input">
        <label
          id="project-issue-tracker"
          title="A place for users to report bugs, issues, and concerns about your project."
        >
          <span class="label__title">Issue tracker</span>
          <span class="label__description">
            A place for users to report bugs, issues, and concerns about your
            project.
          </span>
        </label>
        <input
          id="project-issue-tracker"
          v-model="issuesUrl"
          type="url"
          placeholder="Enter a valid URL"
          maxlength="2048"
          :disabled="!hasPermission"
        />
      </div>
      <div class="adjacent-input">
        <label
          id="project-source-code"
          title="A page/repository containing the source code for your project"
        >
          <span class="label__title">Source code</span>
          <span class="label__description">
            A page/repository containing the source code for your project
          </span>
        </label>
        <input
          id="project-source-code"
          v-model="sourceUrl"
          type="url"
          maxlength="2048"
          placeholder="Enter a valid URL"
          :disabled="!hasPermission"
        />
      </div>
      <div class="adjacent-input">
        <label
          id="project-wiki-page"
          title="A page containing information, documentation, and help for the project."
        >
          <span class="label__title">Wiki page</span>
          <span class="label__description">
            A page containing information, documentation, and help for the
            project.
          </span>
        </label>
        <input
          id="project-wiki-page"
          v-model="wikiUrl"
          type="url"
          maxlength="2048"
          placeholder="Enter a valid URL"
          :disabled="!hasPermission"
        />
      </div>
      <div class="adjacent-input">
        <label
          id="project-discord-invite"
          title="An invitation link to your Discord server."
        >
          <span class="label__title">Discord invite</span>
          <span class="label__description">
            An invitation link to your Discord server.
          </span>
        </label>
        <input
          id="project-discord-invite"
          v-model="discordUrl"
          type="url"
          maxlength="2048"
          placeholder="Enter a valid URL"
          :disabled="!hasPermission"
        />
      </div>
      <span class="label">
        <span class="label__title">Donation links</span>
        <span class="label__description">
          Add donation links for users to support you directly.
        </span>
      </span>

      <div
        v-for="(donationLink, index) in donationLinks"
        :key="`donation-link-${index}`"
        class="input-group donation-link-group"
      >
        <Multiselect
          v-model="donationLink.platform"
          placeholder="Select platform"
          :options="$tag.donationPlatforms.map((x) => x.name)"
          :searchable="false"
          :close-on-select="true"
          :show-labels="false"
          :disabled="!hasPermission"
          @input="updateDonationLinks"
        />
        <input
          v-model="donationLink.url"
          type="url"
          maxlength="2048"
          placeholder="Enter a valid URL"
          :disabled="!hasPermission"
          @input="updateDonationLinks"
        />
      </div>
      <div class="button-group">
        <button
          type="button"
          class="iconified-button brand-button"
          :disabled="!hasChanges"
          @click="saveChanges()"
        >
          <SaveIcon />
          Save changes
        </button>
      </div>
    </section>
  </div>
</template>

<script>
import Multiselect from 'vue-multiselect'
import SaveIcon from '~/assets/images/utils/save.svg?inline'

export default {
  components: {
    Multiselect,
    SaveIcon,
  },
  props: {
    project: {
      type: Object,
      default() {
        return {}
      },
    },
    currentMember: {
      type: Object,
      default() {
        return null
      },
    },
    patchProject: {
      type: Function,
      default() {
        return () => {
          this.$notify({
            group: 'main',
            title: 'An error occurred',
            text: 'Patch project function not found',
            type: 'error',
          })
        }
      },
    },
  },
  data() {
    return {
      issuesUrl: '',
      sourceUrl: '',
      wikiUrl: '',
      discordUrl: '',

      donationLinks: [],
    }
  },
  fetch() {
    this.issuesUrl = this.project.issues_url
    this.sourceUrl = this.project.source_url
    this.wikiUrl = this.project.wiki_url
    this.discordUrl = this.project.discord_url

    this.resetDonationLinks()
  },
  computed: {
    hasPermission() {
      const EDIT_DETAILS = 1 << 2
      return (this.currentMember.permissions & EDIT_DETAILS) === EDIT_DETAILS
    },
    patchData() {
      const data = {}

      if (this.checkDifference(this.issuesUrl, this.project.issues_url)) {
        data.issues_url = this.issuesUrl
      }
      if (this.checkDifference(this.sourceUrl, this.project.source_url)) {
        data.source_url = this.sourceUrl
      }
      if (this.checkDifference(this.wikiUrl, this.project.wiki_url)) {
        data.wiki_url = this.wikiUrl
      }
      if (this.checkDifference(this.discordUrl, this.project.discord_url)) {
        data.discord_url = this.discordUrl
      }

      const donationLinks = this.donationLinks.filter(
        (link) => link.url && link.platform
      )
      donationLinks.forEach((link) => {
        link.id = this.$tag.donationPlatforms.find(
          (platform) => platform.name === link.platform
        ).short
      })
      if (
        donationLinks !== this.project.donation_urls &&
        !(
          this.project.donation_urls &&
          this.project.donation_urls.length === 0 &&
          donationLinks.length === 0
        )
      ) {
        data.donation_urls = donationLinks
      }

      return data
    },
    hasChanges() {
      return Object.keys(this.patchData).length > 0
    },
  },
  methods: {
    async saveChanges() {
      if (this.patchData && (await this.patchProject(this.patchData))) {
        this.resetDonationLinks()
      }
    },
    updateDonationLinks() {
      this.donationLinks.forEach((link) => {
        if (link.url) {
          const url = link.url.toLowerCase()
          if (url.includes('patreon.com')) {
            link.platform = 'Patreon'
          } else if (url.includes('ko-fi.com')) {
            link.platform = 'Ko-fi'
          } else if (url.includes('paypal.com')) {
            link.platform = 'Paypal'
          } else if (url.includes('buymeacoffee.com')) {
            link.platform = 'Buy Me a Coffee'
          } else if (url.includes('github.com/sponsors')) {
            link.platform = 'GitHub Sponsors'
          }
        }
      })
      if (!this.donationLinks.find((link) => !(link.url && link.platform))) {
        this.donationLinks.push({
          id: null,
          platform: null,
          url: null,
        })
      }
    },
    resetDonationLinks() {
      this.donationLinks = JSON.parse(
        JSON.stringify(this.project.donation_urls)
      )
      this.donationLinks.push({
        id: null,
        platform: null,
        url: null,
      })
    },
    checkDifference(a, b) {
      if (!a && !b) {
        return false
      }
      return a !== b
    },
  },
}
</script>
<style lang="scss" scoped>
.donation-link-group {
  input {
    flex-grow: 2;
    max-width: 26rem;
  }
}
</style>
