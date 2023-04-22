<template>
  <div v-if="$route.name.startsWith('type-id-settings')" class="normal-page">
    <div class="normal-page__sidebar">
      <aside class="universal-card">
        <Breadcrumbs
          current-title="Settings"
          :link-stack="[
            { href: `/dashboard/projects`, label: 'Projects' },
            {
              href: `/${project.project_type}/${project.slug ? project.slug : project.id}`,
              label: project.title,
              allowTrimming: true,
            },
          ]"
        />
        <div class="settings-header">
          <Avatar
            :src="project.icon_url"
            :alt="project.title"
            size="sm"
            class="settings-header__icon"
          />
          <div class="settings-header__text">
            <h1 class="wrap-as-needed">
              {{ project.title }}
            </h1>
            <Badge :type="project.status" />
          </div>
        </div>
        <h2>Project settings</h2>
        <NavStack>
          <NavStackItem
            :link="`/${project.project_type}/${project.slug ? project.slug : project.id}/settings`"
            label="General"
          >
            <SettingsIcon />
          </NavStackItem>
          <NavStackItem
            :link="`/${project.project_type}/${
              project.slug ? project.slug : project.id
            }/settings/tags`"
            label="Tags"
          >
            <CategoriesIcon />
          </NavStackItem>
          <NavStackItem
            :link="`/${project.project_type}/${
              project.slug ? project.slug : project.id
            }/settings/description`"
            label="Description"
          >
            <DescriptionIcon />
          </NavStackItem>
          <NavStackItem
            :link="`/${project.project_type}/${
              project.slug ? project.slug : project.id
            }/settings/license`"
            label="License"
          >
            <LicenseIcon />
          </NavStackItem>
          <NavStackItem
            :link="`/${project.project_type}/${
              project.slug ? project.slug : project.id
            }/settings/links`"
            label="Links"
          >
            <LinksIcon />
          </NavStackItem>
          <NavStackItem
            :link="`/${project.project_type}/${
              project.slug ? project.slug : project.id
            }/settings/members`"
            label="Members"
          >
            <UsersIcon />
          </NavStackItem>
          <h3>Upload</h3>
          <NavStackItem
            :link="`/${project.project_type}/${project.slug ? project.slug : project.id}/gallery`"
            label="Gallery"
            chevron
          >
            <GalleryIcon />
          </NavStackItem>
          <NavStackItem
            :link="`/${project.project_type}/${project.slug ? project.slug : project.id}/versions`"
            label="Versions"
            chevron
          >
            <VersionIcon />
          </NavStackItem>
        </NavStack>
      </aside>
    </div>
    <div class="normal-page__content">
      <ProjectPublishingChecklist
        v-if="currentMember"
        :project="project"
        :versions="versions"
        :current-member="currentMember"
        :is-settings="$route.name.startsWith('type-id-settings')"
        :route-name="$route.name"
        :set-processing="setProcessing"
        :collapsed="collapsedChecklist"
        :toggle-collapsed="() => (collapsedChecklist = !collapsedChecklist)"
      />
      <NuxtPage
        v-model:project="project"
        v-model:versions="versions"
        v-model:featured-versions="featuredVersions"
        v-model:members="members"
        v-model:all-members="allMembers"
        v-model:dependencies="dependencies"
        :current-member="currentMember"
        :patch-project="patchProject"
        :patch-icon="patchIcon"
        :update-icon="resetProject"
        :route="route"
      />
    </div>
  </div>
  <div v-else>
    <Head>
      <Title> {{ project.title }} - Minecraft {{ projectTypeDisplay }} </Title>
      <Meta name="og:title" :content="`${project.title} - Minecraft ${projectTypeDisplay}`" />
      <Meta
        name="description"
        :content="`${project.description} - Download the Minecraft ${projectTypeDisplay} ${
          project.title
        } by ${members.find((x) => x.role === 'Owner').user.username} on Modrinth`"
      />
      <Meta
        name="apple-mobile-web-app-title"
        :content="`${project.title} - Minecraft ${projectTypeDisplay}`"
      />
      <Meta name="og:description" :content="project.description" />
      <Meta
        name="og:image"
        :content="project.icon_url ? project.icon_url : 'https://cdn.modrinth.com/placeholder.png'"
      />
      <Meta
        name="robots"
        :content="
          project.status === 'approved' || project.status === 'archived' ? 'all' : 'noindex'
        "
      />
    </Head>
    <ModalModeration
      v-if="$auth.user"
      ref="modalModeration"
      :project="project"
      :status="moderationStatus"
      :on-close="resetProject"
    />
    <Modal ref="modalLicense" :header="project.license.name ? project.license.name : 'License'">
      <div class="modal-license">
        <div class="markdown-body" v-html="renderString(licenseText)" />
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
          <div class="project__header__content universal-card full-width-inputs">
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
            <nuxt-link
              class="title-link project-type"
              :to="`/${$getProjectTypeForUrl(project.actualProjectType, project.loaders)}s`"
            >
              <BoxIcon />
              <span>{{
                $formatProjectType(
                  $getProjectTypeForDisplay(project.actualProjectType, project.loaders)
                )
              }}</span>
            </nuxt-link>
            <p class="description">
              {{ project.description }}
            </p>
            <Categories
              :categories="project.categories.concat(project.additional_categories)"
              :type="project.actualProjectType"
              class="categories"
            >
              <Badge
                v-if="$auth.user && currentMember"
                :type="project.status"
                class="status-badge"
              />
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
                v-tooltip="$dayjs(project.published).format('MMMM D, YYYY [at] h:mm:ss A')"
                class="date"
              >
                <CalendarIcon aria-hidden="true" />
                <span class="label">Created</span>
                <span class="value">{{ fromNow(project.published) }}</span>
              </div>
              <div
                v-tooltip="$dayjs(project.updated).format('MMMM D, YYYY [at] h:mm:ss A')"
                class="date"
              >
                <UpdateIcon aria-hidden="true" />
                <span class="label">Updated</span>
                <span class="value">{{ fromNow(project.updated) }}</span>
              </div>
            </div>
            <hr class="card-divider" />
            <div class="input-group">
              <template v-if="$auth.user">
                <button class="iconified-button" @click="$refs.modal_project_report.show()">
                  <ReportIcon aria-hidden="true" />
                  Report
                </button>
                <button
                  v-if="!user.follows.find((x) => x.id === project.id)"
                  class="iconified-button"
                  @click="userFollowProject(project)"
                >
                  <HeartIcon aria-hidden="true" />
                  Follow
                </button>
                <button
                  v-if="user.follows.find((x) => x.id === project.id)"
                  class="iconified-button"
                  @click="userUnfollowProject(project)"
                >
                  <HeartIcon fill="currentColor" aria-hidden="true" />
                  Unfollow
                </button>
              </template>
              <template v-else>
                <a class="iconified-button" :href="getAuthUrl()" rel="noopener nofollow">
                  <ReportIcon aria-hidden="true" />
                  Report
                </a>
                <a class="iconified-button" :href="getAuthUrl()" rel="noopener nofollow">
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
          <h2 class="card-header">Message from the moderators:</h2>
          <div v-if="project.moderator_message.body">
            <p v-if="project.moderator_message.message" class="mod-message__title">
              {{ project.moderator_message.message }}
            </p>
          </div>
          <div
            class="markdown-body"
            v-html="
              renderString(
                project.moderator_message.body
                  ? project.moderator_message.body
                  : project.moderator_message.message
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
          <h2>Moderation actions</h2>
          <div class="input-stack">
            <button
              v-if="
                !$tag.approvedStatuses.includes(project.status) || project.status === 'processing'
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
                $tag.approvedStatuses.includes(project.status) || project.status === 'processing'
              "
              class="iconified-button danger-button"
              @click="openModerationModal('withheld')"
            >
              <EyeIcon />
              Withhold
            </button>
            <button
              v-if="
                $tag.approvedStatuses.includes(project.status) || project.status === 'processing'
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
      <section class="normal-page__content">
        <ProjectPublishingChecklist
          v-if="currentMember"
          :project="project"
          :versions="versions"
          :current-member="currentMember"
          :is-settings="$route.name.startsWith('type-id-settings')"
          :route-name="$route.name"
          :set-processing="setProcessing"
          :collapsed="collapsedChecklist"
          :toggle-collapsed="() => (collapsedChecklist = !collapsedChecklist)"
        />
        <div v-if="project.status === 'withheld'" class="card warning" aria-label="Warning">
          {{ project.title }} is not viewable in search because it has been found to be in violation
          of one of <nuxt-link to="/legal/rules"> Modrinth's content rules </nuxt-link>. Modrinth
          makes no guarantees as to whether {{ project.title }} is safe for use in a multiplayer
          context.
        </div>
        <div v-if="project.status === 'archived'" class="card warning" aria-label="Warning">
          {{ project.title }} has been archived. {{ project.title }} will not receive any further
          updates unless the author decides to unarchive the project.
        </div>
        <div
          v-if="project.project_type === 'modpack'"
          class="card information"
          aria-label="Information"
        >
          To install {{ project.title }}, visit
          <a href="https://docs.modrinth.com/docs/modpacks/playing_modpacks/" :target="$external()"
            >our documentation</a
          >
          which provides instructions on using
          <a href="https://atlauncher.com/about" :target="$external()" rel="noopener"> ATLauncher</a
          >, <a href="https://multimc.org/" :target="$external()" rel="noopener">MultiMC</a>, and
          <a href="https://prismlauncher.org" :target="$external()" rel="noopener">
            Prism Launcher</a
          >.
        </div>
        <Promotion v-if="$tag.approvedStatuses.includes(project.status)" />
        <div class="navigation-card">
          <NavRow
            :links="[
              {
                label: 'Description',
                href: `/${project.project_type}/${project.slug ? project.slug : project.id}`,
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
                shown: versions.length > 0,
              },
              {
                label: 'Versions',
                href: `/${project.project_type}/${
                  project.slug ? project.slug : project.id
                }/versions`,
                shown: versions.length > 0 || !!currentMember,
              },
            ]"
          />
          <div v-if="$auth.user && currentMember" class="input-group">
            <nuxt-link
              :to="`/${project.project_type}/${project.slug ? project.slug : project.id}/settings`"
              class="iconified-button"
            >
              <SettingsIcon /> Settings
            </nuxt-link>
          </div>
        </div>
        <NuxtPage
          v-model:project="project"
          v-model:versions="versions"
          v-model:featured-versions="featuredVersions"
          v-model:members="members"
          v-model:all-members="allMembers"
          v-model:dependencies="dependencies"
          :current-member="currentMember"
          :route="route"
        />
      </section>
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
          <h2 class="card-header">External resources</h2>
          <div class="links">
            <a
              v-if="project.issues_url"
              :href="project.issues_url"
              class="title"
              :target="$external()"
              rel="noopener nofollow ugc"
            >
              <IssuesIcon aria-hidden="true" />
              <span>Issues</span>
            </a>
            <a
              v-if="project.source_url"
              :href="project.source_url"
              class="title"
              :target="$external()"
              rel="noopener nofollow ugc"
            >
              <CodeIcon aria-hidden="true" />
              <span>Source</span>
            </a>
            <a
              v-if="project.wiki_url"
              :href="project.wiki_url"
              class="title"
              :target="$external()"
              rel="noopener nofollow ugc"
            >
              <WikiIcon aria-hidden="true" />
              <span>Wiki</span>
            </a>
            <a
              v-if="project.discord_url"
              :href="project.discord_url"
              :target="$external()"
              rel="noopener nofollow ugc"
            >
              <DiscordIcon class="shrink" aria-hidden="true" />
              <span>Discord</span>
            </a>
            <a
              v-for="(donation, index) in project.donation_urls"
              :key="index"
              :href="donation.url"
              :target="$external()"
              rel="noopener nofollow ugc"
            >
              <BuyMeACoffeeLogo v-if="donation.id === 'bmac'" aria-hidden="true" />
              <PatreonIcon v-else-if="donation.id === 'patreon'" aria-hidden="true" />
              <KoFiIcon v-else-if="donation.id === 'ko-fi'" aria-hidden="true" />
              <PayPalIcon v-else-if="donation.id === 'paypal'" aria-hidden="true" />
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
            <h2 class="card-header">Featured versions</h2>
            <nuxt-link
              v-if="$route.name !== 'type-id-versions' && (versions.length > 0 || currentMember)"
              :to="`/${project.project_type}/${
                project.slug ? project.slug : project.id
              }/versions#all-versions`"
              class="goto-link"
            >
              See all
              <ChevronRightIcon class="featured-header-chevron" aria-hidden="true" />
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
                version.primaryFile.filename + ' (' + $formatBytes(version.primaryFile.size) + ')'
              "
              :href="version.primaryFile.url"
              class="download square-button brand-button"
              :aria-label="`Download ${version.name}`"
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
              <div v-if="version.game_versions.length > 0" class="game-version item">
                {{ version.loaders.map((x) => $formatCategory(x)).join(', ') }}
                {{ $formatVersion(version.game_versions) }}
              </div>
              <Badge v-if="version.version_type === 'release'" type="release" color="green" />
              <Badge v-else-if="version.version_type === 'beta'" type="beta" color="orange" />
              <Badge v-else-if="version.version_type === 'alpha'" type="alpha" color="red" />
            </div>
          </div>
          <hr class="card-divider" />
        </template>
        <h2 class="card-header">Project members</h2>
        <nuxt-link
          v-for="member in members"
          :key="member.user.id"
          class="team-member columns button-transparent"
          :to="'/user/' + member.user.username"
        >
          <Avatar :src="member.avatar_url" :alt="member.username" size="sm" circle />

          <div class="member-info">
            <p class="name">{{ member.name }}</p>
            <p class="role">
              {{ member.role }}
            </p>
          </div>
        </nuxt-link>
        <hr class="card-divider" />
        <h2 class="card-header">Technical information</h2>
        <div class="infos">
          <div class="info">
            <div class="key">License</div>
            <div class="value lowercase">
              <a
                v-if="project.license.url"
                class="text-link"
                :href="project.license.url"
                rel="noopener nofollow ugc"
              >
                {{ licenseIdDisplay }}
              </a>
              <span
                v-else-if="
                  project.license.id === 'LicenseRef-All-Rights-Reserved' ||
                  !project.license.id.includes('LicenseRef')
                "
                class="text-link"
                @click="getLicenseData()"
              >
                {{ licenseIdDisplay }}
              </span>
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
    </div>
  </div>
</template>
<script setup>
import CalendarIcon from '~/assets/images/utils/calendar.svg'
import CheckIcon from '~/assets/images/utils/check.svg'
import ClearIcon from '~/assets/images/utils/clear.svg'
import DownloadIcon from '~/assets/images/utils/download.svg'
import UpdateIcon from '~/assets/images/utils/updated.svg'
import CodeIcon from '~/assets/images/sidebar/mod.svg'
import ReportIcon from '~/assets/images/utils/report.svg'
import HeartIcon from '~/assets/images/utils/heart.svg'
import IssuesIcon from '~/assets/images/utils/issues.svg'
import WikiIcon from '~/assets/images/utils/wiki.svg'
import DiscordIcon from '~/assets/images/external/discord.svg'
import BuyMeACoffeeLogo from '~/assets/images/external/bmac.svg'
import PatreonIcon from '~/assets/images/external/patreon.svg'
import KoFiIcon from '~/assets/images/external/kofi.svg'
import PayPalIcon from '~/assets/images/external/paypal.svg'
import OpenCollectiveIcon from '~/assets/images/external/opencollective.svg'
import UnknownIcon from '~/assets/images/utils/unknown-donation.svg'
import ChevronRightIcon from '~/assets/images/utils/chevron-right.svg'
import EyeIcon from '~/assets/images/utils/eye.svg'
import BoxIcon from '~/assets/images/utils/box.svg'
import Promotion from '~/components/ads/Promotion'
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
import SettingsIcon from '~/assets/images/utils/settings.svg'
import UsersIcon from '~/assets/images/utils/users.svg'
import CategoriesIcon from '~/assets/images/utils/tags.svg'
import DescriptionIcon from '~/assets/images/utils/align-left.svg'
import LinksIcon from '~/assets/images/utils/link.svg'
import LicenseIcon from '~/assets/images/utils/copyright.svg'
import GalleryIcon from '~/assets/images/utils/image.svg'
import VersionIcon from '~/assets/images/utils/version.svg'
import CrossIcon from '~/assets/images/utils/x.svg'
import EditIcon from '~/assets/images/utils/edit.svg'
import ModerationIcon from '~/assets/images/sidebar/admin.svg'
import { renderString } from '~/helpers/parse'
import Breadcrumbs from '~/components/ui/Breadcrumbs.vue'

const data = useNuxtApp()
const route = useRoute()

const user = await useUser()

if (
  !route.params.id ||
  !(
    data.$tag.projectTypes.find((x) => x.id === route.params.type) ||
    route.params.type === 'project'
  )
) {
  throw createError({
    fatal: true,
    statusCode: 404,
    message: 'The page could not be found',
  })
}

let project, allMembers, dependencies, featuredVersions, versions
try {
  ;[
    { data: project },
    { data: allMembers },
    { data: dependencies },
    { data: featuredVersions },
    { data: versions },
  ] = await Promise.all([
    useAsyncData(
      `project/${route.params.id}`,
      () => useBaseFetch(`project/${route.params.id}`, data.$defaultHeaders()),
      {
        transform: (project) => {
          if (project) {
            project.actualProjectType = JSON.parse(JSON.stringify(project.project_type))

            project.project_type = data.$getProjectTypeForUrl(project.project_type, project.loaders)

            if (process.client && history.state && history.state.overrideProjectType) {
              project.project_type = history.state.overrideProjectType
            }
          }

          return project
        },
      }
    ),
    useAsyncData(
      `project/${route.params.id}/members`,
      () => useBaseFetch(`project/${route.params.id}/members`, data.$defaultHeaders()),
      {
        transform: (members) => {
          members.forEach((it, index) => {
            members[index].avatar_url = it.user.avatar_url
            members[index].name = it.user.username
          })

          return members
        },
      }
    ),
    useAsyncData(`project/${route.params.id}/dependencies`, () =>
      useBaseFetch(`project/${route.params.id}/dependencies`, data.$defaultHeaders())
    ),
    useAsyncData(`project/${route.params.id}/version?featured=true`, () =>
      useBaseFetch(`project/${route.params.id}/version?featured=true`, data.$defaultHeaders())
    ),
    useAsyncData(`project/${route.params.id}/version`, () =>
      useBaseFetch(`project/${route.params.id}/version`, data.$defaultHeaders())
    ),
  ])

  versions = shallowRef(toRaw(versions))
  featuredVersions = shallowRef(toRaw(featuredVersions))
} catch (error) {
  throw createError({
    fatal: true,
    statusCode: 404,
    message: 'Project not found',
  })
}

if (!project.value) {
  throw createError({
    fatal: true,
    statusCode: 404,
    message: 'Project not found',
  })
}

if (project.value.project_type !== route.params.type || route.params.id !== project.value.slug) {
  let path = route.fullPath.split('/')
  path.splice(0, 3)
  path = path.filter((x) => x)

  await navigateTo(
    `/${project.value.project_type}/${project.value.slug}${
      path.length > 0 ? `/${path.join('/')}` : ''
    }`,
    { redirectCode: 301 }
  )
}

const members = ref(allMembers.value.filter((x) => x.accepted))
const currentMember = ref(
  data.$auth.user ? allMembers.value.find((x) => x.user.id === data.$auth.user.id) : null
)

if (
  !currentMember.value &&
  data.$auth.user &&
  data.$tag.staffRoles.includes(data.$auth.user.role)
) {
  currentMember.value = {
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

versions.value = data.$computeVersions(versions.value, allMembers.value)

// Q: Why do this instead of computing the versions of featuredVersions?
// A: It will incorrectly generate the version slugs because it doesn't have the full context of
//    all the versions. For example, if version 1.1.0 for Forge is featured but 1.1.0 for Fabric
//    is not, but the Fabric one was uploaded first, the Forge version would link to the Fabric
///   version
const featuredIds = featuredVersions.value.map((x) => x.id)
featuredVersions.value = versions.value.filter((version) => featuredIds.includes(version.id))

featuredVersions.value.sort((a, b) => {
  const aLatest = a.game_versions[a.game_versions.length - 1]
  const bLatest = b.game_versions[b.game_versions.length - 1]
  const gameVersions = data.$tag.gameVersions.map((e) => e.version)
  return gameVersions.indexOf(aLatest) - gameVersions.indexOf(bLatest)
})

const projectTypeDisplay = computed(() =>
  data.$formatProjectType(
    data.$getProjectTypeForDisplay(project.value.project_type, project.value.loaders)
  )
)
const licenseIdDisplay = computed(() => {
  const id = project.value.license.id

  if (id === 'LicenseRef-All-Rights-Reserved') {
    return 'ARR'
  } else if (id.includes('LicenseRef')) {
    return id.replaceAll('LicenseRef-', '').replaceAll('-', ' ')
  } else {
    return id
  }
})
const featuredGalleryImage = computed(() => project.value.gallery.find((img) => img.featured))
const requestedStatus = computed(() => project.value.requested_status ?? 'approved')

async function resetProject() {
  const newProject = await useBaseFetch(`project/${project.value.id}`, data.$defaultHeaders())

  newProject.actualProjectType = JSON.parse(JSON.stringify(newProject.project_type))

  newProject.project_type = data.$getProjectTypeForUrl(newProject.project_type, newProject.loaders)

  project.value = newProject
}

async function clearMessage() {
  startLoading()

  try {
    await useBaseFetch(`project/${project.value.id}`, {
      method: 'PATCH',
      body: {
        moderation_message: null,
        moderation_message_body: null,
      },
      ...data.$defaultHeaders(),
    })

    project.value.moderator_message = null
  } catch (err) {
    data.$notify({
      group: 'main',
      title: 'An error occurred',
      text: err.data.description,
      type: 'error',
    })
  }

  stopLoading()
}

async function setProcessing() {
  startLoading()

  try {
    await useBaseFetch(`project/${project.value.id}`, {
      method: 'PATCH',
      body: {
        status: 'processing',
      },
      ...data.$defaultHeaders(),
    })

    project.value.status = 'processing'
  } catch (err) {
    data.$notify({
      group: 'main',
      title: 'An error occurred',
      text: err.data.description,
      type: 'error',
    })
  }

  stopLoading()
}

const modalLicense = ref(null)
const licenseText = ref('')
async function getLicenseData() {
  try {
    const text = await useBaseFetch(`tag/license/${project.value.license.id}`)
    licenseText.value = text.body
  } catch {
    licenseText.value = 'License text could not be retrieved.'
  }

  modalLicense.value.show()
}

async function patchProject(resData, quiet = false) {
  let result = false
  startLoading()

  try {
    await useBaseFetch(`project/${project.value.id}`, {
      method: 'PATCH',
      body: resData,
      ...data.$defaultHeaders(),
    })

    for (const key in resData) {
      project.value[key] = resData[key]
    }

    if (resData.license_id) {
      project.value.license.id = resData.license_id
    }
    if (resData.license_url) {
      project.value.license.url = resData.license_url
    }

    result = true
    if (!quiet) {
      data.$notify({
        group: 'main',
        title: 'Project updated',
        text: 'Your project has been updated.',
        type: 'success',
      })
      window.scrollTo({ top: 0, behavior: 'smooth' })
    }
  } catch (err) {
    data.$notify({
      group: 'main',
      title: 'An error occurred',
      text: err.data.description,
      type: 'error',
    })
    window.scrollTo({ top: 0, behavior: 'smooth' })
  }

  stopLoading()

  return result
}

async function patchIcon(icon) {
  let result = false
  startLoading()

  try {
    await useBaseFetch(
      `project/${project.value.id}/icon?ext=${
        icon.type.split('/')[icon.type.split('/').length - 1]
      }`,
      {
        method: 'PATCH',
        body: icon,
        ...data.$defaultHeaders(),
      }
    )
    await resetProject()
    result = true
    data.$notify({
      group: 'main',
      title: 'Project icon updated',
      text: "Your project's icon has been updated.",
      type: 'success',
    })
  } catch (err) {
    data.$notify({
      group: 'main',
      title: 'An error occurred',
      text: err.data.description,
      type: 'error',
    })

    window.scrollTo({ top: 0, behavior: 'smooth' })
  }

  stopLoading()
  return result
}

const modalModeration = ref(null)
const moderationStatus = ref(null)
function openModerationModal(status) {
  moderationStatus.value = status

  modalModeration.value.show()
}

const collapsedChecklist = ref(false)
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

  .project-type {
    text-decoration: none;
    font-weight: 500;

    svg {
      vertical-align: top;
      margin-right: 0.25em;
    }

    &:hover,
    &:focus-visible {
      span {
        text-decoration: underline;
      }
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
  font-size: 1.125rem;
  font-weight: bold;
  color: var(--color-heading);
  margin-top: var(--spacing-card-md);
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
