<template>
  <div>
    <ConfirmPopup
      ref="delete_popup"
      title="Are you sure you want to delete this mod?"
      description="If you proceed, all versions and any attached data will be removed from our servers. This may break other projects, so be careful."
      :has-to-type="true"
      :confirmation-text="mod.title"
      proceed-label="Delete Mod"
      @proceed="deleteMod"
    />
    <div class="section-header columns">
      <h3 class="column-grow-1">General</h3>
    </div>
    <section>
      <h3>Edit Mod</h3>
      <label>
        <span> This leads you to a page where you can edit your mod. </span>
        <nuxt-link class="button" to="edit">Edit</nuxt-link>
      </label>
      <h3>Create Version</h3>
      <label>
        <span>
          This leads to a page where you can create a version for your mod.
        </span>
        <nuxt-link class="button" to="newversion">Create Version</nuxt-link>
      </label>
      <h3>Delete Mod</h3>
      <label>
        <span>
          Clicking on this WILL delete your mod. Do not click on this unless you
          want your mod deleted. If you delete your mod, all versions and any
          attatched data will be removed from our servers. This may break other
          projects, so be careful!
        </span>
        <div
          class="button"
          :disabled="(currentMember.permissions & DELETE_MOD) !== DELETE_MOD"
          @click="showPopup"
        >
          Delete Mod
        </div>
      </label>
    </section>
    <div class="section-header columns">
      <h3 class="column-grow-1">Team members</h3>
      <div class="column">
        <input
          id="username"
          v-model="currentUsername"
          type="text"
          placeholder="Username"
        />
        <label for="username" class="hidden">Username</label>
        <button class="brand-button column" @click="inviteTeamMember">
          Invite
        </button>
      </div>
    </div>
    <div
      v-for="(member, index) in members"
      :key="member.user_id"
      class="member"
      :class="{ open: openTeamMembers.includes(member.user_id) }"
    >
      <div class="member-header">
        <div class="info">
          <img :src="member.avatar_url" :alt="member.name" />
          <div class="text">
            <h4>{{ member.name }}</h4>
            <h3>{{ member.role }}</h3>
          </div>
        </div>
        <div class="side-buttons">
          <span v-if="member.accepted" class="badge green">Accepted</span>
          <span v-else class="badge yellow">Pending</span>
          <button
            class="dropdown-icon"
            @click="
              openTeamMembers.indexOf(member.user_id) === -1
                ? openTeamMembers.push(member.user_id)
                : (openTeamMembers = openTeamMembers.filter(
                    (it) => it !== member.user_id
                  ))
            "
          >
            <DropdownIcon />
          </button>
        </div>
      </div>
      <div class="content">
        <div class="main-info">
          <label>
            Role:
            <input
              v-model="members[index].role"
              type="text"
              :disabled="
                member.role === 'Owner' ||
                (currentMember.permissions & EDIT_MEMBER) !== EDIT_MEMBER
              "
            />
          </label>
        </div>
        <h3>Permissions</h3>
        <div class="permissions">
          <label>
            <input
              type="checkbox"
              :checked="
                (member.permissions & UPLOAD_VERSION) === UPLOAD_VERSION
              "
              :disabled="
                member.role === 'Owner' ||
                (currentMember.permissions & EDIT_MEMBER) !== EDIT_MEMBER ||
                (currentMember.permissions & UPLOAD_VERSION) !== UPLOAD_VERSION
              "
              @change="members[index].permissions ^= UPLOAD_VERSION"
            />
            Upload Version
          </label>
          <label>
            <input
              type="checkbox"
              :checked="
                (member.permissions & DELETE_VERSION) === DELETE_VERSION
              "
              :disabled="
                member.role === 'Owner' ||
                (currentMember.permissions & EDIT_MEMBER) !== EDIT_MEMBER ||
                (currentMember.permissions & DELETE_VERSION) !== DELETE_VERSION
              "
              @change="members[index].permissions ^= DELETE_VERSION"
            />
            Delete Version
          </label>
          <label>
            <input
              type="checkbox"
              :checked="(member.permissions & EDIT_DETAILS) === EDIT_DETAILS"
              :disabled="
                member.role === 'Owner' ||
                (currentMember.permissions & EDIT_MEMBER) !== EDIT_MEMBER ||
                (currentMember.permissions & EDIT_DETAILS) !== EDIT_DETAILS
              "
              @change="members[index].permissions ^= EDIT_DETAILS"
            />
            Edit Details
          </label>
          <label>
            <input
              type="checkbox"
              :checked="(member.permissions & EDIT_BODY) === EDIT_BODY"
              :disabled="
                member.role === 'Owner' ||
                (currentMember.permissions & EDIT_MEMBER) !== EDIT_MEMBER ||
                (currentMember.permissions & EDIT_BODY) !== EDIT_BODY
              "
              @change="members[index].permissions ^= EDIT_BODY"
            />
            Edit Body
          </label>
          <label>
            <input
              type="checkbox"
              :checked="
                (member.permissions & MANAGE_INVITES) === MANAGE_INVITES
              "
              :disabled="
                member.role === 'Owner' ||
                (currentMember.permissions & EDIT_MEMBER) !== EDIT_MEMBER ||
                (currentMember.permissions & MANAGE_INVITES) !== MANAGE_INVITES
              "
              @change="members[index].permissions ^= MANAGE_INVITES"
            />
            Manage Invites
          </label>
          <label>
            <input
              type="checkbox"
              :checked="(member.permissions & REMOVE_MEMBER) === REMOVE_MEMBER"
              :disabled="
                member.role === 'Owner' ||
                (currentMember.permissions & EDIT_MEMBER) !== EDIT_MEMBER ||
                (currentMember.permissions & REMOVE_MEMBER) !== REMOVE_MEMBER
              "
              @change="members[index].permissions ^= REMOVE_MEMBER"
            />
            Remove Member
          </label>
          <label>
            <input
              type="checkbox"
              :checked="(member.permissions & EDIT_MEMBER) === EDIT_MEMBER"
              :disabled="
                member.role === 'Owner' ||
                (currentMember.permissions & EDIT_MEMBER) !== EDIT_MEMBER
              "
              @change="members[index].permissions ^= EDIT_MEMBER"
            />
            Edit Member
          </label>
          <label>
            <input
              type="checkbox"
              :checked="(member.permissions & DELETE_MOD) === DELETE_MOD"
              :disabled="
                member.role === 'Owner' ||
                (currentMember.permissions & EDIT_MEMBER) !== EDIT_MEMBER ||
                (currentMember.permissions & DELETE_MOD) !== DELETE_MOD
              "
              @change="members[index].permissions ^= DELETE_MOD"
            />
            Delete Mod
          </label>
        </div>
        <div class="actions">
          <button
            :disabled="
              member.role === 'Owner' ||
              (currentMember.permissions & EDIT_MEMBER) !== EDIT_MEMBER
            "
            @click="removeTeamMember(index)"
          >
            Remove Member
          </button>
          <button
            :disabled="
              member.role === 'Owner' ||
              (currentMember.permissions & EDIT_MEMBER) !== EDIT_MEMBER
            "
            @click="updateTeamMember(index)"
          >
            Save Changes
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script>
import axios from 'axios'

import ConfirmPopup from '~/components/ui/ConfirmPopup'

import DropdownIcon from '~/assets/images/utils/dropdown.svg?inline'

export default {
  components: { DropdownIcon, ConfirmPopup },
  props: {
    mod: {
      type: Object,
      default() {
        return {}
      },
    },
    members: {
      type: Array,
      default() {
        return []
      },
    },
    currentMember: {
      type: Object,
      default() {
        return null
      },
    },
  },
  data() {
    return {
      currentUsername: '',
      openTeamMembers: [],
    }
  },
  created() {
    this.$emit('update:link-bar', [['Settings', 'settings']])

    this.UPLOAD_VERSION = 1 << 0
    this.DELETE_VERSION = 1 << 1
    this.EDIT_DETAILS = 1 << 2
    this.EDIT_BODY = 1 << 3
    this.MANAGE_INVITES = 1 << 4
    this.REMOVE_MEMBER = 1 << 5
    this.EDIT_MEMBER = 1 << 6
    this.DELETE_MOD = 1 << 7
  },
  methods: {
    async inviteTeamMember() {
      this.$nuxt.$loading.start()

      try {
        const user = (
          await axios.get(
            `https://api.modrinth.com/api/v1/user/${this.currentUsername}`
          )
        ).data

        const data = {
          user_id: user.id,
        }

        await axios.post(
          `https://api.modrinth.com/api/v1/team/${this.mod.team}/members`,
          data,
          this.$auth.headers
        )
        await this.$router.go(null)
      } catch (err) {
        this.$notify({
          group: 'main',
          title: 'An Error Occurred',
          text: err.response.data.description,
          type: 'error',
        })
      }

      this.$nuxt.$loading.finish()
    },
    async removeTeamMember(index) {
      this.$nuxt.$loading.start()

      try {
        await axios.delete(
          `https://api.modrinth.com/api/v1/team/${this.mod.team}/members/${this.members[index].user_id}`,
          this.$auth.headers
        )
        await this.$router.go(null)
      } catch (err) {
        this.$notify({
          group: 'main',
          title: 'An Error Occurred',
          text: err.response.data.description,
          type: 'error',
        })
      }

      this.$nuxt.$loading.finish()
    },
    async updateTeamMember(index) {
      this.$nuxt.$loading.start()

      try {
        const data = {
          permissions: this.members[index].permissions,
          role: this.members[index].role,
        }

        await axios.patch(
          `https://api.modrinth.com/api/v1/team/${this.mod.team}/members/${this.members[index].user_id}`,
          data,
          this.$auth.headers
        )
        await this.$router.go(null)
      } catch (err) {
        this.$notify({
          group: 'main',
          title: 'An Error Occurred',
          text: err.response.data.description,
          type: 'error',
        })
      }

      this.$nuxt.$loading.finish()
    },
    showPopup() {
      this.$refs.delete_popup.show()
    },
    async deleteMod() {
      await axios.delete(
        `https://api.modrinth.com/api/v1/mod/${this.mod.id}`,
        this.$auth.headers
      )
      await this.$router.push('/dashboard/projects')
      this.$notify({
        group: 'main',
        title: 'Action Success',
        text: 'Your mod has been successfully deleted.',
        type: 'success',
      })
    },
  },
}
</script>

<style lang="scss" scoped>
.section-header {
  @extend %card;
  padding: var(--spacing-card-md) var(--spacing-card-lg);
  margin-bottom: var(--spacing-card-md);
  h3 {
    margin: auto 0;
    color: var(--color-text-dark);
    font-weight: var(--font-weight-extrabold);
  }
}

.member {
  @extend %card;
  padding: var(--spacing-card-md) var(--spacing-card-lg);
  margin-bottom: var(--spacing-card-md);

  .member-header {
    display: flex;
    justify-content: space-between;
    .info {
      display: flex;
      img {
        border-radius: var(--size-rounded-icon);
        height: 50px;
        width: 50px;
      }
      .text {
        margin: auto 0 auto 0.5rem;
        h4 {
          font-weight: normal;
          margin: 0;
        }
        h3 {
          text-transform: uppercase;
          margin-top: 0.1rem;
          margin-bottom: 0;
          font-size: var(--font-size-sm);
          font-weight: var(--font-weight-extrabold);
          letter-spacing: 0.02rem;
        }
      }
    }
    .side-buttons {
      display: flex;
      align-items: center;
      .dropdown-icon {
        margin-left: 1rem;
        cursor: pointer;
        color: var(--color-text-dark);
        background-color: unset;
        transition: 150ms ease transform;
        padding: unset;
      }
    }
  }

  .content {
    display: none;

    .main-info {
      margin-bottom: var(--spacing-card-lg);
    }
    .permissions {
      margin: 1rem 0;
      display: grid;
      grid-template-columns: 10rem 10rem 10rem;
      grid-template-rows: 1.5rem 1.5rem 1.5rem;
    }
  }

  &.open {
    .member-header {
      .dropdown-icon {
        transform: rotate(180deg);
      }
    }
    .content {
      display: unset;
      margin: var(--spacing-card-lg);
    }
  }
}

input,
button {
  &:disabled {
    cursor: not-allowed !important;
  }
}

section {
  @extend %card;
  padding: var(--spacing-card-md) var(--spacing-card-lg);
  margin-bottom: var(--spacing-card-md);

  label {
    display: flex;

    span {
      flex: 2;
      padding-right: var(--spacing-card-lg);
    }

    input {
      flex: 3;
      height: fit-content;
    }

    div,
    a {
      text-align: center;
      height: fit-content;
      flex: 1;
    }
    div:hover {
      cursor: pointer;
    }
  }
}
</style>
