<template>
  <div class="normal-page__content">
    <div class="universal-card">
      <div class="label">
        <h3>
          <span class="label__title size-card-header">Manage members</span>
        </h3>
      </div>
      <span class="label">
        <span class="label__title">Invite a member</span>
        <span class="label__description">
          Enter the Modrinth username of the person you'd like to invite to be a member of this
          organization.
        </span>
      </span>
      <div class="input-group">
        <input
          id="username"
          v-model="currentUsername"
          type="text"
          placeholder="Username"
          :disabled="
            !isPermission(
              currentMember.organization_permissions,
              organizationPermissions.MANAGE_INVITES
            )
          "
          @keypress.enter="() => onInviteTeamMember(organization.team, currentUsername)"
        />
        <label for="username" class="hidden">Username</label>
        <Button
          color="primary"
          :disabled="
            !isPermission(
              currentMember.organization_permissions,
              organizationPermissions.MANAGE_INVITES
            )
          "
          @click="() => onInviteTeamMember(organization.team_id, currentUsername)"
        >
          <UserPlusIcon />
          Invite
        </Button>
      </div>
      <div class="adjacent-input">
        <span class="label">
          <span class="label__title">Leave organization</span>
          <span class="label__description">
            Remove yourself as a member of this organization.
          </span>
        </span>
        <Button
          color="danger"
          :disabled="currentMember.is_owner"
          @click="() => onLeaveProject(organization.team_id, auth.user.id)"
        >
          <UserRemoveIcon />
          Leave organization
        </Button>
      </div>
    </div>
    <div
      v-for="(member, index) in allTeamMembers"
      :key="member.user.id"
      class="member universal-card"
      :class="{ open: openTeamMembers.includes(member.user.id) }"
    >
      <div class="member-header">
        <div class="info">
          <Avatar :src="member.user.avatar_url" :alt="member.user.username" size="sm" circle />
          <div class="text">
            <nuxt-link :to="'/user/' + member.user.username" class="name">
              <p>{{ member.user.username }}</p>
              <CrownIcon v-if="member.is_owner" v-tooltip="'Organization owner'" />
            </nuxt-link>
            <p>{{ member.role }}</p>
          </div>
        </div>
        <div class="side-buttons">
          <Badge v-if="member.accepted" type="accepted" />
          <Badge v-else type="pending" />
          <Button
            icon-only
            transparent
            class="dropdown-icon"
            @click="
              openTeamMembers.indexOf(member.user.id) === -1
                ? openTeamMembers.push(member.user.id)
                : (openTeamMembers = openTeamMembers.filter((it) => it !== member.user.id))
            "
          >
            <DropdownIcon />
          </Button>
        </div>
      </div>
      <div class="content">
        <div class="adjacent-input">
          <label :for="`member-${member.user.id}-role`">
            <span class="label__title">Role</span>
            <span class="label__description">
              The title of the role that this member plays for this organization.
            </span>
          </label>
          <input
            :id="`member-${member.user.id}-role`"
            v-model="member.role"
            type="text"
            :disabled="
              !isPermission(
                currentMember.organization_permissions,
                organizationPermissions.EDIT_MEMBER
              )
            "
          />
        </div>
        <div class="adjacent-input">
          <label :for="`member-${member.user.id}-monetization-weight`">
            <span class="label__title">Monetization weight</span>
            <span class="label__description">
              Relative to all other members' monetization weights, this determines what portion of
              the organization projects' revenue goes to this member.
            </span>
          </label>
          <input
            :id="`member-${member.user.id}-monetization-weight`"
            v-model="member.payouts_split"
            type="number"
            :disabled="
              !isPermission(
                currentMember.organization_permissions,
                organizationPermissions.EDIT_MEMBER
              )
            "
          />
        </div>
        <template v-if="!member.is_owner">
          <span class="label">
            <span class="label__title">Project permissions</span>
          </span>
          <div class="permissions">
            <Checkbox
              v-for="[label, permission] in Object.entries(projectPermissions)"
              :key="permission"
              :model-value="isPermission(member.permissions, permission)"
              :disabled="
                !isPermission(
                  currentMember.organization_permissions,
                  organizationPermissions.EDIT_MEMBER_DEFAULT_PERMISSIONS
                ) || !isPermission(currentMember.permissions, permission)
              "
              :label="permToLabel(label)"
              @update:model-value="allTeamMembers[index].permissions ^= permission"
            />
          </div>
        </template>
        <template v-if="!member.is_owner">
          <span class="label">
            <span class="label__title">Organization permissions</span>
          </span>
          <div class="permissions">
            <Checkbox
              v-for="[label, permission] in Object.entries(organizationPermissions)"
              :key="permission"
              :model-value="isPermission(member.organization_permissions, permission)"
              :disabled="
                !isPermission(
                  currentMember.organization_permissions,
                  organizationPermissions.EDIT_MEMBER
                ) || !isPermission(currentMember.organization_permissions, permission)
              "
              :label="permToLabel(label)"
              @update:model-value="allTeamMembers[index].organization_permissions ^= permission"
            />
          </div>
        </template>
        <div class="input-group">
          <Button
            color="primary"
            :disabled="
              !isPermission(
                currentMember.organization_permissions,
                organizationPermissions.EDIT_MEMBER
              )
            "
            @click="onUpdateTeamMember(organization.team_id, member)"
          >
            <SaveIcon />
            Save changes
          </Button>
          <Button
            v-if="!member.is_owner"
            color="danger"
            :disabled="
              !isPermission(
                currentMember.organization_permissions,
                organizationPermissions.EDIT_MEMBER
              ) &&
              !isPermission(
                currentMember.organization_permissions,
                organizationPermissions.REMOVE_MEMBER
              )
            "
            @click="onRemoveMember(organization.team_id, member)"
          >
            <UserRemoveIcon />
            Remove member
          </Button>
          <Button
            v-if="!member.is_owner && currentMember.is_owner && member.accepted"
            @click="() => onTransferOwnership(organization.team_id, member.user.id)"
          >
            <TransferIcon />
            Transfer ownership
          </Button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import {
  Avatar,
  Checkbox,
  SaveIcon,
  Badge,
  TransferIcon,
  UserPlusIcon,
  UserXIcon as UserRemoveIcon,
  DropdownIcon,
  Button,
} from 'omorphia'
import { ref } from 'vue'
import CrownIcon from '~/assets/images/utils/crown.svg?component'

import { removeTeamMember } from '~/helpers/teams.js'
import { isPermission } from '~/utils/permissions.ts'

const { organization, refresh: refreshOrganization, currentMember } = inject('organizationContext')

const auth = await useAuth()

const currentUsername = ref('')
const openTeamMembers = ref([])

const allTeamMembers = ref(organization.value.members)

watch(
  () => organization.value,
  () => {
    allTeamMembers.value = organization.value.members
  }
)

const projectPermissions = {
  UPLOAD_VERSION: 1 << 0,
  DELETE_VERSION: 1 << 1,
  EDIT_DETAILS: 1 << 2,
  EDIT_BODY: 1 << 3,
  MANAGE_INVITES: 1 << 4,
  REMOVE_MEMBER: 1 << 5,
  EDIT_MEMBER: 1 << 6,
  DELETE_PROJECT: 1 << 7,
  VIEW_ANALYTICS: 1 << 8,
  VIEW_PAYOUTS: 1 << 9,
}

const organizationPermissions = {
  EDIT_DETAILS: 1 << 0,
  MANAGE_INVITES: 1 << 1,
  REMOVE_MEMBER: 1 << 2,
  EDIT_MEMBER: 1 << 3,
  ADD_PROJECT: 1 << 4,
  REMOVE_PROJECT: 1 << 5,
  DELETE_ORGANIZATION: 1 << 6,
  EDIT_MEMBER_DEFAULT_PERMISSIONS: 1 << 7,
}

const permToLabel = (key) => {
  const o = key.split('_').join(' ')
  return o.charAt(0).toUpperCase() + o.slice(1).toLowerCase()
}

const leaveProject = async (teamId, uid) => {
  await removeTeamMember(teamId, uid)
  await navigateTo(`/organization/${organization.value.id}`)
}

const onLeaveProject = useClientTry(leaveProject)

const onInviteTeamMember = useClientTry(async (teamId, username) => {
  const user = await useBaseFetch(`user/${username}`)
  const data = {
    user_id: user.id.trim(),
  }
  await useBaseFetch(`team/${teamId}/members`, {
    method: 'POST',
    body: data,
  })
  await refreshOrganization()
  currentUsername.value = ''
  addNotification({
    group: 'main',
    title: 'Member invited',
    text: `${user.username} has been invited to the organization.`,
    type: 'success',
  })
})

const onRemoveMember = useClientTry(async (teamId, member) => {
  await removeTeamMember(teamId, member.user.id)
  await refreshOrganization()
  addNotification({
    group: 'main',
    title: 'Member removed',
    text: `${member.user.username} has been removed from the organization.`,
    type: 'success',
  })
})

const onUpdateTeamMember = useClientTry(async (teamId, member) => {
  const data = !member.is_owner
    ? {
        permissions: member.permissions,
        organization_permissions: member.organization_permissions,
        role: member.role,
        payouts_split: member.payouts_split,
      }
    : {
        payouts_split: member.payouts_split,
        role: member.role,
      }
  await useBaseFetch(`team/${teamId}/members/${member.user.id}`, {
    method: 'PATCH',
    body: data,
  })
  await refreshOrganization()
  addNotification({
    group: 'main',
    title: 'Member updated',
    text: `${member.user.username} has been updated.`,
    type: 'success',
  })
})

const onTransferOwnership = useClientTry(async (teamId, uid) => {
  const data = {
    user_id: uid,
  }
  await useBaseFetch(`team/${teamId}/owner`, {
    method: 'PATCH',
    body: data,
  })
  await refreshOrganization()
  addNotification({
    group: 'main',
    title: 'Ownership transferred',
    text: `The ownership of ${organization.value.name} has been successfully transferred.`,
    type: 'success',
  })
})
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

          display: flex;
          align-items: center;
          gap: 0.25rem;

          svg {
            color: var(--color-orange);
          }
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
        svg {
          transition: 150ms ease transform;
        }
      }
    }
  }
  .content {
    display: none;
    flex-direction: column;
    padding-top: var(--gap-md);
    .main-info {
      margin-bottom: var(--gap-lg);
    }
    .permissions {
      margin-bottom: var(--gap-md);
      max-width: 45rem;
      display: grid;
      grid-template-columns: repeat(auto-fill, minmax(10rem, 1fr));
      grid-gap: 0.5rem;
    }
  }
  &.open {
    .member-header {
      .dropdown-icon svg {
        transform: rotate(180deg);
      }
    }
    .content {
      display: flex;
    }
  }
}
:deep(.checkbox-outer) {
  button.checkbox {
    border: none;
  }
}
</style>
