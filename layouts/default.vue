<template>
  <div class="layout" :class="{ 'expanded-mobile-nav': isBrowseMenuOpen }">
    <header class="site-header" role="presentation">
      <section class="navbar card columns" role="navigation">
        <section class="skip column" role="presentation">
          <a href="#main">Skip to Main Content</a>
          <a
            v-show="!!registeredSkipLink"
            :href="(registeredSkipLink || {}).id"
            >{{ (registeredSkipLink || {}).text }}</a
          >
        </section>
        <section class="logo column" role="presentation">
          <NuxtLink class="button-base" to="/" aria-label="Modrinth home page">
            <ModrinthLogo aria-hidden="true" class="text-logo" />
          </NuxtLink>
        </section>
        <section class="nav-group columns" role="presentation">
          <section class="nav" aria-label="Page links">
            <NavRow
              class="navigation"
              :links="[
                {
                  label: 'Mods',
                  href: '/mods',
                },
                {
                  label: 'Plugins',
                  href: '/plugins',
                },
                {
                  label: 'Resource Packs',
                  href: '/resourcepacks',
                },
                {
                  label: 'Modpacks',
                  href: '/modpacks',
                },
              ]"
            />
          </section>
          <section class="column-grow user-outer" aria-label="Account links">
            <section class="user-controls">
              <nuxt-link
                v-if="$auth.user"
                to="/notifications"
                class="control-button button-transparent"
                :class="{ bubble: $user.notifications.length > 0 }"
                title="Notifications"
              >
                <NotificationIcon aria-hidden="true" />
              </nuxt-link>
              <button
                class="control-button button-transparent"
                title="Switch theme"
                @click="changeTheme"
              >
                <MoonIcon
                  v-if="$colorMode.value === 'light'"
                  aria-hidden="true"
                />
                <SunIcon v-else aria-hidden="true" />
              </button>
              <div
                v-if="$auth.user"
                class="dropdown"
                :class="{ closed: !isDropdownOpen }"
                tabindex="0"
                @mouseover="isDropdownOpen = true"
                @focus="isDropdownOpen = true"
                @mouseleave="isDropdownOpen = false"
              >
                <button class="control" value="Profile Dropdown">
                  <Avatar
                    :src="$auth.user.avatar_url"
                    class="user-icon"
                    alt="Your avatar"
                    aria-hidden="true"
                    circle
                  />
                  <DropdownIcon class="caret" />
                </button>
                <div class="content card">
                  <NuxtLink
                    class="item button-transparent"
                    :to="`/user/${$auth.user.username}`"
                  >
                    <div class="title profile-link">
                      <div class="username">@{{ $auth.user.username }}</div>
                      <div class="prompt">Go to my profile</div>
                    </div>
                  </NuxtLink>
                  <hr class="divider" />
                  <button
                    class="item button-transparent"
                    @click="$refs.modal_creation.show()"
                  >
                    <PlusIcon class="icon" />
                    <span class="title">Create a project</span>
                  </button>
                  <hr class="divider" />
                  <NuxtLink class="item button-transparent" to="/notifications">
                    <NotificationIcon class="icon" />
                    <span class="title">Notifications</span>
                  </NuxtLink>
                  <NuxtLink class="item button-transparent" to="/dashboard">
                    <ChartIcon class="icon" />
                    <span class="title">Dashboard</span
                    ><span class="beta-badge">BETA</span>
                  </NuxtLink>
                  <NuxtLink
                    class="item button-transparent"
                    to="/settings/follows"
                  >
                    <HeartIcon class="icon" />
                    <span class="title">Following</span>
                  </NuxtLink>
                  <NuxtLink class="item button-transparent" to="/settings">
                    <SettingsIcon class="icon" />
                    <span class="title">Settings</span>
                  </NuxtLink>
                  <NuxtLink
                    v-if="
                      $auth.user.role === 'moderator' ||
                      $auth.user.role === 'admin'
                    "
                    class="item button-transparent"
                    to="/moderation"
                  >
                    <ModerationIcon class="icon" />
                    <span class="title">Moderation</span>
                  </NuxtLink>
                  <hr class="divider" />
                  <button class="item button-transparent" @click="logout">
                    <LogOutIcon class="icon" />
                    <span class="dropdown-item__text">Log out</span>
                  </button>
                </div>
              </div>
              <section v-else class="auth-prompt">
                <a
                  :href="authUrl"
                  class="log-in-button header-button brand-button"
                >
                  <GitHubIcon aria-hidden="true" />
                  Sign in with GitHub</a
                >
              </section>
            </section>
          </section>
        </section>
      </section>
      <section class="mobile-navbar" :class="{ expanded: isBrowseMenuOpen }">
        <div class="top-row">
          <NuxtLink
            to="/"
            class="tab button-animation"
            @click.native="isBrowseMenuOpen = false"
          >
            <HomeIcon />
          </NuxtLink>
          <div class="spacer"></div>
          <button
            class="tab browse button-animation"
            @click="toggleBrowseMenu()"
          >
            <DropdownIcon :class="{ closed: !isBrowseMenuOpen }" />
            <span>Browse</span>
          </button>
          <div class="spacer"></div>
          <button class="tab button-animation" @click="toggleMobileMenu()">
            <HamburgerIcon v-if="!isMobileMenuOpen" />
            <CrossIcon v-else />
          </button>
        </div>
        <div
          :class="{ 'disable-children': !isBrowseMenuOpen }"
          class="project-types"
        >
          <NuxtLink
            :tabindex="isBrowseMenuOpen ? 0 : -1"
            to="/mods"
            class="tab iconified-button"
            @click.native="isBrowseMenuOpen = false"
          >
            <span>Mods</span>
          </NuxtLink>
          <NuxtLink
            :tabindex="isBrowseMenuOpen ? 0 : -1"
            to="/plugins"
            class="tab iconified-button"
            @click.native="isBrowseMenuOpen = false"
          >
            <span>Plugins</span>
          </NuxtLink>

          <NuxtLink
            :tabindex="isBrowseMenuOpen ? 0 : -1"
            to="/resourcepacks"
            class="tab iconified-button"
            @click.native="isBrowseMenuOpen = false"
          >
            <span>Resource Packs</span>
          </NuxtLink>
          <NuxtLink
            :tabindex="isBrowseMenuOpen ? 0 : -1"
            to="/modpacks"
            class="tab iconified-button"
            @click.native="isBrowseMenuOpen = false"
          >
            <span>Modpacks</span>
          </NuxtLink>
        </div>
      </section>
      <section class="mobile-menu" :class="{ active: isMobileMenuOpen }">
        <div class="mobile-menu-wrapper">
          <div class="items-container rows">
            <NuxtLink
              v-if="$auth.user"
              class="iconified-button raised-button user-item"
              :to="`/user/${$auth.user.username}`"
            >
              <img
                :src="$auth.user.avatar_url"
                class="user-icon"
                aria-hidden="true"
                alt="User profile icon"
              />
              <div class="profile-link">
                <div class="username">@{{ $auth.user.username }}</div>
                <div class="prompt">Go to my profile</div>
              </div>
            </NuxtLink>
            <button
              v-if="$auth.user"
              class="iconified-button raised-button"
              @click="$refs.modal_creation.show()"
            >
              <PlusIcon class="icon" />
              <span class="dropdown-item__text">Create a project</span>
            </button>
            <NuxtLink
              v-if="$auth.user"
              class="iconified-button raised-button"
              to="/notifications"
            >
              <NotificationIcon class="icon" />
              <span class="dropdown-item__text">Notifications</span>
            </NuxtLink>
            <NuxtLink
              v-if="$auth.user"
              class="iconified-button raised-button"
              to="/dashboard"
            >
              <ChartIcon class="icon" />
              <span class="dropdown-item__text">Dashboard</span>
              <span class="beta-badge">BETA</span>
            </NuxtLink>
            <NuxtLink
              v-if="$auth.user"
              class="iconified-button raised-button"
              to="/settings/follows"
            >
              <HeartIcon class="icon" />
              <span class="dropdown-item__text">Following</span>
            </NuxtLink>
            <NuxtLink class="iconified-button raised-button" to="/settings">
              <SettingsIcon class="icon" />
              <span class="dropdown-item__text">Settings</span>
            </NuxtLink>
            <NuxtLink
              v-if="
                $auth.user &&
                ($auth.user.role === 'moderator' || $auth.user.role === 'admin')
              "
              class="iconified-button raised-button"
              to="/moderation"
            >
              <ModerationIcon class="icon" />
              <span class="dropdown-item__text">Moderation</span>
            </NuxtLink>
            <button class="iconified-button raised-button" @click="changeTheme">
              <MoonIcon v-if="$colorMode.value === 'light'" class="icon" />
              <SunIcon v-else class="icon" />
              <span class="dropdown-item__text">Change theme</span>
            </button>
            <button
              v-if="$auth.user"
              class="iconified-button danger-button"
              @click="logout"
            >
              <LogOutIcon class="icon" />
              <span class="dropdown-item__text">Log out</span>
            </button>
            <a v-else :href="authUrl" class="iconified-button brand-button">
              <GitHubIcon aria-hidden="true" />
              Sign in with GitHub</a
            >
          </div>
        </div>
      </section>
    </header>
    <main>
      <ModalCreation ref="modal_creation" />
      <notifications
        group="main"
        position="bottom right"
        :max="5"
        :class="{ 'browse-menu-open': isBrowseMenuOpen }"
        :ignore-duplicates="true"
        :duration="10000"
      />
      <Nuxt id="main" />
    </main>
    <footer>
      <div class="logo-info" role="region" aria-label="Modrinth information">
        <ModrinthLogo aria-hidden="true" class="text-logo" />
        <p>
          Modrinth is
          <a
            target="_blank"
            href="https://github.com/modrinth"
            class="text-link"
          >
            open source</a
          >.
        </p>
        <p>
          {{ owner }}/{{ slug }} {{ branch }}@<a
            target="_blank"
            :href="'https://github.com/' + owner + '/' + slug + '/tree/' + hash"
            class="text-link"
            >{{ hash.substring(0, 7) }}</a
          >
        </p>
        <p>© Rinth, Inc.</p>
      </div>
      <div class="links links-1" role="region" aria-label="Legal">
        <h4 aria-hidden="true">Legal</h4>
        <nuxt-link to="/legal/terms">Terms</nuxt-link>
        <nuxt-link to="/legal/privacy">Privacy</nuxt-link>
        <nuxt-link to="/legal/rules">Rules</nuxt-link>
        <a
          target="_blank"
          href="https://github.com/modrinth/knossos/blob/master/LICENSE.md"
        >
          License
        </a>
      </div>
      <div class="links links-2" role="region" aria-label="Resources">
        <h4 aria-hidden="true">Resources</h4>
        <a target="_blank" href="https://blog.modrinth.com">Blog</a>
        <a target="_blank" href="https://docs.modrinth.com">Docs</a>
        <a target="_blank" href="https://status.modrinth.com">Status</a>
        <a target="_blank" href="https://github.com/modrinth">GitHub</a>
      </div>
      <div class="links links-3" role="region" aria-label="Interact">
        <h4 aria-hidden="true">Interact</h4>
        <a target="_blank" href="https://discord.gg/EUHuJHt">Discord</a>
        <a target="_blank" href="https://twitter.com/modrinth">Twitter</a>
        <a target="_blank" rel="me" href="https://floss.social/@modrinth">
          Mastodon
        </a>
        <a target="_blank" href="https://crowdin.com/project/modrinth">
          Crowdin
        </a>
      </div>
      <div class="buttons">
        <button class="iconified-button raised-button" @click="changeTheme">
          <MoonIcon v-if="$colorMode.value === 'light'" aria-hidden="true" />
          <SunIcon v-else aria-hidden="true" />
          Change theme
        </button>
        <nuxt-link class="iconified-button raised-button" to="/settings">
          <SettingsIcon aria-hidden="true" />
          Settings
        </nuxt-link>
      </div>
      <div class="not-affiliated-notice">
        NOT AN OFFICIAL MINECRAFT PRODUCT. NOT APPROVED BY OR ASSOCIATED WITH
        MOJANG.
      </div>
    </footer>
  </div>
</template>

<script>
import ClickOutside from 'vue-click-outside'

import ModrinthLogo from '~/assets/images/text-logo.svg?inline'

import HamburgerIcon from '~/assets/images/utils/hamburger.svg?inline'
import CrossIcon from '~/assets/images/utils/x.svg?inline'

import NotificationIcon from '~/assets/images/sidebar/notifications.svg?inline'
import SettingsIcon from '~/assets/images/sidebar/settings.svg?inline'
import ModerationIcon from '~/assets/images/sidebar/admin.svg?inline'
import HomeIcon from '~/assets/images/sidebar/home.svg?inline'

import MoonIcon from '~/assets/images/utils/moon.svg?inline'
import SunIcon from '~/assets/images/utils/sun.svg?inline'
import PlusIcon from '~/assets/images/utils/plus.svg?inline'
import DropdownIcon from '~/assets/images/utils/dropdown.svg?inline'
import LogOutIcon from '~/assets/images/utils/log-out.svg?inline'
import HeartIcon from '~/assets/images/utils/heart.svg?inline'
import ChartIcon from '~/assets/images/utils/chart.svg?inline'

import GitHubIcon from '~/assets/images/utils/github.svg?inline'
import NavRow from '~/components/ui/NavRow'
import ModalCreation from '~/components/ui/ModalCreation'
import Avatar from '~/components/ui/Avatar'

export default {
  components: {
    Avatar,
    ModalCreation,
    NavRow,
    ModrinthLogo,
    MoonIcon,
    SunIcon,
    LogOutIcon,
    GitHubIcon,
    NotificationIcon,
    HomeIcon,
    CrossIcon,
    HamburgerIcon,
    SettingsIcon,
    ModerationIcon,
    PlusIcon,
    DropdownIcon,
    HeartIcon,
    ChartIcon,
  },
  directives: {
    ClickOutside,
  },
  data() {
    return {
      isDropdownOpen: false,
      owner: process.env.owner || 'modrinth',
      slug: process.env.slug || 'knossos',
      branch: process.env.branch || 'master',
      hash: process.env.hash || 'unknown',
      isMobileMenuOpen: false,
      isBrowseMenuOpen: false,
      registeredSkipLink: null,
      hideDropdown: false,
    }
  },
  async fetch() {
    await Promise.all([
      this.$store.dispatch('user/fetchAll', { force: true }),
      this.$store.dispatch('cosmetics/fetchCosmetics', this.$cookies),
    ])
  },
  head() {
    const link = process.env.domain + this.$route.path.replace(/\/+$/, '')

    return {
      link: [
        {
          rel: 'canonical',
          href: link,
        },
      ],
      meta: [
        {
          hid: 'og:url',
          name: 'og:url',
          content: link,
        },
      ],
    }
  },
  computed: {
    authUrl() {
      return `${process.env.authURLBase}auth/init?url=${process.env.domain}${this.$route.path}`
    },
  },
  watch: {
    $route() {
      this.isMobileMenuOpen = false
      document.body.style.overflowY = 'scroll'

      this.$store.dispatch('user/fetchAll')

      document.body.setAttribute('tabindex', '-1')
      document.body.removeAttribute('tabindex')
    },
  },
  beforeCreate() {
    if (this.$route.query.code) {
      this.$router.push(this.$route.path)
    }
  },
  created() {
    this.$nuxt.$on('registerSkipLink', (data) => {
      this.registeredSkipLink = data
    })
  },
  methods: {
    toggleMobileMenu() {
      window.scrollTo(0, 0)
      document.body.scrollTop = 0

      this.isMobileMenuOpen = !this.isMobileMenuOpen

      if (this.isMobileMenuOpen) {
        document.body.style.overflowY = 'hidden'
        this.isBrowseMenuOpen = false
      } else {
        document.body.style.overflowY = 'scroll'
      }
    },
    toggleBrowseMenu() {
      this.isBrowseMenuOpen = !this.isBrowseMenuOpen

      if (this.isBrowseMenuOpen) {
        this.isMobileMenuOpen = false
      }
    },
    async logout() {
      this.$cookies.set('auth-token-reset', true)
      // If users logs out on dashboard, force redirect on the home page to clear cookies
      if (this.$route.path.startsWith('/settings/')) {
        window.location.href = '/settings'
      } else {
        await this.$router.go(null)

        this.$notify({
          group: 'main',
          title: 'Logged Out',
          text: 'You have logged out successfully!',
          type: 'success',
        })
      }
    },
    changeTheme() {
      this.$colorMode.preference =
        this.$colorMode.value === 'dark' ? 'light' : 'dark'
    },
  },
}
</script>

<style lang="scss">
.skip a {
  clip: rect(1px, 1px, 1px, 1px);
  height: 1px;
  overflow: hidden;
  position: absolute;
  white-space: nowrap;
  width: 1px;
}

.skip a:focus {
  clip: auto;
  height: auto;
  overflow: auto;
  position: absolute;
  width: auto;
  padding: 0.5rem 0.75rem;
  background-color: var(--color-brand);
  color: var(--color-brand-inverted);
  border-radius: var(--size-rounded-max);
  margin: 0 0.5rem 0 0;
  box-shadow: inset 0px -1px 1px rgba(17, 24, 39, 0.1);
  z-index: 1;
}

.layout {
  min-height: 100vh;
  background-color: var(--color-bg);
  display: block;

  @media screen and (min-width: 1024px) {
    min-height: calc(100vh - var(--spacing-card-bg));
  }

  @media screen and (max-width: 750px) {
    margin-bottom: calc(var(--size-mobile-navbar-height) + 2rem);
  }

  .site-header {
    margin-top: var(--spacing-card-md);
    margin-bottom: var(--spacing-card-md);
    max-width: 100vw;

    @media screen and (min-width: 1024px) {
      border-radius: var(--size-rounded-sm);
      max-width: 1280px;
      margin-left: auto;
      margin-right: auto;
    }

    .navbar {
      margin: 0 var(--spacing-card-lg);
      max-width: 1280px;
      margin-left: auto;
      margin-right: auto;

      @media screen and (max-width: 750px) {
        justify-content: center;
      }

      section.logo {
        display: flex;
        justify-content: space-between;
        color: var(--color-text-dark);

        a {
          align-items: center;
          display: flex;
        }

        .small-logo {
          display: block;
        }

        svg {
          height: 1.75rem;
          width: auto;
        }

        .badge {
          margin-left: 0.25rem;
          display: none;
          @media screen and (min-width: 430px) {
            display: unset;
          }
        }

        button {
          background: none;
          border: none;
          margin: 0 0 0 0.5rem;
          padding: 0;

          svg {
            height: 1.5rem;
            width: 1.5rem;
          }
        }
      }

      section.nav-group {
        display: flex;
        flex-grow: 5;

        section.nav {
          flex-grow: 5;

          .navigation {
            display: flex;
            width: fit-content;
            position: relative;
            top: 50%;
            transform: translateY(-50%);
            margin-left: 2rem;

            a {
              margin-left: 0;
              margin-right: auto;
            }

            a.tab {
              padding: 0;
              margin-right: 1rem;
              display: flex;
              align-items: flex-start;

              &--alpha::after {
                content: 'Alpha';
                background-color: var(--color-warning-bg);
                color: var(--color-warning-text);
                border-radius: 1rem;
                padding: 0.25rem 0.5rem;
                margin-left: 0.4rem;
                font-size: 0.7rem;
              }
            }
          }
        }

        .user-outer {
          z-index: 5;
        }

        section.user-controls {
          align-items: center;
          display: flex;
          flex-direction: row;
          justify-content: space-between;
          position: relative;
          top: 50%;
          transform: translateY(-50%);
          min-width: 6rem;

          .control-button {
            position: relative;
            display: flex;
            padding: 0.5rem 0.5rem;
            margin: 0 1rem 0 0;
            color: var(--color-text);
            border-radius: 2rem;
            transition: filter 0.1s ease-in-out;
            border: 2px solid transparent;
            box-sizing: border-box;

            svg {
              height: 1.25rem;
              width: 1.25rem;
            }

            &.bubble {
              &::after {
                background-color: var(--color-brand);
                border-radius: var(--size-rounded-max);
                content: '';
                height: 0.5rem;
                position: absolute;
                right: 0.25rem;
                top: 0.5rem;
                width: 0.5rem;
              }
            }

            //&.nuxt-link-exact-active {
            //  color: var(--color-button-text-active);
            //  background-color: var(--color-button-bg);
            //}
          }

          .hide-desktop {
            display: none;
          }

          .dropdown {
            position: relative;

            .control {
              align-items: center;
              background: none;
              display: flex;
              justify-content: center;
              padding: 0;

              .user-icon {
                height: 2rem;
                width: 2rem;
                outline: 2px solid var(--color-raised-bg);
                transition: outline-color 0.1s ease-in-out;
              }

              .caret {
                color: var(--color-button-text);
                margin-left: 0.25rem;
                width: 1rem;
              }
            }

            .content {
              border: 1px solid var(--color-divider-dark);
              list-style: none;
              margin: 0.5rem 0 0 0;
              max-width: 25rem;
              min-width: 12rem;
              opacity: 0;
              padding: 1rem;
              position: absolute;
              right: -1rem;
              transform: scaleY(0.9);
              transform-origin: top;
              transition: all 0.1s ease-in-out 0.05s, color 0s ease-in-out 0s,
                background-color 0s ease-in-out 0s,
                border-color 0s ease-in-out 0s;
              visibility: hidden;
              width: max-content;
              z-index: 1;
              box-shadow: var(--shadow-floating);

              .divider {
                background-color: var(--color-divider-dark);
                border: none;
                color: var(--color-divider-dark);
                height: 1px;
                margin: 0.5rem 0;
              }

              .item {
                align-items: center;
                border-radius: 0.5rem;
                box-sizing: border-box;
                color: inherit;
                display: flex;
                padding: 0.5rem;
                width: 100%;

                .icon {
                  margin-right: 0.5rem;
                  height: 20px;
                  width: 20px;
                }

                &.nuxt-link-exact-active {
                  color: var(--color-button-text-active);
                  background-color: var(--color-button-bg);

                  .profile-link {
                    .username {
                      margin-block: 0.7rem;
                    }
                    .prompt {
                      display: none;
                    }
                  }
                }
              }

              .profile-link {
                .prompt {
                  margin-top: 0.25rem;
                  color: var(--color-text-secondary);
                }
              }
            }

            @media screen and (max-width: 1300px) {
              .content {
                margin-right: 1rem;
              }
            }
          }

          .dropdown:hover .user-icon {
            outline-color: var(--color-brand);
          }

          .dropdown:hover:not(.closed) .content,
          .dropdown:focus:not(.closed) .content,
          .dropdown:focus-within:not(.closed) .content {
            opacity: 1;
            transform: scaleY(1);
            visibility: visible;
          }
        }

        section.auth-prompt {
          display: flex;
          align-items: center;
          height: 100%;
          margin: 0;

          .log-in-button {
            margin: 0 auto;
          }
        }
      }

      @media screen and (max-width: 750px) {
        display: none;
      }
    }

    .mobile-navbar {
      display: none;
      width: 100%;
      transition: height 0.25s ease-in-out;
      height: var(--size-mobile-navbar-height);
      position: fixed;
      left: 0;
      bottom: 0;
      background-color: var(--color-raised-bg);
      box-shadow: 0 0 20px 2px rgba(0, 0, 0, 0.3);
      z-index: 6;
      flex-direction: column;
      border-radius: var(--size-rounded-card) var(--size-rounded-card) 0 0;

      overflow: hidden;

      .tab {
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

        svg {
          height: 1.75rem;
          width: 1.75rem;
        }

        &:hover,
        &:focus {
          color: var(--color-text);
        }

        &.nuxt-link-exact-active {
          svg {
            color: var(--color-brand);
          }

          color: var(--color-text);
        }
      }

      .top-row {
        min-height: var(--size-mobile-navbar-height);
        display: flex;
        width: 100%;

        .browse {
          flex-grow: 10;

          svg {
            transition: transform 0.125s ease-in-out;

            &.closed {
              transform: rotate(180deg);
            }
          }
        }

        .tab {
          &:first-child {
            margin-left: 2rem;
          }

          &:last-child {
            margin-right: 2rem;
          }
        }

        .spacer {
          flex-grow: 1;
        }
      }

      .disable-children {
        a {
          pointer-events: none;
        }
      }

      .project-types {
        margin-top: 0.5rem;
        display: flex;
        justify-content: center;
        flex-wrap: wrap;
        row-gap: 0.5rem;
        margin-inline: var(--spacing-card-sm);

        .tab {
          flex: 0 0 fit-content;
          background-color: var(--color-button-bg);
          padding: 0.4rem 1.25rem;
          margin: 0 0.25rem;
          max-height: unset;

          &.nuxt-link-exact-active {
            color: var(--color-brand-inverted);
            background-color: var(--color-brand);
          }
        }
      }

      @media screen and (max-width: 750px) {
        display: flex;
      }

      &.expanded {
        height: var(--size-mobile-navbar-height-expanded);
      }
    }
  }

  .mobile-menu {
    display: none;
    position: absolute;
    top: 0;
    left: 0;
    background-color: var(--color-bg);
    height: 100%;
    width: 100%;
    z-index: 5;

    .mobile-menu-wrapper {
      max-height: calc(100vh - var(--size-mobile-navbar-height));
      margin-bottom: var(--size-mobile-navbar-height);
      overflow-y: auto;
      margin-top: auto;

      .items-container {
        margin: 1rem 2rem;

        .iconified-button {
          box-sizing: border-box;
          padding: 0.85rem 1.5rem;
          align-items: center;
          justify-content: center;
          display: flex;
          column-gap: 0.25rem;
          width: 100%;
          max-width: 20rem;
          max-height: unset;

          svg {
            height: 1.25rem;
            width: 1.25rem;
          }

          &.nuxt-link-exact-active {
            color: var(--color-brand-inverted);
            background-color: var(--color-brand);

            .profile-link {
              .prompt {
                display: none;
              }
            }

            .beta-badge {
              background-color: var(--color-brand-inverted);
              color: var(--color-text-dark);
            }
          }

          &.user-item {
            flex-direction: column;
            row-gap: 0.5rem;
            width: fit-content;
            max-width: 16rem;
            flex-grow: 0;
            padding-inline: 3rem;

            .profile-link {
              text-align: center;

              .prompt {
                color: var(--color-text-secondary);
              }
            }

            .user-icon {
              width: 4rem;
              height: 4rem;
              border-radius: var(--size-rounded-max);
            }
          }
        }
      }
    }

    div {
      flex-grow: 1;
      justify-content: end;
      align-items: center;
      row-gap: 1rem;
    }

    &.active {
      display: flex;

      @media screen and (min-width: 750px) {
        display: none;
      }
    }
  }

  main {
    grid-area: main;
  }

  footer {
    margin: 6rem 0 2rem 0;
    text-align: center;
    display: grid;
    grid-template:
      'logo-info  logo-info  logo-info' auto
      'links-1    links-2    links-3' auto
      'buttons    buttons    buttons' auto
      'notice     notice     notice' auto
      / 1fr 1fr 1fr;

    .logo-info {
      margin-left: auto;
      margin-right: auto;
      max-width: 20rem;
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
      width: fit-content;
      grid-template:
        'logo-info  links-1 links-2 links-3 buttons' auto
        'notice     notice  notice  notice  notice' auto;
      text-align: unset;

      .logo-info {
        margin-right: 2rem;
      }

      .links {
        margin-right: 2rem;
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
</style>
<style src="vue-multiselect/dist/vue-multiselect.min.css"></style>
