<template>
  <div>
    <Modal
      ref="modal_license"
      :header="project.license.name ? project.license.name : 'License'"
    >
      <div class="modal-license">
        <div class="markdown-body" v-html="$xss($md.render(licenseText))" />
      </div>
    </Modal>
    <ModalReport
      v-if="$auth.user"
      ref="modal_project_report"
      :item-id="project.id"
      item-type="project"
    />
    <div
      :class="{
        'normal-page': true,
        'alt-layout': $cosmetics.projectLayout,
      }"
    >
      <article class="normal-page__sidebar">
        <div class="header card">
          <nuxt-link
            :to="
              '/' +
              project.project_type +
              '/' +
              (project.slug ? project.slug : project.id)
            "
          >
            <Avatar :src="project.icon_url" :alt="project.title" size="md" />
          </nuxt-link>
          <nuxt-link
            :to="
              '/' +
              project.project_type +
              '/' +
              (project.slug ? project.slug : project.id)
            "
          >
            <h1 class="title">{{ project.title }}</h1>
          </nuxt-link>
          <div
            v-if="
              project.project_type !== 'resourcepack' &&
              project.project_type !== 'plugin' &&
              project.project_type !== 'shader' &&
              project.project_type !== 'datapack'
            "
          >
            <div
              v-if="
                project.client_side === 'optional' &&
                project.server_side === 'optional'
              "
              class="side-descriptor"
            >
              <InfoIcon aria-hidden="true" />
              Universal {{ projectTypeDisplay }}
            </div>
            <div
              v-else-if="
                (project.client_side === 'optional' ||
                  project.client_side === 'required') &&
                (project.server_side === 'optional' ||
                  project.server_side === 'unsupported')
              "
              class="side-descriptor"
            >
              <InfoIcon aria-hidden="true" />
              Client {{ projectTypeDisplay }}
            </div>
            <div
              v-else-if="
                (project.server_side === 'optional' ||
                  project.server_side === 'required') &&
                (project.client_side === 'optional' ||
                  project.client_side === 'unsupported')
              "
              class="side-descriptor"
            >
              <InfoIcon aria-hidden="true" />
              Server {{ projectTypeDisplay }}
            </div>
          </div>

          <p class="description">
            {{ project.description }}
          </p>
          <Categories
            :categories="project.categories"
            :type="project.actualProjectType"
            class="categories"
          />
          <hr class="card-divider" />
          <div class="primary-stat">
            <DownloadIcon class="primary-stat__icon" aria-hidden="true" />
            <div class="primary-stat__text">
              <span class="primary-stat__counter">
                {{ $formatNumber(project.downloads) }}
              </span>
              download<span v-if="project.downloads !== 1">s</span>
            </div>
          </div>
          <div class="primary-stat">
            <HeartIcon class="primary-stat__icon" aria-hidden="true" />
            <div class="primary-stat__text">
              <span class="primary-stat__counter">
                {{ $formatNumber(project.followers) }}
              </span>
              follower<span v-if="project.followers !== 1">s</span>
            </div>
          </div>
          <div class="dates">
            <div
              v-tooltip="
                $dayjs(project.published).format('MMMM D, YYYY [at] h:mm:ss A')
              "
              class="date"
            >
              <CalendarIcon aria-hidden="true" />
              <span class="label">Created</span>
              <span class="value">{{
                $dayjs(project.published).fromNow()
              }}</span>
            </div>
            <div
              v-tooltip="
                $dayjs(project.updated).format('MMMM D, YYYY [at] h:mm:ss A')
              "
              class="date"
            >
              <UpdateIcon aria-hidden="true" />
              <span class="label">Updated</span>
              <span class="value">{{ $dayjs(project.updated).fromNow() }}</span>
            </div>
          </div>
          <hr class="card-divider" />
          <div class="input-group">
            <template v-if="$auth.user">
              <button
                class="iconified-button"
                @click="$refs.modal_project_report.show()"
              >
                <ReportIcon aria-hidden="true" />
                Report
              </button>
              <button
                v-if="!$user.follows.find((x) => x.id === project.id)"
                class="iconified-button"
                @click="$store.dispatch('user/followProject', project)"
              >
                <HeartIcon aria-hidden="true" />
                Follow
              </button>
              <button
                v-if="$user.follows.find((x) => x.id === project.id)"
                class="iconified-button"
                @click="$store.dispatch('user/unfollowProject', project)"
              >
                <HeartIcon fill="currentColor" aria-hidden="true" />
                Unfollow
              </button>
            </template>
            <template v-else>
              <a
                class="iconified-button"
                :href="authUrl"
                rel="noopener noreferrer nofollow"
              >
                <ReportIcon aria-hidden="true" />
                Report
              </a>
              <a
                class="iconified-button"
                :href="authUrl"
                rel="noopener noreferrer nofollow"
              >
                <HeartIcon aria-hidden="true" />
                Follow
              </a>
            </template>
          </div>
        </div>
        <div
          v-if="
            currentMember &&
            (project.status !== 'approved' ||
              (project.moderator_message &&
                (project.moderator_message.message ||
                  project.moderator_message.body)))
          "
          class="project-status card"
        >
          <h3 class="card-header">Project status</h3>
          <div class="status-info"></div>
          <p>
            Your project is currently:
            <Badge :type="project.status" />
          </p>
          <div class="message">
            <p v-if="project.status === 'rejected'">
              Your project has been rejected by Modrinth's staff. In most cases,
              you can resubmit for review after addressing the staff's message,
              which is below. Do not resubmit until you've addressed the message
              from the moderators!
            </p>
            <p v-if="project.status === 'processing'">
              Your project is currently not viewable by people who are not part
              of your team. Please wait for our moderators to manually review
              your project to see if it abides by our
              <nuxt-link class="text-link" to="/legal/rules"
                >content rules!
              </nuxt-link>
            </p>
            <p v-if="project.status === 'draft'">
              Your project is currently not viewable by people who are not part
              of your team. If you would like to publish your project, click the
              button below to send your project in for review.
            </p>
            <div v-if="project.moderator_message">
              <hr class="card-divider" />
              <div v-if="project.moderator_message.body">
                <h3 class="card-header">
                  Message from the Modrinth moderators:
                </h3>
                <p
                  v-if="project.moderator_message.message"
                  class="mod-message__title"
                >
                  {{ project.moderator_message.message }}
                </p>
                <div
                  v-highlightjs
                  class="markdown-body"
                  v-html="$xss($md.render(project.moderator_message.body))"
                />
              </div>
              <div v-else>
                <h3 class="card-header">
                  Message from the Modrinth moderators:
                </h3>
                <p>{{ project.moderator_message.message }}</p>
              </div>
              <hr class="card-divider" />
            </div>
          </div>
          <div class="buttons status-buttons">
            <button
              v-if="
                project.status === 'rejected' ||
                project.status === 'unlisted' ||
                project.status === 'abandoned'
              "
              class="iconified-button brand-button"
              @click="submitForReview"
            >
              <CheckIcon />
              Resubmit for review
            </button>
            <button
              v-if="project.status === 'draft'"
              class="iconified-button brand-button"
              @click="submitForReview"
            >
              <CheckIcon />
              Submit for review
            </button>
            <button
              v-if="project.status === 'approved'"
              class="iconified-button"
              @click="clearMessage"
            >
              <ClearIcon />
              Clear message
            </button>
          </div>
          <div v-if="showKnownErrors" class="known-errors">
            <ul>
              <li v-if="project.body === ''">
                Your project must have a body to submit for review.
              </li>
              <li v-if="project.versions.length < 1">
                Your project must have at least one version to submit for
                review.
              </li>
              <li
                v-if="
                  project.client_side === 'unknown' ||
                  project.server_side === 'unknown'
                "
              >
                Your project must have the supported environments selected.
              </li>
            </ul>
          </div>
        </div>
      </article>
      <div class="card normal-page__info">
        <template
          v-if="
            project.issues_url ||
            project.source_url ||
            project.wiki_url ||
            project.discord_url ||
            project.donation_urls.length > 0
          "
        >
          <h3 class="card-header">External resources</h3>
          <div class="links">
            <a
              v-if="project.issues_url"
              :href="project.issues_url"
              class="title"
              :target="$external()"
              rel="noopener noreferrer nofollow ugc"
            >
              <IssuesIcon aria-hidden="true" />
              <span>Issues</span>
            </a>
            <a
              v-if="project.source_url"
              :href="project.source_url"
              class="title"
              :target="$external()"
              rel="noopener noreferrer nofollow ugc"
            >
              <CodeIcon aria-hidden="true" />
              <span>Source</span>
            </a>
            <a
              v-if="project.wiki_url"
              :href="project.wiki_url"
              class="title"
              :target="$external()"
              rel="noopener noreferrer nofollow ugc"
            >
              <WikiIcon aria-hidden="true" />
              <span>Wiki</span>
            </a>
            <a
              v-if="project.discord_url"
              :href="project.discord_url"
              :target="$external()"
              rel="noopener noreferrer nofollow ugc"
            >
              <DiscordIcon class="shrink" aria-hidden="true" />
              <span>Discord</span>
            </a>
            <a
              v-for="(donation, index) in project.donation_urls"
              :key="index"
              :href="donation.url"
              :target="$external()"
              rel="noopener noreferrer nofollow ugc"
            >
              <BuyMeACoffeeLogo
                v-if="donation.id === 'bmac'"
                aria-hidden="true"
              />
              <PatreonIcon
                v-else-if="donation.id === 'patreon'"
                aria-hidden="true"
              />
              <KoFiIcon
                v-else-if="donation.id === 'ko-fi'"
                aria-hidden="true"
              />
              <PayPalIcon
                v-else-if="donation.id === 'paypal'"
                aria-hidden="true"
              />
              <OpenCollectiveIcon
                v-else-if="donation.id === 'open-collective'"
                aria-hidden="true"
              />
              <HeartIcon v-else-if="donation.id === 'github'" />
              <UnknownIcon v-else />
              <span v-if="donation.id === 'bmac'">Buy Me a Coffee</span>
              <span v-else-if="donation.id === 'patreon'">Patreon</span>
              <span v-else-if="donation.id === 'paypal'">PayPal</span>
              <span v-else-if="donation.id === 'ko-fi'">Ko-fi</span>
              <span v-else-if="donation.id === 'github'">GitHub Sponsors</span>
              <span v-else>Donate</span>
            </a>
          </div>
          <hr class="card-divider" />
        </template>
        <template v-if="featuredVersions.length > 0">
          <div class="featured-header">
            <h3 class="card-header">Featured versions</h3>
            <nuxt-link
              v-if="project.versions.length > 0 || currentMember"
              :to="`/${project.project_type}/${
                project.slug ? project.slug : project.id
              }/versions`"
              class="goto-link"
            >
              See all
              <ChevronRightIcon
                class="featured-header-chevron"
                aria-hidden="true"
              />
            </nuxt-link>
          </div>
          <div
            v-for="version in featuredVersions"
            :key="version.id"
            class="featured-version button-transparent"
            @click="
              $router.push(
                `/${project.project_type}/${
                  project.slug ? project.slug : project.id
                }/version/${encodeURI(version.displayUrlEnding)}`
              )
            "
          >
            <a
              v-tooltip="
                findPrimary(version).filename +
                ' (' +
                $formatBytes(findPrimary(version).size) +
                ')'
              "
              :href="findPrimary(version).url"
              class="download download-button square-button brand-button"
              :title="`Download ${version.name}`"
              @click.stop="(event) => event.stopPropagation()"
            >
              <DownloadIcon aria-hidden="true" />
            </a>
            <div class="info">
              <nuxt-link
                :to="`/${project.project_type}/${
                  project.slug ? project.slug : project.id
                }/version/${encodeURI(version.displayUrlEnding)}`"
                class="top"
              >
                {{ version.name }}
              </nuxt-link>
              <div
                v-if="version.game_versions.length > 0"
                class="game-version item"
              >
                {{ version.loaders.map((x) => $formatCategory(x)).join(', ') }}
                {{ $formatVersion(version.game_versions) }}
              </div>
              <Badge
                v-if="version.version_type === 'release'"
                type="release"
                color="green"
              />
              <Badge
                v-else-if="version.version_type === 'beta'"
                type="beta"
                color="orange"
              />
              <Badge
                v-else-if="version.version_type === 'alpha'"
                type="alpha"
                color="red"
              />
            </div>
          </div>
          <hr class="card-divider" />
        </template>
        <h3 class="card-header">Project members</h3>
        <div
          v-for="member in members"
          :key="member.user.id"
          class="team-member columns button-transparent"
          @click="$router.push('/user/' + member.user.username)"
        >
          <Avatar
            :src="member.avatar_url"
            :alt="member.username"
            size="sm"
            circle
          />

          <div class="member-info">
            <nuxt-link :to="'/user/' + member.user.username" class="name">
              <p>{{ member.name }}</p>
            </nuxt-link>
            <p class="role">{{ member.role }}</p>
          </div>
        </div>
        <hr class="card-divider" />
        <h3 class="card-header">Technical information</h3>
        <div class="infos">
          <div class="info">
            <div class="key">License</div>
            <div class="value lowercase">
              <a
                v-if="project.license.url"
                class="text-link"
                :href="project.license.url"
              >
                {{ licenseIdDisplay }}
              </a>
              <a
                v-else-if="
                  project.license.id === 'LicenseRef-All-Rights-Reserved' ||
                  !project.license.id.includes('LicenseRef')
                "
                class="text-link"
                @click="getLicenseData()"
              >
                {{ licenseIdDisplay }}
              </a>
              <span v-else>{{ licenseIdDisplay }}</span>
            </div>
          </div>
          <div
            v-if="
              project.project_type !== 'resourcepack' &&
              project.project_type !== 'plugin' &&
              project.project_type !== 'shader' &&
              project.project_type !== 'datapack'
            "
            class="info"
          >
            <div class="key">Client side</div>
            <div class="value">
              {{ project.client_side }}
            </div>
          </div>
          <div
            v-if="
              project.project_type !== 'resourcepack' &&
              project.project_type !== 'plugin' &&
              project.project_type !== 'shader' &&
              project.project_type !== 'datapack'
            "
            class="info"
          >
            <div class="key">Server side</div>
            <div class="value">
              {{ project.server_side }}
            </div>
          </div>
          <div class="info">
            <div class="key">Project ID</div>
            <div class="value lowercase">
              <CopyCode :text="project.id" />
            </div>
          </div>
        </div>
      </div>
      <section class="normal-page__content">
        <div
          v-if="project.status === 'unlisted'"
          class="card warning"
          aria-label="Warning"
        >
          {{ project.title }} is not viewable in search — either because the
          author has marked it as such or because it has been found to be in
          violation of one of
          <nuxt-link to="/legal/rules">Modrinth's content rules</nuxt-link>.
          Modrinth makes no guarantees as to whether {{ project.title }} is safe
          for use in a multiplayer context.
        </div>
        <div
          v-if="project.status === 'archived'"
          class="card warning"
          aria-label="Warning"
        >
          {{ project.title }} has been archived by the project author.
          {{ project.title }} will not receive any further updates unless the
          author decides to unarchive the project.
        </div>
        <div
          v-if="project.status === 'abandoned'"
          class="card warning"
          aria-label="Warning"
        >
          {{ project.title }} has been marked as abandoned by Modrinth's
          moderators. {{ project.title }} will not receive any further updates
          unless the author decides to return.
        </div>
        <div
          v-if="project.project_type === 'modpack'"
          class="card warning"
          aria-label="Warning"
        >
          To install {{ project.title }}, visit
          <a
            href="https://docs.modrinth.com/docs/modpacks/playing_modpacks/"
            :target="$external()"
            >our documentation</a
          >
          which provides instructions on using
          <a
            href="https://atlauncher.com/about"
            :target="$external()"
            rel="noopener noreferrer nofollow"
          >
            ATLauncher</a
          >,
          <a
            href="https://multimc.org/"
            :target="$external()"
            rel="noopener noreferrer nofollow"
            >MultiMC</a
          >, and
          <a
            href="https://prismlauncher.org"
            :target="$external()"
            rel="noopener noreferrer nofollow"
          >
            Prism Launcher</a
          >.
        </div>
        <Advertisement
          v-if="project.status === 'approved' || project.status === 'unlisted'"
          type="banner"
          small-screen="square"
        />
        <NavRow
          :links="[
            {
              label: 'Description',
              href: `/${project.project_type}/${
                project.slug ? project.slug : project.id
              }`,
            },
            {
              label: 'Gallery',
              href: `/${project.project_type}/${
                project.slug ? project.slug : project.id
              }/gallery`,
              shown: project.gallery.length > 0 || !!currentMember,
            },
            {
              label: 'Changelog',
              href: `/${project.project_type}/${
                project.slug ? project.slug : project.id
              }/changelog`,
              shown: project.versions.length > 0,
            },
            {
              label: 'Versions',
              href: `/${project.project_type}/${
                project.slug ? project.slug : project.id
              }/versions`,
              shown: project.versions.length > 0 || !!currentMember,
            },
            {
              label: 'Settings',
              href: `/${project.project_type}/${
                project.slug ? project.slug : project.id
              }/settings`,
              shown: !!currentMember,
            },
          ]"
          class="card"
        />
        <NuxtChild
          :project.sync="project"
          :versions.sync="versions"
          :featured-versions.sync="featuredVersions"
          :members.sync="members"
          :current-member="currentMember"
          :all-members.sync="allMembers"
          :dependencies.sync="dependencies"
        />
      </section>
    </div>
  </div>
</template>

<script>
import CalendarIcon from '~/assets/images/utils/calendar.svg?inline'
import CheckIcon from '~/assets/images/utils/check.svg?inline'
import ClearIcon from '~/assets/images/utils/clear.svg?inline'
import DownloadIcon from '~/assets/images/utils/download.svg?inline'
import UpdateIcon from '~/assets/images/utils/updated.svg?inline'
import CodeIcon from '~/assets/images/sidebar/mod.svg?inline'
import ReportIcon from '~/assets/images/utils/report.svg?inline'
import HeartIcon from '~/assets/images/utils/heart.svg?inline'
import InfoIcon from '~/assets/images/utils/info.svg?inline'
import IssuesIcon from '~/assets/images/utils/issues.svg?inline'
import WikiIcon from '~/assets/images/utils/wiki.svg?inline'
import DiscordIcon from '~/assets/images/external/discord.svg?inline'
import BuyMeACoffeeLogo from '~/assets/images/external/bmac.svg?inline'
import PatreonIcon from '~/assets/images/external/patreon.svg?inline'
import KoFiIcon from '~/assets/images/external/kofi.svg?inline'
import PayPalIcon from '~/assets/images/external/paypal.svg?inline'
import OpenCollectiveIcon from '~/assets/images/external/opencollective.svg?inline'
import UnknownIcon from '~/assets/images/utils/unknown-donation.svg?inline'
import ChevronRightIcon from '~/assets/images/utils/chevron-right.svg?inline'
import Advertisement from '~/components/ads/Advertisement'
import Badge from '~/components/ui/Badge'
import Categories from '~/components/ui/search/Categories'
import Modal from '~/components/ui/Modal'
import ModalReport from '~/components/ui/ModalReport'
import NavRow from '~/components/ui/NavRow'
import CopyCode from '~/components/ui/CopyCode'
import Avatar from '~/components/ui/Avatar'

export default {
  components: {
    Avatar,
    CopyCode,
    NavRow,
    Badge,
    Advertisement,
    Modal,
    ModalReport,
    IssuesIcon,
    DownloadIcon,
    CalendarIcon,
    CheckIcon,
    ClearIcon,
    UpdateIcon,
    CodeIcon,
    ReportIcon,
    HeartIcon,
    InfoIcon,
    WikiIcon,
    DiscordIcon,
    BuyMeACoffeeLogo,
    PayPalIcon,
    OpenCollectiveIcon,
    UnknownIcon,
    Categories,
    PatreonIcon,
    KoFiIcon,
    ChevronRightIcon,
  },
  async asyncData(data) {
    try {
      if (
        !data.params.id ||
        !(
          data.$tag.projectTypes.find((x) => x.id === data.params.type) ||
          data.params.type === 'project'
        )
      ) {
        data.error({
          statusCode: 404,
          message: 'The page could not be found',
        })

        return
      }

      const [project, members, dependencies, versions, featuredVersions] = (
        await Promise.all([
          data.$axios.get(`project/${data.params.id}`, data.$defaultHeaders()),
          data.$axios.get(
            `project/${data.params.id}/members`,
            data.$defaultHeaders()
          ),
          data.$axios.get(
            `project/${data.params.id}/dependencies`,
            data.$defaultHeaders()
          ),
          data.$axios.get(
            `project/${data.params.id}/version`,
            data.$defaultHeaders()
          ),
          data.$axios.get(
            `project/${data.params.id}/version?featured=true`,
            data.$defaultHeaders()
          ),
        ])
      ).map((it) => it.data)

      const projectLoaders = {}

      for (const version of versions) {
        for (const loader of version.loaders) {
          projectLoaders[loader] = true
        }
      }

      project.actualProjectType = JSON.parse(
        JSON.stringify(project.project_type)
      )

      project.project_type = data.params.overrideProjectType
        ? data.params.overrideProjectType
        : data.$getProjectTypeForUrl(
            project.project_type,
            Object.keys(projectLoaders)
          )

      if (
        project.project_type !== data.params.type ||
        data.params.id !== project.slug
      ) {
        let route = data.route.fullPath.split('/')
        route.splice(0, 3)
        route = route.filter((x) => x)

        data.redirect(
          301,
          `/${project.project_type}/${project.slug}${
            route.length > 0 ? `/${route.join('/')}` : ''
          }`
        )

        return
      }

      members.forEach((it, index) => {
        members[index].avatar_url = it.user.avatar_url
        members[index].name = it.user.username
      })

      let currentMember = data.$auth.user
        ? members.find((x) => x.user.id === data.$auth.user.id)
        : null

      if (
        !currentMember &&
        data.$auth.user &&
        (data.$auth.user.role === 'admin' ||
          data.$auth.user.role === 'moderator')
      ) {
        currentMember = {
          team_id: project.team_id,
          user: data.$auth.user,
          role: data.$auth.role,
          permissions: data.$auth.user.role === 'admin' ? 1023 : 12,
          accepted: true,
          payouts_split: 0,
          avatar_url: data.$auth.user.avatar_url,
          name: data.$auth.user.username,
        }
      }

      if (project.body_url && !project.body) {
        project.body = (await data.$axios.get(project.body_url)).data
      }

      const loaders = []

      versions.forEach((version) => {
        version.loaders.forEach((loader) => {
          if (!loaders.includes(loader)) {
            loaders.push(loader)
          }
        })
      })

      return {
        project,
        versions,
        featuredVersions,
        members: members.filter((x) => x.accepted),
        allMembers: members,
        currentMember,
        dependencies,
        loaders,
      }
    } catch {
      data.error({
        statusCode: 404,
        message: 'Project not found',
      })
    }
  },
  data() {
    return {
      showKnownErrors: false,
      licenseText: '',
    }
  },
  fetch() {
    this.versions = this.$computeVersions(this.versions)
    this.featuredVersions = this.$computeVersions(this.featuredVersions)
  },
  head() {
    const title = `${this.project.title} - Minecraft ${this.$formatProjectType(
      this.projectTypeDisplay
    )}`

    return {
      title,
      meta: [
        {
          hid: 'og:title',
          name: 'og:title',
          content: title,
        },
        {
          hid: 'apple-mobile-web-app-title',
          name: 'apple-mobile-web-app-title',
          content: title,
        },
        {
          hid: 'og:description',
          name: 'og:description',
          content: this.project.description,
        },
        {
          hid: 'description',
          name: 'description',
          content: `${
            this.project.description
          } - Download the Minecraft ${this.$formatProjectType(
            this.projectTypeDisplay
          )} ${this.project.title} by ${
            this.members.find((x) => x.role === 'Owner').user.username
          } on Modrinth`,
        },
        {
          hid: 'og:image',
          name: 'og:image',
          content: this.project.icon_url
            ? this.project.icon_url
            : 'https://cdn.modrinth.com/placeholder.png',
        },
        {
          hid: 'robots',
          name: 'robots',
          content:
            this.project.status === 'approved' ||
            this.project.status === 'archived'
              ? 'all'
              : 'noindex',
        },
      ],
    }
  },
  computed: {
    authUrl() {
      return `${process.env.authURLBase}auth/init?url=${process.env.domain}${this.$route.path}`
    },
    projectTypeDisplay() {
      return this.$getProjectTypeForDisplay(
        this.project.project_type,
        this.loaders
      )
    },
    licenseIdDisplay() {
      const id = this.project.license.id

      if (id === 'LicenseRef-All-Rights-Reserved') {
        return 'ARR'
      } else if (id.includes('LicenseRef')) {
        return id.replaceAll('LicenseRef-', '').replaceAll('-', ' ')
      } else {
        return id
      }
    },
  },
  methods: {
    async resetProject() {
      const project = (
        await this.$axios.get(
          `project/${this.project.id}`,
          this.$defaultHeaders()
        )
      ).data

      const projectLoaders = {}

      for (const version of this.versions) {
        for (const loader of version.loaders) {
          projectLoaders[loader] = true
        }
      }

      project.actualProjectType = JSON.parse(
        JSON.stringify(project.project_type)
      )

      project.project_type = this.$getProjectTypeForUrl(
        project.project_type,
        Object.keys(projectLoaders)
      )

      this.project = project
    },
    findPrimary(version) {
      let file = version.files.find((x) => x.primary)

      if (!file) {
        file = version.files[0]
      }

      if (!file) {
        file = { url: `/project/${this.project.id}/version/${version.id}` }
      }

      return file
    },
    async clearMessage() {
      this.$nuxt.$loading.start()

      try {
        await this.$axios.patch(
          `project/${this.project.id}`,
          {
            moderation_message: null,
            moderation_message_body: null,
          },
          this.$defaultHeaders()
        )

        this.project.moderator_message = null
      } catch (err) {
        this.$notify({
          group: 'main',
          title: 'An error occurred',
          text: err.response.data.description,
          type: 'error',
        })
      }

      this.$nuxt.$loading.finish()
    },
    async submitForReview() {
      if (
        this.project.body === '' ||
        this.versions.length < 1 ||
        this.project.client_side === 'unknown' ||
        this.project.server_side === 'unknown'
      ) {
        this.showKnownErrors = true
      } else {
        this.$nuxt.$loading.start()

        try {
          await this.$axios.patch(
            `project/${this.project.id}`,
            {
              status: 'processing',
            },
            this.$defaultHeaders()
          )

          this.project.status = 'processing'
        } catch (err) {
          this.$notify({
            group: 'main',
            title: 'An error occurred',
            text: err.response.data.description,
            type: 'error',
          })
        }

        this.$nuxt.$loading.finish()
      }
    },
    async getLicenseData() {
      try {
        const text = await this.$axios.get(
          `tag/license/${this.project.license.id}`
        )
        this.licenseText = text.data.body
      } catch {
        this.licenseText = 'License text could not be retrieved.'
      }

      this.$refs.modal_license.show()
    },
  },
}
</script>
<style lang="scss" scoped>
.header {
  grid-area: header;
  .title {
    overflow-wrap: break-word;
    margin: 0.25rem 0;
    color: var(--color-text-dark);
    font-size: var(--font-size-xl);
  }

  .side-descriptor {
    display: flex;
    align-items: center;
    color: var(--color-text-dark);
    font-weight: bold;
    font-size: var(--font-size-sm);
    margin-bottom: 0.5rem;

    svg {
      height: 1.25rem;
      margin-right: 0.125rem;
    }
  }

  .description {
    line-height: 1.3;
    overflow-wrap: break-word;

    margin-top: var(--spacing-card-sm);
    margin-bottom: 0.5rem;
    font-size: var(--font-size-nm);
  }

  .categories {
    margin: 0.25rem 0;
    color: var(--color-text-secondary);
    font-size: var(--font-size-nm);
  }

  .dates {
    margin: 0.75rem 0;

    .date {
      color: var(--color-text-secondary);
      font-size: var(--font-size-nm);
      display: flex;
      align-items: center;
      margin-bottom: 0.25rem;
      cursor: default;

      .label {
        margin-right: 0.25rem;
      }

      svg {
        height: 1rem;
        margin-right: 0.25rem;
      }
    }
  }
}

.project-info {
  height: auto;
  overflow: hidden;
}

.card-header {
  font-weight: bold;
  color: var(--color-heading);
  margin-bottom: 0.3rem;
  width: fit-content;
}

.featured-header {
  display: flex;
  justify-content: space-between;
  align-items: baseline;

  .card-header {
    height: 23px;
  }

  .goto-link {
    margin-bottom: 0.3rem;
  }
}

.featured-version {
  display: flex;
  flex-direction: row;
  padding: 0.5rem;

  .download {
    height: 2.5rem;
    width: 2.5rem;
    margin-right: 0.75rem;

    svg {
      width: 1.5rem;
      height: 1.5rem;
    }
  }

  .info {
    display: flex;
    flex-direction: column;

    .top {
      font-weight: bold;
      word-wrap: break-word;
      overflow-wrap: anywhere;
    }
  }
}

.links {
  a {
    display: inline-flex;
    align-items: center;
    border-radius: 1rem;

    svg,
    img {
      height: 1rem;
      width: 1rem;
    }

    span {
      margin-left: 0.25rem;
      text-decoration: underline;
      line-height: 2rem;
    }

    &:focus-visible,
    &:hover {
      svg,
      img,
      span {
        color: var(--color-heading);
      }
    }

    &:active {
      svg,
      img,
      span {
        color: var(--color-text-dark);
      }
    }

    &:not(:last-child)::after {
      content: '•';
      margin: 0 0.25rem;
    }
  }
}

.team-member {
  align-items: center;
  padding: 0.25rem 0.5rem;

  .member-info {
    overflow: hidden;
    margin: auto 0 auto 0.75rem;

    .name {
      font-weight: bold;
    }

    p {
      font-size: var(--font-size-sm);
      margin: 0.2rem 0;
    }
  }
}

.infos {
  .info {
    display: flex;
    margin: 0.5rem 0;

    .key {
      font-weight: bold;
      color: var(--color-text-secondary);
      width: 40%;
    }

    .value {
      width: 50%;

      &::first-letter {
        text-transform: capitalize;
      }

      &.lowercase {
        &::first-letter {
          text-transform: none;
        }
      }
    }

    .uppercase {
      text-transform: uppercase;
    }
  }
}

@media screen and (max-width: 550px) {
  .title a {
    display: none;
  }
}

@media screen and (max-width: 800px) {
  .project-navigation {
    display: block;
    overflow-x: auto;
    overflow-wrap: break-word;
    overflow-y: hidden;
  }
}

@media screen and (min-width: 1024px) {
  .content {
    max-width: calc(1280px - 21rem);
  }
}

.status-buttons {
  margin-top: var(--spacing-card-sm);
}

.mod-message__title {
  font-weight: bold;
  margin-bottom: var(--spacing-card-xs);
  font-size: 1.125rem;
}

.modal-license {
  padding: var(--spacing-card-bg);
}
</style>
