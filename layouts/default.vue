<template>
  <div class="layout">
    <aside>
      <label class="hidden" for="toggle-nav-menu">Toggle Nav Menu</label>
      <input
        id="toggle-nav-menu"
        class="hamburger-button"
        alt="Open navigation menu"
        type="checkbox"
        @click="toggleNavMenu()"
      />
      <!-- TODO: Probably shouldn't be a Unicode symbol -->
      <div class="hamburger-icon">☰</div>
      <nuxt-link to="/" class="logo-wrapper">
        <img class="logo" src="~/assets/images/logo.svg" alt="modrinth-logo" />
        <span class="name">modrinth</span>
      </nuxt-link>
      <nav>
        <section class="links">
          <h3>Projects</h3>
          <section>
            <nuxt-link to="/modpacks">
              <svg
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2"
                stroke-linecap="round"
                stroke-linejoin="round"
              >
                <line x1="16.5" y1="9.4" x2="7.5" y2="4.21" />
                <path
                  d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z"
                />
                <polyline points="3.27 6.96 12 12.01 20.73 6.96" />
                <line x1="12" y1="22.08" x2="12" y2="12" />
              </svg>
              <span> Modpacks </span>
            </nuxt-link>
            <nuxt-link to="/mods">
              <svg
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2"
                stroke-linecap="round"
                stroke-linejoin="round"
              >
                <polyline points="16 18 22 12 16 6" />
                <polyline points="8 6 2 12 8 18" />
              </svg>
              <span>Mods</span>
            </nuxt-link>
          </section>

          <h3>Community</h3>
          <section>
            <nuxt-link to="/support">
              <svg
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2"
                stroke-linecap="round"
                stroke-linejoin="round"
              >
                <circle cx="12" cy="12" r="10" />
                <circle cx="12" cy="12" r="4" />
                <line x1="4.93" y1="4.93" x2="9.17" y2="9.17" />
                <line x1="14.83" y1="14.83" x2="19.07" y2="19.07" />
                <line x1="14.83" y1="9.17" x2="18.36" y2="5.64" />
                <line x1="4.93" y1="19.07" x2="9.17" y2="14.83" />
              </svg>
              <span>Support</span>
            </nuxt-link>
            <nuxt-link to="/guides">
              <svg
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2"
                stroke-linecap="round"
                stroke-linejoin="round"
              >
                <path d="M2 3h6a4 4 0 0 1 4 4v14a3 3 0 0 0-3-3H2z" />
                <path d="M22 3h-6a4 4 0 0 0-4 4v14a3 3 0 0 1 3-3h7z" />
              </svg>
              <span>Guides</span>
            </nuxt-link>
          </section>

          <h3 v-if="this.$auth.loggedIn">Dashboard</h3>
          <section v-if="this.$auth.loggedIn">
            <nuxt-link to="/dashboard/projects">
              <svg
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2"
                stroke-linecap="round"
                stroke-linejoin="round"
              >
                <path
                  d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"
                />
              </svg>
              <span>My projects</span>
            </nuxt-link>
            <nuxt-link to="/dashboard/analytics">
              <svg
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2"
                stroke-linecap="round"
                stroke-linejoin="round"
              >
                <polyline points="22 12 18 12 15 21 9 3 6 12 2 12" />
              </svg>
              <span>Analytics</span>
            </nuxt-link>
          </section>
        </section>
        <div class="disclosure">
          <span>
            Modrinth is open source software. You may view the source code at
            our
            <a href="https://github.com/modrinth/knossos">GitHub Repository</a>.
          </span>
        </div>
        <section class="user-actions">
          <a
            v-if="!this.$auth.loggedIn"
            :href="
              'https://api.modrinth.com/api/v1/auth/init?url=http://modrinth.com' +
              this.$route.path
            "
            class="log-in-button"
          >
            Log In
          </a>
          <div v-if="this.$auth.loggedIn" class="avatar">
            <img :src="this.$auth.user.avatar_url" alt="avatar" />
            <span> {{ this.$auth.user.username }} </span>
          </div>
          <div v-if="this.$auth.loggedIn" class="notifications">
            <div v-if="showPopup" class="user-actions-popup">
              <div class="popup-inner">
                <p>
                  Modrinth ID: <strong>{{ this.$auth.user.id }}</strong>
                </p>
                <hr />
                <p class="hover">
                  <nuxt-link :to="'/user/' + this.$auth.user.id">
                    My profile
                  </nuxt-link>
                </p>
                <p class="hover">My teams</p>
                <hr />
                <p class="hover" @click="logout">Logout</p>
              </div>
            </div>
            <SettingsIcon @click="showPopup = !showPopup" />
          </div>
          <div class="theme">
            <svg
              v-if="$colorMode.value === 'light'"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
              @click="
                $colorMode.preference =
                  $colorMode.value === 'dark' ? 'light' : 'dark'
              "
            >
              <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"></path>
            </svg>
            <svg
              v-else
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
              @click="
                $colorMode.preference =
                  $colorMode.value === 'dark' ? 'light' : 'dark'
              "
            >
              <circle cx="12" cy="12" r="5"></circle>
              <line x1="12" y1="1" x2="12" y2="3"></line>
              <line x1="12" y1="21" x2="12" y2="23"></line>
              <line x1="4.22" y1="4.22" x2="5.64" y2="5.64"></line>
              <line x1="18.36" y1="18.36" x2="19.78" y2="19.78"></line>
              <line x1="1" y1="12" x2="3" y2="12"></line>
              <line x1="21" y1="12" x2="23" y2="12"></line>
              <line x1="4.22" y1="19.78" x2="5.64" y2="18.36"></line>
              <line x1="18.36" y1="5.64" x2="19.78" y2="4.22"></line>
            </svg>
          </div>
        </section>
      </nav>
    </aside>
    <main>
      <nuxt />
    </main>
  </div>
</template>

<script>
import SettingsIcon from '~/assets/images/utils/settings.svg?inline'

export default {
  components: {
    SettingsIcon,
  },
  async fetch() {
    if (this.$route.query.code)
      await this.$auth.setUserToken(this.$route.query.code)
  },
  data() {
    return {
      showPopup: false,
    }
  },
  methods: {
    toggleNavMenu() {
      document.body.style.overflow =
        document.body.style.overflow !== 'hidden' ? 'hidden' : 'auto'
    },
    logout() {
      this.$auth.setToken('local', false)
      this.$router.go(null)
    },
  },
}
</script>

<style lang="scss">
.layout {
  display: flex;
  flex-flow: column;
  min-height: 100vh;
  width: 100%;

  // Desktop
  @media screen and (min-width: 1145px) {
    flex-flow: row;
  }

  aside {
    top: 0;
    position: sticky;
    border-right: 0;
    display: flex; // Flex here to safely expand navigation height
    flex-direction: column;
    width: 100vw;
    max-height: 100vh;
    background: var(--color-bg);
    z-index: 10;

    .logo-wrapper {
      align-items: center;
      display: flex;
      height: 3.5rem;
      width: 100vw;
      font-family: 'Montserrat', sans-serif;

      .logo {
        height: 2rem;
        width: auto;
        margin-left: 2.5rem;
      }

      .name {
        font-family: 'Montserrat Alternates', serif;
        margin-left: 0.4rem;
        font-size: 1.3rem;
      }
    }

    .hamburger-button {
      position: absolute;
      display: block;
      left: 10px;
      opacity: 0;
      margin: 0;
      top: 1.2rem;
      width: 30px;
      height: 30px;
      cursor: pointer;
    }

    .hamburger-icon {
      display: block;
      position: absolute;
      left: 15px;
      top: 1.2rem;
      pointer-events: none;
    }

    .hamburger-button:checked ~ nav {
      left: 0;
    }

    nav {
      display: flex;
      flex-direction: column;
      flex-grow: 1;
      justify-content: space-between;
      position: absolute;
      height: calc(100vh - 3.5rem);
      width: 100vw;
      left: -100vw;
      top: 3.5rem;
      transition: left 150ms;
      background: var(--color-bg);
      overflow-y: auto;
      z-index: 10;

      // Larger screens that still need a collapsed menu
      @media screen and (min-width: 900px) {
        width: 300px;
        left: -300px;
      }

      & > * {
        padding: 0 0.75rem;
      }

      .links {
        h3 {
          color: #718096;
          font-size: 0.8rem;
          letter-spacing: 0.02rem;
          margin-bottom: 0.5rem;
          margin-top: 1.5rem;
          text-transform: uppercase;
        }

        section {
          border-left: 4px solid var(--color-grey-3);

          a {
            align-items: center;
            border-radius: 0 0.25rem 0.25rem 0;
            color: var(--color-grey-5);
            display: flex;
            margin-bottom: 0.25rem;
            padding: 0.5rem 1rem;

            &:hover,
            &:focus,
            &.nuxt-link-active {
              background-color: var(--color-grey-1);
              color: var(--color-text);
            }

            &.nuxt-link-active {
              box-shadow: -4px 0 0 0 var(--color-brand);
            }

            svg {
              height: 1rem;
              width: 1rem;
              flex-shrink: 0;
            }

            span {
              margin-left: 0.5rem;
            }
          }
        }
      }
      .user-actions {
        align-items: center;
        border-top: 2px solid var(--color-grey-2);
        display: flex;
        justify-content: space-between;
        margin-top: 1rem;
        padding-bottom: 1rem;
        padding-top: 1rem;

        & > * {
          align-items: center;
          display: flex;
        }

        svg {
          color: var(--color-grey-5);

          &:hover,
          &:focus {
            color: inherit;
          }
        }

        .avatar {
          img {
            border-radius: 50%;
            height: 2rem;
            margin-right: 0.5rem;
            width: 2rem;
          }
        }

        .theme {
          cursor: pointer;
        }

        .log-in-button {
          text-align: center;
          padding: 8px 40px;
          border-radius: 5px;
          color: var(--color-grey-5);
          background-color: var(--color-grey-1);
          margin-left: 2.5rem;
        }

        .notifications {
          svg {
            cursor: pointer;
          }
        }

        .user-actions-popup {
          position: relative;

          .popup-inner {
            width: 120px;
            border: 2px var(--color-grey-2) solid;
            background-color: var(--color-bg);
            color: var(--color-grey-5);
            font-size: 15px;
            padding: 8px 0;
            position: absolute;
            z-index: 1;
            margin-bottom: 20px;
            bottom: 100%;
            margin-left: -50px;

            hr {
              color: var(--color-grey-2);
              height: 1px;
            }
            p {
              padding: 8px;
              margin: 0;
            }

            .hover {
              cursor: pointer;

              &:hover,
              &:focus {
                background-color: var(--color-brand);
              }
            }
          }
          .popup-inner::after {
            content: '';
            position: absolute;
            top: 100%;
            left: 45%;
            border-width: 7px;
            border-style: solid;
            border-color: var(--color-grey-2) transparent transparent
              transparent;
          }
        }
      }
    }

    // Desktop
    @media screen and (min-width: 1145px) {
      border-right: 1px solid var(--color-grey-2);
      min-width: 270px;
      max-width: 270px;

      nav {
        height: 100%;
        left: 0;
        width: 100%;
        transition: none;
        position: static;
      }

      .logo-wrapper {
        padding: 0 0 0 1.5rem;
        width: 100%;
        .logo {
          margin: 0;
        }
      }

      .hamburger-button,
      .hamburger-icon {
        display: none;
      }
    }
  }
  main {
    background-color: var(--color-grey-0);
    flex-grow: 1;

    header {
      align-items: center;
      background-color: var(--color-bg);
      box-shadow: 0 1px 1px 0 var(--color-grey-2);
      display: flex;
      height: 3.5rem;
      justify-content: space-between;
      padding: 0 3rem 0 1rem;

      .search-wrapper {
        align-items: center;
        display: flex;
        flex-direction: row-reverse;
        width: 100%;

        input {
          border: none;
          font-size: 1rem;
          padding: 1rem;
          width: 100%;

          &::placeholder {
            color: var(--color-grey-5);
          }

          &:hover,
          &:focus {
            & + svg {
              color: inherit;
            }

            &::placeholder {
              color: var(--color-grey-7);
            }
          }
        }

        svg {
          color: var(--color-grey-5);
        }
      }
    }

    .content {
      // Default is for small phone sizes (like iPhone 5/SE)
      padding: 0.5rem 0.35rem 0.5rem 0.35rem;

      // Larger phones
      @media screen and (min-width: 500px) {
        padding: 1rem 0.5rem 1rem 0.5rem;
      }

      // Desktop
      @media screen and (min-width: 1145px) {
        padding: 1rem;
      }
    }
  }
}

.disclosure {
  margin-top: auto;
  max-width: 250px;
  color: var(--color-grey-3);

  a {
    text-decoration: var(--color-grey-2) underline;
  }
}

// Hack for very small (iPhone 5/SE) sized phones
// an x overflow existed and I was unable to figure out why
@media screen and (max-width: 360px) {
  body {
    overflow-x: hidden !important;
  }
}
</style>
