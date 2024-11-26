<template>
  <div>
    <ModalConfirm
      ref="modal_remove"
      title="您确定要从团队中移除该项目吗？"
      description="如果继续，该项目将不再由该团队管理。"
      proceed-label="移除"
      :noblur="!(cosmetics?.advancedRendering ?? true)"
      @proceed="onRemoveFromOrg"
    />
    <Card>
      <div class="label">
        <h3>
          <span class="label__title size-card-header">管理成员</span>
        </h3>
      </div>
      <span class="label">
        <span class="label__title">邀请成员</span>
        <span class="label__description">
          输入您想要邀请成为该项目成员的人员的 BBSMC 用户名。
        </span>
      </span>
      <div class="input-group">
        <input
          id="username"
          v-model="currentUsername"
          type="text"
          placeholder="用户名"
          :disabled="(props.currentMember?.permissions & MANAGE_INVITES) !== MANAGE_INVITES"
          @keypress.enter="inviteTeamMember()"
        />
        <label for="username" class="hidden">Username</label>
        <button
          class="iconified-button brand-button"
          :disabled="(props.currentMember?.permissions & MANAGE_INVITES) !== MANAGE_INVITES"
          @click="inviteTeamMember()"
        >
          <UserPlusIcon />
          邀请
        </button>
      </div>
      <div class="adjacent-input">
        <span class="label">
          <span class="label__title">退出项目</span>
          <span class="label__description"> 从该项目离开,不再成为该项目的成员 </span>
        </span>
        <button
          class="iconified-button danger-button"
          :disabled="props.currentMember?.is_owner"
          :title="props.currentMember?.is_owner ? '你无法离开！你是老大！' : ''"
          @click="leaveProject()"
        >
          <UserRemoveIcon />
          退出
        </button>
      </div>
    </Card>
    <div
      v-for="(member, index) in allTeamMembers"
      :key="member.user.id"
      class="universal-card member"
      :class="{ open: openTeamMembers.includes(member.user.id) }"
    >
      <div class="member-header">
        <div class="info">
          <Avatar :src="member.avatar_url" :alt="member.username" size="sm" circle />
          <div class="text">
            <nuxt-link :to="'/user/' + member.user.username" class="name">
              <p>{{ member.name }}</p>
              <CrownIcon v-if="member.is_owner" v-tooltip="'Project owner'" />
            </nuxt-link>
            <p>{{ member.role }}</p>
          </div>
        </div>
        <div class="side-buttons">
          <Badge v-if="member.accepted" type="accepted" />
          <Badge v-else type="pending" />
          <button
            class="square-button dropdown-icon"
            @click="
              openTeamMembers.indexOf(member.user.id) === -1
                ? openTeamMembers.push(member.user.id)
                : (openTeamMembers = openTeamMembers.filter((it) => it !== member.user.id))
            "
          >
            <DropdownIcon />
          </button>
        </div>
      </div>
      <div class="content">
        <div class="adjacent-input">
          <label :for="`member-${allTeamMembers[index].user.username}-role`">
            <span class="label__title">角色</span>
            <span class="label__description"> 该成员在此资源中的头衔。 </span>
          </label>
          <input
            :id="`member-${allTeamMembers[index].user.username}-role`"
            v-model="allTeamMembers[index].role"
            type="text"
            :disabled="(props.currentMember?.permissions & EDIT_MEMBER) !== EDIT_MEMBER"
          />
        </div>
        <!--        <div class="adjacent-input">-->
        <!--          <label :for="`member-${allTeamMembers[index].user.username}-monetization-weight`">-->
        <!--            <span class="label__title">盈利权重</span>-->
        <!--            <span class="label__description">-->
        <!--              相对于所有其他成员的货币化权重，这决定了该项目的收入中有多少部分归该成员所有。-->
        <!--            </span>-->
        <!--          </label>-->
        <!--          <input-->
        <!--            :id="`member-${allTeamMembers[index].user.username}-monetization-weight`"-->
        <!--            v-model="allTeamMembers[index].payouts_split"-->
        <!--            type="number"-->
        <!--            :disabled="(props.currentMember?.permissions & EDIT_MEMBER) !== EDIT_MEMBER"-->
        <!--          />-->
        <!--        </div>-->
        <template v-if="!member.is_owner">
          <span class="label">
            <span class="label__title">Permissions</span>
          </span>
          <div v-if="allTeamMembers[index]" class="permissions">
            <Checkbox
              :model-value="(member.permissions & UPLOAD_VERSION) === UPLOAD_VERSION"
              :disabled="
                (props.currentMember?.permissions & EDIT_MEMBER) !== EDIT_MEMBER ||
                (props.currentMember?.permissions & UPLOAD_VERSION) !== UPLOAD_VERSION
              "
              label="上传版本"
              @update:model-value="allTeamMembers[index].permissions ^= UPLOAD_VERSION"
            />
            <Checkbox
              :model-value="(member.permissions & DELETE_VERSION) === DELETE_VERSION"
              :disabled="
                (props.currentMember?.permissions & EDIT_MEMBER) !== EDIT_MEMBER ||
                (props.currentMember?.permissions & DELETE_VERSION) !== DELETE_VERSION
              "
              label="删除版本"
              @update:model-value="allTeamMembers[index].permissions ^= DELETE_VERSION"
            />
            <Checkbox
              :model-value="(member.permissions & EDIT_DETAILS) === EDIT_DETAILS"
              :disabled="
                (props.currentMember?.permissions & EDIT_MEMBER) !== EDIT_MEMBER ||
                (props.currentMember?.permissions & EDIT_DETAILS) !== EDIT_DETAILS
              "
              label="编辑详细信息"
              @update:model-value="allTeamMembers[index].permissions ^= EDIT_DETAILS"
            />
            <Checkbox
              :model-value="(member.permissions & EDIT_BODY) === EDIT_BODY"
              :disabled="
                (props.currentMember?.permissions & EDIT_MEMBER) !== EDIT_MEMBER ||
                (props.currentMember?.permissions & EDIT_BODY) !== EDIT_BODY
              "
              label="编辑正文"
              @update:model-value="allTeamMembers[index].permissions ^= EDIT_BODY"
            />
            <Checkbox
              :model-value="(member.permissions & MANAGE_INVITES) === MANAGE_INVITES"
              :disabled="
                (props.currentMember?.permissions & EDIT_MEMBER) !== EDIT_MEMBER ||
                (props.currentMember?.permissions & MANAGE_INVITES) !== MANAGE_INVITES
              "
              label="邀请成员"
              @update:model-value="allTeamMembers[index].permissions ^= MANAGE_INVITES"
            />
            <Checkbox
              :model-value="(member.permissions & REMOVE_MEMBER) === REMOVE_MEMBER"
              :disabled="
                (props.currentMember?.permissions & EDIT_MEMBER) !== EDIT_MEMBER ||
                (props.currentMember?.permissions & REMOVE_MEMBER) !== REMOVE_MEMBER
              "
              label="移除成员"
              @update:model-value="allTeamMembers[index].permissions ^= REMOVE_MEMBER"
            />
            <Checkbox
              :model-value="(member.permissions & EDIT_MEMBER) === EDIT_MEMBER"
              :disabled="(props.currentMember?.permissions & EDIT_MEMBER) !== EDIT_MEMBER"
              label="设置成员"
              @update:model-value="allTeamMembers[index].permissions ^= EDIT_MEMBER"
            />
            <Checkbox
              :model-value="(member.permissions & DELETE_PROJECT) === DELETE_PROJECT"
              :disabled="
                (props.currentMember?.permissions & EDIT_MEMBER) !== EDIT_MEMBER ||
                (props.currentMember?.permissions & DELETE_PROJECT) !== DELETE_PROJECT
              "
              label="删除资源"
              @update:model-value="allTeamMembers[index].permissions ^= DELETE_PROJECT"
            />
            <Checkbox
              :model-value="(member.permissions & VIEW_ANALYTICS) === VIEW_ANALYTICS"
              :disabled="
                (props.currentMember?.permissions & EDIT_MEMBER) !== EDIT_MEMBER ||
                (props.currentMember?.permissions & VIEW_ANALYTICS) !== VIEW_ANALYTICS
              "
              label="查看分析"
              @update:model-value="allTeamMembers[index].permissions ^= VIEW_ANALYTICS"
            />
            <Checkbox
              :model-value="(member.permissions & VIEW_PAYOUTS) === VIEW_PAYOUTS"
              :disabled="
                (props.currentMember?.permissions & EDIT_MEMBER) !== EDIT_MEMBER ||
                (props.currentMember?.permissions & VIEW_PAYOUTS) !== VIEW_PAYOUTS
              "
              label="查看收入"
              @update:model-value="allTeamMembers[index].permissions ^= VIEW_PAYOUTS"
            />
          </div>
        </template>
        <div class="input-group">
          <button
            class="iconified-button brand-button"
            :disabled="(props.currentMember?.permissions & EDIT_MEMBER) !== EDIT_MEMBER"
            @click="updateTeamMember(index)"
          >
            <SaveIcon />
            保存
          </button>
          <button
            v-if="!member.is_owner"
            class="iconified-button danger-button"
            :disabled="(props.currentMember?.permissions & EDIT_MEMBER) !== EDIT_MEMBER"
            @click="removeTeamMember(index)"
          >
            <UserRemoveIcon />
            移除成员
          </button>
          <button
            v-if="!member.is_owner && props.currentMember?.is_owner && member.accepted"
            class="iconified-button"
            @click="transferOwnership(index)"
          >
            <TransferIcon />
            转让所有权
          </button>
        </div>
      </div>
    </div>
    <section class="universal-card">
      <div class="label">
        <span class="label__title size-card-header">团队</span>
      </div>
      <div v-if="props.organization">
        <p>
          {{ props.organization.name }}正在管理该资源. 成员的默认权限设置请前往
          <nuxt-link :to="`/organization/${props.organization.slug}/settings/members`">
            团队设置</nuxt-link
          >. 您可以在下面覆盖成员在该项目的权限
        </p>
        <nuxt-link
          :to="`/organization/${props.organization.slug}`"
          class="universal-card button-base recessed org"
        >
          <Avatar :src="props.organization.icon_url" :alt="props.organization.name" size="md" />
          <div class="details">
            <div class="title">
              {{ props.organization.name }}
            </div>
            <div class="description">
              {{ props.organization.description }}
            </div>
            <span class="stat-bar">
              <div class="stats">
                <UsersIcon />
                <span>
                  {{ acceptedOrgMembers.length }} 位成员<template
                    v-if="acceptedOrgMembers.length !== 1"
                  ></template>
                </span>
              </div>
            </span>
          </div>
        </nuxt-link>
      </div>
      <p v-else>
        此项目不受任何团队管理。如果您是任何团队的成员，您可以将管理权转让给其中一个团队。
      </p>
      <div v-if="!props.organization" class="input-group">
        <Multiselect
          id="organization-picker"
          v-model="selectedOrganization"
          class="large-multiselect"
          placeholder="选择一个团队"
          track-by="id"
          label="name"
          open-direction="top"
          :close-on-select="true"
          :show-labels="false"
          :allow-empty="false"
          :options="organizations || []"
          :disabled="!props.currentMember?.is_owner || organizations?.length === 0"
        />
        <button class="btn btn-primary" :disabled="!selectedOrganization" @click="onAddToOrg">
          <CheckIcon />
          转移管理权
        </button>
      </div>
      <button v-if="props.organization" class="btn" @click="$refs.modal_remove.show()">
        <OrganizationIcon />
        从团队中移除
      </button>
    </section>
    <div
      v-for="(member, index) in allOrgMembers"
      :key="member.user.id"
      class="universal-card member"
      :class="{ open: openTeamMembers.includes(member.user.id) }"
    >
      <div class="member-header">
        <div class="info">
          <Avatar :src="member.user.avatar_url" :alt="member.user.username" size="sm" circle />
          <div class="text">
            <nuxt-link :to="'/user/' + member.user.username" class="name">
              <p>{{ member.user.username }}</p>
              <CrownIcon v-if="member.is_owner" v-tooltip="'团队创始人'" />
            </nuxt-link>
            <p>{{ member.role }}</p>
          </div>
        </div>
        <div class="side-buttons">
          <Badge v-if="member.accepted" type="accepted" />
          <Badge v-else type="pending" />
          <button
            class="square-button dropdown-icon"
            @click="
              openTeamMembers.indexOf(member.user.id) === -1
                ? openTeamMembers.push(member.user.id)
                : (openTeamMembers = openTeamMembers.filter((it) => it !== member.user.id))
            "
          >
            <DropdownIcon />
          </button>
        </div>
      </div>
      <div class="content">
        <div class="adjacent-input">
          <label :for="`member-${allOrgMembers[index].user.username}-override-perms`">
            <span class="label__title">覆盖</span>
            <span class="label__description">
              覆盖团队默认值权限并为资源中的用户分配在该资源中的自定义权限、角色
            </span>
          </label>
          <input
            :id="`member-${allOrgMembers[index].user.username}-override-perms`"
            v-model="allOrgMembers[index].override"
            class="switch stylized-toggle"
            type="checkbox"
            :disabled="(props.currentMember?.permissions & EDIT_MEMBER) !== EDIT_MEMBER"
          />
        </div>
        <div class="adjacent-input">
          <label :for="`member-${allOrgMembers[index].user.username}-role`">
            <span class="label__title">角色</span>
            <span class="label__description"> 该成员在此资源中的头衔。 </span>
          </label>
          <input
            :id="`member-${allOrgMembers[index].user.username}-role`"
            v-model="allOrgMembers[index].role"
            type="text"
            :disabled="
              (props.currentMember?.permissions & EDIT_MEMBER) !== EDIT_MEMBER ||
              !allOrgMembers[index].override
            "
          />
        </div>
        <!--        <div class="adjacent-input">-->
        <!--          <label :for="`member-${allOrgMembers[index].user.username}-monetization-weight`">-->
        <!--            <span class="label__title">Monetization weight</span>-->
        <!--            <span class="label__description">-->
        <!--              Relative to all other members' monetization weights, this determines what portion of-->
        <!--              this project's revenue goes to this member.-->
        <!--            </span>-->
        <!--          </label>-->
        <!--          <input-->
        <!--            :id="`member-${allOrgMembers[index].user.username}-monetization-weight`"-->
        <!--            v-model="allOrgMembers[index].payouts_split"-->
        <!--            type="number"-->
        <!--            :disabled="-->
        <!--              (props.currentMember?.permissions & EDIT_MEMBER) !== EDIT_MEMBER ||-->
        <!--              !allOrgMembers[index].override-->
        <!--            "-->
        <!--          />-->
        <!--        </div>-->
        <template v-if="!member.is_owner">
          <span class="label">
            <span class="label__title">权限</span>
          </span>
          <div class="permissions">
            <Checkbox
              :model-value="(member.permissions & UPLOAD_VERSION) === UPLOAD_VERSION"
              :disabled="
                (props.currentMember?.permissions & EDIT_MEMBER) !== EDIT_MEMBER ||
                (props.currentMember?.permissions & UPLOAD_VERSION) !== UPLOAD_VERSION ||
                !allOrgMembers[index].override
              "
              label="上传新版本"
              @update:model-value="allOrgMembers[index].permissions ^= UPLOAD_VERSION"
            />
            <Checkbox
              :model-value="(member.permissions & DELETE_VERSION) === DELETE_VERSION"
              :disabled="
                (props.currentMember?.permissions & EDIT_MEMBER) !== EDIT_MEMBER ||
                (props.currentMember?.permissions & DELETE_VERSION) !== DELETE_VERSION ||
                !allOrgMembers[index].override
              "
              label="删除版本"
              @update:model-value="allOrgMembers[index].permissions ^= DELETE_VERSION"
            />
            <Checkbox
              :model-value="(member.permissions & EDIT_DETAILS) === EDIT_DETAILS"
              :disabled="
                (props.currentMember?.permissions & EDIT_MEMBER) !== EDIT_MEMBER ||
                (props.currentMember?.permissions & EDIT_DETAILS) !== EDIT_DETAILS ||
                !allOrgMembers[index].override
              "
              label="编辑详情"
              @update:model-value="allOrgMembers[index].permissions ^= EDIT_DETAILS"
            />
            <Checkbox
              :model-value="(member.permissions & EDIT_BODY) === EDIT_BODY"
              :disabled="
                (props.currentMember?.permissions & EDIT_MEMBER) !== EDIT_MEMBER ||
                (props.currentMember?.permissions & EDIT_BODY) !== EDIT_BODY ||
                !allOrgMembers[index].override
              "
              label="编辑正文"
              @update:model-value="allOrgMembers[index].permissions ^= EDIT_BODY"
            />
            <Checkbox
              :model-value="(member.permissions & MANAGE_INVITES) === MANAGE_INVITES"
              :disabled="
                (props.currentMember?.permissions & EDIT_MEMBER) !== EDIT_MEMBER ||
                (props.currentMember?.permissions & MANAGE_INVITES) !== MANAGE_INVITES ||
                !allOrgMembers[index].override
              "
              label="邀请成员"
              @update:model-value="allOrgMembers[index].permissions ^= MANAGE_INVITES"
            />
            <Checkbox
              :model-value="(member.permissions & REMOVE_MEMBER) === REMOVE_MEMBER"
              :disabled="
                (props.currentMember?.permissions & EDIT_MEMBER) !== EDIT_MEMBER ||
                (props.currentMember?.permissions & REMOVE_MEMBER) !== REMOVE_MEMBER ||
                !allOrgMembers[index].override
              "
              label="移除成员"
              @update:model-value="allOrgMembers[index].permissions ^= REMOVE_MEMBER"
            />
            <Checkbox
              :model-value="(member.permissions & EDIT_MEMBER) === EDIT_MEMBER"
              :disabled="
                (props.currentMember?.permissions & EDIT_MEMBER) !== EDIT_MEMBER ||
                !allOrgMembers[index].override
              "
              label="编辑成员"
              @update:model-value="allOrgMembers[index].permissions ^= EDIT_MEMBER"
            />
            <Checkbox
              :model-value="(member.permissions & DELETE_PROJECT) === DELETE_PROJECT"
              :disabled="
                (props.currentMember?.permissions & EDIT_MEMBER) !== EDIT_MEMBER ||
                (props.currentMember?.permissions & DELETE_PROJECT) !== DELETE_PROJECT ||
                !allOrgMembers[index].override
              "
              label="删除资源"
              @update:model-value="allOrgMembers[index].permissions ^= DELETE_PROJECT"
            />
            <Checkbox
              :model-value="(member.permissions & VIEW_ANALYTICS) === VIEW_ANALYTICS"
              :disabled="
                (props.currentMember?.permissions & EDIT_MEMBER) !== EDIT_MEMBER ||
                (props.currentMember?.permissions & VIEW_ANALYTICS) !== VIEW_ANALYTICS ||
                !allOrgMembers[index].override
              "
              label="查看分析"
              @update:model-value="allOrgMembers[index].permissions ^= VIEW_ANALYTICS"
            />
            <!--            <Checkbox-->
            <!--              :model-value="(member.permissions & VIEW_PAYOUTS) === VIEW_PAYOUTS"-->
            <!--              :disabled="-->
            <!--                (props.currentMember?.permissions & EDIT_MEMBER) !== EDIT_MEMBER ||-->
            <!--                (props.currentMember?.permissions & VIEW_PAYOUTS) !== VIEW_PAYOUTS ||-->
            <!--                !allOrgMembers[index].override-->
            <!--              "-->
            <!--              label="View revenue"-->
            <!--              @update:model-value="allOrgMembers[index].permissions ^= VIEW_PAYOUTS"-->
            <!--            />-->
          </div>
        </template>
        <div class="input-group">
          <button
            class="iconified-button brand-button"
            :disabled="(props.currentMember?.permissions & EDIT_MEMBER) !== EDIT_MEMBER"
            @click="updateOrgMember(index)"
          >
            <SaveIcon />
            保存
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { Multiselect } from "vue-multiselect";
import { TransferIcon, CheckIcon, UsersIcon } from "@modrinth/assets";
import { Avatar, Badge, Card, Checkbox } from "@modrinth/ui";

import ModalConfirm from "~/components/ui/ModalConfirm.vue";
import DropdownIcon from "~/assets/images/utils/dropdown.svg?component";
import SaveIcon from "~/assets/images/utils/save.svg?component";
import UserPlusIcon from "~/assets/images/utils/user-plus.svg?component";
import UserRemoveIcon from "~/assets/images/utils/user-x.svg?component";
import OrganizationIcon from "~/assets/images/utils/organization.svg?component";
import CrownIcon from "~/assets/images/utils/crown.svg?component";

import { removeSelfFromTeam } from "~/helpers/teams.js";

const props = defineProps({
  project: {
    type: Object,
    default() {
      return {};
    },
  },
  organization: {
    type: Object,
    default() {
      return {};
    },
  },
  allMembers: {
    type: Array,
    default() {
      return [];
    },
  },
  currentMember: {
    type: Object,
    default() {
      return null;
    },
  },
  resetProject: {
    type: Function,
    required: true,
    default: () => {},
  },
  resetOrganization: {
    type: Function,
    required: true,
    default: () => {},
  },
  resetMembers: {
    type: Function,
    required: true,
    default: () => {},
  },
});

const cosmetics = useCosmetics();
const auth = await useAuth();

const allTeamMembers = ref([]);
const allOrgMembers = ref([]);

const acceptedOrgMembers = computed(() => {
  return props.organization?.members?.filter((x) => x.accepted) || [];
});

function initMembers() {
  const orgMembers = props.organization?.members || [];

  const selectedMembersForOrg = orgMembers.map((partialOrgMember) => {
    const foundMember = props.allMembers.find((tM) => tM.user.id === partialOrgMember.user.id);
    const returnVal = foundMember ?? partialOrgMember;

    // If replacing a partial with a full member, we need to mark as such.
    returnVal.override = !!foundMember;
    returnVal.oldOverride = !!foundMember;

    returnVal.is_owner = partialOrgMember.is_owner;

    return returnVal;
  });

  allOrgMembers.value = selectedMembersForOrg;

  allTeamMembers.value = props.allMembers.filter(
    (x) => !selectedMembersForOrg.some((y) => y.user.id === x.user.id),
  );
}

watch(
  [
    () => props.allMembers,
    () => props.organization,
    () => props.project,
    () => props.currentMember,
  ],
  initMembers,
);
initMembers();

const currentUsername = ref("");
const openTeamMembers = ref([]);
const selectedOrganization = ref(null);

const { data: organizations } = useAsyncData("organizations", () => {
  return useBaseFetch("user/" + auth.value?.user.id + "/organizations", {
    apiVersion: 3,
  });
});

const UPLOAD_VERSION = 1 << 0;
const DELETE_VERSION = 1 << 1;
const EDIT_DETAILS = 1 << 2;
const EDIT_BODY = 1 << 3;
const MANAGE_INVITES = 1 << 4;
const REMOVE_MEMBER = 1 << 5;
const EDIT_MEMBER = 1 << 6;
const DELETE_PROJECT = 1 << 7;
const VIEW_ANALYTICS = 1 << 8;
const VIEW_PAYOUTS = 1 << 9;

const onAddToOrg = useClientTry(async () => {
  if (!selectedOrganization.value) return;

  await useBaseFetch(`organization/${selectedOrganization.value.id}/projects`, {
    method: "POST",
    body: JSON.stringify({
      project_id: props.project.id,
    }),
    apiVersion: 3,
  });

  await updateMembers();

  addNotification({
    group: "main",
    title: "资源移交",
    text: "您的资源已转移到该团队。",
    type: "success",
  });
});

const onRemoveFromOrg = useClientTry(async () => {
  if (!props.project.organization || !auth.value?.user?.id) return;

  await useBaseFetch(`organization/${props.project.organization}/projects/${props.project.id}`, {
    method: "DELETE",
    body: JSON.stringify({
      new_owner: auth.value.user.id,
    }),
    apiVersion: 3,
  });

  await updateMembers();

  addNotification({
    group: "main",
    title: "资源已删除",
    text: "您的资源已从团队中删除.",
    type: "success",
  });
});

const leaveProject = async () => {
  await removeSelfFromTeam(props.project.team);
  navigateTo("/dashboard/projects");
};

const inviteTeamMember = async () => {
  startLoading();

  try {
    const user = await useBaseFetch(`user/${currentUsername.value}`);
    const data = {
      user_id: user.id.trim(),
    };
    await useBaseFetch(`team/${props.project.team}/members`, {
      method: "POST",
      body: data,
    });
    currentUsername.value = "";
    await updateMembers();
  } catch (err) {
    addNotification({
      group: "main",
      title: "发生错误",
      text: err?.data?.description || err?.message || err || "未知错误",
      type: "error",
    });
  }

  stopLoading();
};

const removeTeamMember = async (index) => {
  startLoading();

  try {
    await useBaseFetch(
      `team/${props.project.team}/members/${allTeamMembers.value[index].user.id}`,
      {
        method: "DELETE",
      },
    );
    await updateMembers();
  } catch (err) {
    addNotification({
      group: "main",
      title: "发生错误",
      text: err?.data?.description || err?.message || err || "未知错误",
      type: "error",
    });
  }

  stopLoading();
};

const updateTeamMember = async (index) => {
  startLoading();

  try {
    const data = !allTeamMembers.value[index].is_owner
      ? {
          permissions: allTeamMembers.value[index].permissions,
          role: allTeamMembers.value[index].role,
          payouts_split: allTeamMembers.value[index].payouts_split,
        }
      : {
          payouts_split: allTeamMembers.value[index].payouts_split,
          role: allTeamMembers.value[index].role,
        };

    await useBaseFetch(
      `team/${props.project.team}/members/${allTeamMembers.value[index].user.id}`,
      {
        method: "PATCH",
        body: data,
      },
    );
    await updateMembers();
    addNotification({
      group: "main",
      title: "成员更新",
      text: "您的资源成员已更新。",
      type: "success",
    });
  } catch (err) {
    addNotification({
      group: "main",
      title: "发生错误",
      text: err?.data?.description || err?.message || err || "未知错误",
      type: "error",
    });
  }

  stopLoading();
};

const transferOwnership = async (index) => {
  startLoading();

  try {
    await useBaseFetch(`team/${props.project.team}/owner`, {
      method: "PATCH",
      body: {
        user_id: allTeamMembers.value[index].user.id,
      },
    });
    await updateMembers();
  } catch (err) {
    addNotification({
      group: "main",
      title: "发生错误",
      text: err?.data?.description || err?.message || err || "Unknown error",
      type: "error",
    });
  }

  stopLoading();
};

async function updateOrgMember(index) {
  startLoading();

  try {
    if (allOrgMembers.value[index].override && !allOrgMembers.value[index].oldOverride) {
      await useBaseFetch(`team/${props.project.team}/members`, {
        method: "POST",
        body: {
          permissions: allOrgMembers.value[index].permissions,
          role: allOrgMembers.value[index].role,
          payouts_split: allOrgMembers.value[index].payouts_split,
          user_id: allOrgMembers.value[index].user.id,
        },
      });
    } else if (!allOrgMembers.value[index].override && allOrgMembers.value[index].oldOverride) {
      await useBaseFetch(
        `team/${props.project.team}/members/${allOrgMembers.value[index].user.id}`,
        {
          method: "DELETE",
        },
      );
    } else {
      await useBaseFetch(
        `team/${props.project.team}/members/${allOrgMembers.value[index].user.id}`,
        {
          method: "PATCH",
          body: {
            permissions: allOrgMembers.value[index].permissions,
            role: allOrgMembers.value[index].role,
            payouts_split: allOrgMembers.value[index].payouts_split,
          },
        },
      );
    }
    await updateMembers();
  } catch (err) {
    addNotification({
      group: "main",
      title: "发生错误",
      text: err?.data?.description || err?.message || err || "Unknown error",
      type: "error",
    });
  }

  stopLoading();
}

const updateMembers = async () => {
  await Promise.all([props.resetProject(), props.resetOrganization(), props.resetMembers()]);
};
</script>

<style lang="scss" scoped>
.org {
  display: grid;
  grid-template-columns: auto 1fr;
  gap: var(--gap-md);

  .details {
    display: flex;
    flex-direction: column;
    gap: var(--gap-sm);

    .title {
      color: var(--color-contrast);
      font-weight: 600;
      font-size: var(--font-size-lg);
    }

    .description {
      color: var(--color-secondary);
    }

    .stat-bar {
      display: flex;
      align-items: center;
      gap: var(--gap-md);
      margin-top: auto;
    }

    .stats {
      display: flex;
      align-items: center;
      gap: var(--gap-xs);

      svg {
        color: var(--color-secondary);
      }
    }
  }
}

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
      .dropdown-icon svg {
        transform: rotate(180deg);
      }
    }
    .content {
      display: flex;
    }
  }
}

.large-multiselect {
  max-width: 24rem;
}
</style>
