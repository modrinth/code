<script setup lang="ts">
import { Avatar, ButtonStyled, NewModal, OverflowMenu } from '@modrinth/ui'
import { UserPlusIcon, MoreVerticalIcon, MailIcon, SettingsIcon, TrashIcon } from '@modrinth/assets'
import { ref, onUnmounted } from 'vue'
import { friend_listener } from '@/helpers/events'
import { friends, friend_statuses, add_friend, remove_friend } from '@/helpers/friends'
import { get_user_many } from '@/helpers/cache'
import { handleError } from '@/store/notifications.js'
import ContextMenu from '@/components/ui/ContextMenu.vue'

defineProps<{
  credentials: any
  signIn: () => void
}>()

const userFriends = ref([
  {
    name: 'Prospector',
    avatar: 'https://cdn.modrinth.com/user/Dc7EYhxG/32e8b1f7d18288262d1ed92cbdf43272d21b4fcd.png',
    status: 'Playing Simply Optimized',
    online: true,
  },
  {
    name: 'Minenash',
    avatar: 'https://avatars3.githubusercontent.com/u/12193049?v=4',
    online: true,
  },
  {
    name: 'coolbot100s',
    avatar: 'https://avatars.githubusercontent.com/u/76798835?v=4',
    online: false,
  },
])

const username = ref('')
const addFriendModal = ref()
async function addFriend() {
  addFriendModal.value.hide()
  await add_friend(username.value).catch(handleError)
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

async function removeFriend(userId: string) {
  await remove_friend(userId).catch(handleError)
  await loadFriends()
}

const loading = ref(false)
async function loadFriends(timeout = false) {
  loading.value = timeout

  try {
    const friendsList = await friends()

    if (friendsList.length === 0) {
      userFriends.value = []
    } else {
      const friendStatuses = await friend_statuses()
      const users = await get_user_many(friendsList.map((x) => x.id))

      userFriends.value = friendsList.map((friend) => {
        const user = users.find((x) => x.id === friend.id)
        const status = friendStatuses.find((x) => x.user_id === friend.id)
        return {
          ...friend,
          status: status?.profile_name,
          last_updated: status?.last_update,
          avatar: user?.avatar_url,
          username: user?.username,
          online: !!status,
        }
      })

      // TODO: sort values
      // TODO: remove pending
    }

    loading.value = false
  } catch (e) {
    console.error('Error loading friends', e)
    if (timeout) {
      setTimeout(() => loadFriends(), 15 * 1000)
    }
  }
}

loadFriends(true)

const unlisten = await friend_listener(() => loadFriends())
onUnmounted(() => {
  unlisten()
})

// TODO: Manage friends menu
// TODO: Remove friends menu
</script>

<template>
  <NewModal ref="addFriendModal" header="Add a friend">
    <div class="mb-4">
      <h2 class="m-0 text-xl">Username</h2>
      <p class="m-0 mt-1 leading-tight">You can add friends with their Modrinth username.</p>
      <input v-model="username" class="mt-2" type="text" placeholder="Enter username..." />
    </div>
    <ButtonStyled color="brand">
      <button class="ml-auto" :disabled="username.length === 0" @click="addFriend">
        <UserPlusIcon />
        Add friend
      </button>
    </ButtonStyled>
  </NewModal>
  <div class="flex justify-between items-center">
    <h3 class="text-lg m-0">Friends</h3>
    <OverflowMenu
      :options="[
        {
          id: 'add-friend',
          action: () => addFriendModal.show(),
        },
        {
          id: 'manage-friends',
          action: () => {},
        },
        {
          id: 'view-requests',
          action: () => {},
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
      </template>
      <template #view-requests>
        <MailIcon aria-hidden="true" />
        View friend requests
      </template>
    </OverflowMenu>
  </div>
  <div class="flex flex-col gap-2 mt-2">
    <template v-if="userFriends.length === 0">
      <div class="text-sm">
        <div class="mb-2">You have no friends :C</div>
        <div v-if="!credentials">
          <span class="text-link cursor-pointer" @click="signIn">Sign in</span> to add friends!
        </div>
        <div v-else>
          Why don't you
          <span class="text-link cursor-pointer" @click="addFriendModal.show()">add one</span>?
        </div>
      </div>
    </template>
    <template v-else-if="loading">
      <div v-for="n in 5" :key="n" class="flex gap-2 items-center animate-pulse">
        <div class="min-w-9 min-h-9 bg-button-bg rounded-full"></div>
        <div class="flex flex-col w-full">
          <div class="h-3 bg-button-bg rounded-full w-1/2 mb-1"></div>
          <div class="h-2.5 bg-button-bg rounded-full w-3/4"></div>
        </div>
      </div>
    </template>
    <template v-else>
      <ContextMenu ref="friendOptions" @option-clicked="handleFriendOptions">
        <template #remove-friend> <TrashIcon /> Remove friend </template>
      </ContextMenu>
      <div
        v-for="friend in userFriends.slice(0, 5)"
        :key="friend.name"
        class="flex gap-2 items-center"
        :class="{ grayscale: !friend.online }"
        @contextmenu.prevent.stop="
          (event) =>
            friendOptions.showMenu(event, friend.id, [
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
