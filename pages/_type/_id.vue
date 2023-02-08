<template>
  <div v-if="isSettings" class="normal-page">
    <div class="normal-page__sidebar">
      <aside class="universal-card">
        <div class="settings-header">
          <Avatar
            :src="project.icon_url"
            :alt="project.title"
            size="sm"
            class="settings-header__icon"
          />
          <div class="settings-header__text">
            <h1 class="wrap-as-needed">{{ project.title }}</h1>
            <Badge :type="project.status" />
          </div>
        </div>
        <h2>Project settings</h2>
        <NavStack>
          <NavStackItem
            :link="`/${project.project_type}/${project.slug}/settings`"
            label="General"
          >
            <SettingsIcon />
          </NavStackItem>
          <NavStackItem
            :link="`/${project.project_type}/${project.slug}/settings/tags`"
            label="Tags"
          >
            <CategoriesIcon />
          </NavStackItem>
          <NavStackItem
            :link="`/${project.project_type}/${project.slug}/settings/description`"
            label="Description"
          >
            <DescriptionIcon />
          </NavStackItem>
          <NavStackItem
            :link="`/${project.project_type}/${project.slug}/settings/license`"
            label="License"
          >
            <LicenseIcon />
          </NavStackItem>
          <NavStackItem
            :link="`/${project.project_type}/${project.slug}/settings/links`"
            label="Links"
          >
            <LinksIcon />
          </NavStackItem>
          <NavStackItem
            :link="`/${project.project_type}/${project.slug}/settings/members`"
            label="Members"
          >
            <UsersIcon />
          </NavStackItem>
          <h3>Relevant pages</h3>
          <NavStackItem
            :link="`/${project.project_type}/${project.slug}`"
            label="View project"
            chevron
          >
            <EyeIcon />
          </NavStackItem>
          <NavStackItem
            :link="`/${project.project_type}/${project.slug}/gallery`"
            label="Gallery"
            chevron
          >
            <GalleryIcon />
          </NavStackItem>
          <NavStackItem
            :link="`/${project.project_type}/${project.slug}/versions`"
            label="Versions"
            chevron
          >
            <VersionIcon />
          </NavStackItem>
          <NavStackItem link="/dashboard/projects" label="All projects" chevron>
            <SettingsIcon />
          </NavStackItem>
        </NavStack>
      </aside>
    </div>
    <div class="normal-page__content">
      <ProjectPublishingChecklist
        :project="project"
        :versions="versions"
        :current-member="currentMember"
        :is-settings="isSettings"
        :route-name="routeName"
        :set-processing="setProcessing"
        :collapsed="collapsedChecklist"
        :toggle-collapsed="toggleChecklistCollapse"
      />
      <NuxtChild
        :project.sync="project"
        :versions.sync="versions"
        :featured-versions.sync="featuredVersions"
        :members.sync="members"
        :current-member="currentMember"
        :all-members.sync="allMembers"
        :dependencies.sync="dependencies"
        :patch-project="patchProject"
        :patch-icon="patchIcon"
        :update-icon="updateIcon"
      />
    </div>
  </div>
  <div v-else>
    <ModalModeration
      ref="modal_moderation"
      :project="project"
      :status="moderationStatus"
      :on-close="resetProject"
    />
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
      <div class="normal-page__sidebar">
        <div
          class="header project__header base-card padding-0"
          :class="{ 'has-featured-image': featuredGalleryImage }"
        >
          <nuxt-link
            class="project__gallery"
            tabindex="-1"
            :to="
              '/' +
              project.project_type +
              '/' +
              (project.slug ? project.slug : project.id) +
              '/gallery'
            "
          >
            <img
              v-if="featuredGalleryImage"
              :src="featuredGalleryImage.url"
              :alt="
                featuredGalleryImage.description
                  ? featuredGalleryImage.description
                  : featuredGalleryImage.title
              "
            />
          </nuxt-link>
          <div
            class="project__header__content universal-card full-width-inputs"
          >
            <Avatar
              :src="project.icon_url"
              :alt="project.title"
              size="md"
              class="project__icon"
              no-shadow
            />
            <h1 class="title">
              {{ project.title }}
            </h1>
            <Badge
              v-if="$auth.user && currentMember"
              :type="project.status"
              class="status-badge"
            />
            <p class="description">
              {{ project.description }}
            </p>
            <Categories
              :categories="project.categories"
              :type="project.actualProjectType"
              class="categories"
            >
              <EnvironmentIndicator
                :client-side="project.client_side"
                :server-side="project.server_side"
                :type="project.project_type"
              />
            </Categories>
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
                  $dayjs(project.published).format(
                    'MMMM D, YYYY [at] h:mm:ss A'
                  )
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
                <span class="value">{{
                  $dayjs(project.updated).fromNow()
                }}</span>
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
                <a class="iconified-button" :href="authUrl">
                  <ReportIcon aria-hidden="true" />
                  Report
                </a>
                <a class="iconified-button" :href="authUrl">
                  <HeartIcon aria-hidden="true" />
                  Follow
                </a>
              </template>
            </div>
          </div>
        </div>
        <div
          v-if="currentMember && project.moderator_message"
          class="universal-card moderation-card"
        >
          <h3 class="card-header">Message from the moderators:</h3>
          <div v-if="project.moderator_message.body">
            <p
              v-if="project.moderator_message.message"
              class="mod-message__title"
            >
              {{ project.moderator_message.message }}
            </p>
          </div>
          <div
            v-highlightjs
            class="markdown-body"
            v-html="
              $xss(
                $md.render(
                  project.moderator_message.body
                    ? project.moderator_message.body
                    : project.moderator_message.message
                )
              )
            "
          />
          <div class="buttons status-buttons">
            <button
              v-if="$tag.approvedStatuses.includes(project.status)"
              class="iconified-button"
              @click="clearMessage"
            >
              <ClearIcon />
              Clear message
            </button>
          </div>
        </div>
        <div
          v-if="$auth.user && $tag.staffRoles.includes($auth.user.role)"
          class="universal-card moderation-card"
        >
          <h3>Moderation actions</h3>
          <div class="input-stack">
            <button
              v-if="
                !$tag.approvedStatuses.includes(project.status) ||
                project.status === 'processing'
              "
              class="iconified-button brand-button"
              @click="openModerationModal(requestedStatus)"
            >
              <CheckIcon />
              Approve
              {{ requestedStatus !== 'approved' ? `(${requestedStatus})` : '' }}
            </button>
            <button
              v-if="
                $tag.approvedStatuses.includes(project.status) ||
                project.status === 'processing'
              "
              class="iconified-button danger-button"
              @click="openModerationModal('withheld')"
            >
              <EyeIcon />
              Withhold
            </button>
            <button
              v-if="
                $tag.approvedStatuses.includes(project.status) ||
                project.status === 'processing'
              "
              class="iconified-button danger-button"
              @click="openModerationModal('rejected')"
            >
              <CrossIcon />
              Reject
            </button>
            <button class="iconified-button" @click="openModerationModal(null)">
              <EditIcon />
              Edit message
            </button>
            <nuxt-link class="iconified-button" to="/moderation">
              <ModerationIcon />
              Visit moderation queue
            </nuxt-link>
          </div>
        </div>
      </div>
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
        <ProjectPublishingChecklist
          :project="project"
          :versions="versions"
          :current-member="currentMember"
          :is-settings="isSettings"
          :route-name="routeName"
          :set-processing="setProcessing"
          :collapsed="collapsedChecklist"
          :toggle-collapsed="toggleChecklistCollapse"
        />
        <div
          v-if="project.status === 'withheld'"
          class="card warning"
          aria-label="Warning"
        >
          {{ project.title }} is not viewable in search because it has been
          found to be in violation of one of
          <nuxt-link to="/legal/rules">Modrinth's content rules</nuxt-link>.
          Modrinth makes no guarantees as to whether {{ project.title }} is safe
          for use in a multiplayer context.
        </div>
        <div
          v-if="project.status === 'archived'"
          class="card warning"
          aria-label="Warning"
        >
          {{ project.title }} has been archived and will not receive any further
          updates unless the author decides to unarchive the project.
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
          v-if="$tag.approvedStatuses.includes(project.status)"
          type="banner"
          small-screen="square"
        />
        <div class="navigation-card">
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
            ]"
          />
          <div v-if="$auth.user && currentMember" class="input-group">
            <nuxt-link
              :to="`/${project.project_type}/${project.slug}/settings`"
              class="iconified-button"
            >
              <SettingsIcon /> Settings
            </nuxt-link>
          </div>
        </div>
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
import EyeIcon from '~/assets/images/utils/eye.svg?inline'
import Advertisement from '~/components/ads/Advertisement'
import Badge from '~/components/ui/Badge'
import Categories from '~/components/ui/search/Categories'
import EnvironmentIndicator from '~/components/ui/EnvironmentIndicator'
import Modal from '~/components/ui/Modal'
import ModalReport from '~/components/ui/ModalReport'
import ModalModeration from '~/components/ui/ModalModeration'
import NavRow from '~/components/ui/NavRow'
import CopyCode from '~/components/ui/CopyCode'
import Avatar from '~/components/ui/Avatar'
import NavStack from '~/components/ui/NavStack'
import NavStackItem from '~/components/ui/NavStackItem'
import ProjectPublishingChecklist from '~/components/ui/ProjectPublishingChecklist'
import SettingsIcon from '~/assets/images/utils/settings.svg?inline'
import UsersIcon from '~/assets/images/utils/users.svg?inline'
import CategoriesIcon from '~/assets/images/utils/tags.svg?inline'
import DescriptionIcon from '~/assets/images/utils/align-left.svg?inline'
import LinksIcon from '~/assets/images/utils/link.svg?inline'
import LicenseIcon from '~/assets/images/utils/copyright.svg?inline'
import GalleryIcon from '~/assets/images/utils/image.svg?inline'
import VersionIcon from '~/assets/images/utils/version.svg?inline'
import CrossIcon from '~/assets/images/utils/x.svg?inline'
import EditIcon from '~/assets/images/utils/edit.svg?inline'
import ModerationIcon from '~/assets/images/sidebar/admin.svg?inline'

export default {
  components: {
    Avatar,
    CopyCode,
    NavRow,
    Badge,
    Advertisement,
    Modal,
    ModalReport,
    ModalModeration,
    ProjectPublishingChecklist,
    EnvironmentIndicator,
    IssuesIcon,
    DownloadIcon,
    CalendarIcon,
    CheckIcon,
    ClearIcon,
    UpdateIcon,
    CodeIcon,
    ReportIcon,
    HeartIcon,
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
    NavStack,
    NavStackItem,
    SettingsIcon,
    EyeIcon,
    CrossIcon,
    EditIcon,
    ModerationIcon,
    GalleryIcon,
    VersionIcon,
    UsersIcon,
    CategoriesIcon,
    DescriptionIcon,
    LinksIcon,
    LicenseIcon,
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
        data.$tag.staffRoles.includes(data.$auth.user.role)
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
      licenseText: '',
      isSettings: false,
      routeName: '',
      from: '',
      collapsedChecklist: false,
      moderationStatus: null,
    }
  },
  fetch() {
    this.reset()
    this.versions = this.$computeVersions(this.versions)
    this.featuredVersions = this.$computeVersions(this.featuredVersions)

    this.featuredVersions.sort((a, b) => {
      const aLatest = a.game_versions[a.game_versions.length - 1]
      const bLatest = b.game_versions[b.game_versions.length - 1]
      const gameVersions = this.$tag.gameVersions.map((e) => e.version)
      return gameVersions.indexOf(aLatest) - gameVersions.indexOf(bLatest)
    })
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
    featuredGalleryImage() {
      return this.project.gallery.find((img) => img.featured)
    },
    requestedStatus() {
      return this.project.requested_status ?? 'approved'
    },
  },
  watch: {
    '$route.path': {
      async handler() {
        await this.reset()
      },
    },
  },
  methods: {
    reset() {
      // First time going to settings, this will run, but not subsequent times.
      if (!this.isSettings) {
        this.from = this.$nuxt.context.from ? this.$nuxt.context.from.name : ''
      }
      this.routeName = this.$route.name
      this.isSettings = this.routeName.startsWith('type-id-settings')
    },
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
    toggleChecklistCollapse() {
      this.collapsedChecklist = !this.collapsedChecklist
    },
    async setProcessing() {
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
    async patchProject(data, quiet = false) {
      let result = false
      this.$nuxt.$loading.start()

      try {
        await this.$axios.patch(
          `project/${this.project.id}`,
          data,
          this.$defaultHeaders()
        )

        if (this.iconChanged) {
          await this.$axios.patch(
            `project/${this.project.id}/icon?ext=${
              this.icon.type.split('/')[this.icon.type.split('/').length - 1]
            }`,
            this.icon,
            this.$defaultHeaders()
          )
        }

        for (const key in data) {
          this.project[key] = data[key]
        }

        if (data.license_id) {
          this.project.license.id = data.license_id
        }
        if (data.license_url) {
          this.project.license.url = data.license_url
        }

        this.$emit('update:project', this.project)
        result = true
        if (!quiet) {
          this.$notify({
            group: 'main',
            title: 'Project updated',
            text: 'Your project has been updated.',
            type: 'success',
          })
          window.scrollTo({ top: 0, behavior: 'smooth' })
        }
      } catch (err) {
        this.$notify({
          group: 'main',
          title: 'An error occurred',
          text: err.response.data.description,
          type: 'error',
        })
        window.scrollTo({ top: 0, behavior: 'smooth' })
      }

      this.$nuxt.$loading.finish()

      return result
    },
    async patchIcon(icon) {
      let result = false
      this.$nuxt.$loading.start()

      try {
        await this.$axios.patch(
          `project/${this.project.id}/icon?ext=${
            icon.type.split('/')[icon.type.split('/').length - 1]
          }`,
          icon,
          this.$defaultHeaders()
        )
        await this.updateIcon()
        result = true
        this.$notify({
          group: 'main',
          title: 'Project icon updated',
          text: "Your project's icon has been updated.",
          type: 'success',
        })
      } catch (err) {
        this.$notify({
          group: 'main',
          title: 'An error occurred',
          text: err.response.data.description,
          type: 'error',
        })

        window.scrollTo({ top: 0, behavior: 'smooth' })
      }

      this.$nuxt.$loading.finish()
      return result
    },
    async updateIcon() {
      const response = await this.$axios.get(
        `project/${this.project.id}`,
        this.$defaultHeaders()
      )
      this.project.icon_url = response.data.icon_url
    },
    openModerationModal(status) {
      this.moderationStatus = status

      this.$refs.modal_moderation.show()
    },
  },
}
</script>
<style lang="scss" scoped>
.header {
  grid-area: header;
  .title {
    overflow-wrap: break-word;
    margin: var(--spacing-card-xs) 0;
    color: var(--color-text-dark);
    font-size: var(--font-size-xl);
  }

  .status-badge {
    margin-top: var(--spacing-card-sm);
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

.project__header {
  overflow: hidden;
  .project__gallery {
    display: none;
  }
  &.has-featured-image {
    .project__gallery {
      display: inline-block;
      width: 100%;
      height: 10rem;
      background-color: var(--color-button-bg-active);
      img {
        width: 100%;
        height: 10rem;
        object-fit: cover;
      }
    }
    .project__icon {
      margin-top: calc(-3rem - var(--spacing-card-lg) - 4px);
      margin-left: -4px;
      z-index: 1;
      border: 4px solid var(--color-raised-bg);
      border-bottom: none;
    }
  }
  .project__header__content {
    margin: 0;
    background: none;
    border-radius: unset;
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
.settings-header {
  display: flex;
  flex-direction: row;
  gap: var(--spacing-card-sm);
  align-items: center;
  margin-bottom: var(--spacing-card-bg);

  .settings-header__icon {
    flex-shrink: 0;
  }

  .settings-header__text {
    h1 {
      font-size: var(--font-size-md);
      margin-top: 0;
      margin-bottom: var(--spacing-card-sm);
    }
  }
}

.normal-page__sidebar .mod-button {
  margin-top: var(--spacing-card-sm);
}
</style>
