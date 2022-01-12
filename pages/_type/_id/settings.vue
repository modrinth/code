<template>
  <div>
    <ConfirmPopup
      ref="delete_popup"
      title="Are you sure you want to delete this project?"
      description="If you proceed, all versions and any attached data will be removed from our servers. This may break other projects, so be careful."
      :has-to-type="true"
      :confirmation-text="project.title"
      proceed-label="Delete project"
      @proceed="deleteProject"
    />
    <div class="card">
      <h3>General</h3>
    </div>
    <section class="card">
      <h3>Edit project</h3>
      <label>
        <span> This leads you to a page where you can edit your project. </span>
        <nuxt-link class="iconified-button" to="edit">Edit</nuxt-link>
      </label>
      <h3>Create version</h3>
      <label>
        <span>
          This leads to a page where you can create a version for your project.
        </span>
        <nuxt-link
          class="iconified-button"
          to="version/create"
          :disabled="
            (currentMember.permissions & UPLOAD_VERSION) !== UPLOAD_VERSION
          "
          >Create version</nuxt-link
        >
      </label>
      <h3>Delete project</h3>
      <label>
        <span>
          Removes your project from Modrinth's servers and search. Clicking on
          this will delete your project, so be extra careful!
        </span>
        <div
          class="iconified-button"
          :disabled="
            (currentMember.permissions & DELETE_PROJECT) !== DELETE_PROJECT
          "
          @click="showPopup"
        >
          Delete project
        </div>
      </label>
    </section>
    <div class="card columns team-invite">
      <h3>Team members</h3>
      <div
        v-if="(currentMember.permissions & MANAGE_INVITES) === MANAGE_INVITES"
        class="column"
      >
        <input
          id="username"
          v-model="currentUsername"
          type="text"
          placeholder="Username"
        />
        <label for="username" class="hidden">Username</label>
        <button
          class="iconified-button brand-button-colors column"
          @click="inviteTeamMember"
        >
          <PlusIcon />
          Invite
        </button>
      </div>
    </div>
    <div
      v-for="(member, index) in allTeamMembers"
      :key="member.user.id"
      class="card member"
      :class="{ open: openTeamMembers.includes(member.user.id) }"
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
          <Badge v-if="member.accepted" type="accepted" color="green" />
          <Badge v-else type="pending" color="yellow" />
          <button
            class="dropdown-icon"
            @click="
              openTeamMembers.indexOf(member.user.id) === -1
                ? openTeamMembers.push(member.user.id)
                : (openTeamMembers = openTeamMembers.filter(
                    (it) => it !== member.user.id
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
              v-model="allTeamMembers[index].role"
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
          <Checkbox
            :value="
              (member.permissions & UPLOAD_VERSION) === UPLOAD_VERSION ||
              member.role === 'Owner'
            "
            :disabled="
              member.role === 'Owner' ||
              (currentMember.permissions & EDIT_MEMBER) !== EDIT_MEMBER ||
              (currentMember.permissions & UPLOAD_VERSION) !== UPLOAD_VERSION
            "
            label="Upload version"
            @input="allTeamMembers[index].permissions ^= UPLOAD_VERSION"
          />
          <Checkbox
            :value="
              (member.permissions & DELETE_VERSION) === DELETE_VERSION ||
              member.role === 'Owner'
            "
            :disabled="
              member.role === 'Owner' ||
              (currentMember.permissions & EDIT_MEMBER) !== EDIT_MEMBER ||
              (currentMember.permissions & DELETE_VERSION) !== DELETE_VERSION
            "
            label="Delete version"
            @input="allTeamMembers[index].permissions ^= DELETE_VERSION"
          />
          <Checkbox
            :value="
              (member.permissions & EDIT_DETAILS) === EDIT_DETAILS ||
              member.role === 'Owner'
            "
            :disabled="
              member.role === 'Owner' ||
              (currentMember.permissions & EDIT_MEMBER) !== EDIT_MEMBER ||
              (currentMember.permissions & EDIT_DETAILS) !== EDIT_DETAILS
            "
            label="Edit details"
            @input="allTeamMembers[index].permissions ^= EDIT_DETAILS"
          />
          <Checkbox
            :value="
              (member.permissions & EDIT_BODY) === EDIT_BODY ||
              member.role === 'Owner'
            "
            :disabled="
              member.role === 'Owner' ||
              (currentMember.permissions & EDIT_MEMBER) !== EDIT_MEMBER ||
              (currentMember.permissions & EDIT_BODY) !== EDIT_BODY
            "
            label="Edit body"
            @input="allTeamMembers[index].permissions ^= EDIT_BODY"
          />
          <Checkbox
            :value="
              (member.permissions & MANAGE_INVITES) === MANAGE_INVITES ||
              member.role === 'Owner'
            "
            :disabled="
              member.role === 'Owner' ||
              (currentMember.permissions & EDIT_MEMBER) !== EDIT_MEMBER ||
              (currentMember.permissions & MANAGE_INVITES) !== MANAGE_INVITES
            "
            label="Manage invites"
            @input="allTeamMembers[index].permissions ^= MANAGE_INVITES"
          />
          <Checkbox
            :value="
              (member.permissions & REMOVE_MEMBER) === REMOVE_MEMBER ||
              member.role === 'Owner'
            "
            :disabled="
              member.role === 'Owner' ||
              (currentMember.permissions & EDIT_MEMBER) !== EDIT_MEMBER ||
              (currentMember.permissions & REMOVE_MEMBER) !== REMOVE_MEMBER
            "
            label="Remove member"
            @input="allTeamMembers[index].permissions ^= REMOVE_MEMBER"
          />
          <Checkbox
            :value="
              (member.permissions & EDIT_MEMBER) === EDIT_MEMBER ||
              member.role === 'Owner'
            "
            :disabled="
              member.role === 'Owner' ||
              (currentMember.permissions & EDIT_MEMBER) !== EDIT_MEMBER
            "
            label="Edit member"
            @input="allTeamMembers[index].permissions ^= EDIT_MEMBER"
          />
          <Checkbox
            :value="
              (member.permissions & DELETE_PROJECT) === DELETE_PROJECT ||
              member.role === 'Owner'
            "
            :disabled="
              member.role === 'Owner' ||
              (currentMember.permissions & EDIT_MEMBER) !== EDIT_MEMBER ||
              (currentMember.permissions & DELETE_PROJECT) !== DELETE_PROJECT
            "
            label="Delete project"
            @input="allTeamMembers[index].permissions ^= DELETE_PROJECT"
          />
        </div>
        <div class="actions">
          <button
            class="iconified-button"
            :disabled="
              member.role === 'Owner' ||
              (currentMember.permissions & EDIT_MEMBER) !== EDIT_MEMBER
            "
            @click="removeTeamMember(index)"
          >
            <TrashIcon />
            Remove member
          </button>
          <button
            v-if="
              member.role !== 'Owner' &&
              currentMember.role === 'Owner' &&
              member.accepted
            "
            class="iconified-button"
            @click="transferOwnership(index)"
          >
            <UserIcon />
            Transfer ownership
          </button>
          <button
            class="iconified-button brand-button-colors"
            :disabled="
              (currentMember.permissions & EDIT_MEMBER) !== EDIT_MEMBER
            "
            @click="updateTeamMember(index)"
          >
            <CheckIcon />
            Save changes
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script>
import ConfirmPopup from '~/components/ui/ConfirmPopup'
import Checkbox from '~/components/ui/Checkbox'
import Badge from '~/components/ui/Badge'

import DropdownIcon from '~/assets/images/utils/dropdown.svg?inline'
import PlusIcon from '~/assets/images/utils/plus.svg?inline'
import CheckIcon from '~/assets/images/utils/check.svg?inline'
import TrashIcon from '~/assets/images/utils/trash.svg?inline'
import UserIcon from '~/assets/images/utils/user.svg?inline'

export default {
  components: {
    DropdownIcon,
    ConfirmPopup,
    Checkbox,
    Badge,
    PlusIcon,
    CheckIcon,
    TrashIcon,
    UserIcon,
  },
  props: {
    project: {
      type: Object,
      default() {
        return {}
      },
    },
    allMembers: {
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
      allTeamMembers: [],
    }
  },
  fetch() {
    this.allTeamMembers = this.allMembers
  },
  created() {
    this.UPLOAD_VERSION = 1 << 0
    this.DELETE_VERSION = 1 << 1
    this.EDIT_DETAILS = 1 << 2
    this.EDIT_BODY = 1 << 3
    this.MANAGE_INVITES = 1 << 4
    this.REMOVE_MEMBER = 1 << 5
    this.EDIT_MEMBER = 1 << 6
    this.DELETE_PROJECT = 1 << 7
  },
  methods: {
    async inviteTeamMember() {
      this.$nuxt.$loading.start()

      try {
        const user = (await this.$axios.get(`user/${this.currentUsername}`))
          .data

        const data = {
          user_id: user.id,
        }

        await this.$axios.post(
          `team/${this.project.team}/members`,
          data,
          this.$auth.headers
        )
        await this.updateMembers()
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
    async removeTeamMember(index) {
      this.$nuxt.$loading.start()

      try {
        await this.$axios.delete(
          `team/${this.project.team}/members/${this.allTeamMembers[index].user.id}`,
          this.$auth.headers
        )
        await this.updateMembers()
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
    async updateTeamMember(index) {
      this.$nuxt.$loading.start()

      try {
        const data = {
          permissions: this.allTeamMembers[index].permissions,
          role: this.allTeamMembers[index].role,
        }

        await this.$axios.patch(
          `team/${this.project.team}/members/${this.allTeamMembers[index].user.id}`,
          data,
          this.$auth.headers
        )
        await this.updateMembers()
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
    async transferOwnership(index) {
      this.$nuxt.$loading.start()

      try {
        await this.$axios.patch(
          `team/${this.project.team}/owner`,
          {
            user_id: this.allTeamMembers[index].user.id,
          },
          this.$auth.headers
        )
        await this.updateMembers()
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
    showPopup() {
      if (
        (this.currentMember.permissions & this.DELETE_PROJECT) ===
        this.DELETE_PROJECT
      ) {
        this.$refs.delete_popup.show()
      }
    },
    async deleteProject() {
      await this.$axios.delete(`project/${this.project.id}`, this.$auth.headers)
      await this.$store.dispatch('user/fetchProjects')
      await this.$router.push(`/user/${this.$auth.user.username}`)
      this.$notify({
        group: 'main',
        title: 'Action Success',
        text: 'Your _type has been successfully deleted.',
        type: 'success',
      })
    },
    async updateMembers() {
      this.allTeamMembers = (
        await this.$axios.get(
          `team/${this.project.team}/members`,
          this.$auth.headers
        )
      ).data.map((it) => ({
        avatar_url: it.user.avatar_url,
        name: it.user.username,
        ...it,
      }))
    },
  },
}
</script>

<style lang="scss" scoped>
.member {
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

      @media screen and (min-width: 1024px) {
        label {
          align-items: center;
          input {
            margin-left: 1rem;
          }
        }
      }
    }
    .permissions {
      margin: 1rem 0;
      max-width: 45rem;
      display: grid;
      grid-template-columns: repeat(auto-fill, minmax(10rem, 1fr));
      grid-gap: 0.5rem;

      label {
        flex-direction: row;
        input {
          flex: none;
          margin-right: 0.5rem;
        }
      }
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
      display: block;
      text-align: center;
      height: fit-content;
      flex: 1;
      @media screen and (max-width: 1024px) {
        margin: 0.5rem 0 1rem 0;
      }
    }
    div:hover {
      cursor: pointer;
    }
  }
}

.team-invite {
  @media screen and (max-width: 1024px) {
    flex-direction: column;
    h3 {
      margin-bottom: 0.5rem;
    }
  }

  h3 {
    margin-right: auto;
  }

  > div {
    display: flex;
    align-items: center;

    input {
      margin-right: 1rem;
    }

    @media screen and (max-width: 500px) {
      display: flex;
      flex-direction: column;

      input {
        margin: 0;
      }

      button {
        margin-top: 0.5rem;
      }
    }
  }
}

.actions {
  display: flex;

  button {
    margin-right: 0.5rem;

    &:first-child {
      margin-left: auto;
    }
  }
}
</style>
