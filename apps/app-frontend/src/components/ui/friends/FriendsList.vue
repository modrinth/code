<script setup lang="ts">
import { Avatar, ButtonStyled, OverflowMenu, useRelativeTime } from '@modrinth/ui'
import {
  UserPlusIcon,
  MoreVerticalIcon,
  MailIcon,
  SettingsIcon,
  TrashIcon,
  XIcon,
} from '@modrinth/assets'
import { ref, onUnmounted, watch, computed } from 'vue'
import { friend_listener } from '@/helpers/events'
import { friends, friend_statuses, add_friend, remove_friend } from '@/helpers/friends'
import { get_user_many } from '@/helpers/cache'
import { handleError } from '@/store/notifications.js'
import ContextMenu from '@/components/ui/ContextMenu.vue'
import type { Dayjs } from 'dayjs'
import dayjs from 'dayjs'
import ModalWrapper from '@/components/ui/modal/ModalWrapper.vue'

const formatRelativeTime = useRelativeTime()

const props = defineProps<{
  credentials: unknown | null
  signIn: () => void
}>()

const userCredentials = computed(() => props.credentials)

const search = ref('')
const manageFriendsModal = ref()
const friendInvitesModal = ref()

const username = ref('')
const addFriendModal = ref()
async function addFriendFromModal() {
  addFriendModal.value.hide()
  await add_friend(username.value).catch(handleError)
  username.value = ''
  await loadFriends()
}

const friendOptions = ref()
async function handleFriendOptions(args) {
  switch (args.option) {
    case 'remove-friend':
      await removeFriend(args.item)
      break
  }
}

async function addFriend(friend: Friend) {
  await add_friend(
    friend.id === userCredentials.value.user_id ? friend.friend_id : friend.id,
  ).catch(handleError)
  await loadFriends()
}

async function removeFriend(friend: Friend) {
  await remove_friend(
    friend.id === userCredentials.value.user_id ? friend.friend_id : friend.id,
  ).catch(handleError)
  await loadFriends()
}

type Friend = {
  id: string
  friend_id: string | null
  status: string | null
  last_updated: Dayjs | null
  created: Dayjs
  username: string
  accepted: boolean
  online: boolean
  avatar: string
}

const userFriends = ref<Friend[]>([])
const acceptedFriends = computed(() =>
  userFriends.value
    .filter((x) => x.accepted)
    .toSorted((a, b) => {
      if (a.last_updated === null && b.last_updated === null) {
        return 0 // Both are null, equal in sorting
      }
      if (a.last_updated === null) {
        return 1 // `a` is null, move it after `b`
      }
      if (b.last_updated === null) {
        return -1 // `b` is null, move it after `a`
      }
      // Both are non-null, sort by date
      return b.last_updated.diff(a.last_updated)
    }),
)
const pendingFriends = computed(() =>
  userFriends.value.filter((x) => !x.accepted).toSorted((a, b) => b.created.diff(a.created)),
)

const loading = ref(true)
async function loadFriends(timeout = false) {
  loading.value = timeout

  try {
    const friendsList = await friends()

    if (friendsList.length === 0) {
      userFriends.value = []
    } else {
      const friendStatuses = await friend_statuses()
      const users = await get_user_many(
        friendsList.map((x) => (x.id === userCredentials.value.user_id ? x.friend_id : x.id)),
      )

      userFriends.value = friendsList.map((friend) => {
        const user = users.find((x) => x.id === friend.id || x.id === friend.friend_id)
        const status = friendStatuses.find(
          (x) => x.user_id === friend.id || x.user_id === friend.friend_id,
        )
        return {
          id: friend.id,
          friend_id: friend.friend_id,
          status: status?.profile_name,
          last_updated: status && status.last_update ? dayjs(status.last_update) : null,
          created: dayjs(friend.created),
          avatar: user?.avatar_url,
          username: user?.username,
          online: !!status,
          accepted: friend.accepted,
        }
      })
    }

    loading.value = false
  } catch (e) {
    console.error('Error loading friends', e)
    if (timeout) {
      setTimeout(() => loadFriends(), 15 * 1000)
    }
  }
}

watch(
  userCredentials,
  () => {
    if (userCredentials.value === undefined) {
      userFriends.value = []
    } else if (userCredentials.value === null) {
      userFriends.value = []
      loading.value = false
    } else {
      loadFriends(true)
    }
  },
  { immediate: true },
)

const unlisten = await friend_listener(() => loadFriends())
onUnmounted(() => {
  unlisten()
})
</script>

<template>
  <ModalWrapper ref="manageFriendsModal" header="Manage friends">
    <p v-if="acceptedFriends.length === 0">Add friends to share what you're playing!</p>
    <div v-else class="flex flex-col gap-4 min-w-[20rem]">
      <input v-model="search" type="text" placeholder="Search friends..." class="w-full" />
      <div
        v-for="friend in acceptedFriends.filter(
          (x) => !search || x.username.toLowerCase().includes(search),
        )"
        :key="friend.username"
        class="flex gap-2 items-center"
      >
        <div class="relative">
          <Avatar :src="friend.avatar" class="w-12 h-12 rounded-full" size="2.25rem" circle />
          <span
            v-if="friend.online"
            class="bottom-1 right-0 absolute w-3 h-3 bg-brand border-2 border-black border-solid rounded-full"
          />
        </div>
        <div>{{ friend.username }}</div>
        <div class="ml-auto">
          <ButtonStyled>
            <button @click="removeFriend(friend)">
              <XIcon />
              Remove
            </button>
          </ButtonStyled>
        </div>
      </div>
    </div>
  </ModalWrapper>
  <ModalWrapper ref="friendInvitesModal" header="View friend requests">
    <p v-if="pendingFriends.length === 0">You have no pending friend requests :C</p>
    <div v-else class="flex flex-col gap-4">
      <div v-for="friend in pendingFriends" :key="friend.username" class="flex gap-2">
        <Avatar :src="friend.avatar" class="w-12 h-12 rounded-full" size="2.25rem" circle />
        <div class="flex flex-col gap-2">
          <div>
            <p class="m-0">
              <template v-if="friend.id === userCredentials.user_id">
                <span class="font-bold">{{ friend.username }}</span> sent you a friend request
              </template>
              <template v-else>
                You sent <span class="font-bold">{{ friend.username }}</span> a friend request
              </template>
            </p>
            <p class="m-0 text-sm text-secondary">
              {{ formatRelativeTime(friend.created.toISOString()) }}
            </p>
          </div>
          <div class="flex gap-2">
            <template v-if="friend.id === userCredentials.user_id">
              <ButtonStyled color="brand">
                <button @click="addFriend(friend)">
                  <UserPlusIcon />
                  Accept
                </button>
              </ButtonStyled>
              <ButtonStyled>
                <button @click="removeFriend(friend)">
                  <XIcon />
                  Ignore
                </button>
              </ButtonStyled>
            </template>
            <template v-else>
              <ButtonStyled>
                <button @click="removeFriend(friend)">
                  <XIcon />
                  Cancel
                </button>
              </ButtonStyled>
            </template>
          </div>
        </div>
      </div>
    </div>
  </ModalWrapper>
  <ModalWrapper ref="addFriendModal" header="Add a friend">
    <div class="mb-4">
      <h2 class="m-0 text-lg font-extrabold text-contrast">Username</h2>
      <p class="m-0 mt-1 leading-tight">You can add friends with their Modrinth username.</p>
      <input v-model="username" class="mt-2 w-full" type="text" placeholder="Enter username..." />
    </div>
    <ButtonStyled color="brand">
      <button :disabled="username.length === 0" @click="addFriendFromModal">
        <UserPlusIcon />
        Add friend
      </button>
    </ButtonStyled>
  </ModalWrapper>
  <div class="flex justify-between items-center">
    <h3 class="text-lg m-0">Friends</h3>
    <ButtonStyled v-if="userCredentials" type="transparent" circular>
      <OverflowMenu
        :options="[
          {
            id: 'add-friend',
            action: () => addFriendModal.show(),
          },
          {
            id: 'manage-friends',
            action: () => manageFriendsModal.show(),
            shown: acceptedFriends.length > 0,
          },
          {
            id: 'view-requests',
            action: () => friendInvitesModal.show(),
            shown: pendingFriends.length > 0,
          },
        ]"
        aria-label="More options"
      >
        <MoreVerticalIcon aria-hidden="true" />
        <template #add-friend>
          <UserPlusIcon aria-hidden="true" />
          Add friend
        </template>
        <template #manage-friends>
          <SettingsIcon aria-hidden="true" />
          Manage friends
          <div
            v-if="acceptedFriends.length > 0"
            class="bg-button-bg w-6 h-6 rounded-full flex items-center justify-center"
          >
            {{ acceptedFriends.length }}
          </div>
        </template>
        <template #view-requests>
          <MailIcon aria-hidden="true" />
          View friend requests
          <div
            v-if="pendingFriends.length > 0"
            class="bg-button-bg w-6 h-6 rounded-full flex items-center justify-center"
          >
            {{ pendingFriends.length }}
          </div>
        </template>
      </OverflowMenu>
    </ButtonStyled>
  </div>
  <div class="flex flex-col gap-2 mt-2">
    <template v-if="loading">
      <div v-for="n in 5" :key="n" class="flex gap-2 items-center animate-pulse">
        <div class="min-w-9 min-h-9 bg-button-bg rounded-full"></div>
        <div class="flex flex-col w-full">
          <div class="h-3 bg-button-bg rounded-full w-1/2 mb-1"></div>
          <div class="h-2.5 bg-button-bg rounded-full w-3/4"></div>
        </div>
      </div>
    </template>
    <template v-else-if="acceptedFriends.length === 0">
      <div class="text-sm">
        <div v-if="!userCredentials">
          <span class="text-link cursor-pointer" @click="signIn">Sign in</span> to add friends!
        </div>
        <div v-else>
          <span class="text-link cursor-pointer" @click="addFriendModal.show()">Add friends</span>
          to share what you're playing!
        </div>
      </div>
    </template>
    <template v-else>
      <ContextMenu ref="friendOptions" @option-clicked="handleFriendOptions">
        <template #remove-friend> <TrashIcon /> Remove friend </template>
      </ContextMenu>
      <div
        v-for="friend in acceptedFriends.slice(0, 5)"
        :key="friend.username"
        class="flex gap-2 items-center"
        :class="{ grayscale: !friend.online }"
        @contextmenu.prevent.stop="
          (event) =>
            friendOptions.showMenu(event, friend, [
              {
                name: 'remove-friend',
                color: 'danger',
              },
            ])
        "
      >
        <div class="relative">
          <Avatar :src="friend.avatar" class="w-12 h-12 rounded-full" size="2.25rem" circle />
          <span
            v-if="friend.online"
            class="bottom-1 right-0 absolute w-3 h-3 bg-brand border-2 border-black border-solid rounded-full"
          />
        </div>
        <div class="flex flex-col">
          <span class="text-md m-0 font-medium" :class="{ 'text-secondary': !friend.online }">
            {{ friend.username }}
          </span>
          <span v-if="friend.status" class="m-0 text-xs">{{ friend.status }}</span>
        </div>
      </div>
    </template>
  </div>
</template>
