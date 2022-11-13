<template>
  <div>
    <ModalConfirm
      ref="modal_confirm"
      title="Are you sure you want to delete this project?"
      description="If you proceed, all versions and any attached data will be removed from our servers. This may break other projects, so be careful."
      :has-to-type="true"
      :confirmation-text="project.title"
      proceed-label="Delete"
      @proceed="deleteProject"
    />
    <div class="universal-card">
      <h2>General settings</h2>
      <div class="adjacent-input">
        <label>
          <span class="label__title">Edit project information</span>
          <span class="label__description">
            Edit your project's name, description, categories, and more.
          </span>
        </label>
        <nuxt-link class="iconified-button" to="edit"
          ><EditIcon />Edit</nuxt-link
        >
      </div>
      <div class="adjacent-input">
        <span class="label">
          <span class="label__title">Delete project</span>
          <span class="label__description">
            Removes your project from Modrinth's servers and search. Clicking on
            this will delete your project, so be extra careful!
          </span>
        </span>
        <button
          class="iconified-button danger-button"
          :disabled="
            (currentMember.permissions & DELETE_PROJECT) !== DELETE_PROJECT
          "
          @click="$refs.modal_confirm.show()"
        >
          <TrashIcon />Delete project
        </button>
      </div>
    </div>
    <div class="universal-card">
      <h2>Manage members</h2>
      <div class="adjacent-input">
        <span class="label">
          <span class="label__title">Invite a member</span>
          <span class="label__description">
            Enter the Modrinth username of the person you'd like to invite to be
            a member of this project.
          </span>
        </span>
        <div
          v-if="(currentMember.permissions & MANAGE_INVITES) === MANAGE_INVITES"
          class="input-group"
        >
          <input
            id="username"
            v-model="currentUsername"
            type="text"
            placeholder="Username"
          />
          <label for="username" class="hidden">Username</label>
          <button
            class="iconified-button brand-button"
            @click="inviteTeamMember"
          >
            <PlusIcon />
            Invite
          </button>
        </div>
      </div>
    </div>
    <div
      v-for="(member, index) in allTeamMembers"
      :key="member.user.id"
      class="universal-card member"
      :class="{ open: openTeamMembers.includes(member.user.id) }"
    >
      <div class="member-header">
        <div class="info">
          <Avatar
            :src="member.avatar_url"
            :alt="member.username"
            size="sm"
            circle
          />
          <div class="text">
            <nuxt-link :to="'/user/' + member.user.username" class="name">
              <p>{{ member.name }}</p>
            </nuxt-link>
            <p>{{ member.role }}</p>
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
        <div v-if="member.oldRole !== 'Owner'" class="adjacent-input">
          <label :for="`member-${allTeamMembers[index].user.username}-role`">
            <span class="label__title">Role</span>
            <span class="label__description">
              The title of the role that this member plays for this project.
            </span>
          </label>
          <input
            :id="`member-${allTeamMembers[index].user.username}-role`"
            v-model="allTeamMembers[index].role"
            type="text"
            :class="{ 'known-error': member.role === 'Owner' }"
            :disabled="
              (currentMember.permissions & EDIT_MEMBER) !== EDIT_MEMBER
            "
          />
        </div>
        <div class="adjacent-input">
          <label
            :for="`member-${allTeamMembers[index].user.username}-monetization-weight`"
          >
            <span class="label__title">Monetization weight</span>
            <span class="label__description">
              Relative to all other members' monetization weights, this
              determines what portion of this project's revenue goes to this
              member.
            </span>
          </label>
          <input
            :id="`member-${allTeamMembers[index].user.username}-monetization-weight`"
            v-model="allTeamMembers[index].payouts_split"
            type="number"
            :disabled="
              (currentMember.permissions & EDIT_MEMBER) !== EDIT_MEMBER
            "
          />
        </div>
        <ul
          v-if="member.role === 'Owner' && member.oldRole !== 'Owner'"
          class="known-errors"
        >
          <li>A project can only have one 'Owner'.</li>
        </ul>
        <template v-if="member.oldRole !== 'Owner'">
          <span class="label">
            <span class="label__title">Permissions</span>
          </span>
          <div class="permissions">
            <Checkbox
              :value="(member.permissions & UPLOAD_VERSION) === UPLOAD_VERSION"
              :disabled="
                (currentMember.permissions & EDIT_MEMBER) !== EDIT_MEMBER ||
                (currentMember.permissions & UPLOAD_VERSION) !== UPLOAD_VERSION
              "
              label="Upload version"
              @input="allTeamMembers[index].permissions ^= UPLOAD_VERSION"
            />
            <Checkbox
              :value="(member.permissions & DELETE_VERSION) === DELETE_VERSION"
              :disabled="
                (currentMember.permissions & EDIT_MEMBER) !== EDIT_MEMBER ||
                (currentMember.permissions & DELETE_VERSION) !== DELETE_VERSION
              "
              label="Delete version"
              @input="allTeamMembers[index].permissions ^= DELETE_VERSION"
            />
            <Checkbox
              :value="(member.permissions & EDIT_DETAILS) === EDIT_DETAILS"
              :disabled="
                (currentMember.permissions & EDIT_MEMBER) !== EDIT_MEMBER ||
                (currentMember.permissions & EDIT_DETAILS) !== EDIT_DETAILS
              "
              label="Edit details"
              @input="allTeamMembers[index].permissions ^= EDIT_DETAILS"
            />
            <Checkbox
              :value="(member.permissions & EDIT_BODY) === EDIT_BODY"
              :disabled="
                (currentMember.permissions & EDIT_MEMBER) !== EDIT_MEMBER ||
                (currentMember.permissions & EDIT_BODY) !== EDIT_BODY
              "
              label="Edit body"
              @input="allTeamMembers[index].permissions ^= EDIT_BODY"
            />
            <Checkbox
              :value="(member.permissions & MANAGE_INVITES) === MANAGE_INVITES"
              :disabled="
                (currentMember.permissions & EDIT_MEMBER) !== EDIT_MEMBER ||
                (currentMember.permissions & MANAGE_INVITES) !== MANAGE_INVITES
              "
              label="Manage invites"
              @input="allTeamMembers[index].permissions ^= MANAGE_INVITES"
            />
            <Checkbox
              :value="(member.permissions & REMOVE_MEMBER) === REMOVE_MEMBER"
              :disabled="
                (currentMember.permissions & EDIT_MEMBER) !== EDIT_MEMBER ||
                (currentMember.permissions & REMOVE_MEMBER) !== REMOVE_MEMBER
              "
              label="Remove member"
              @input="allTeamMembers[index].permissions ^= REMOVE_MEMBER"
            />
            <Checkbox
              :value="(member.permissions & EDIT_MEMBER) === EDIT_MEMBER"
              :disabled="
                (currentMember.permissions & EDIT_MEMBER) !== EDIT_MEMBER
              "
              label="Edit member"
              @input="allTeamMembers[index].permissions ^= EDIT_MEMBER"
            />
            <Checkbox
              :value="(member.permissions & DELETE_PROJECT) === DELETE_PROJECT"
              :disabled="
                (currentMember.permissions & EDIT_MEMBER) !== EDIT_MEMBER ||
                (currentMember.permissions & DELETE_PROJECT) !== DELETE_PROJECT
              "
              label="Delete project"
              @input="allTeamMembers[index].permissions ^= DELETE_PROJECT"
            />
            <Checkbox
              :value="(member.permissions & VIEW_ANALYTICS) === VIEW_ANALYTICS"
              :disabled="
                (currentMember.permissions & EDIT_MEMBER) !== EDIT_MEMBER ||
                (currentMember.permissions & VIEW_ANALYTICS) !== VIEW_ANALYTICS
              "
              label="View analytics"
              @input="allTeamMembers[index].permissions ^= VIEW_ANALYTICS"
            />
            <Checkbox
              :value="(member.permissions & VIEW_PAYOUTS) === VIEW_PAYOUTS"
              :disabled="
                (currentMember.permissions & EDIT_MEMBER) !== EDIT_MEMBER ||
                (currentMember.permissions & VIEW_PAYOUTS) !== VIEW_PAYOUTS
              "
              label="View revenue"
              @input="allTeamMembers[index].permissions ^= VIEW_PAYOUTS"
            />
          </div>
        </template>
        <div class="button-group push-right">
          <button
            v-if="member.oldRole !== 'Owner'"
            class="iconified-button"
            :disabled="
              (currentMember.permissions & EDIT_MEMBER) !== EDIT_MEMBER
            "
            @click="removeTeamMember(index)"
          >
            <TrashIcon />
            Remove member
          </button>
          <button
            v-if="
              member.oldRole !== 'Owner' &&
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
            class="iconified-button brand-button"
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
import ModalConfirm from '~/components/ui/ModalConfirm'
import Checkbox from '~/components/ui/Checkbox'
import Badge from '~/components/ui/Badge'

import DropdownIcon from '~/assets/images/utils/dropdown.svg?inline'
import PlusIcon from '~/assets/images/utils/plus.svg?inline'
import CheckIcon from '~/assets/images/utils/check.svg?inline'
import EditIcon from '~/assets/images/utils/edit.svg?inline'
import TrashIcon from '~/assets/images/utils/trash.svg?inline'
import UserIcon from '~/assets/images/utils/user.svg?inline'
import Avatar from '~/components/ui/Avatar'

export default {
  components: {
    Avatar,
    DropdownIcon,
    ModalConfirm,
    Checkbox,
    Badge,
    PlusIcon,
    CheckIcon,
    EditIcon,
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

    this.allTeamMembers.forEach((x) => (x.oldRole = x.role))
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
    this.VIEW_ANALYTICS = 1 << 8
    this.VIEW_PAYOUTS = 1 << 9
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
          this.$defaultHeaders()
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
          this.$defaultHeaders()
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
        const data =
          this.allTeamMembers[index].oldRole !== 'Owner'
            ? {
                permissions: this.allTeamMembers[index].permissions,
                role: this.allTeamMembers[index].role,
                payouts_split: this.allTeamMembers[index].payouts_split,
              }
            : {
                payouts_split: this.allTeamMembers[index].payouts_split,
              }

        await this.$axios.patch(
          `team/${this.project.team}/members/${this.allTeamMembers[index].user.id}`,
          data,
          this.$defaultHeaders()
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
          this.$defaultHeaders()
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
    async deleteProject() {
      await this.$axios.delete(
        `project/${this.project.id}`,
        this.$defaultHeaders()
      )
      await this.$store.dispatch('user/fetchProjects')
      await this.$router.push(`/user/${this.$auth.user.username}`)
      this.$notify({
        group: 'main',
        title: 'Action Success',
        text: 'Your project has been successfully deleted.',
        type: 'success',
      })
    },
    async updateMembers() {
      this.allTeamMembers = (
        await this.$axios.get(
          `team/${this.project.team}/members`,
          this.$defaultHeaders()
        )
      ).data.map((it) => ({
        avatar_url: it.user.avatar_url,
        name: it.user.username,
        oldRole: it.role,
        ...it,
      }))
    },
  },
}
</script>

<style lang="scss" scoped>
.member {
  .member-header {
    display: flex;
    justify-content: space-between;
    .info {
      display: flex;
      .text {
        margin: auto 0 auto 0.5rem;
        font-size: var(--font-size-sm);
        .name {
          font-weight: bold;
        }
        p {
          margin: 0.2rem 0;
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
    flex-direction: column;
    padding-top: var(--spacing-card-md);

    .main-info {
      margin-bottom: var(--spacing-card-lg);
    }
    .permissions {
      margin-bottom: var(--spacing-card-md);
      max-width: 45rem;
      display: grid;
      grid-template-columns: repeat(auto-fill, minmax(10rem, 1fr));
      grid-gap: 0.5rem;
    }
  }

  &.open {
    .member-header {
      .dropdown-icon {
        transform: rotate(180deg);
      }
    }
    .content {
      display: flex;
    }
  }
}
</style>
