<template>
  <div class="pointer-events-none fixed inset-0 z-[-1]">
    <div id="fixed-background-teleport" class="relative"></div>
  </div>
  <div class="pointer-events-none absolute inset-0 z-[-1]">
    <div id="absolute-background-teleport" class="relative"></div>
  </div>
  <div ref="main_page" class="layout" :class="{ 'expanded-mobile-nav': isBrowseMenuOpen }">
    <div
      v-if="auth.user && !auth.user.email_verified && route.path !== '/auth/verify-email'"
      class="email-nag"
    >
      <template v-if="auth.user.email">
        <span>{{ formatMessage(verifyEmailBannerMessages.title) }}</span>
        <button class="btn" @click="resendVerifyEmail">
          {{ formatMessage(verifyEmailBannerMessages.action) }}
        </button>
      </template>
      <template v-else>
        <span>{{ formatMessage(addEmailBannerMessages.title) }}</span>
        <nuxt-link class="btn" to="/settings/account">
          <SettingsIcon aria-hidden="true" />
          {{ formatMessage(addEmailBannerMessages.action) }}
        </nuxt-link>
      </template>
    </div>
    <div
      v-if="
        user &&
        user.subscriptions &&
        user.subscriptions.some((x) => x.status === 'payment-failed') &&
        route.path !== '/settings/billing'
      "
      class="email-nag"
    >
      <span>{{ formatMessage(subscriptionPaymentFailedBannerMessages.title) }}</span>
      <nuxt-link class="btn" to="/settings/billing">
        <SettingsIcon aria-hidden="true" />
        {{ formatMessage(subscriptionPaymentFailedBannerMessages.action) }}
      </nuxt-link>
    </div>
    <div
      v-if="
        config.public.apiBaseUrl.startsWith('https://staging-api.modrinth.com') &&
        !cosmetics.hideStagingBanner
      "
      class="site-banner site-banner--warning [&>*]:z-[6]"
    >
      <div class="site-banner__title">
        <IssuesIcon aria-hidden="true" />
        <span>{{ formatMessage(stagingBannerMessages.title) }}</span>
      </div>
      <div class="site-banner__description">
        {{ formatMessage(stagingBannerMessages.description) }}
      </div>
      <div class="site-banner__actions">
        <Button transparent icon-only :action="hideStagingBanner" aria-label="Close banner"
          ><XIcon aria-hidden="true"
        /></Button>
      </div>
    </div>
    <header
      class="experimental-styles-within desktop-only relative z-[5] mx-auto grid max-w-[1280px] grid-cols-[1fr_auto] items-center gap-2 px-6 py-4 lg:grid-cols-[auto_1fr_auto]"
    >
      <div>
        <NuxtLink to="/" aria-label="Modrinth home page">
          <BrandTextLogo aria-hidden="true" class="h-7 w-auto text-contrast" />
        </NuxtLink>
      </div>
      <div
        :class="`col-span-2 row-start-2 flex flex-wrap justify-center ${flags.projectTypesPrimaryNav ? 'gap-2' : 'gap-4'} lg:col-span-1 lg:row-start-auto`"
      >
        <template v-if="flags.projectTypesPrimaryNav">
          <ButtonStyled
            type="transparent"
            :highlighted="route.name === 'search-mods' || route.path.startsWith('/mod/')"
            :highlighted-style="
              route.name === 'search-mods' ? 'main-nav-primary' : 'main-nav-secondary'
            "
          >
            <nuxt-link to="/mods"> <BoxIcon aria-hidden="true" /> Mods </nuxt-link>
          </ButtonStyled>
          <ButtonStyled
            type="transparent"
            :highlighted="
              route.name === 'search-resourcepacks' || route.path.startsWith('/resourcepack/')
            "
            :highlighted-style="
              route.name === 'search-resourcepacks' ? 'main-nav-primary' : 'main-nav-secondary'
            "
          >
            <nuxt-link to="/resourcepacks">
              <PaintBrushIcon aria-hidden="true" /> Resource Packs
            </nuxt-link>
          </ButtonStyled>
          <ButtonStyled
            type="transparent"
            :highlighted="route.name === 'search-datapacks' || route.path.startsWith('/datapack/')"
            :highlighted-style="
              route.name === 'search-datapacks' ? 'main-nav-primary' : 'main-nav-secondary'
            "
          >
            <nuxt-link to="/datapacks"> <BracesIcon aria-hidden="true" /> Data Packs </nuxt-link>
          </ButtonStyled>
          <ButtonStyled
            type="transparent"
            :highlighted="route.name === 'search-modpacks' || route.path.startsWith('/modpack/')"
            :highlighted-style="
              route.name === 'search-modpacks' ? 'main-nav-primary' : 'main-nav-secondary'
            "
          >
            <nuxt-link to="/modpacks"> <PackageOpenIcon aria-hidden="true" /> Modpacks </nuxt-link>
          </ButtonStyled>
          <ButtonStyled
            type="transparent"
            :highlighted="route.name === 'search-shaders' || route.path.startsWith('/shader/')"
            :highlighted-style="
              route.name === 'search-shaders' ? 'main-nav-primary' : 'main-nav-secondary'
            "
          >
            <nuxt-link to="/shaders"> <GlassesIcon aria-hidden="true" /> Shaders </nuxt-link>
          </ButtonStyled>
          <ButtonStyled
            type="transparent"
            :highlighted="route.name === 'search-plugins' || route.path.startsWith('/plugin/')"
            :highlighted-style="
              route.name === 'search-plugins' ? 'main-nav-primary' : 'main-nav-secondary'
            "
          >
            <nuxt-link to="/plugins"> <PlugIcon aria-hidden="true" /> Plugins </nuxt-link>
          </ButtonStyled>
        </template>
        <template v-else>
          <ButtonStyled
            type="transparent"
            :highlighted="isDiscovering || isDiscoveringSubpage"
            :highlighted-style="isDiscoveringSubpage ? 'main-nav-secondary' : 'main-nav-primary'"
          >
            <TeleportOverflowMenu
              :options="[
                {
                  id: 'mods',
                  action: '/mods',
                },
                {
                  id: 'resourcepacks',
                  action: '/resourcepacks',
                },
                {
                  id: 'datapacks',
                  action: '/datapacks',
                },
                {
                  id: 'shaders',
                  action: '/shaders',
                },
                {
                  id: 'modpacks',
                  action: '/modpacks',
                },
                {
                  id: 'plugins',
                  action: '/plugins',
                },
              ]"
              hoverable
            >
              <BoxIcon
                v-if="route.name === 'search-mods' || route.path.startsWith('/mod/')"
                aria-hidden="true"
              />
              <PaintBrushIcon
                v-else-if="
                  route.name === 'search-resourcepacks' || route.path.startsWith('/resourcepack/')
                "
                aria-hidden="true"
              />
              <BracesIcon
                v-else-if="route.name === 'search-datapacks' || route.path.startsWith('/datapack/')"
                aria-hidden="true"
              />
              <PackageOpenIcon
                v-else-if="route.name === 'search-modpacks' || route.path.startsWith('/modpack/')"
                aria-hidden="true"
              />
              <GlassesIcon
                v-else-if="route.name === 'search-shaders' || route.path.startsWith('/shader/')"
                aria-hidden="true"
              />
              <PlugIcon
                v-else-if="route.name === 'search-plugins' || route.path.startsWith('/plugin/')"
                aria-hidden="true"
              />
              <CompassIcon v-else aria-hidden="true" />
              <span class="hidden md:contents">Discover content</span>
              <span class="contents md:hidden">Discover</span>
              <DropdownIcon aria-hidden="true" class="h-5 w-5 text-secondary" />

              <template #mods> <BoxIcon aria-hidden="true" /> Mods </template>
              <template #resourcepacks>
                <PaintBrushIcon aria-hidden="true" /> Resource Packs
              </template>
              <template #datapacks> <BracesIcon aria-hidden="true" /> Data Packs </template>
              <template #plugins> <PlugIcon aria-hidden="true" /> Plugins </template>
              <template #shaders> <GlassesIcon aria-hidden="true" /> Shaders </template>
              <template #modpacks> <PackageOpenIcon aria-hidden="true" /> Modpacks </template>
            </TeleportOverflowMenu>
          </ButtonStyled>

          <ButtonStyled
            type="transparent"
            :highlighted="
              route.name?.startsWith('servers') ||
              (route.name?.startsWith('search-') && route.query.sid)
            "
            :highlighted-style="
              route.name === 'servers' ? 'main-nav-primary' : 'main-nav-secondary'
            "
          >
            <nuxt-link to="/servers">
              <ServerIcon aria-hidden="true" />
              Host a server
            </nuxt-link>
          </ButtonStyled>
          <ButtonStyled type="transparent" :highlighted="route.name === 'app'">
            <nuxt-link to="/app">
              <DownloadIcon aria-hidden="true" />
              <span class="hidden md:contents">Get Modrinth App</span>
              <span class="contents md:hidden">Modrinth App</span>
            </nuxt-link>
          </ButtonStyled>
        </template>
      </div>
      <div class="flex items-center gap-2">
        <ButtonStyled type="transparent">
          <OverflowMenu
            v-if="auth.user"
            class="btn-dropdown-animation flex items-center gap-1 rounded-xl bg-transparent px-2 py-1"
            position="bottom"
            direction="left"
            :dropdown-id="createPopoutId"
            aria-label="Create new..."
            :options="[
              {
                id: 'new-project',
                action: (event) => $refs.modal_creation.show(event),
              },
              {
                id: 'new-collection',
                action: (event) => $refs.modal_collection_creation.show(event),
              },
              { divider: true },
              {
                id: 'new-organization',
                action: (event) => $refs.modal_organization_creation.show(event),
              },
            ]"
          >
            <PlusIcon aria-hidden="true" />
            <DropdownIcon aria-hidden="true" class="h-5 w-5 text-secondary" />
            <template #new-project> <BoxIcon aria-hidden="true" /> New project </template>
            <!-- <template #import-project> <BoxImportIcon /> Import project </template>-->
            <template #new-collection>
              <CollectionIcon aria-hidden="true" /> New collection
            </template>
            <template #new-organization>
              <OrganizationIcon aria-hidden="true" /> New organization
            </template>
          </OverflowMenu>
        </ButtonStyled>
        <OverflowMenu
          v-if="auth.user"
          :dropdown-id="userPopoutId"
          class="btn-dropdown-animation flex items-center gap-1 rounded-xl bg-transparent px-2 py-1"
          :options="userMenuOptions"
        >
          <Avatar :src="auth.user.avatar_url" aria-hidden="true" circle />
          <DropdownIcon class="h-5 w-5 text-secondary" />
          <template #profile> <UserIcon aria-hidden="true" /> Profile </template>
          <template #notifications> <BellIcon aria-hidden="true" /> Notifications </template>
          <template #saved> <BookmarkIcon aria-hidden="true" /> Saved projects </template>
          <template #servers> <ServerIcon aria-hidden="true" /> My servers </template>
          <template #plus>
            <ArrowBigUpDashIcon aria-hidden="true" /> Upgrade to Modrinth+
          </template>
          <template #settings> <SettingsIcon aria-hidden="true" /> Settings </template>
          <template #flags> <ReportIcon aria-hidden="true" /> Feature flags </template>
          <template #projects> <BoxIcon aria-hidden="true" /> Projects </template>
          <template #organizations>
            <OrganizationIcon aria-hidden="true" /> Organizations
          </template>
          <template #revenue> <CurrencyIcon aria-hidden="true" /> Revenue </template>
          <template #analytics> <ChartIcon aria-hidden="true" /> Analytics </template>
          <template #moderation> <ModerationIcon aria-hidden="true" /> Moderation </template>
          <template #sign-out> <LogOutIcon aria-hidden="true" /> Sign out </template>
        </OverflowMenu>
        <ButtonStyled v-else color="brand">
          <nuxt-link to="/auth/sign-in">
            <LogInIcon aria-hidden="true" />
            Sign in
          </nuxt-link>
        </ButtonStyled>
      </div>
    </header>
    <header class="mobile-navigation mobile-only">
      <div
        class="nav-menu nav-menu-browse"
        :class="{ expanded: isBrowseMenuOpen }"
        @focusin="isBrowseMenuOpen = true"
        @focusout="isBrowseMenuOpen = false"
      >
        <div class="links cascade-links">
          <NuxtLink
            v-for="navRoute in navRoutes"
            :key="navRoute.href"
            :to="navRoute.href"
            class="iconified-button"
          >
            {{ navRoute.label }}
          </NuxtLink>
        </div>
      </div>
      <div
        class="nav-menu nav-menu-mobile"
        :class="{ expanded: isMobileMenuOpen }"
        @focusin="isMobileMenuOpen = true"
        @focusout="isMobileMenuOpen = false"
      >
        <div class="account-container">
          <NuxtLink
            v-if="auth.user"
            :to="`/user/${auth.user.username}`"
            class="iconified-button account-button"
          >
            <Avatar
              :src="auth.user.avatar_url"
              class="user-icon"
              :alt="formatMessage(messages.yourAvatarAlt)"
              aria-hidden="true"
              circle
            />
            <div class="account-text">
              <div>@{{ auth.user.username }}</div>
              <div>{{ formatMessage(commonMessages.visitYourProfile) }}</div>
            </div>
          </NuxtLink>
          <nuxt-link v-else class="iconified-button brand-button" to="/auth/sign-in">
            <LogInIcon aria-hidden="true" /> {{ formatMessage(commonMessages.signInButton) }}
          </nuxt-link>
        </div>
        <div class="links">
          <template v-if="auth.user">
            <button class="iconified-button danger-button" @click="logoutUser()">
              <LogOutIcon aria-hidden="true" />
              {{ formatMessage(commonMessages.signOutButton) }}
            </button>
            <button class="iconified-button" @click="$refs.modal_creation.show()">
              <PlusIcon aria-hidden="true" />
              {{ formatMessage(commonMessages.createAProjectButton) }}
            </button>
            <NuxtLink class="iconified-button" to="/dashboard/collections">
              <LibraryIcon class="icon" />
              {{ formatMessage(commonMessages.collectionsLabel) }}
            </NuxtLink>
            <NuxtLink class="iconified-button" to="/servers/manage">
              <ServerIcon class="icon" />
              {{ formatMessage(commonMessages.serversLabel) }}
            </NuxtLink>
            <NuxtLink
              v-if="auth.user.role === 'moderator' || auth.user.role === 'admin'"
              class="iconified-button"
              to="/moderation"
            >
              <ModerationIcon aria-hidden="true" />
              {{ formatMessage(commonMessages.moderationLabel) }}
            </NuxtLink>
            <NuxtLink v-if="flags.developerMode" class="iconified-button" to="/flags">
              <ReportIcon aria-hidden="true" />
              Feature flags
            </NuxtLink>
          </template>
          <NuxtLink class="iconified-button" to="/settings">
            <SettingsIcon aria-hidden="true" />
            {{ formatMessage(commonMessages.settingsLabel) }}
          </NuxtLink>
          <button class="iconified-button" @click="changeTheme">
            <MoonIcon v-if="$theme.active === 'light'" class="icon" />
            <SunIcon v-else class="icon" />
            <span class="dropdown-item__text">
              {{ formatMessage(messages.changeTheme) }}
            </span>
          </button>
        </div>
      </div>
      <div class="mobile-navbar" :class="{ expanded: isBrowseMenuOpen || isMobileMenuOpen }">
        <NuxtLink
          to="/"
          class="tab button-animation"
          :title="formatMessage(navMenuMessages.home)"
          aria-label="Home"
        >
          <HomeIcon aria-hidden="true" />
        </NuxtLink>
        <button
          class="tab button-animation"
          :class="{ 'router-link-exact-active': isBrowseMenuOpen }"
          :title="formatMessage(navMenuMessages.search)"
          aria-label="Search"
          @click="toggleBrowseMenu()"
        >
          <template v-if="auth.user">
            <SearchIcon aria-hidden="true" />
          </template>
          <template v-else>
            <SearchIcon aria-hidden="true" class="smaller" />
            {{ formatMessage(navMenuMessages.search) }}
          </template>
        </button>
        <template v-if="auth.user">
          <NuxtLink
            to="/dashboard/notifications"
            class="tab button-animation"
            aria-label="Notifications"
            :class="{
              'no-active': isMobileMenuOpen || isBrowseMenuOpen,
            }"
            :title="formatMessage(commonMessages.notificationsLabel)"
            @click="
              () => {
                isMobileMenuOpen = false;
                isBrowseMenuOpen = false;
              }
            "
          >
            <NotificationIcon aria-hidden="true" />
          </NuxtLink>
          <NuxtLink
            to="/dashboard"
            class="tab button-animation"
            aria-label="Dashboard"
            :title="formatMessage(commonMessages.dashboardLabel)"
          >
            <ChartIcon aria-hidden="true" />
          </NuxtLink>
        </template>
        <button
          class="tab button-animation"
          :title="formatMessage(messages.toggleMenu)"
          :aria-label="isMobileMenuOpen ? 'Close menu' : 'Open menu'"
          @click="toggleMobileMenu()"
        >
          <template v-if="!auth.user">
            <HamburgerIcon v-if="!isMobileMenuOpen" aria-hidden="true" />
            <CrossIcon v-else aria-hidden="true" />
          </template>
          <template v-else>
            <Avatar
              :src="auth.user.avatar_url"
              class="user-icon"
              :class="{ expanded: isMobileMenuOpen }"
              :alt="formatMessage(messages.yourAvatarAlt)"
              aria-hidden="true"
              circle
            />
          </template>
        </button>
      </div>
    </header>
    <main>
      <ModalCreation v-if="auth.user" ref="modal_creation" />
      <CollectionCreateModal ref="modal_collection_creation" />
      <OrganizationCreateModal ref="modal_organization_creation" />
      <slot id="main" />
    </main>
    <footer>
      <div class="logo-info" role="region" aria-label="Modrinth information">
        <BrandTextLogo
          aria-hidden="true"
          class="text-logo button-base mx-auto mb-4 lg:mx-0"
          @click="developerModeIncrement()"
        />
        <p class="mb-4">
          <IntlFormatted :message-id="footerMessages.openSource">
            <template #github-link="{ children }">
              <a
                :target="$external()"
                href="https://github.com/modrinth"
                class="text-link"
                rel="noopener"
              >
                <component :is="() => children" />
              </a>
            </template>
          </IntlFormatted>
        </p>
        <p class="mb-4">
          {{ config.public.branch }}@<a
            :target="$external()"
            :href="
              'https://github.com/' +
              config.public.owner +
              '/' +
              config.public.slug +
              '/tree/' +
              config.public.hash
            "
            class="text-link"
            rel="noopener"
            >{{ config.public.hash.substring(0, 7) }}</a
          >
        </p>
        <p>© Rinth, Inc.</p>
      </div>
      <div class="links links-1" role="region" aria-label="Legal">
        <h4 aria-hidden="true">{{ formatMessage(footerMessages.companyTitle) }}</h4>
        <nuxt-link to="/legal/terms"> {{ formatMessage(footerMessages.terms) }}</nuxt-link>
        <nuxt-link to="/legal/privacy"> {{ formatMessage(footerMessages.privacy) }}</nuxt-link>
        <nuxt-link to="/legal/rules"> {{ formatMessage(footerMessages.rules) }}</nuxt-link>
        <a :target="$external()" href="https://careers.modrinth.com">
          {{ formatMessage(footerMessages.careers) }}
          <span v-if="false" class="count-bubble">0</span>
        </a>
      </div>
      <div class="links links-2" role="region" aria-label="Resources">
        <h4 aria-hidden="true">{{ formatMessage(footerMessages.resourcesTitle) }}</h4>
        <a :target="$external()" href="https://support.modrinth.com">
          {{ formatMessage(footerMessages.support) }}
        </a>
        <a :target="$external()" href="https://blog.modrinth.com">
          {{ formatMessage(footerMessages.blog) }}
        </a>
        <a :target="$external()" href="https://docs.modrinth.com">
          {{ formatMessage(footerMessages.docs) }}
        </a>
        <a :target="$external()" href="https://status.modrinth.com">
          {{ formatMessage(footerMessages.status) }}
        </a>
      </div>
      <div class="links links-3" role="region" aria-label="Interact">
        <h4 aria-hidden="true">{{ formatMessage(footerMessages.interactTitle) }}</h4>
        <a rel="noopener" :target="$external()" href="https://discord.modrinth.com"> Discord </a>
        <a rel="noopener" :target="$external()" href="https://x.com/modrinth"> X (Twitter) </a>
        <a rel="noopener" :target="$external()" href="https://floss.social/@modrinth"> Mastodon </a>
        <a rel="noopener" :target="$external()" href="https://crowdin.com/project/modrinth">
          Crowdin
        </a>
      </div>
      <div class="buttons">
        <nuxt-link class="btn btn-outline btn-primary" to="/app">
          <DownloadIcon aria-hidden="true" />
          {{ formatMessage(messages.getModrinthApp) }}
        </nuxt-link>
        <button class="iconified-button raised-button" @click="changeTheme">
          <MoonIcon v-if="$theme.active === 'light'" aria-hidden="true" />
          <SunIcon v-else aria-hidden="true" />
          {{ formatMessage(messages.changeTheme) }}
        </button>
        <nuxt-link class="iconified-button raised-button" to="/settings">
          <SettingsIcon aria-hidden="true" />
          {{ formatMessage(commonMessages.settingsLabel) }}
        </nuxt-link>
      </div>
      <div class="not-affiliated-notice">
        {{ formatMessage(footerMessages.legalDisclaimer) }}
      </div>
    </footer>
  </div>
</template>
<script setup>
import {
  ArrowBigUpDashIcon,
  BookmarkIcon,
  ServerIcon,
  LogInIcon,
  DownloadIcon,
  LibraryIcon,
  XIcon,
  IssuesIcon,
  ReportIcon,
  CompassIcon,
  HamburgerIcon,
  SearchIcon,
  BellIcon,
  SettingsIcon,
  HomeIcon,
  MoonIcon,
  SunIcon,
  PlugIcon,
  PlusIcon,
  DropdownIcon,
  LogOutIcon,
  ChartIcon,
  BoxIcon,
  CollectionIcon,
  OrganizationIcon,
  UserIcon,
  CurrencyIcon,
  BracesIcon,
  GlassesIcon,
  PaintBrushIcon,
  PackageOpenIcon,
  XIcon as CrossIcon,
  ScaleIcon as ModerationIcon,
  BellIcon as NotificationIcon,
} from "@modrinth/assets";
import { Button, ButtonStyled, OverflowMenu, Avatar, commonMessages } from "@modrinth/ui";

import ModalCreation from "~/components/ui/ModalCreation.vue";
import { getProjectTypeMessage } from "~/utils/i18n-project-type.ts";
import CollectionCreateModal from "~/components/ui/CollectionCreateModal.vue";
import OrganizationCreateModal from "~/components/ui/OrganizationCreateModal.vue";
import TeleportOverflowMenu from "~/components/ui/servers/TeleportOverflowMenu.vue";

const { formatMessage } = useVIntl();

const app = useNuxtApp();
const auth = await useAuth();
const user = await useUser();

const cosmetics = useCosmetics();
const flags = useFeatureFlags();

const config = useRuntimeConfig();
const route = useNativeRoute();
const link = config.public.siteUrl + route.path.replace(/\/+$/, "");

const createPopoutId = useId();
const userPopoutId = useId();

const verifyEmailBannerMessages = defineMessages({
  title: {
    id: "layout.banner.verify-email.title",
    defaultMessage: "For security purposes, please verify your email address on Modrinth.",
  },
  action: {
    id: "layout.banner.verify-email.action",
    defaultMessage: "Re-send verification email",
  },
});

const addEmailBannerMessages = defineMessages({
  title: {
    id: "layout.banner.add-email.title",
    defaultMessage: "For security purposes, please enter your email on Modrinth.",
  },
  action: {
    id: "layout.banner.add-email.button",
    defaultMessage: "Visit account settings",
  },
});

const subscriptionPaymentFailedBannerMessages = defineMessages({
  title: {
    id: "layout.banner.subscription-payment-failed.title",
    defaultMessage:
      "Your subscription failed to renew. Please update your payment method to prevent losing access.",
  },
  action: {
    id: "layout.banner.subscription-payment-failed.button",
    defaultMessage: "Update billing info",
  },
});

const stagingBannerMessages = defineMessages({
  title: {
    id: "layout.banner.staging.title",
    defaultMessage: "You’re viewing Modrinth’s staging environment.",
  },
  description: {
    id: "layout.banner.staging.description",
    defaultMessage:
      "The staging environment is completely separate from the production Modrinth database. This is used for testing and debugging purposes, and may be running in-development versions of the Modrinth backend or frontend newer than the production instance.",
  },
});

const navMenuMessages = defineMessages({
  home: {
    id: "layout.nav.home",
    defaultMessage: "Home",
  },
  search: {
    id: "layout.nav.search",
    defaultMessage: "Search",
  },
});

const messages = defineMessages({
  toggleMenu: {
    id: "layout.menu-toggle.action",
    defaultMessage: "Toggle menu",
  },
  yourAvatarAlt: {
    id: "layout.avatar.alt",
    defaultMessage: "Your avatar",
  },
  getModrinthApp: {
    id: "layout.action.get-modrinth-app",
    defaultMessage: "Get Modrinth App",
  },
  changeTheme: {
    id: "layout.action.change-theme",
    defaultMessage: "Change theme",
  },
});

const footerMessages = defineMessages({
  openSource: {
    id: "layout.footer.open-source",
    defaultMessage: "Modrinth is <github-link>open source</github-link>.",
  },
  companyTitle: {
    id: "layout.footer.company.title",
    defaultMessage: "Company",
  },
  terms: {
    id: "layout.footer.company.terms",
    defaultMessage: "Terms",
  },
  privacy: {
    id: "layout.footer.company.privacy",
    defaultMessage: "Privacy",
  },
  rules: {
    id: "layout.footer.company.rules",
    defaultMessage: "Rules",
  },
  careers: {
    id: "layout.footer.company.careers",
    defaultMessage: "Careers",
  },
  resourcesTitle: {
    id: "layout.footer.resources.title",
    defaultMessage: "Resources",
  },
  support: {
    id: "layout.footer.resources.support",
    defaultMessage: "Support",
  },
  blog: {
    id: "layout.footer.resources.blog",
    defaultMessage: "Blog",
  },
  docs: {
    id: "layout.footer.resources.docs",
    defaultMessage: "Docs",
  },
  status: {
    id: "layout.footer.resources.status",
    defaultMessage: "Status",
  },
  interactTitle: {
    id: "layout.footer.interact.title",
    defaultMessage: "Interact",
  },
  legalDisclaimer: {
    id: "layout.footer.legal-disclaimer",
    defaultMessage:
      "NOT AN OFFICIAL MINECRAFT SERVICE. NOT APPROVED BY OR ASSOCIATED WITH MOJANG OR MICROSOFT.",
  },
});

useHead({
  link: [
    {
      rel: "canonical",
      href: link,
    },
  ],
});
useSeoMeta({
  title: "Modrinth",
  description: () =>
    formatMessage({
      id: "layout.meta.description",
      defaultMessage:
        "Download Minecraft mods, plugins, datapacks, shaders, resourcepacks, and modpacks on Modrinth. " +
        "Discover and publish projects on Modrinth with a modern, easy to use interface and API.",
    }),
  publisher: "Modrinth",
  themeColor: "#1bd96a",
  colorScheme: "dark light",

  // OpenGraph
  ogTitle: "Modrinth",
  ogSiteName: "Modrinth",
  ogDescription: () =>
    formatMessage({
      id: "layout.meta.og-description",
      defaultMessage: "Discover and publish Minecraft content!",
    }),
  ogType: "website",
  ogImage: "https://cdn.modrinth.com/modrinth-new.png",
  ogUrl: link,

  // Twitter
  twitterCard: "summary",
  twitterSite: "@modrinth",
});

const developerModeCounter = ref(0);

const isMobileMenuOpen = ref(false);
const isBrowseMenuOpen = ref(false);
const navRoutes = computed(() => [
  {
    id: "mods",
    label: formatMessage(getProjectTypeMessage("mod", true)),
    href: "/mods",
  },
  {
    label: formatMessage(getProjectTypeMessage("plugin", true)),
    href: "/plugins",
  },
  {
    label: formatMessage(getProjectTypeMessage("datapack", true)),
    href: "/datapacks",
  },
  {
    label: formatMessage(getProjectTypeMessage("shader", true)),
    href: "/shaders",
  },
  {
    label: formatMessage(getProjectTypeMessage("resourcepack", true)),
    href: "/resourcepacks",
  },
  {
    label: formatMessage(getProjectTypeMessage("modpack", true)),
    href: "/modpacks",
  },
]);

const userMenuOptions = computed(() => {
  let options = [
    {
      id: "profile",
      link: `/user/${auth.value.user.username}`,
    },
    {
      id: "plus",
      link: "/plus",
      color: "purple",
      shown: !flags.value.hidePlusPromoInUserMenu && !isPermission(auth.value.user.badges, 1 << 0),
    },
    {
      id: "notifications",
      link: "/dashboard/notifications",
    },
    {
      id: "saved",
      link: "/dashboard/collections",
    },
    {
      id: "servers",
      link: "/servers/manage",
    },
    {
      id: "flags",
      link: "/flags",
      shown: flags.value.developerMode,
    },
    {
      id: "settings",
      link: "/settings",
    },
  ];

  // TODO: Only show if user has projects
  options = [
    ...options,
    {
      divider: true,
    },
    {
      id: "projects",
      link: "/dashboard/projects",
    },
    {
      id: "organizations",
      link: "/dashboard/organizations",
    },
    {
      id: "revenue",
      link: "/dashboard/revenue",
    },
    {
      id: "analytics",
      link: "/dashboard/analytics",
    },
  ];

  if (
    (auth.value && auth.value.user && auth.value.user.role === "moderator") ||
    auth.value.user.role === "admin"
  ) {
    options = [
      ...options,
      {
        divider: true,
      },
      {
        id: "moderation",
        color: "orange",
        link: "/moderation/review",
      },
    ];
  }

  options = [
    ...options,
    {
      divider: true,
    },
    {
      id: "sign-out",
      color: "danger",
      action: () => logoutUser(),
      hoverFilled: true,
    },
  ];
  return options;
});

const isDiscovering = computed(
  () => route.name && route.name.startsWith("search-") && !route.query.sid,
);

const isDiscoveringSubpage = computed(
  () => route.name && route.name.startsWith("type-id") && !route.query.sid,
);

onMounted(() => {
  if (window && import.meta.client) {
    window.history.scrollRestoration = "auto";
  }

  runAnalytics();
});

watch(
  () => route.path,
  () => {
    isMobileMenuOpen.value = false;
    isBrowseMenuOpen.value = false;

    if (import.meta.client) {
      document.body.style.overflowY = "scroll";
      document.body.setAttribute("tabindex", "-1");
      document.body.removeAttribute("tabindex");
    }

    updateCurrentDate();
    runAnalytics();
  },
);

function developerModeIncrement() {
  if (developerModeCounter.value >= 5) {
    flags.value.developerMode = !flags.value.developerMode;
    developerModeCounter.value = 0;
    saveFeatureFlags();
    if (flags.value.developerMode) {
      app.$notify({
        group: "main",
        title: "Developer mode activated",
        text: "Developer mode has been enabled",
        type: "success",
      });
    } else {
      app.$notify({
        group: "main",
        title: "Developer mode deactivated",
        text: "Developer mode has been disabled",
        type: "success",
      });
    }
  } else {
    developerModeCounter.value++;
  }
}

async function logoutUser() {
  await logout();
}

function runAnalytics() {
  const config = useRuntimeConfig();
  const replacedUrl = config.public.apiBaseUrl.replace("v2/", "");

  try {
    setTimeout(() => {
      $fetch(`${replacedUrl}analytics/view`, {
        method: "POST",
        body: {
          url: window.location.href,
        },
        headers: {
          Authorization: auth.value.token,
        },
      })
        .then(() => {})
        .catch(() => {});
    });
  } catch (e) {
    console.error(`Sending analytics failed (CORS error? If so, ignore)`, e);
  }
}
function toggleMobileMenu() {
  isMobileMenuOpen.value = !isMobileMenuOpen.value;
  if (isMobileMenuOpen.value) {
    isBrowseMenuOpen.value = false;
  }
}
function toggleBrowseMenu() {
  isBrowseMenuOpen.value = !isBrowseMenuOpen.value;

  if (isBrowseMenuOpen.value) {
    isMobileMenuOpen.value = false;
  }
}

const { cycle: changeTheme } = useTheme();

function hideStagingBanner() {
  cosmetics.value.hideStagingBanner = true;
}
</script>

<style lang="scss">
@import "~/assets/styles/global.scss";
// @import '@modrinth/assets';

.layout {
  min-height: 100vh;
  display: block;

  @media screen and (min-width: 1024px) {
    min-height: calc(100vh - var(--spacing-card-bg));
  }

  @media screen and (max-width: 750px) {
    margin-bottom: calc(var(--size-mobile-navbar-height) + 2rem);
  }

  main {
    grid-area: main;
  }

  footer {
    margin: 6rem 0 2rem 0;
    text-align: center;
    display: grid;
    grid-template:
      "logo-info  logo-info  logo-info" auto
      "links-1    links-2    links-3" auto
      "buttons    buttons    buttons" auto
      "notice     notice     notice" auto
      / 1fr 1fr 1fr;
    max-width: 1280px;

    .logo-info {
      margin-left: auto;
      margin-right: auto;
      max-width: 15rem;
      margin-bottom: 1rem;
      grid-area: logo-info;

      .text-logo {
        width: 10rem;
        height: auto;
      }
    }

    .links {
      display: flex;
      flex-direction: column;
      margin-bottom: 1rem;

      h4 {
        color: var(--color-text-dark);
        margin: 0 0 1rem 0;
      }

      a {
        margin: 0 0 1rem 0;
      }

      &.links-1 {
        grid-area: links-1;
      }

      &.links-2 {
        grid-area: links-2;
      }

      &.links-3 {
        grid-area: links-3;
      }

      .count-bubble {
        font-size: 1rem;
        border-radius: 5rem;
        background: var(--color-brand);
        color: var(--color-text-inverted);
        padding: 0 0.35rem;
        margin-left: 0.25rem;
      }
    }

    .buttons {
      margin-left: auto;
      margin-right: auto;
      grid-area: buttons;

      button,
      a {
        margin-bottom: 0.5rem;
        margin-left: auto;
        margin-right: auto;
      }
    }

    .not-affiliated-notice {
      grid-area: notice;
      font-size: var(--font-size-xs);
      text-align: center;
      font-weight: 500;
      margin-top: var(--spacing-card-md);
    }

    @media screen and (min-width: 1024px) {
      display: grid;
      margin-inline: auto;
      grid-template:
        "logo-info  links-1 links-2 links-3 buttons" auto
        "notice     notice  notice  notice  notice" auto;
      text-align: unset;

      .logo-info {
        margin-right: 4rem;
      }

      .links {
        margin-right: 4rem;
      }

      .buttons {
        width: unset;
        margin-left: 0;

        button,
        a {
          margin-right: unset;
        }
      }

      .not-affiliated-notice {
        margin-top: 0;
      }
    }
  }
}

@media (min-width: 1024px) {
  .layout {
    main {
      .alpha-alert {
        margin: 1rem;

        .wrapper {
          padding: 1rem 2rem 1rem 1rem;
        }
      }
    }
  }
}

.email-nag {
  z-index: 6;
  position: relative;
  background-color: var(--color-raised-bg);
  width: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 1rem;
  padding: 0.5rem 1rem;
}

.site-banner--warning {
  // On some pages, there's gradient backgrounds that seep underneath
  // the banner, so we need to add a solid color underlay.
  background-color: black;
  border-bottom: 2px solid var(--color-red);
  display: grid;
  gap: 0.5rem;
  grid-template: "title actions" "description actions";
  padding-block: var(--gap-xl);
  padding-inline: max(calc((100% - 80rem) / 2 + var(--gap-md)), var(--gap-xl));
  z-index: 4;
  position: relative;

  &::before {
    content: "";
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background-color: var(--color-red-bg);
    z-index: 5;
  }

  .site-banner__title {
    grid-area: title;
    display: flex;
    gap: 0.5rem;
    align-items: center;
    font-weight: bold;
    font-size: var(--font-size-md);
    color: var(--color-contrast);

    svg {
      color: var(--color-red);
      width: 1.5rem;
      height: 1.5rem;
      flex-shrink: 0;
    }
  }

  .site-banner__description {
    grid-area: description;
  }

  .site-banner__actions {
    grid-area: actions;
  }

  a {
    color: var(--color-red);
  }
}

@media (max-width: 1200px) {
  .app-btn {
    display: none;
  }
}

.mobile-navigation {
  display: none;

  .nav-menu {
    width: 100%;
    position: fixed;
    bottom: calc(var(--size-mobile-navbar-height) - var(--size-rounded-card));
    padding-bottom: var(--size-rounded-card);
    left: 0;
    background-color: var(--color-raised-bg);
    z-index: 6;
    transform: translateY(100%);
    transition: transform 0.4s cubic-bezier(0.54, 0.84, 0.42, 1);
    border-radius: var(--size-rounded-card) var(--size-rounded-card) 0 0;
    box-shadow: 0 0 20px 2px rgba(0, 0, 0, 0);

    .links,
    .account-container {
      display: grid;
      grid-template-columns: repeat(1, 1fr);
      grid-gap: 1rem;
      justify-content: center;
      padding: 1rem;

      .iconified-button {
        width: 100%;
        max-width: 500px;
        padding: 0.75rem;
        justify-content: center;
        font-weight: 600;
        font-size: 1rem;
        margin: 0 auto;
      }
    }

    .cascade-links {
      @media screen and (min-width: 354px) {
        grid-template-columns: repeat(2, 1fr);
      }
      @media screen and (min-width: 674px) {
        grid-template-columns: repeat(3, 1fr);
      }
    }

    &-browse {
      &.expanded {
        transform: translateY(0);
        box-shadow: 0 0 20px 2px rgba(0, 0, 0, 0.3);
      }
    }

    &-mobile {
      .account-container {
        padding-bottom: 0;

        .account-button {
          padding: var(--spacing-card-md);
          display: flex;
          align-items: center;
          justify-content: center;
          gap: 0.5rem;

          .user-icon {
            width: 2.25rem;
            height: 2.25rem;
          }

          .account-text {
            flex-grow: 0;
          }
        }
      }

      &.expanded {
        transform: translateY(0);
        box-shadow: 0 0 20px 2px rgba(0, 0, 0, 0.3);
      }
    }
  }

  .mobile-navbar {
    display: flex;
    height: calc(var(--size-mobile-navbar-height) + env(safe-area-inset-bottom));
    border-radius: var(--size-rounded-card) var(--size-rounded-card) 0 0;
    padding-bottom: env(safe-area-inset-bottom);
    position: fixed;
    left: 0;
    bottom: 0;
    background-color: var(--color-raised-bg);
    box-shadow: 0 0 20px 2px rgba(0, 0, 0, 0.3);
    z-index: 7;
    width: 100%;
    align-items: center;
    justify-content: space-between;
    transition: border-radius 0.3s ease-out;
    border-top: 2px solid rgba(0, 0, 0, 0);
    box-sizing: border-box;

    &.expanded {
      box-shadow: none;
      border-radius: 0;
    }

    .tab {
      position: relative;
      background: none;
      display: flex;
      flex-basis: 0;
      justify-content: center;
      align-items: center;
      flex-direction: row;
      gap: 0.25rem;
      font-weight: bold;
      padding: 0;
      transition: color ease-in-out 0.15s;
      color: var(--color-text-inactive);
      text-align: center;

      &.browse {
        svg {
          transform: rotate(180deg);
          transition: transform ease-in-out 0.3s;

          &.closed {
            transform: rotate(0deg);
          }
        }
      }

      &.bubble {
        &::after {
          background-color: var(--color-brand);
          border-radius: var(--size-rounded-max);
          content: "";
          height: 0.5rem;
          position: absolute;
          left: 1.5rem;
          top: 0;
          width: 0.5rem;
        }
      }

      svg {
        height: 1.75rem;
        width: 1.75rem;

        &.smaller {
          width: 1.25rem;
          height: 1.25rem;
        }
      }

      .user-icon {
        width: 2rem;
        height: 2rem;
        transition: border ease-in-out 0.15s;
        border: 0 solid var(--color-brand);
        box-sizing: border-box;

        &.expanded {
          border: 2px solid var(--color-brand);
        }
      }

      &:hover,
      &:focus {
        color: var(--color-text);
      }

      &:first-child {
        margin-left: 2rem;
      }

      &:last-child {
        margin-right: 2rem;
      }

      &.router-link-exact-active:not(&.no-active) {
        svg {
          color: var(--color-brand);
        }

        color: var(--color-brand);
      }
    }
  }
}

@media (any-hover: none) and (max-width: 640px) {
  .desktop-only {
    display: none;
  }
}

@media (any-hover: none) and (max-width: 640px) {
  .mobile-navigation {
    display: flex;
  }

  main {
    padding-top: 1.5rem;
  }
}
</style>
<style src="vue-multiselect/dist/vue-multiselect.css"></style>
