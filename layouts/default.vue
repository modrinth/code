<template>
  <div ref="main_page" class="layout" :class="{ 'expanded-mobile-nav': isBrowseMenuOpen }">
    <header class="site-header" role="presentation">
      <section class="navbar columns" role="navigation">
        <section class="logo column" role="presentation">
          <NuxtLink class="button-base" to="/" aria-label="Modrinth home page">
            <BrandTextLogo aria-hidden="true" class="text-logo" />
          </NuxtLink>
        </section>
        <section class="nav-group columns" role="presentation">
          <section class="nav" aria-label="Page links">
            <NavRow class="navigation" :links="navRoutes" />
          </section>
          <section class="column-grow user-outer" aria-label="Account links">
            <section class="user-controls">
              <nuxt-link
                v-if="auth.user"
                to="/notifications"
                class="control-button button-transparent"
                :class="{ bubble: user.notifications.length > 0 }"
                title="Notifications"
              >
                <NotificationIcon aria-hidden="true" />
              </nuxt-link>
              <button
                class="control-button button-transparent"
                title="Switch theme"
                @click="changeTheme"
              >
                <MoonIcon v-if="$colorMode.value === 'light'" aria-hidden="true" />
                <SunIcon v-else aria-hidden="true" />
              </button>
              <div
                v-if="auth.user"
                class="dropdown"
                :class="{ closed: !isDropdownOpen }"
                tabindex="0"
                @mouseover="isDropdownOpen = true"
                @focus="isDropdownOpen = true"
                @mouseleave="isDropdownOpen = false"
              >
                <button class="control" value="Profile Dropdown">
                  <Avatar
                    :src="auth.user.avatar_url"
                    class="user-icon"
                    alt="Your avatar"
                    aria-hidden="true"
                    circle
                  />
                  <DropdownIcon class="caret" />
                </button>
                <div class="content card">
                  <NuxtLink class="item button-transparent" :to="`/user/${auth.user.username}`">
                    <div class="title profile-link">
                      <div class="username">@{{ auth.user.username }}</div>
                      <div class="prompt">Visit your profile</div>
                    </div>
                  </NuxtLink>
                  <hr class="divider" />
                  <button class="item button-transparent" @click="$refs.modal_creation.show()">
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
                    <span class="title">Dashboard</span><span class="beta-badge">BETA</span>
                  </NuxtLink>
                  <NuxtLink class="item button-transparent" to="/settings/follows">
                    <HeartIcon class="icon" />
                    <span class="title">Following</span>
                  </NuxtLink>
                  <NuxtLink class="item button-transparent" to="/settings">
                    <SettingsIcon class="icon" />
                    <span class="title">Settings</span>
                  </NuxtLink>
                  <NuxtLink
                    v-if="$tag.staffRoles.includes($auth.user.role)"
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
                  :href="getAuthUrl()"
                  class="log-in-button header-button brand-button"
                  rel="noopener nofollow"
                >
                  <GitHubIcon aria-hidden="true" />
                  Sign in with GitHub</a
                >
              </section>
            </section>
          </section>
        </section>
      </section>
      <section class="mobile-navigation">
        <div class="nav-menu nav-menu-browse" :class="{ expanded: isBrowseMenuOpen }">
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
        <div class="nav-menu nav-menu-mobile" :class="{ expanded: isMobileMenuOpen }">
          <div class="account-container">
            <NuxtLink
              v-if="auth.user"
              :to="`/user/${auth.user.username}`"
              class="iconified-button account-button"
            >
              <Avatar
                :src="auth.user.avatar_url"
                class="user-icon"
                alt="Your avatar"
                aria-hidden="true"
                circle
              />
              <div class="account-text">
                <div>@{{ auth.user.username }}</div>
                <div>Visit your profile</div>
              </div>
            </NuxtLink>
            <NuxtLink v-else class="iconified-button brand-button" :to="getAuthUrl()">
              <GitHubIcon aria-hidden="true" />
              Sign in with GitHub
            </NuxtLink>
          </div>
          <div class="links">
            <template v-if="auth.user">
              <button class="iconified-button danger-button" @click="logout">
                <LogOutIcon aria-hidden="true" />
                Log out
              </button>
              <button class="iconified-button" @click="$refs.modal_creation.show()">
                <PlusIcon aria-hidden="true" />
                Create a project
              </button>
              <NuxtLink class="iconified-button" to="/settings/follows">
                <HeartIcon aria-hidden="true" />
                Following
              </NuxtLink>
              <NuxtLink
                v-if="auth.user.role === 'moderator' || auth.user.role === 'admin'"
                class="iconified-button"
                to="/moderation"
              >
                <ModerationIcon aria-hidden="true" />
                Moderation
              </NuxtLink>
            </template>
            <NuxtLink class="iconified-button" to="/settings">
              <SettingsIcon aria-hidden="true" />
              Settings
            </NuxtLink>
            <button class="iconified-button" @click="changeTheme">
              <MoonIcon v-if="$colorMode.value === 'light'" class="icon" />
              <SunIcon v-else class="icon" />
              <span class="dropdown-item__text">Change theme</span>
            </button>
          </div>
        </div>
        <div class="mobile-navbar" :class="{ expanded: isBrowseMenuOpen || isMobileMenuOpen }">
          <NuxtLink to="/" class="tab button-animation" title="Home">
            <HomeIcon />
          </NuxtLink>
          <button
            class="tab button-animation"
            :class="{ 'router-link-exact-active': isBrowseMenuOpen }"
            title="Search"
            @click="toggleBrowseMenu()"
          >
            <template v-if="auth.user">
              <SearchIcon />
            </template>
            <template v-else>
              <SearchIcon class="smaller" />
              Search
            </template>
          </button>
          <template v-if="auth.user">
            <NuxtLink
              to="/notifications"
              class="tab button-animation"
              :class="{
                bubble: user.notifications.length > 0,
                'no-active': isMobileMenuOpen || isBrowseMenuOpen,
              }"
              title="Notifications"
              @click="
                () => {
                  isMobileMenuOpen = false
                  isBrowseMenuOpen = false
                }
              "
            >
              <NotificationIcon />
            </NuxtLink>
            <NuxtLink to="/dashboard" class="tab button-animation" title="Dashboard">
              <ChartIcon />
            </NuxtLink>
          </template>
          <button
            class="tab button-animation"
            title="Toggle Mobile Menu"
            @click="toggleMobileMenu()"
          >
            <template v-if="!auth.user">
              <HamburgerIcon v-if="!isMobileMenuOpen" />
              <CrossIcon v-else />
            </template>
            <template v-else>
              <Avatar
                :src="auth.user.avatar_url"
                class="user-icon"
                :class="{ expanded: isMobileMenuOpen }"
                alt="Your avatar"
                aria-hidden="true"
                circle
              />
            </template>
          </button>
        </div>
      </section>
    </header>
    <main>
      <ModalCreation v-if="auth.user" ref="modal_creation" />
      <slot id="main" />
    </main>
    <footer>
      <div class="logo-info" role="region" aria-label="Modrinth information">
        <BrandTextLogo aria-hidden="true" class="text-logo" />
        <p>
          Modrinth is
          <a
            :target="$external()"
            href="https://github.com/modrinth"
            class="text-link"
            rel="noopener"
          >
            open source</a
          >.
        </p>
        <p>
          {{ config.public.owner }}/{{ config.public.slug }} {{ config.public.branch }}@<a
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
        <h4 aria-hidden="true">Company</h4>
        <nuxt-link to="/legal/terms"> Terms </nuxt-link>
        <nuxt-link to="/legal/privacy"> Privacy </nuxt-link>
        <nuxt-link to="/legal/rules"> Rules </nuxt-link>
        <a :target="$external()" href="https://careers.modrinth.com"> Careers </a>
      </div>
      <div class="links links-2" role="region" aria-label="Resources">
        <h4 aria-hidden="true">Resources</h4>
        <a :target="$external()" href="https://blog.modrinth.com">Blog</a>
        <a :target="$external()" href="https://docs.modrinth.com">Docs</a>
        <a :target="$external()" href="https://status.modrinth.com">Status</a>
        <a rel="noopener" :target="$external()" href="https://github.com/modrinth">GitHub</a>
      </div>
      <div class="links links-3" role="region" aria-label="Interact">
        <h4 aria-hidden="true">Interact</h4>
        <a rel="noopener" :target="$external()" href="https://discord.gg/EUHuJHt"> Discord </a>
        <a rel="noopener" :target="$external()" href="https://twitter.com/modrinth"> Twitter </a>
        <a rel="noopener" :target="$external()" href="https://floss.social/@modrinth"> Mastodon </a>
        <a rel="noopener" :target="$external()" href="https://crowdin.com/project/modrinth">
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
        NOT AN OFFICIAL MINECRAFT PRODUCT. NOT APPROVED BY OR ASSOCIATED WITH MOJANG.
      </div>
    </footer>
  </div>
</template>
<script setup>
import HamburgerIcon from '~/assets/images/utils/hamburger.svg'
import CrossIcon from '~/assets/images/utils/x.svg'
import SearchIcon from '~/assets/images/utils/search.svg'

import NotificationIcon from '~/assets/images/sidebar/notifications.svg'
import SettingsIcon from '~/assets/images/sidebar/settings.svg'
import ModerationIcon from '~/assets/images/sidebar/admin.svg'
import HomeIcon from '~/assets/images/sidebar/home.svg'

import MoonIcon from '~/assets/images/utils/moon.svg'
import SunIcon from '~/assets/images/utils/sun.svg'
import PlusIcon from '~/assets/images/utils/plus.svg'
import DropdownIcon from '~/assets/images/utils/dropdown.svg'
import LogOutIcon from '~/assets/images/utils/log-out.svg'
import HeartIcon from '~/assets/images/utils/heart.svg'
import ChartIcon from '~/assets/images/utils/chart.svg'

import GitHubIcon from '~/assets/images/utils/github.svg'
import NavRow from '~/components/ui/NavRow'
import ModalCreation from '~/components/ui/ModalCreation'
import Avatar from '~/components/ui/Avatar'

const auth = await useAuth()
const user = await useUser()

const config = useRuntimeConfig()
const route = useRoute()
const link = config.public.siteUrl + route.path.replace(/\/+$/, '')
useHead({
  meta: [{ name: 'og:url', content: link }],
  link: [
    {
      rel: 'canonical',
      href: link,
    },
  ],
})
</script>
<script>
export default defineNuxtComponent({
  data() {
    return {
      isDropdownOpen: false,
      isMobileMenuOpen: false,
      isBrowseMenuOpen: false,
      registeredSkipLink: null,
      hideDropdown: false,
      navRoutes: [
        {
          label: 'Mods',
          href: '/mods',
        },
        {
          label: 'Plugins',
          href: '/plugins',
        },
        {
          label: 'Data Packs',
          href: '/datapacks',
        },
        {
          label: 'Shaders',
          href: '/shaders',
        },
        {
          label: 'Resource Packs',
          href: '/resourcepacks',
        },
        {
          label: 'Modpacks',
          href: '/modpacks',
        },
      ],
    }
  },
  computed: {
    isOnSearchPage() {
      return this.navRoutes.some((route) => this.$route.path.startsWith(route.href))
    },
  },
  watch: {
    '$route.path'() {
      this.isMobileMenuOpen = false
      this.isBrowseMenuOpen = false

      if (process.client) {
        document.body.style.overflowY = 'scroll'
        document.body.setAttribute('tabindex', '-1')
        document.body.removeAttribute('tabindex')
      }

      updateCurrentDate()
      this.runAnalytics()
    },
  },
  mounted() {
    this.runAnalytics()
    if (this.$route.query.code) {
      window.history.replaceState(history.state, null, this.$route.path)
    }
  },
  methods: {
    runAnalytics() {
      const config = useRuntimeConfig()

      setTimeout(() => {
        $fetch(`${config.public.ariadneBaseUrl}view`, {
          method: 'POST',
          body: {
            url: window.location.href,
          },
        })
          .then(() => {})
          .catch((e) => {
            console.error('An error occurred while registering the visit: ', e)
          })
      })
    },
    toggleMobileMenu() {
      this.isMobileMenuOpen = !this.isMobileMenuOpen
      if (this.isMobileMenuOpen) {
        this.isBrowseMenuOpen = false
      }
    },
    toggleBrowseMenu() {
      this.isBrowseMenuOpen = !this.isBrowseMenuOpen

      if (this.isBrowseMenuOpen) {
        this.isMobileMenuOpen = false
      }
    },
    async logout() {
      useCookie('auth-token').value = null

      // If users logs out on dashboard, force redirect on the home page to clear cookies
      if (
        this.$route.path.startsWith('/settings/') ||
        this.$route.path.startsWith('/dashboard/') ||
        this.$route.path.startsWith('/moderation') ||
        this.$route.path.startsWith('/notifications')
      ) {
        window.location.href = '/'
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
      updateTheme(this.$colorMode.value === 'dark' ? 'light' : 'dark', true)
    },
  },
})
</script>

<style lang="scss">
@import '~/assets/styles/global.scss';

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
    max-width: 100vw;

    @media screen and (min-width: 1024px) {
      margin-top: var(--spacing-card-md);
      margin-bottom: var(--spacing-card-md);
    }

    @media screen and (min-width: 1280px) {
      border-radius: var(--size-rounded-sm);
      max-width: 1280px;
      margin-left: auto;
      margin-right: auto;
    }

    .navbar {
      padding: 0 var(--spacing-card-lg);
      margin: 0 var(--spacing-card-lg);
      max-width: 1280px;
      margin-left: auto;
      margin-right: auto;

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
            grid-gap: 1.5rem;

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
          gap: 1rem;

          .control-button {
            position: relative;
            display: flex;
            padding: 0.5rem;
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

            //&.router-link-exact-active {
            //  color: var(--color-button-text-active);
            //  background-color: var(--color-button-bg);
            //}
          }

          .hide-desktop {
            display: none;
          }

          .dropdown {
            position: relative;
            margin-left: 0.5rem;

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
                background-color 0s ease-in-out 0s, border-color 0s ease-in-out 0s;
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

                &.router-link-exact-active {
                  color: var(--color-button-text-active);
                  background-color: var(--color-button-bg);
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

      @media screen and (max-width: 1095px) {
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
        height: var(--size-mobile-navbar-height);
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
              content: '';
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
      @media screen and (max-width: 1095px) {
        display: flex;
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

      @media screen and (min-width: 1095px) {
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
        padding: 0 0.25rem;
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
        'logo-info  links-1 links-2 links-3 buttons' auto
        'notice     notice  notice  notice  notice' auto;
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
</style>
<style src="vue-multiselect/dist/vue-multiselect.css"></style>
