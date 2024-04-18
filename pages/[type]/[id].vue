<template>
  <div v-if="route.name.startsWith('type-id-settings')" class="normal-page">
    <div class="normal-page__sidebar">
      <aside class="universal-card">
        <Breadcrumbs
          current-title="Settings"
          :link-stack="[
            {
              href: organization
                ? `/organization/${organization.slug}/settings/projects`
                : `/dashboard/projects`,
              label: 'Projects',
            },
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
          <h3>View</h3>
          <NavStackItem
            :link="`/${project.project_type}/${
              project.slug ? project.slug : project.id
            }/settings/analytics`"
            label="Analytics"
            chevron
          >
            <ChartIcon />
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
      <ProjectMemberHeader
        v-if="currentMember"
        :project="project"
        :versions="versions"
        :current-member="currentMember"
        :is-settings="route.name.startsWith('type-id-settings')"
        :route-name="route.name"
        :set-processing="setProcessing"
        :collapsed="collapsedChecklist"
        :toggle-collapsed="() => (collapsedChecklist = !collapsedChecklist)"
        :all-members="allMembers"
        :update-members="updateMembers"
        :auth="auth"
        :tags="tags"
      />
      <NuxtPage
        v-model:project="project"
        v-model:versions="versions"
        v-model:featured-versions="featuredVersions"
        v-model:members="members"
        v-model:all-members="allMembers"
        v-model:dependencies="dependencies"
        v-model:organization="organization"
        :current-member="currentMember"
        :patch-project="patchProject"
        :patch-icon="patchIcon"
        :reset-project="resetProject"
        :reset-organization="resetOrganization"
        :reset-members="resetMembers"
        :route="route"
      />
    </div>
  </div>
  <div v-else>
    <Modal ref="modalLicense" :header="project.license.name ? project.license.name : 'License'">
      <div class="modal-license">
        <div class="markdown-body" v-html="renderString(licenseText)" />
      </div>
    </Modal>
    <CollectionCreateModal ref="modal_collection" :project-ids="[project.id]" />
    <div
      :class="{
        'normal-page': true,
        'alt-layout': cosmetics.projectLayout,
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
          <div class="project__header__content padding-lg full-width-inputs">
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
                v-if="auth.user && currentMember"
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
                v-tooltip="$dayjs(project.published).format('MMMM D, YYYY [at] h:mm A')"
                class="date"
              >
                <CalendarIcon aria-hidden="true" />
                <span class="label">Created</span>
                <span class="value">{{ fromNow(project.published) }}</span>
              </div>
              <div
                v-tooltip="$dayjs(project.updated).format('MMMM D, YYYY [at] h:mm A')"
                class="date"
              >
                <UpdateIcon aria-hidden="true" />
                <span class="label">Updated</span>
                <span class="value">{{ fromNow(project.updated) }}</span>
              </div>
              <div
                v-if="project.status === 'processing' && project.queued"
                v-tooltip="$dayjs(project.queued).format('MMMM D, YYYY [at] h:mm A')"
                class="date"
              >
                <QueuedIcon aria-hidden="true" />
                <span class="label">Submitted</span>
                <span class="value">{{ fromNow(project.queued) }}</span>
              </div>
            </div>
            <hr class="card-divider" />
            <div class="input-group">
              <template v-if="auth.user">
                <button
                  v-if="!user.follows.find((x) => x.id === project.id)"
                  class="btn"
                  @click="userFollowProject(project)"
                >
                  <HeartIcon aria-hidden="true" />
                  Follow
                </button>
                <button
                  v-if="user.follows.find((x) => x.id === project.id)"
                  class="btn"
                  @click="userUnfollowProject(project)"
                >
                  <HeartIcon fill="currentColor" aria-hidden="true" />
                  Unfollow
                </button>
                <PopoutMenu class="btn" direction="right" position="bottom" from="top-right">
                  <BookmarkIcon aria-hidden="true" />
                  Save
                  <template #menu>
                    <input
                      v-model="displayCollectionsSearch"
                      type="text"
                      placeholder="Search collections..."
                      class="search-input menu-search"
                    />
                    <div v-if="collections.length > 0" class="collections-list">
                      <Checkbox
                        v-for="option in collections"
                        :key="option.id"
                        :model-value="option.projects.includes(project.id)"
                        class="popout-checkbox"
                        @update:model-value="() => onUserCollectProject(option, project.id)"
                      >
                        {{ option.name }}
                      </Checkbox>
                    </div>
                    <div v-else class="menu-text">
                      <p class="popout-text">No collections found.</p>
                    </div>
                    <button class="btn collection-button" @click="$refs.modal_collection.show()">
                      <PlusIcon />
                      Create new collection
                    </button>
                  </template>
                </PopoutMenu>
                <OverflowMenu
                  class="btn icon-only"
                  :options="[
                    {
                      id: 'report',
                      action: () => reportProject(project.id),
                      color: 'red',
                      hoverOnly: true,
                    },
                    { id: 'copy-id', action: () => copyId() },
                  ]"
                  :direction="cosmetics.projectLayout ? 'left' : 'right'"
                >
                  <MoreHorizontalIcon />
                  <template #report> <ReportIcon /> Report</template>
                  <template #copy-id> <ClipboardCopyIcon /> Copy ID</template>
                </OverflowMenu>
              </template>
              <template v-else>
                <nuxt-link class="iconified-button" to="/auth/sign-in">
                  <HeartIcon aria-hidden="true" />
                  Follow
                </nuxt-link>
                <nuxt-link class="iconified-button" to="/auth/sign-in">
                  <BookmarkIcon aria-hidden="true" />
                  Save
                </nuxt-link>
                <OverflowMenu
                  class="btn icon-only"
                  :options="[
                    {
                      id: 'report',
                      action: () => navigateTo('/auth/sign-in'),
                      color: 'red',
                      hoverOnly: true,
                    },
                    { id: 'copy-id', action: () => copyId() },
                  ]"
                  :direction="cosmetics.projectLayout ? 'left' : 'right'"
                >
                  <MoreHorizontalIcon />
                  <template #report> <ReportIcon /> Report</template>
                  <template #copy-id> <ClipboardCopyIcon /> Copy ID</template>
                </OverflowMenu>
              </template>
            </div>
          </div>
        </div>
      </div>
      <section class="normal-page__content">
        <ProjectMemberHeader
          v-if="currentMember"
          :project="project"
          :versions="versions"
          :current-member="currentMember"
          :is-settings="route.name.startsWith('type-id-settings')"
          :route-name="route.name"
          :set-processing="setProcessing"
          :collapsed="collapsedChecklist"
          :toggle-collapsed="() => (collapsedChecklist = !collapsedChecklist)"
          :all-members="allMembers"
          :update-members="updateMembers"
          :auth="auth"
          :tags="tags"
        />
        <MessageBanner v-if="project.status === 'archived'" message-type="warning">
          {{ project.title }} has been archived. {{ project.title }} will not receive any further
          updates unless the author decides to unarchive the project.
        </MessageBanner>
        <MessageBanner v-if="project.project_type === 'modpack'" message-type="information">
          To install {{ project.title }}, download
          <nuxt-link to="/app">the Modrinth App</nuxt-link>. For instructions with other launchers,
          please see
          <a href="https://docs.modrinth.com/docs/modpacks/playing_modpacks/" :target="$external()"
            >our documentation</a
          >.
        </MessageBanner>
        <Promotion v-if="tags.approvedStatuses.includes(project.status)" />
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
              {
                label: 'Moderation',
                href: `/${project.project_type}/${
                  project.slug ? project.slug : project.id
                }/moderation`,
                shown:
                  !!currentMember &&
                  (isRejected(project) || isUnderReview(project) || isStaff(auth.user)),
              },
            ]"
          />
          <div v-if="auth.user && currentMember" class="input-group">
            <button
              v-if="tags.staffRoles.includes(auth.user.role) && !showModerationChecklist"
              class="iconified-button"
              @click="showModerationChecklist = true"
            >
              <EyeIcon /> Checklist
            </button>
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
          v-model:organization="organization"
          :current-member="currentMember"
          :reset-project="resetProject"
          :reset-organization="resetOrganization"
          :reset-members="resetMembers"
          :route="route"
        />
      </section>
      <div class="universal-card normal-page__info">
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
              v-if="route.name !== 'type-id-versions' && (versions.length > 0 || currentMember)"
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
          v-if="organization"
          class="team-member columns button-transparent"
          :to="`/organization/${organization.slug}`"
        >
          <Avatar :src="organization.icon_url" :alt="organization.name" size="sm" />
          <div class="member-info">
            <p class="name">
              {{ organization.name }}
            </p>
            <p class="role"><OrganizationIcon /> Organization</p>
          </div>
        </nuxt-link>
        <nuxt-link
          v-for="member in members"
          :key="member.user.id"
          class="team-member columns button-transparent"
          :to="'/user/' + member.user.username"
        >
          <Avatar :src="member.avatar_url" :alt="member.username" size="sm" circle />

          <div class="member-info">
            <p class="name">
              {{ member.name }} <CrownIcon v-if="member.is_owner" v-tooltip="'Project owner'" />
            </p>
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
          <div class="input-group">
            <a
              v-if="
                config.public.apiBaseUrl.startsWith('https://api.modrinth.com') &&
                config.public.siteUrl !== 'https://modrinth.com'
              "
              class="iconified-button"
              :href="`https://modrinth.com/${project.project_type}/${
                project.slug ? project.slug : project.id
              }`"
              rel="noopener nofollow"
              target="_blank"
            >
              <ExternalIcon aria-hidden="true" />
              View on modrinth.com
            </a>
            <a
              v-else-if="
                config.public.apiBaseUrl.startsWith('https://staging-api.modrinth.com') &&
                config.public.siteUrl !== 'https://staging.modrinth.com'
              "
              class="iconified-button"
              :href="`https://staging.modrinth.com/${project.project_type}/${
                project.slug ? project.slug : project.id
              }`"
              rel="noopener nofollow"
              target="_blank"
            >
              <ExternalIcon aria-hidden="true" />
              View on staging.modrinth.com
            </a>
          </div>
        </div>
      </div>
    </div>
    <ModerationChecklist
      v-if="auth.user && tags.staffRoles.includes(auth.user.role) && showModerationChecklist"
      :project="project"
      :future-projects="futureProjects"
      :reset-project="resetProject"
    />
  </div>
</template>
<script setup>
import {
  Promotion,
  OverflowMenu,
  PopoutMenu,
  BookmarkIcon,
  MoreHorizontalIcon,
  ClipboardCopyIcon,
  PlusIcon,
  Checkbox,
  ChartIcon,
  EyeIcon,
  renderString,
  isRejected,
  isUnderReview,
  isStaff,
} from 'omorphia'
import CrownIcon from '~/assets/images/utils/crown.svg'
import CalendarIcon from '~/assets/images/utils/calendar.svg'
import DownloadIcon from '~/assets/images/utils/download.svg'
import UpdateIcon from '~/assets/images/utils/updated.svg'
import QueuedIcon from '~/assets/images/utils/list-end.svg'
import CodeIcon from '~/assets/images/sidebar/mod.svg'
import ExternalIcon from '~/assets/images/utils/external.svg'
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
import BoxIcon from '~/assets/images/utils/box.svg'
import Badge from '~/components/ui/Badge.vue'
import Categories from '~/components/ui/search/Categories.vue'
import EnvironmentIndicator from '~/components/ui/EnvironmentIndicator.vue'
import Modal from '~/components/ui/Modal.vue'
import NavRow from '~/components/ui/NavRow.vue'
import CopyCode from '~/components/ui/CopyCode.vue'
import Avatar from '~/components/ui/Avatar.vue'
import NavStack from '~/components/ui/NavStack.vue'
import NavStackItem from '~/components/ui/NavStackItem.vue'
import ProjectMemberHeader from '~/components/ui/ProjectMemberHeader.vue'
import MessageBanner from '~/components/ui/MessageBanner.vue'
import SettingsIcon from '~/assets/images/utils/settings.svg'
import UsersIcon from '~/assets/images/utils/users.svg'
import CategoriesIcon from '~/assets/images/utils/tags.svg'
import DescriptionIcon from '~/assets/images/utils/align-left.svg'
import LinksIcon from '~/assets/images/utils/link.svg'
import LicenseIcon from '~/assets/images/utils/copyright.svg'
import GalleryIcon from '~/assets/images/utils/image.svg'
import VersionIcon from '~/assets/images/utils/version.svg'
import { reportProject } from '~/utils/report-helpers.ts'
import Breadcrumbs from '~/components/ui/Breadcrumbs.vue'
import { userCollectProject } from '~/composables/user.js'
import CollectionCreateModal from '~/components/ui/CollectionCreateModal.vue'
import OrganizationIcon from '~/assets/images/utils/organization.svg'
import ModerationChecklist from '~/components/ui/ModerationChecklist.vue'

const data = useNuxtApp()
const route = useRoute()
const config = useRuntimeConfig()

const auth = await useAuth()
const user = await useUser()
const cosmetics = useCosmetics()
const tags = useTags()

const displayCollectionsSearch = ref('')
const collections = computed(() =>
  user.value && user.value.collections
    ? user.value.collections.filter((x) =>
        x.name.toLowerCase().includes(displayCollectionsSearch.value.toLowerCase())
      )
    : []
)

if (
  !route.params.id ||
  !(
    tags.value.projectTypes.find((x) => x.id === route.params.type) ||
    route.params.type === 'project'
  )
) {
  throw createError({
    fatal: true,
    statusCode: 404,
    message: 'The page could not be found',
  })
}

let project,
  resetProject,
  allMembers,
  resetMembers,
  dependencies,
  featuredVersions,
  versions,
  organization,
  resetOrganization
try {
  ;[
    { data: project, refresh: resetProject },
    { data: allMembers, refresh: resetMembers },
    { data: dependencies },
    { data: featuredVersions },
    { data: versions },
    { data: organization, refresh: resetOrganization },
  ] = await Promise.all([
    useAsyncData(`project/${route.params.id}`, () => useBaseFetch(`project/${route.params.id}`), {
      transform: (project) => {
        if (project) {
          project.actualProjectType = JSON.parse(JSON.stringify(project.project_type))
          project.project_type = data.$getProjectTypeForUrl(
            project.project_type,
            project.loaders,
            tags.value
          )
        }

        return project
      },
    }),
    useAsyncData(
      `project/${route.params.id}/members`,
      () => useBaseFetch(`project/${route.params.id}/members`, { apiVersion: 3 }),
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
      useBaseFetch(`project/${route.params.id}/dependencies`)
    ),
    useAsyncData(`project/${route.params.id}/version?featured=true`, () =>
      useBaseFetch(`project/${route.params.id}/version?featured=true`)
    ),
    useAsyncData(`project/${route.params.id}/version`, () =>
      useBaseFetch(`project/${route.params.id}/version`)
    ),
    useAsyncData(`project/${route.params.id}/organization`, () =>
      useBaseFetch(`project/${route.params.id}/organization`, { apiVersion: 3 })
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
    { redirectCode: 301, replace: true }
  )
}

// Members should be an array of all members, without the accepted ones, and with the user with the Owner role at the start
// The rest of the members should be sorted by role, then by name
const members = computed(() => {
  const acceptedMembers = allMembers.value.filter((x) => x.accepted)
  const owner = acceptedMembers.find((x) => x.is_owner)
  const rest = acceptedMembers.filter((x) => !x.is_owner) || []

  rest.sort((a, b) => {
    if (a.role === b.role) {
      return a.user.username.localeCompare(b.user.username)
    } else {
      return a.role.localeCompare(b.role)
    }
  })

  return owner ? [owner, ...rest] : rest
})

const currentMember = computed(() => {
  let val = auth.value.user ? allMembers.value.find((x) => x.user.id === auth.value.user.id) : null

  if (!val && auth.value.user && organization.value && organization.value.members) {
    val = organization.value.members.find((x) => x.user.id === auth.value.user.id)
  }

  if (!val && auth.value.user && tags.value.staffRoles.includes(auth.value.user.role)) {
    val = {
      team_id: project.team_id,
      user: auth.value.user,
      role: auth.value.role,
      permissions: auth.value.user.role === 'admin' ? 1023 : 12,
      accepted: true,
      payouts_split: 0,
      avatar_url: auth.value.user.avatar_url,
      name: auth.value.user.username,
    }
  }

  return val
})

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
  const gameVersions = tags.value.gameVersions.map((e) => e.version)
  return gameVersions.indexOf(aLatest) - gameVersions.indexOf(bLatest)
})

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

const projectTypeDisplay = computed(() =>
  data.$formatProjectType(
    data.$getProjectTypeForDisplay(project.value.project_type, project.value.loaders)
  )
)

const title = computed(() => `${project.value.title} - Minecraft ${projectTypeDisplay.value}`)
const description = computed(
  () =>
    `${project.value.description} - Download the Minecraft ${projectTypeDisplay.value} ${
      project.value.title
    } by ${members.value.find((x) => x.is_owner)?.user?.username || 'a Creator'} on Modrinth`
)

if (!route.name.startsWith('type-id-settings')) {
  useSeoMeta({
    title: () => title.value,
    description: () => description.value,
    ogTitle: () => title.value,
    ogDescription: () => project.value.description,
    ogImage: () => project.value.icon_url ?? 'https://cdn.modrinth.com/placeholder.png',
    robots: () =>
      project.value.status === 'approved' || project.value.status === 'archived'
        ? 'all'
        : 'noindex',
  })
}

const onUserCollectProject = useClientTry(userCollectProject)

async function setProcessing() {
  startLoading()

  try {
    await useBaseFetch(`project/${project.value.id}`, {
      method: 'PATCH',
      body: {
        status: 'processing',
      },
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
    licenseText.value = text.body || 'License text could not be retrieved.'
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

async function updateMembers() {
  allMembers.value = await useAsyncData(
    `project/${route.params.id}/members`,
    () => useBaseFetch(`project/${route.params.id}/members`),
    {
      transform: (members) => {
        members.forEach((it, index) => {
          members[index].avatar_url = it.user.avatar_url
          members[index].name = it.user.username
        })

        return members
      },
    }
  )
}

async function copyId() {
  await navigator.clipboard.writeText(project.value.id)
}

const collapsedChecklist = ref(false)

const showModerationChecklist = ref(false)
const futureProjects = ref([])
if (process.client && history && history.state && history.state.showChecklist) {
  showModerationChecklist.value = true
  futureProjects.value = history.state.projects
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
  .project__gallery {
    display: none;
  }
  &.has-featured-image {
    .project__gallery {
      display: inline-block;
      width: 100%;
      height: 10rem;
      img {
        width: 100%;
        height: 10rem;
        object-fit: cover;
        background-color: var(--color-button-bg-active);
        border-radius: var(--size-rounded-card) var(--size-rounded-card) 0 0;
      }
    }
    .project__icon {
      margin-top: calc(-3rem - var(--spacing-card-lg) - 4px);
      margin-left: -4px;
      z-index: 1;
      box-shadow: -2px -2px 0 2px var(--color-raised-bg), 2px -2px 0 2px var(--color-raised-bg);
    }
  }
  .project__header__content {
    margin: 0;
    background: none;
    border-radius: unset;
  }
  .input-group {
    flex-wrap: nowrap;
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
      display: flex;
      align-items: center;
      gap: 0.25rem;

      svg {
        color: var(--color-orange);
      }
    }

    p {
      font-size: var(--font-size-sm);
      margin: 0.2rem 0;
    }

    .role {
      display: flex;
      align-items: center;
      gap: 0.25rem;
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

.popout-checkbox {
  padding: var(--gap-sm) var(--gap-md);
  white-space: nowrap;
  &:hover {
    filter: brightness(0.95);
  }
}

.popout-heading {
  padding: var(--gap-sm) var(--gap-md);
  padding-bottom: 0;
  font-size: var(--font-size-nm);
  color: var(--color-secondary);
}

.collection-button {
  margin: var(--gap-sm) var(--gap-md);
  white-space: nowrap;
}

.menu-text {
  padding: 0 var(--gap-md);
  font-size: var(--font-size-nm);
  color: var(--color-secondary);
}

.menu-search {
  margin: var(--gap-sm) var(--gap-md);
  width: calc(100% - var(--gap-md) * 2);
}

.collections-list {
  max-height: 40rem;
  overflow-y: auto;
  background-color: var(--color-bg);
  border-radius: var(--radius-md);
  margin: var(--gap-sm) var(--gap-md);
  padding: var(--gap-sm);
}
</style>
