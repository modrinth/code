<template>
  <div class="layout">
    <header class="site-header">
      <section class="navbar columns">
        <section class="logo column">
          <NuxtLink to="/">
            <ModrinthLogoSmall aria-label="modrinth" class="small-logo" />
            <ModrinthLogo
              v-if="$colorMode.value === 'light'"
              aria-label="modrinth"
              class="text-logo"
            />
            <ModrinthLogoWhite v-else aria-label="modrinth" class="text-logo" />
          </NuxtLink>
          <span class="badge yellow">Beta</span>
        </section>
        <section class="menu-icon">
          <button class="theme-toggle" @click="changeTheme">
            <MoonIcon v-if="$colorMode.value === 'light'" />
            <SunIcon v-else />
          </button>
          <button @click="toggleNavBar">
            <HamburgerIcon />
          </button>
        </section>
        <section ref="nav" class="right-group columns">
          <section class="column-grow-5 nav">
            <div class="tabs">
              <NuxtLink
                to="/mods"
                class="tab"
                :class="{
                  'active-path': this.$route.path.startsWith('/mod'),
                }"
              >
                <span>Mods</span>
              </NuxtLink>
              <div v-if="this.$auth.user" class="section">
                <NuxtLink
                  to="/dashboard/projects"
                  class="tab"
                  :class="{
                    'active-path': this.$route.path.startsWith('/dashboard'),
                  }"
                >
                  <span>Dashboard</span>
                </NuxtLink>
              </div>
            </div>
          </section>
          <section class="column-grow">
            <template v-if="this.$auth.user">
              <section class="user-controls">
                <div
                  v-click-outside="hideDropdown"
                  class="dropdown"
                  :class="{ open: isDropdownOpen }"
                >
                  <button class="control" @click="toggleDropdown">
                    <div class="avatar">
                      <span>{{ this.$auth.user.username }}</span>
                      <AvatarIcon
                        :notif-count="this.$user.notifications.count"
                        :dropdown-bg="isDropdownOpen"
                      />
                    </div>
                    <DropdownIcon class="dropdown-icon" />
                  </button>
                  <div class="content">
                    <ul v-if="isDropdownOpen" @click="hideDropdown">
                      <li>
                        <NuxtLink :to="userUrl">
                          <UserIcon />
                          <span>Profile</span>
                        </NuxtLink>
                      </li>
                      <li>
                        <NuxtLink to="/dashboard/notifications">
                          <NotificationIcon />
                          <span>Notifications</span>
                        </NuxtLink>
                      </li>
                      <li>
                        <NuxtLink to="/dashboard/settings">
                          <SettingsIcon />
                          <span>Settings</span>
                        </NuxtLink>
                      </li>
                      <!--<li v-tooltip="'Not implemented yet'" class="hidden">
                          <NuxtLink :to="userTeamsUrl" disabled>
                            <UsersIcon />
                            <span>Teams</span>
                          </NuxtLink>
                        </li>-->
                      <li>
                        <button @click="changeTheme">
                          <MoonIcon v-if="$colorMode.value === 'light'" />
                          <SunIcon v-else />
                          <span v-if="$colorMode.value === 'light'">
                            Dark Mode</span
                          >
                          <span v-else>Light Mode</span>
                        </button>
                      </li>
                      <hr />
                      <li>
                        <button @click="logout">
                          <LogOutIcon />
                          <span>Log Out</span>
                        </button>
                      </li>
                    </ul>
                  </div>
                </div>
              </section>
            </template>
            <template v-else>
              <section class="auth-prompt">
                <a class="desktop-header-mode-switch" @click="changeTheme">
                  <MoonIcon v-if="$colorMode.value === 'light'" />
                  <SunIcon v-else />
                </a>
                <a :href="authUrl" class="log-in-button"
                  ><GitHubIcon aria-hidden="true" />Sign In with GitHub</a
                >
              </section>
            </template>
          </section>
        </section>
      </section>
    </header>
    <main>
      <CookieConsent />
      <notifications
        group="main"
        position="bottom right"
        :max="5"
        :ignore-duplicates="true"
      />
      <!--<notifications
        group="ads"
        position="bottom right"
        :duration="-1"
        :ignore-duplicates="true"
      />-->
      <Nuxt />
    </main>
  </div>
</template>

<script>
import ClickOutside from 'vue-click-outside'

import ModrinthLogo from '~/assets/images/text-logo.svg?inline'
import ModrinthLogoWhite from '~/assets/images/text-logo-white.svg?inline'
import ModrinthLogoSmall from '~/assets/images/logo.svg?inline'

import HamburgerIcon from '~/assets/images/utils/hamburger.svg?inline'

import NotificationIcon from '~/assets/images/sidebar/notifications.svg?inline'
import SettingsIcon from '~/assets/images/sidebar/settings.svg?inline'

import DropdownIcon from '~/assets/images/utils/dropdown.svg?inline'
import MoonIcon from '~/assets/images/utils/moon.svg?inline'
import SunIcon from '~/assets/images/utils/sun.svg?inline'

import UserIcon from '~/assets/images/utils/user.svg?inline'
import LogOutIcon from '~/assets/images/utils/log-out.svg?inline'
import GitHubIcon from '~/assets/images/utils/github.svg?inline'

import CookieConsent from '~/components/ads/CookieConsent'

import AvatarIcon from '~/components/ui/AvatarIcon'

export default {
  components: {
    ModrinthLogo,
    ModrinthLogoWhite,
    ModrinthLogoSmall,
    DropdownIcon,
    MoonIcon,
    SunIcon,
    UserIcon,
    LogOutIcon,
    GitHubIcon,
    NotificationIcon,
    HamburgerIcon,
    CookieConsent,
    AvatarIcon,
    SettingsIcon,
  },
  directives: {
    ClickOutside,
  },
  data() {
    return {
      isDropdownOpen: false,
    }
  },
  computed: {
    authUrl() {
      return `${this.$axios.defaults.baseURL}auth/init?url=${this.$store.app.$config.utils.domain}${this.$route.path}`
    },
    userUrl() {
      return `/user/${this.$auth.user.id}`
    },
    userTeamsUrl() {
      return `${this.userUrl}/teams`
    },
  },
  watch: {
    $route() {
      this.$refs.nav.className = 'right-group'
      document.body.style.overflow = 'auto'
      this.$store.dispatch('user/fetchNotifications')
    },
  },
  created() {
    this.$store.dispatch('user/fetchNotifications', { force: true })
  },
  beforeCreate() {
    if (this.$route.query.code) {
      this.$router.push(this.$route.path)
    }
  },
  methods: {
    toggleNavBar() {
      window.scrollTo(0, 0)
      const currentlyActive = this.$refs.nav.className === 'right-group active'
      this.$refs.nav.className = `right-group${
        currentlyActive ? '' : ' active'
      }`
      document.body.scrollTop = 0
      document.body.style.overflow =
        document.body.style.overflow !== 'hidden' ? 'hidden' : 'auto'
    },
    toggleDropdown() {
      this.isDropdownOpen = !this.isDropdownOpen
    },
    hideDropdown() {
      this.isDropdownOpen = false
    },
    async logout() {
      this.$cookies.set('auth-token-reset', true)
      // If users logs out on dashboard, force redirect on the home page to clear cookies
      if (this.$route.path.startsWith('/dashboard')) {
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
      this.$colorMode.preference =
        this.$colorMode.value === 'dark' ? 'light' : 'dark'
    },
  },
}
</script>

<style lang="scss">
.layout {
  background-color: var(--color-bg);
  display: block;

  .site-header {
    height: var(--size-navbar-height);
    background-color: var(--color-raised-bg);
    max-width: 100vw;
    .navbar {
      margin: 0 0.5rem;
      @media screen and (min-width: 450px) {
        margin: 0 var(--spacing-card-lg);
      }
      section.logo {
        align-items: center;
        display: flex;
        justify-content: space-between;
        padding: 1rem 0;
        margin-left: 1rem;
        color: var(--color-text-dark);
        .small-logo {
          display: block;
        }
        svg {
          display: none;
          height: 1.75rem;
          width: auto;
        }
        @media screen and (min-width: 350px) {
          .small-logo {
            display: none;
          }
          svg {
            display: unset;
          }
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

      section.menu-icon {
        display: flex;
        margin-left: auto;
        align-items: center;
        margin-right: 1rem;
      }

      .desktop-header-mode-switch {
        margin-right: 1rem;
        cursor: pointer;
      }

      section.right-group {
        display: flex;
        flex-grow: 5;

        flex-direction: column-reverse;

        overflow-y: auto;
        position: fixed;
        width: 100vw;
        top: var(--size-navbar-height);
        height: calc(100vh - var(--size-navbar-height));
        right: -100vw;
        background-color: var(--color-raised-bg);
        transition: right 150ms;
        z-index: 100;

        &.active {
          right: 0;
        }

        section.nav {
          .tabs {
            flex-direction: column;

            .section {
              border-top: 3px solid var(--color-brand-disabled);
              margin-top: 0.75rem;
              padding-top: 0.75rem;
            }
            .tab {
              font-size: var(--font-size-md);

              span {
                margin: 0 auto;
              }
            }
          }
        }
        section.user-controls {
          align-items: center;
          display: flex;
          justify-content: space-between;
          position: relative;
          top: 50%;
          transform: translateY(-50%);
          min-width: 12rem;

          width: 13rem;
          margin: 0 auto;

          .dropdown {
            position: relative;
            display: inline-block;
            flex-grow: 1;
            &:hover .control {
              border-radius: var(--size-rounded-control);
              background: var(--color-button-bg);
            }
            &.open {
              .control {
                background: var(--color-button-bg);
                border-radius: var(--size-rounded-control)
                  var(--size-rounded-control) 0 0;
                .dropdown-icon {
                  transform: rotate(180deg);
                }
              }
              .content {
                display: unset;
              }
            }
            .control {
              border-radius: var(--size-rounded-control);
              align-items: center;
              display: flex;
              padding: 0.3rem 0.75rem;
              position: relative;
              z-index: 11;
              width: 100%;
              .avatar {
                align-items: center;
                display: flex;
                flex-grow: 1;
                span {
                  display: block;
                  overflow: hidden;
                  text-overflow: ellipsis;
                  white-space: nowrap;
                  color: var(--color-text-dark);
                  font-weight: var(--font-weight-medium);
                }
              }
              .dropdown-icon {
                color: var(--color-text-dark);
                transition: 150ms ease transform;
              }
            }
            .content {
              margin: 0 0 0 0;
              min-width: 10rem;
              width: 100%;
              position: fixed;
              display: none;
            }
            button {
              background-color: transparent;
              color: var(--color-text-dark);
              margin: 0;
              padding: 0;
              font-weight: var(--font-weight-medium);
            }
            ul {
              background-color: var(--color-button-bg);
              border-radius: 0 0 var(--size-rounded-control)
                var(--size-rounded-control);
              box-shadow: var(--shadow-dropdown);
              display: flex;
              flex-direction: column;
              margin: 0;
              list-style: none;
              padding: 0.5rem 0;
              z-index: 1;
              hr {
                background-color: var(--color-divider-dark);
                border: none;
                color: var(--color-divider-dark);
                height: 2px;
                margin: 0.5rem 0;
              }
              li {
                margin: 0;
                &:hover,
                &:focus,
                &:active {
                  background-color: var(--color-button-bg-active);
                  color: var(--color-text-dark);
                }
                a,
                button {
                  align-items: center;
                  display: flex;
                  padding: 0.75rem 1.5rem;
                  color: var(--color-text-dark);
                  svg {
                    color: inherit;
                    height: 1rem;
                    width: 1rem;
                  }
                  span {
                    margin-left: 0.5rem;
                  }
                }
              }
            }
          }
        }

        section.auth-prompt {
          display: flex;
          align-items: center;
          height: 100%;

          .log-in-button {
            margin: 0 auto;

            text-align: center;
            border-radius: var(--size-rounded-control);
            background-color: var(--color-brand);
            white-space: nowrap;
            outline: none;
            color: var(--color-brand-inverted);
            display: block;
            padding: 0.5rem 0.75rem;
            svg {
              vertical-align: middle;
              margin-right: 0.5rem;
            }
            &:hover,
            &:focus {
              background-color: var(--color-brand-2);
            }
          }
        }
      }
      @media screen and (min-width: 1024px) {
        max-width: 1280px;
        margin-left: auto;
        margin-right: auto;

        section.menu-icon {
          display: none;
        }

        section.mobile-header-mode-switch {
          display: none;
        }

        section.right-group {
          flex-direction: unset;
          overflow-y: unset;
          position: unset;
          width: unset;
          top: unset;
          height: unset;
          right: unset;
          background-color: unset;
          transition: unset;
          z-index: unset;

          section.nav {
            .tabs {
              flex-direction: unset;
              position: relative;
              top: 50%;
              transform: translateY(-50%);

              .section {
                margin-top: unset;
                padding-top: unset;
                border-top: unset;

                border-left: 3px solid var(--color-brand-disabled);
                margin-left: 0.75rem;
                padding-left: 0.75rem;
              }
            }
          }

          section.user-controls {
            width: unset;
            margin: unset;
          }

          section.auth-prompt {
            margin: 0;
          }
        }
      }
      @media only screen and (max-width: 1024px) {
        .desktop-header-mode-switch {
          display: none;
        }
      }
    }
  }

  main {
    grid-area: main;
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
