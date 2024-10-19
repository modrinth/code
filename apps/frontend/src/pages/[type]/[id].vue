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
            <SettingsIcon aria-hidden="true" />
          </NavStackItem>
          <NavStackItem
            :link="`/${project.project_type}/${
              project.slug ? project.slug : project.id
            }/settings/tags`"
            label="Tags"
          >
            <TagsIcon aria-hidden="true" />
          </NavStackItem>
          <NavStackItem
            :link="`/${project.project_type}/${
              project.slug ? project.slug : project.id
            }/settings/description`"
            label="Description"
          >
            <DescriptionIcon aria-hidden="true" />
          </NavStackItem>
          <NavStackItem
            :link="`/${project.project_type}/${
              project.slug ? project.slug : project.id
            }/settings/license`"
            label="License"
          >
            <CopyrightIcon aria-hidden="true" />
          </NavStackItem>
          <NavStackItem
            :link="`/${project.project_type}/${
              project.slug ? project.slug : project.id
            }/settings/links`"
            label="Links"
          >
            <LinksIcon aria-hidden="true" />
          </NavStackItem>
          <NavStackItem
            :link="`/${project.project_type}/${
              project.slug ? project.slug : project.id
            }/settings/members`"
            label="Members"
          >
            <UsersIcon aria-hidden="true" />
          </NavStackItem>
          <h3>View</h3>
          <NavStackItem
            :link="`/${project.project_type}/${
              project.slug ? project.slug : project.id
            }/settings/analytics`"
            label="Analytics"
            chevron
          >
            <ChartIcon aria-hidden="true" />
          </NavStackItem>
          <h3>Upload</h3>
          <NavStackItem
            :link="`/${project.project_type}/${project.slug ? project.slug : project.id}/gallery`"
            label="Gallery"
            chevron
          >
            <GalleryIcon aria-hidden="true" />
          </NavStackItem>
          <NavStackItem
            :link="`/${project.project_type}/${project.slug ? project.slug : project.id}/versions`"
            label="Versions"
            chevron
          >
            <VersionIcon aria-hidden="true" />
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
  <div v-else class="experimental-styles-within">
    <NewModal ref="settingsModal">
      <template #title>
        <Avatar :src="project.icon_url" :alt="project.title" class="icon" size="32px" />
        <span class="text-lg font-extrabold text-contrast"> Settings </span>
      </template>
    </NewModal>
    <NewModal ref="modalLicense" :header="project.license.name ? project.license.name : 'License'">
      <template #title>
        <Avatar :src="project.icon_url" :alt="project.title" class="icon" size="32px" no-shadow />
        <span class="text-lg font-extrabold text-contrast">
          {{ project.license.name ? project.license.name : "License" }}
        </span>
      </template>
      <div
        class="markdown-body"
        v-html="
          renderString(licenseText).isEmpty ? 'Loading license text...' : renderString(licenseText)
        "
      />
    </NewModal>
    <div
      class="over-the-top-download-animation"
      :class="{ 'animation-hidden': !overTheTopDownloadAnimation }"
    >
      <div>
        <div
          class="animation-ring-3 flex items-center justify-center rounded-full border-4 border-solid border-brand bg-brand-highlight opacity-40"
        ></div>
        <div
          class="animation-ring-2 flex items-center justify-center rounded-full border-4 border-solid border-brand bg-brand-highlight opacity-60"
        ></div>
        <div
          class="animation-ring-1 flex items-center justify-center rounded-full border-4 border-solid border-brand bg-brand-highlight"
        >
          <DownloadIcon class="h-20 w-20 text-contrast" />
        </div>
      </div>
    </div>
    <NewModal ref="downloadModal">
      <template #title>
        <Avatar :src="project.icon_url" :alt="project.title" class="icon" size="32px" />
        <div class="truncate text-lg font-extrabold text-contrast">
          Download {{ project.title }}
        </div>
      </template>
      <template #default>
        <div class="mx-auto flex max-w-[40rem] flex-col gap-4 md:w-[30rem]">
          <div
            v-if="
              project.project_type !== 'plugin' ||
              project.loaders.some((x) => !tags.loaderData.allPluginLoaders.includes(x))
            "
            class="modrinth-app-section contents"
          >
            <div class="mx-auto flex w-fit flex-col">
              <ButtonStyled color="brand">
                <a
                  class="w-fit"
                  :href="`modrinth://mod/${project.slug}`"
                  @click="() => installWithApp()"
                >
                  <ModrinthIcon aria-hidden="true" />
                  Install with Modrinth App
                  <ExternalIcon aria-hidden="true" />
                </a>
              </ButtonStyled>
              <Accordion ref="getModrinthAppAccordion">
                <nuxt-link
                  class="mt-2 flex justify-center text-brand-blue hover:underline"
                  to="/app"
                >
                  Don't have Modrinth App?
                </nuxt-link>
              </Accordion>
            </div>

            <div class="flex items-center gap-4 px-4">
              <div class="flex h-[2px] w-full rounded-2xl bg-button-bg"></div>
              <span class="flex-shrink-0 text-sm font-semibold text-secondary"> or </span>
              <div class="flex h-[2px] w-full rounded-2xl bg-button-bg"></div>
            </div>
          </div>
          <div class="mx-auto flex w-fit flex-col gap-2">
            <ButtonStyled v-if="project.game_versions.length === 1">
              <div class="disabled button-like">
                <GameIcon aria-hidden="true" />
                {{
                  currentGameVersion
                    ? `Game version: ${currentGameVersion}`
                    : "Error: no game versions found"
                }}
                <InfoIcon
                  v-tooltip="`${project.title} is only available for ${currentGameVersion}`"
                  class="ml-auto size-5"
                />
              </div>
            </ButtonStyled>
            <Accordion
              v-else
              ref="gameVersionAccordion"
              class="accordion-with-bg"
              @on-open="
                () => {
                  if (platformAccordion) {
                    platformAccordion.close();
                  }
                }
              "
            >
              <template #title>
                <GameIcon aria-hidden="true" />
                {{
                  currentGameVersion ? `Game version: ${currentGameVersion}` : "Select game version"
                }}
              </template>
              <div class="iconified-input mb-2 flex w-full">
                <label for="game-versions-filtering" hidden>Search game versions...</label>
                <SearchIcon aria-hidden="true" />
                <input
                  id="game-versions-filtering"
                  ref="gameVersionFilterInput"
                  v-model="versionFilter"
                  type="search"
                  autocomplete="off"
                  placeholder="Search game versions..."
                />
              </div>
              <ScrollablePanel :class="project.game_versions.length > 4 ? 'h-[15rem]' : ''">
                <ButtonStyled
                  v-for="version in project.game_versions
                    .filter(
                      (x) =>
                        (versionFilter && x.includes(versionFilter)) ||
                        (!versionFilter &&
                          (showAllVersions || (!x.includes('w') && !x.includes('-')))),
                    )
                    .slice()
                    .reverse()"
                  :key="version"
                  :color="currentGameVersion === version ? 'brand' : 'standard'"
                >
                  <button
                    v-tooltip="
                      !possibleGameVersions.includes(version)
                        ? `${project.title} does not support ${version} for ${formatCategory(currentPlatform)}`
                        : null
                    "
                    :class="{
                      'looks-disabled !text-brand-red': !possibleGameVersions.includes(version),
                    }"
                    @click="
                      () => {
                        userSelectedGameVersion = version;
                        gameVersionAccordion.close();
                        if (!currentPlatform && platformAccordion) {
                          platformAccordion.open();
                        }
                      }
                    "
                  >
                    {{ version }}
                    <CheckIcon v-if="userSelectedGameVersion === version" />
                  </button>
                </ButtonStyled>
              </ScrollablePanel>
              <Checkbox
                v-model="showAllVersions"
                class="mx-1"
                :label="`Show all versions`"
                :disabled="!!versionFilter"
              />
            </Accordion>
            <ButtonStyled
              v-if="project.loaders.length === 1 && project.project_type !== 'resourcepack'"
            >
              <div class="disabled button-like">
                <WrenchIcon aria-hidden="true" />
                {{
                  currentPlatform
                    ? `Platform: ${formatCategory(currentPlatform)}`
                    : "Error: no platforms found"
                }}
                <InfoIcon
                  v-tooltip="
                    `${project.title} is only available for ${formatCategory(currentPlatform)}`
                  "
                  class="ml-auto size-5"
                />
              </div>
            </ButtonStyled>
            <Accordion
              v-else-if="project.project_type !== 'resourcepack'"
              ref="platformAccordion"
              class="accordion-with-bg"
              @on-open="
                () => {
                  if (gameVersionAccordion) {
                    gameVersionAccordion.close();
                  }
                }
              "
            >
              <template #title>
                <WrenchIcon aria-hidden="true" />
                {{
                  currentPlatform
                    ? `Platform: ${formatCategory(currentPlatform)}`
                    : "Select platform"
                }}
              </template>
              <ScrollablePanel :class="project.loaders.length > 4 ? 'h-[15rem]' : ''">
                <ButtonStyled
                  v-for="platform in project.loaders.slice().reverse()"
                  :key="platform"
                  :color="currentPlatform === platform ? 'brand' : 'standard'"
                >
                  <button
                    v-tooltip="
                      !possiblePlatforms.includes(platform)
                        ? `${project.title} does not support ${formatCategory(platform)} for ${currentGameVersion}`
                        : null
                    "
                    :class="{
                      'looks-disabled !text-brand-red': !possiblePlatforms.includes(platform),
                    }"
                    @click="
                      () => {
                        userSelectedPlatform = platform;

                        platformAccordion.close();
                        if (!currentGameVersion && gameVersionAccordion) {
                          gameVersionAccordion.open();
                        }
                      }
                    "
                  >
                    {{ formatCategory(platform) }}
                    <CheckIcon v-if="userSelectedPlatform === platform" />
                  </button>
                </ButtonStyled>
              </ScrollablePanel>
            </Accordion>
          </div>

          <AutomaticAccordion div class="flex flex-col gap-2">
            <VersionSummary
              v-if="filteredRelease"
              :version="filteredRelease"
              @on-download="onDownload"
              @on-navigate="downloadModal.hide"
            />
            <VersionSummary
              v-if="filteredBeta"
              :version="filteredBeta"
              @on-download="onDownload"
              @on-navigate="downloadModal.hide"
            />
            <VersionSummary
              v-if="filteredAlpha"
              :version="filteredAlpha"
              @on-download="onDownload"
              @on-navigate="downloadModal.hide"
            />
            <p
              v-if="
                currentPlatform &&
                currentGameVersion &&
                !filteredRelease &&
                !filteredBeta &&
                !filteredAlpha
              "
            >
              No versions available for {{ currentGameVersion }} and
              {{ formatCategory(currentPlatform) }}.
            </p>
          </AutomaticAccordion>
        </div>
      </template>
    </NewModal>
    <CollectionCreateModal ref="modal_collection" :project-ids="[project.id]" />
    <div
      class="new-page sidebar"
      :class="{
        'alt-layout': cosmetics.leftContentLayout,
      }"
    >
      <div class="normal-page__header relative my-4">
        <ContentPageHeader>
          <template #icon>
            <Avatar :src="project.icon_url" :alt="project.title" size="96px" />
          </template>
          <template #title>
            {{ project.title }}
          </template>
          <template #title-suffix>
            <Badge v-if="auth.user && currentMember" :type="project.status" class="status-badge" />
          </template>
          <template #summary>
            {{ project.description }}
          </template>
          <template #stats>
            <div
              class="flex items-center gap-2 border-0 border-r border-solid border-button-bg pr-4 font-semibold"
            >
              <DownloadIcon class="h-6 w-6 text-secondary" />
              {{ $formatNumber(project.downloads) }}
            </div>
            <div
              class="flex items-center gap-2 border-0 border-solid border-button-bg pr-4 md:border-r"
            >
              <HeartIcon class="h-6 w-6 text-secondary" />
              <span class="font-semibold">
                {{ $formatNumber(project.followers) }}
              </span>
            </div>
            <div class="hidden items-center gap-2 md:flex">
              <TagsIcon class="h-6 w-6 text-secondary" />
              <div class="flex flex-wrap gap-2">
                <div
                  v-for="(category, index) in project.categories"
                  :key="index"
                  class="tag-list__item"
                >
                  {{ formatCategory(category) }}
                </div>
              </div>
            </div>
          </template>
          <template #actions>
            <div class="hidden sm:contents">
              <ButtonStyled
                size="large"
                :color="route.name === 'type-id-version-version' ? `standard` : `brand`"
              >
                <button @click="(event) => downloadModal.show(event)">
                  <DownloadIcon aria-hidden="true" />
                  Download
                </button>
              </ButtonStyled>
            </div>
            <div class="contents sm:hidden">
              <ButtonStyled
                size="large"
                circular
                :color="route.name === 'type-id-version-version' ? `standard` : `brand`"
              >
                <button
                  aria-label="Download"
                  class="flex sm:hidden"
                  @click="(event) => downloadModal.show(event)"
                >
                  <DownloadIcon aria-hidden="true" />
                </button>
              </ButtonStyled>
            </div>
            <ButtonStyled
              size="large"
              circular
              :color="following ? 'red' : 'standard'"
              color-fill="none"
              hover-color-fill="background"
            >
              <button
                v-if="auth.user"
                v-tooltip="following ? `Unfollow` : `Follow`"
                :aria-label="following ? `Unfollow` : `Follow`"
                @click="userFollowProject(project)"
              >
                <HeartIcon :fill="following ? 'currentColor' : 'none'" aria-hidden="true" />
              </button>
              <nuxt-link v-else v-tooltip="'Follow'" to="/auth/sign-in" aria-label="Follow">
                <HeartIcon aria-hidden="true" />
              </nuxt-link>
            </ButtonStyled>
            <ButtonStyled size="large" circular>
              <PopoutMenu v-if="auth.user" v-tooltip="'Save'" from="top-right" aria-label="Save">
                <BookmarkIcon
                  aria-hidden="true"
                  :fill="
                    collections.some((x) => x.projects.includes(project.id))
                      ? 'currentColor'
                      : 'none'
                  "
                />
                <template #menu>
                  <input
                    v-model="displayCollectionsSearch"
                    type="text"
                    placeholder="Search collections..."
                    class="search-input menu-search"
                  />
                  <div v-if="collections.length > 0" class="collections-list">
                    <Checkbox
                      v-for="option in collections
                        .slice()
                        .sort((a, b) => a.name.localeCompare(b.name))"
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
                  <button
                    class="btn collection-button"
                    @click="(event) => $refs.modal_collection.show(event)"
                  >
                    <PlusIcon aria-hidden="true" />
                    Create new collection
                  </button>
                </template>
              </PopoutMenu>
              <nuxt-link v-else v-tooltip="'Save'" to="/auth/sign-in" aria-label="Save">
                <BookmarkIcon aria-hidden="true" />
              </nuxt-link>
            </ButtonStyled>
            <ButtonStyled v-if="auth.user && currentMember" size="large" circular>
              <nuxt-link
                :to="`/${project.project_type}/${project.slug ? project.slug : project.id}/settings`"
              >
                <SettingsIcon aria-hidden="true" />
              </nuxt-link>
            </ButtonStyled>
            <ButtonStyled size="large" circular type="transparent">
              <OverflowMenu
                :options="[
                  {
                    id: 'analytics',
                    link: `/${project.project_type}/${project.slug ? project.slug : project.id}/settings/analytics`,
                    hoverOnly: true,
                    shown: auth.user && !!currentMember,
                  },
                  {
                    divider: true,
                    shown: auth.user && !!currentMember,
                  },
                  {
                    id: 'moderation-checklist',
                    action: () => (showModerationChecklist = true),
                    color: 'orange',
                    hoverOnly: true,
                    shown:
                      auth.user &&
                      tags.staffRoles.includes(auth.user.role) &&
                      !showModerationChecklist,
                  },
                  {
                    divider: true,
                    shown:
                      auth.user &&
                      tags.staffRoles.includes(auth.user.role) &&
                      !showModerationChecklist,
                  },
                  {
                    id: 'report',
                    action: () =>
                      auth.user ? reportProject(project.id) : navigateTo('/auth/sign-in'),
                    color: 'red',
                    hoverOnly: true,
                  },
                  { id: 'copy-id', action: () => copyId() },
                ]"
                aria-label="More options"
              >
                <MoreVerticalIcon aria-hidden="true" />
                <template #analytics>
                  <ChartIcon aria-hidden="true" />
                  Analytics
                </template>
                <template #moderation-checklist>
                  <ScaleIcon aria-hidden="true" />
                  Review project
                </template>
                <template #report>
                  <ReportIcon aria-hidden="true" />
                  Report
                </template>
                <template #copy-id>
                  <ClipboardCopyIcon aria-hidden="true" />
                  Copy ID
                </template>
              </OverflowMenu>
            </ButtonStyled>
          </template>
        </ContentPageHeader>
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
        <MessageBanner v-if="project.status === 'archived'" message-type="warning" class="mb-4">
          {{ project.title }} has been archived. {{ project.title }} will not receive any further
          updates unless the author decides to unarchive the project.
        </MessageBanner>
      </div>
      <div class="normal-page__sidebar">
        <div v-if="versions.length > 0" class="card flex-card experimental-styles-within">
          <h2>{{ formatMessage(compatibilityMessages.title) }}</h2>
          <section>
            <h3>{{ formatMessage(compatibilityMessages.minecraftJava) }}</h3>
            <div class="tag-list">
              <div
                v-for="version in getVersionsToDisplay(project)"
                :key="`version-tag-${version}`"
                class="tag-list__item"
              >
                {{ version }}
              </div>
            </div>
          </section>
          <section v-if="project.project_type !== 'resourcepack'">
            <h3>{{ formatMessage(compatibilityMessages.platforms) }}</h3>
            <div class="tag-list">
              <div
                v-for="platform in project.loaders"
                :key="`platform-tag-${platform}`"
                :class="`tag-list__item`"
                :style="`--_color: var(--color-platform-${platform})`"
              >
                <svg v-html="tags.loaders.find((x) => x.name === platform).icon"></svg>
                {{ formatCategory(platform) }}
              </div>
            </div>
          </section>
          <section
            v-if="
              (project.actualProjectType === 'mod' || project.project_type === 'modpack') &&
              !(project.client_side === 'unsupported' && project.server_side === 'unsupported') &&
              !(project.client_side === 'unknown' && project.server_side === 'unknown')
            "
          >
            <h3>{{ formatMessage(compatibilityMessages.environments) }}</h3>
            <div class="tag-list">
              <div
                v-if="
                  (project.client_side === 'required' && project.server_side !== 'required') ||
                  (project.client_side === 'optional' && project.server_side === 'optional')
                "
                class="tag-list__item"
              >
                <ClientIcon aria-hidden="true" />
                Client-side
              </div>
              <div
                v-if="
                  (project.server_side === 'required' && project.client_side !== 'required') ||
                  (project.client_side === 'optional' && project.server_side === 'optional')
                "
                class="tag-list__item"
              >
                <ServerIcon aria-hidden="true" />
                Server-side
              </div>
              <div v-if="false" class="tag-list__item">
                <UserIcon aria-hidden="true" />
                Singleplayer
              </div>
              <div
                v-if="
                  project.project_type !== 'datapack' &&
                  ((project.client_side === 'required' && project.server_side === 'required') ||
                    project.client_side === 'optional' ||
                    (project.client_side === 'required' && project.server_side === 'optional') ||
                    project.server_side === 'optional' ||
                    (project.server_side === 'required' && project.client_side === 'optional'))
                "
                class="tag-list__item"
              >
                <MonitorSmartphoneIcon aria-hidden="true" />
                Client and server
              </div>
            </div>
          </section>
        </div>
        <AdPlaceholder
          v-if="
            (!auth.user || !isPermission(auth.user.badges, 1 << 0) || flags.showAdsWithPlus) &&
            tags.approvedStatuses.includes(project.status)
          "
        />
        <div
          v-if="
            project.issues_url ||
            project.source_url ||
            project.wiki_url ||
            project.discord_url ||
            project.donation_urls.length > 0
          "
          class="card flex-card experimental-styles-within"
        >
          <h2>{{ formatMessage(linksMessages.title) }}</h2>
          <div class="links-list">
            <a
              v-if="project.issues_url"
              :href="project.issues_url"
              :target="$external()"
              rel="noopener nofollow ugc"
            >
              <IssuesIcon aria-hidden="true" />
              {{ formatMessage(linksMessages.issues) }}
              <ExternalIcon aria-hidden="true" class="external-icon" />
            </a>
            <a
              v-if="project.source_url"
              :href="project.source_url"
              :target="$external()"
              rel="noopener nofollow ugc"
            >
              <CodeIcon aria-hidden="true" />
              {{ formatMessage(linksMessages.source) }}
              <ExternalIcon aria-hidden="true" class="external-icon" />
            </a>
            <a
              v-if="project.wiki_url"
              :href="project.wiki_url"
              :target="$external()"
              rel="noopener nofollow ugc"
            >
              <WikiIcon aria-hidden="true" />
              {{ formatMessage(linksMessages.wiki) }}
              <ExternalIcon aria-hidden="true" class="external-icon" />
            </a>
            <a
              v-if="project.discord_url"
              :href="project.discord_url"
              :target="$external()"
              rel="noopener nofollow ugc"
            >
              <DiscordIcon class="shrink" aria-hidden="true" />
              {{ formatMessage(linksMessages.discord) }}
              <ExternalIcon aria-hidden="true" class="external-icon" />
            </a>
            <hr
              v-if="
                (project.issues_url ||
                  project.source_url ||
                  project.wiki_url ||
                  project.discord_url) &&
                project.donation_urls.length > 0
              "
            />
            <a
              v-for="(donation, index) in project.donation_urls"
              :key="index"
              :href="donation.url"
              :target="$external()"
              rel="noopener nofollow ugc"
            >
              <BuyMeACoffeeIcon v-if="donation.id === 'bmac'" aria-hidden="true" />
              <PatreonIcon v-else-if="donation.id === 'patreon'" aria-hidden="true" />
              <KoFiIcon v-else-if="donation.id === 'ko-fi'" aria-hidden="true" />
              <PayPalIcon v-else-if="donation.id === 'paypal'" aria-hidden="true" />
              <OpenCollectiveIcon
                v-else-if="donation.id === 'open-collective'"
                aria-hidden="true"
              />
              <HeartIcon v-else-if="donation.id === 'github'" />
              <CurrencyIcon v-else />
              <span v-if="donation.id === 'bmac'">{{
                formatMessage(linksMessages.donateBmac)
              }}</span>
              <span v-else-if="donation.id === 'patreon'">{{
                formatMessage(linksMessages.donatePatreon)
              }}</span>
              <span v-else-if="donation.id === 'paypal'">{{
                formatMessage(linksMessages.donatePayPal)
              }}</span>
              <span v-else-if="donation.id === 'ko-fi'">{{
                formatMessage(linksMessages.donateKoFi)
              }}</span>
              <span v-else-if="donation.id === 'github'">{{
                formatMessage(linksMessages.donateGithub)
              }}</span>
              <span v-else>{{ formatMessage(linksMessages.donateGeneric) }}</span>
              <ExternalIcon aria-hidden="true" class="external-icon" />
            </a>
          </div>
        </div>
        <div class="card flex-card experimental-styles-within">
          <h2>{{ formatMessage(creatorsMessages.title) }}</h2>
          <div class="details-list">
            <template v-if="organization">
              <nuxt-link
                class="details-list__item details-list__item--type-large"
                :to="`/organization/${organization.slug}`"
              >
                <Avatar :src="organization.icon_url" :alt="organization.name" size="32px" />
                <div class="rows">
                  <span>
                    {{ organization.name }}
                  </span>
                  <span class="details-list__item__text--style-secondary">Organization</span>
                </div>
              </nuxt-link>
              <hr v-if="members.length > 0" />
            </template>
            <nuxt-link
              v-for="member in members"
              :key="`member-${member.id}`"
              class="details-list__item details-list__item--type-large"
              :to="'/user/' + member.user.username"
            >
              <Avatar :src="member.avatar_url" :alt="member.name" size="32px" circle />
              <div class="rows">
                <span class="flex items-center gap-1">
                  {{ member.name }}
                  <CrownIcon
                    v-if="member.is_owner"
                    v-tooltip="formatMessage(creatorsMessages.owner)"
                    class="text-brand-orange"
                  />
                </span>
                <span class="details-list__item__text--style-secondary">{{ member.role }}</span>
              </div>
            </nuxt-link>
          </div>
        </div>
        <div class="card flex-card experimental-styles-within">
          <h2>{{ formatMessage(detailsMessages.title) }}</h2>
          <div class="details-list">
            <div class="details-list__item">
              <BookTextIcon aria-hidden="true" />
              <div>
                Licensed
                <a
                  v-if="project.license.url"
                  class="text-link hover:underline"
                  :href="project.license.url"
                  :target="$external()"
                  rel="noopener nofollow ugc"
                >
                  {{ licenseIdDisplay }}
                  <ExternalIcon aria-hidden="true" class="external-icon ml-1 mt-[-1px] inline" />
                </a>
                <span
                  v-else-if="
                    project.license.id === 'LicenseRef-All-Rights-Reserved' ||
                    !project.license.id.includes('LicenseRef')
                  "
                  class="text-link hover:underline"
                  @click="(event) => getLicenseData(event)"
                >
                  {{ licenseIdDisplay }}
                </span>
                <span v-else>{{ licenseIdDisplay }}</span>
              </div>
            </div>
            <div
              v-if="project.approved"
              v-tooltip="$dayjs(project.approved).format('MMMM D, YYYY [at] h:mm A')"
              class="details-list__item"
            >
              <CalendarIcon aria-hidden="true" />
              <div>
                {{ formatMessage(detailsMessages.published, { date: publishedDate }) }}
              </div>
            </div>
            <div
              v-else
              v-tooltip="$dayjs(project.published).format('MMMM D, YYYY [at] h:mm A')"
              class="details-list__item"
            >
              <CalendarIcon aria-hidden="true" />
              <div>
                {{ formatMessage(detailsMessages.created, { date: createdDate }) }}
              </div>
            </div>
            <div
              v-if="project.status === 'processing' && project.queued"
              v-tooltip="$dayjs(project.queued).format('MMMM D, YYYY [at] h:mm A')"
              class="details-list__item"
            >
              <ScaleIcon aria-hidden="true" />
              <div>
                {{ formatMessage(detailsMessages.submitted, { date: submittedDate }) }}
              </div>
            </div>
            <div
              v-if="versions.length > 0 && project.updated"
              v-tooltip="$dayjs(project.updated).format('MMMM D, YYYY [at] h:mm A')"
              class="details-list__item"
            >
              <VersionIcon aria-hidden="true" />
              <div>
                {{ formatMessage(detailsMessages.updated, { date: updatedDate }) }}
              </div>
            </div>
          </div>
        </div>
      </div>
      <div class="normal-page__content">
        <div class="overflow-x-auto">
          <NavTabs :links="navLinks" class="mb-4" />
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
          @on-download="triggerDownloadAnimation"
        />
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
  ScaleIcon,
  AlignLeftIcon as DescriptionIcon,
  BookmarkIcon,
  ChartIcon,
  CheckIcon,
  ClipboardCopyIcon,
  CopyrightIcon,
  DownloadIcon,
  ExternalIcon,
  GameIcon,
  HeartIcon,
  ImageIcon as GalleryIcon,
  InfoIcon,
  LinkIcon as LinksIcon,
  MoreVerticalIcon,
  PlusIcon,
  ReportIcon,
  SearchIcon,
  SettingsIcon,
  TagsIcon,
  UsersIcon,
  VersionIcon,
  WrenchIcon,
  ClientIcon,
  BookTextIcon,
  MonitorSmartphoneIcon,
  WikiIcon,
  DiscordIcon,
  CalendarIcon,
  KoFiIcon,
  BuyMeACoffeeIcon,
  IssuesIcon,
  UserIcon,
  PayPalIcon,
  ServerIcon,
  PatreonIcon,
  CrownIcon,
  OpenCollectiveIcon,
  CodeIcon,
  CurrencyIcon,
} from "@modrinth/assets";
import {
  Avatar,
  ButtonStyled,
  Checkbox,
  NewModal,
  OverflowMenu,
  PopoutMenu,
  ScrollablePanel,
  ContentPageHeader,
} from "@modrinth/ui";
import { formatCategory, isRejected, isStaff, isUnderReview, renderString } from "@modrinth/utils";
import dayjs from "dayjs";
import Badge from "~/components/ui/Badge.vue";
import NavTabs from "~/components/ui/NavTabs.vue";
import NavStack from "~/components/ui/NavStack.vue";
import NavStackItem from "~/components/ui/NavStackItem.vue";
import ProjectMemberHeader from "~/components/ui/ProjectMemberHeader.vue";
import MessageBanner from "~/components/ui/MessageBanner.vue";
import { reportProject } from "~/utils/report-helpers.ts";
import Breadcrumbs from "~/components/ui/Breadcrumbs.vue";
import { userCollectProject } from "~/composables/user.js";
import CollectionCreateModal from "~/components/ui/CollectionCreateModal.vue";
import ModerationChecklist from "~/components/ui/ModerationChecklist.vue";
import Accordion from "~/components/ui/Accordion.vue";
import ModrinthIcon from "~/assets/images/utils/modrinth.svg?component";
import VersionSummary from "~/components/ui/VersionSummary.vue";
import AutomaticAccordion from "~/components/ui/AutomaticAccordion.vue";
import { getVersionsToDisplay } from "~/helpers/projects.js";
import AdPlaceholder from "~/components/ui/AdPlaceholder.vue";

const data = useNuxtApp();
const route = useNativeRoute();

const auth = await useAuth();
const user = await useUser();

const tags = useTags();
const flags = useFeatureFlags();
const cosmetics = useCosmetics();

const { formatMessage } = useVIntl();

const settingsModal = ref();
const downloadModal = ref();
const overTheTopDownloadAnimation = ref();

const userSelectedGameVersion = ref(null);
const userSelectedPlatform = ref(null);
const showAllVersions = ref(false);

const gameVersionFilterInput = ref();

const versionFilter = ref("");

const currentGameVersion = computed(() => {
  return (
    userSelectedGameVersion.value ||
    (project.value.game_versions.length === 1 && project.value.game_versions[0])
  );
});

const possibleGameVersions = computed(() => {
  return versions.value
    .filter((x) => !currentPlatform.value || x.loaders.includes(currentPlatform.value))
    .flatMap((x) => x.game_versions);
});

const possiblePlatforms = computed(() => {
  return versions.value
    .filter((x) => !currentGameVersion.value || x.game_versions.includes(currentGameVersion.value))
    .flatMap((x) => x.loaders);
});

const currentPlatform = computed(() => {
  return (
    userSelectedPlatform.value || (project.value.loaders.length === 1 && project.value.loaders[0])
  );
});

function installWithApp() {
  setTimeout(() => {
    getModrinthAppAccordion.value.open();
  }, 1500);
}

const gameVersionAccordion = ref();
const platformAccordion = ref();
const getModrinthAppAccordion = ref();

const formatRelativeTime = useRelativeTime();

const compatibilityMessages = defineMessages({
  title: {
    id: "project.about.compatibility.title",
    defaultMessage: "Compatibility",
  },
  minecraftJava: {
    id: "project.about.compatibility.game.minecraftJava",
    defaultMessage: "Minecraft: Java Edition",
  },
  platforms: {
    id: "project.about.compatibility.platforms",
    defaultMessage: "Platforms",
  },
  environments: {
    id: "project.about.compatibility.environments",
    defaultMessage: "Supported environments",
  },
});
const linksMessages = defineMessages({
  title: {
    id: "project.about.links.title",
    defaultMessage: "Links",
  },
  issues: {
    id: "project.about.links.issues",
    defaultMessage: "Report issues",
  },
  source: {
    id: "project.about.links.source",
    defaultMessage: "View source",
  },
  wiki: {
    id: "project.about.links.wiki",
    defaultMessage: "Visit wiki",
  },
  discord: {
    id: "project.about.links.discord",
    defaultMessage: "Join Discord server",
  },
  donateGeneric: {
    id: "project.about.links.donate.generic",
    defaultMessage: "Donate",
  },
  donateGitHub: {
    id: "project.about.links.donate.github",
    defaultMessage: "Sponsor on GitHub",
  },
  donateBmac: {
    id: "project.about.links.donate.bmac",
    defaultMessage: "Buy Me a Coffee",
  },
  donatePatreon: {
    id: "project.about.links.donate.patreon",
    defaultMessage: "Donate on Patreon",
  },
  donatePayPal: {
    id: "project.about.links.donate.paypal",
    defaultMessage: "Donate on PayPal",
  },
  donateKoFi: {
    id: "project.about.links.donate.kofi",
    defaultMessage: "Donate on Ko-fi",
  },
  donateGithub: {
    id: "project.about.links.donate.github",
    defaultMessage: "Sponsor on GitHub",
  },
});
const creatorsMessages = defineMessages({
  title: {
    id: "project.about.creators.title",
    defaultMessage: "Creators",
  },
  owner: {
    id: "project.about.creators.owner",
    defaultMessage: "Project owner",
  },
});
const detailsMessages = defineMessages({
  title: {
    id: "project.about.details.title",
    defaultMessage: "Details",
  },
  licensed: {
    id: "project.about.details.licensed",
    defaultMessage: "Licensed {license}",
  },
  created: {
    id: "project.about.details.created",
    defaultMessage: "Created {date}",
  },
  submitted: {
    id: "project.about.details.submitted",
    defaultMessage: "Submitted {date}",
  },
  published: {
    id: "project.about.details.published",
    defaultMessage: "Published {date}",
  },
  updated: {
    id: "project.about.details.updated",
    defaultMessage: "Updated {date}",
  },
});

const modalLicense = ref(null);
const licenseText = ref("");

const createdDate = computed(() =>
  project.value.published ? formatRelativeTime(project.value.published) : "unknown",
);
const submittedDate = computed(() =>
  project.value.queued ? formatRelativeTime(project.value.queued) : "unknown",
);
const publishedDate = computed(() =>
  project.value.approved ? formatRelativeTime(project.value.approved) : "unknown",
);
const updatedDate = computed(() =>
  project.value.updated ? formatRelativeTime(project.value.updated) : "unknown",
);

const licenseIdDisplay = computed(() => {
  const id = project.value.license.id;

  if (id === "LicenseRef-All-Rights-Reserved") {
    return "ARR";
  } else if (id.includes("LicenseRef")) {
    return id.replaceAll("LicenseRef-", "").replaceAll("-", " ");
  } else {
    return id;
  }
});

async function getLicenseData(event) {
  modalLicense.value.show(event);

  try {
    const text = await useBaseFetch(`tag/license/${project.value.license.id}`);
    licenseText.value = text.body || "License text could not be retrieved.";
  } catch {
    licenseText.value = "License text could not be retrieved.";
  }
}

const filteredVersions = computed(() => {
  return versions.value.filter(
    (x) =>
      x.game_versions.includes(currentGameVersion.value) &&
      x.loaders.includes(currentPlatform.value),
  );
});

const filteredRelease = computed(() => {
  return filteredVersions.value.find((x) => x.version_type === "release");
});

const filteredBeta = computed(() => {
  return filteredVersions.value.find(
    (x) =>
      x.version_type === "beta" &&
      (!filteredRelease.value ||
        dayjs(x.date_published).isAfter(dayjs(filteredRelease.value.date_published))),
  );
});

const filteredAlpha = computed(() => {
  return filteredVersions.value.find(
    (x) =>
      x.version_type === "alpha" &&
      (!filteredRelease.value ||
        dayjs(x.date_published).isAfter(dayjs(filteredRelease.value.date_published))) &&
      (!filteredBeta.value ||
        dayjs(x.date_published).isAfter(dayjs(filteredBeta.value.date_published))),
  );
});

const messages = defineMessages({
  downloadsStat: {
    id: "project.stats.downloads-label",
    defaultMessage: "download{count, plural, one {} other {s}}",
  },
  followersStat: {
    id: "project.stats.followers-label",
    defaultMessage: "follower{count, plural, one {} other {s}}",
  },
  descriptionTab: {
    id: "project.description.title",
    defaultMessage: "Description",
  },
  galleryTab: {
    id: "project.gallery.title",
    defaultMessage: "Gallery",
  },
  versionsTab: {
    id: "project.versions.title",
    defaultMessage: "Versions",
  },
  moderationTab: {
    id: "project.moderation.title",
    defaultMessage: "Moderation",
  },
});

const displayCollectionsSearch = ref("");
const collections = computed(() =>
  user.value && user.value.collections
    ? user.value.collections.filter((x) =>
        x.name.toLowerCase().includes(displayCollectionsSearch.value.toLowerCase()),
      )
    : [],
);

if (
  !route.params.id ||
  !(
    tags.value.projectTypes.find((x) => x.id === route.params.type) ||
    route.params.type === "project"
  )
) {
  throw createError({
    fatal: true,
    statusCode: 404,
    message: "The page could not be found",
  });
}

let project,
  resetProject,
  allMembers,
  resetMembers,
  dependencies,
  featuredVersions,
  versions,
  organization,
  resetOrganization;
try {
  [
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
          project.actualProjectType = JSON.parse(JSON.stringify(project.project_type));
          project.project_type = data.$getProjectTypeForUrl(
            project.project_type,
            project.loaders,
            tags.value,
          );
        }

        return project;
      },
    }),
    useAsyncData(
      `project/${route.params.id}/members`,
      () => useBaseFetch(`project/${route.params.id}/members`, { apiVersion: 3 }),
      {
        transform: (members) => {
          members.forEach((it, index) => {
            members[index].avatar_url = it.user.avatar_url;
            members[index].name = it.user.username;
          });

          return members;
        },
      },
    ),
    useAsyncData(`project/${route.params.id}/dependencies`, () =>
      useBaseFetch(`project/${route.params.id}/dependencies`),
    ),
    useAsyncData(`project/${route.params.id}/version?featured=true`, () =>
      useBaseFetch(`project/${route.params.id}/version?featured=true`),
    ),
    useAsyncData(`project/${route.params.id}/version`, () =>
      useBaseFetch(`project/${route.params.id}/version`),
    ),
    useAsyncData(`project/${route.params.id}/organization`, () =>
      useBaseFetch(`project/${route.params.id}/organization`, { apiVersion: 3 }),
    ),
  ]);

  versions = shallowRef(toRaw(versions));
  featuredVersions = shallowRef(toRaw(featuredVersions));
} catch {
  throw createError({
    fatal: true,
    statusCode: 404,
    message: "Project not found",
  });
}

if (!project.value) {
  throw createError({
    fatal: true,
    statusCode: 404,
    message: "Project not found",
  });
}

if (project.value.project_type !== route.params.type || route.params.id !== project.value.slug) {
  let path = route.fullPath.split("/");
  path.splice(0, 3);
  path = path.filter((x) => x);

  await navigateTo(
    `/${project.value.project_type}/${project.value.slug}${
      path.length > 0 ? `/${path.join("/")}` : ""
    }`,
    { redirectCode: 301, replace: true },
  );
}

// Members should be an array of all members, without the accepted ones, and with the user with the Owner role at the start
// The rest of the members should be sorted by role, then by name
const members = computed(() => {
  const acceptedMembers = allMembers.value.filter((x) => x.accepted);
  const owner = acceptedMembers.find((x) =>
    organization.value
      ? organization.value.members.some(
          (orgMember) => orgMember.user.id === x.user.id && orgMember.is_owner,
        )
      : x.is_owner,
  );

  const rest = acceptedMembers.filter((x) => !owner || x.user.id !== owner.user.id) || [];

  rest.sort((a, b) => {
    if (a.role === b.role) {
      return a.user.username.localeCompare(b.user.username);
    } else {
      return a.role.localeCompare(b.role);
    }
  });

  return owner ? [owner, ...rest] : rest;
});

const currentMember = computed(() => {
  let val = auth.value.user ? allMembers.value.find((x) => x.user.id === auth.value.user.id) : null;

  if (!val && auth.value.user && organization.value && organization.value.members) {
    val = organization.value.members.find((x) => x.user.id === auth.value.user.id);
  }

  if (!val && auth.value.user && tags.value.staffRoles.includes(auth.value.user.role)) {
    val = {
      team_id: project.team_id,
      user: auth.value.user,
      role: auth.value.role,
      permissions: auth.value.user.role === "admin" ? 1023 : 12,
      accepted: true,
      payouts_split: 0,
      avatar_url: auth.value.user.avatar_url,
      name: auth.value.user.username,
    };
  }

  return val;
});

versions.value = data.$computeVersions(versions.value, allMembers.value);

// Q: Why do this instead of computing the versions of featuredVersions?
// A: It will incorrectly generate the version slugs because it doesn't have the full context of
//    all the versions. For example, if version 1.1.0 for Forge is featured but 1.1.0 for Fabric
//    is not, but the Fabric one was uploaded first, the Forge version would link to the Fabric
///   version
const featuredIds = featuredVersions.value.map((x) => x.id);
featuredVersions.value = versions.value.filter((version) => featuredIds.includes(version.id));

featuredVersions.value.sort((a, b) => {
  const aLatest = a.game_versions[a.game_versions.length - 1];
  const bLatest = b.game_versions[b.game_versions.length - 1];
  const gameVersions = tags.value.gameVersions.map((e) => e.version);
  return gameVersions.indexOf(aLatest) - gameVersions.indexOf(bLatest);
});

const projectTypeDisplay = computed(() =>
  data.$formatProjectType(
    data.$getProjectTypeForDisplay(project.value.project_type, project.value.loaders),
  ),
);

const following = computed(
  () =>
    user.value && user.value.follows && user.value.follows.find((x) => x.id === project.value.id),
);

const title = computed(() => `${project.value.title} - Minecraft ${projectTypeDisplay.value}`);
const description = computed(
  () =>
    `${project.value.description} - Download the Minecraft ${projectTypeDisplay.value} ${
      project.value.title
    } by ${members.value.find((x) => x.is_owner)?.user?.username || "a Creator"} on Modrinth`,
);

if (!route.name.startsWith("type-id-settings")) {
  useSeoMeta({
    title: () => title.value,
    description: () => description.value,
    ogTitle: () => title.value,
    ogDescription: () => project.value.description,
    ogImage: () => project.value.icon_url ?? "https://cdn.modrinth.com/placeholder.png",
    robots: () =>
      project.value.status === "approved" || project.value.status === "archived"
        ? "all"
        : "noindex",
  });
}

const onUserCollectProject = useClientTry(userCollectProject);

async function setProcessing() {
  startLoading();

  try {
    await useBaseFetch(`project/${project.value.id}`, {
      method: "PATCH",
      body: {
        status: "processing",
      },
    });

    project.value.status = "processing";
  } catch (err) {
    data.$notify({
      group: "main",
      title: "An error occurred",
      text: err.data.description,
      type: "error",
    });
  }

  stopLoading();
}

async function patchProject(resData, quiet = false) {
  let result = false;
  startLoading();

  try {
    await useBaseFetch(`project/${project.value.id}`, {
      method: "PATCH",
      body: resData,
    });

    for (const key in resData) {
      project.value[key] = resData[key];
    }

    if (resData.license_id) {
      project.value.license.id = resData.license_id;
    }
    if (resData.license_url) {
      project.value.license.url = resData.license_url;
    }

    result = true;
    if (!quiet) {
      data.$notify({
        group: "main",
        title: "Project updated",
        text: "Your project has been updated.",
        type: "success",
      });
      window.scrollTo({ top: 0, behavior: "smooth" });
    }
  } catch (err) {
    data.$notify({
      group: "main",
      title: "An error occurred",
      text: err.data.description,
      type: "error",
    });
    window.scrollTo({ top: 0, behavior: "smooth" });
  }

  stopLoading();

  return result;
}

async function patchIcon(icon) {
  let result = false;
  startLoading();

  try {
    await useBaseFetch(
      `project/${project.value.id}/icon?ext=${
        icon.type.split("/")[icon.type.split("/").length - 1]
      }`,
      {
        method: "PATCH",
        body: icon,
      },
    );
    await resetProject();
    result = true;
    data.$notify({
      group: "main",
      title: "Project icon updated",
      text: "Your project's icon has been updated.",
      type: "success",
    });
  } catch (err) {
    data.$notify({
      group: "main",
      title: "An error occurred",
      text: err.data.description,
      type: "error",
    });

    window.scrollTo({ top: 0, behavior: "smooth" });
  }

  stopLoading();
  return result;
}

async function updateMembers() {
  allMembers.value = await useAsyncData(
    `project/${route.params.id}/members`,
    () => useBaseFetch(`project/${route.params.id}/members`),
    {
      transform: (members) => {
        members.forEach((it, index) => {
          members[index].avatar_url = it.user.avatar_url;
          members[index].name = it.user.username;
        });

        return members;
      },
    },
  );
}

async function copyId() {
  await navigator.clipboard.writeText(project.value.id);
}

const collapsedChecklist = ref(false);

const showModerationChecklist = ref(false);
const futureProjects = ref([]);
if (import.meta.client && history && history.state && history.state.showChecklist) {
  showModerationChecklist.value = true;
  futureProjects.value = history.state.projects;
}

function closeDownloadModal(event) {
  downloadModal.value.hide(event);
  userSelectedPlatform.value = null;
  userSelectedGameVersion.value = null;
  showAllVersions.value = false;
}

function triggerDownloadAnimation() {
  overTheTopDownloadAnimation.value = true;
  setTimeout(() => (overTheTopDownloadAnimation.value = false), 500);
}

function onDownload(event) {
  triggerDownloadAnimation();
  setTimeout(() => {
    closeDownloadModal(event);
  }, 400);
}

const navLinks = computed(() => {
  const projectUrl = `/${project.value.project_type}/${project.value.slug ? project.value.slug : project.value.id}`;

  return [
    {
      label: formatMessage(messages.descriptionTab),
      href: projectUrl,
    },
    {
      label: formatMessage(messages.galleryTab),
      href: `${projectUrl}/gallery`,
      shown: project.value.gallery.length > 0 || !!currentMember.value,
    },
    {
      label: "Changelog",
      href: `${projectUrl}/changelog`,
      shown: versions.value.length > 0,
    },
    {
      label: formatMessage(messages.versionsTab),
      href: `${projectUrl}/versions`,
      shown: versions.value.length > 0 || !!currentMember.value,
      subpages: [`${projectUrl}/version/`],
    },
    {
      label: formatMessage(messages.moderationTab),
      href: `${projectUrl}/moderation`,
      shown:
        !!currentMember.value &&
        (isRejected(project.value) || isUnderReview(project.value) || isStaff(auth.value.user)),
    },
  ];
});
</script>
<style lang="scss" scoped>
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

.normal-page__info:empty {
  display: none;
}

:deep(.accordion-with-bg) {
  @apply rounded-2xl bg-bg p-2;
  --scrollable-pane-bg: var(--color-bg);
}

.over-the-top-download-animation {
  position: fixed;
  z-index: 100;
  inset: 0;
  display: flex;
  justify-content: center;
  align-items: center;
  pointer-events: none;
  scale: 0.5;
  transition: all 0.5s ease-out;
  opacity: 1;

  &.animation-hidden {
    scale: 0.8;
    opacity: 0;

    .animation-ring-1 {
      width: 25rem;
      height: 25rem;
    }
    .animation-ring-2 {
      width: 50rem;
      height: 50rem;
    }
    .animation-ring-3 {
      width: 100rem;
      height: 100rem;
    }
  }

  > div {
    position: relative;
    display: flex;
    justify-content: center;
    align-items: center;
    width: fit-content;
    height: fit-content;

    > * {
      position: absolute;
      scale: 1;
      transition: all 0.2s ease-out;
      width: 20rem;
      height: 20rem;
    }
  }
}

@media (hover: none) and (max-width: 767px) {
  .modrinth-app-section {
    display: none;
  }
}
</style>
