<template>
  <div>
    <ModalConfirm
      ref="modal_confirm"
      title="你确定要注销该账户吗?"
      description="这将会 **立即删除您的所有用户数据和关注**. 这不会删除您的已发布的资源. 注销后将无法恢复账户."
      proceed-label="删除此账户"
      :confirmation-text="auth.user.username"
      :has-to-type="true"
      @proceed="deleteAccount"
    />
    <Modal ref="changeEmailModal" :header="`${auth.user.email ? '修改' : '新增'} 电子邮箱`">
      <div class="universal-modal">
        <p>您的帐户信息不会公开显示</p>
        <label for="email-input"><span class="label__title">电子邮箱地址</span> </label>
        <input
          id="email-input"
          v-model="email"
          maxlength="2048"
          type="email"
          :placeholder="`输入邮箱地址...`"
          @keyup.enter="saveEmail()"
        />
        <div class="input-group push-right">
          <button class="iconified-button" @click="$refs.changeEmailModal.hide()">
            <XIcon />
            取消
          </button>
          <button
            type="button"
            class="iconified-button brand-button"
            :disabled="!email"
            @click="saveEmail()"
          >
            <SaveIcon />
            保存
          </button>
        </div>
      </div>
    </Modal>
    <Modal
      ref="managePasswordModal"
      :header="`${removePasswordMode ? '删除' : auth.user.has_password ? '修改' : '设置'}密码`"
    >
      <div class="universal-modal">
        <ul
          v-if="newPassword !== confirmNewPassword && confirmNewPassword.length > 0"
          class="known-errors"
        >
          <li>输入的密码不匹配！</li>
        </ul>
        <label v-if="removePasswordMode" for="old-password">
          <span class="label__title">确认密码</span>
          <span class="label__description">请输入您的密码以继续。</span>
        </label>
        <label v-else-if="auth.user.has_password" for="old-password">
          <span class="label__title">当前密码</span>
        </label>
        <input
          v-if="auth.user.has_password"
          id="old-password"
          v-model="oldPassword"
          maxlength="2048"
          type="password"
          autocomplete="current-password"
          :placeholder="`${removePasswordMode ? '确认' : '当前'} 密码`"
        />
        <template v-if="!removePasswordMode">
          <label for="new-password"><span class="label__title">新密码</span></label>
          <input
            id="new-password"
            v-model="newPassword"
            maxlength="2048"
            type="password"
            autocomplete="new-password"
            placeholder="新密码"
          />
          <label for="confirm-new-password"><span class="label__title">再次输入密码</span></label>
          <input
            id="confirm-new-password"
            v-model="confirmNewPassword"
            maxlength="2048"
            type="password"
            autocomplete="new-password"
            placeholder="再次输入一次新密码"
          />
        </template>
        <p></p>
        <div class="input-group push-right">
          <button class="iconified-button" @click="$refs.managePasswordModal.hide()">
            <XIcon />
            取消
          </button>
          <template v-if="removePasswordMode">
            <button
              type="button"
              class="iconified-button danger-button"
              :disabled="!oldPassword"
              @click="savePassword"
            >
              <TrashIcon />
              删除密码
            </button>
          </template>
          <template v-else>
            <button
              v-if="auth.user.has_password && auth.user.auth_providers.length > 0"
              type="button"
              class="iconified-button danger-button"
              @click="removePasswordMode = true"
            >
              <TrashIcon />
              删除密码
            </button>
            <button
              type="button"
              class="iconified-button brand-button"
              :disabled="
                newPassword.length == 0 ||
                (auth.user.has_password && oldPassword.length == 0) ||
                newPassword !== confirmNewPassword
              "
              @click="savePassword"
            >
              <SaveIcon />
              保存密码
            </button>
          </template>
        </div>
      </div>
    </Modal>
    <Modal
      ref="manageTwoFactorModal"
      :header="`${
        auth.user.has_totp && twoFactorStep === 0 ? 'Remove' : 'Setup'
      } two-factor authentication`"
    >
      <div class="universal-modal">
        <template v-if="auth.user.has_totp && twoFactorStep === 0">
          <label for="two-factor-code">
            <span class="label__title">Enter two-factor code</span>
            <span class="label__description">Please enter a two-factor code to proceed.</span>
          </label>
          <input
            id="two-factor-code"
            v-model="twoFactorCode"
            maxlength="11"
            type="text"
            placeholder="Enter code..."
            @keyup.enter="removeTwoFactor()"
          />
          <p v-if="twoFactorIncorrect" class="known-errors">The code entered is incorrect!</p>
          <div class="input-group push-right">
            <button class="iconified-button" @click="$refs.manageTwoFactorModal.hide()">
              <XIcon />
              Cancel
            </button>
            <button class="iconified-button danger-button" @click="removeTwoFactor">
              <TrashIcon />
              Remove 2FA
            </button>
          </div>
        </template>
        <template v-else>
          <template v-if="twoFactorStep === 0">
            <p>
              Two-factor authentication keeps your account secure by requiring access to a second
              device in order to sign in.
              <br /><br />
              Scan the QR code with <a href="https://authy.com/">Authy</a>,
              <a href="https://www.microsoft.com/en-us/security/mobile-authenticator-app">
                Microsoft Authenticator</a
              >, or any other 2FA app to begin.
            </p>
            <qrcode-vue
              v-if="twoFactorSecret"
              :value="`otpauth://totp/${encodeURIComponent(
                auth.user.email,
              )}?secret=${twoFactorSecret}&issuer=BBSMC`"
              :size="250"
              :margin="2"
              level="H"
            />
            <p>
              If the QR code does not scan, you can manually enter the secret:
              <strong>{{ twoFactorSecret }}</strong>
            </p>
          </template>
          <template v-if="twoFactorStep === 1">
            <label for="verify-code">
              <span class="label__title">Verify code</span>
              <span class="label__description"
                >Enter the one-time code from authenticator to verify access.
              </span>
            </label>
            <input
              id="verify-code"
              v-model="twoFactorCode"
              maxlength="6"
              type="text"
              autocomplete="one-time-code"
              placeholder="Enter code..."
              @keyup.enter="verifyTwoFactorCode()"
            />
            <p v-if="twoFactorIncorrect" class="known-errors">The code entered is incorrect!</p>
          </template>
          <template v-if="twoFactorStep === 2">
            <p>
              Download and save these back-up codes in a safe place. You can use these in-place of a
              2FA code if you ever lose access to your device! You should protect these codes like
              your password.
            </p>
            <p>Backup codes can only be used once.</p>
            <ul>
              <li v-for="code in backupCodes" :key="code">{{ code }}</li>
            </ul>
          </template>
          <div class="input-group push-right">
            <button v-if="twoFactorStep === 1" class="iconified-button" @click="twoFactorStep = 0">
              <LeftArrowIcon />
              Back
            </button>
            <button
              v-if="twoFactorStep !== 2"
              class="iconified-button"
              @click="$refs.manageTwoFactorModal.hide()"
            >
              <XIcon />
              Cancel
            </button>
            <button
              v-if="twoFactorStep <= 1"
              class="iconified-button brand-button"
              @click="twoFactorStep === 1 ? verifyTwoFactorCode() : (twoFactorStep = 1)"
            >
              <RightArrowIcon />
              Continue
            </button>
            <button
              v-if="twoFactorStep === 2"
              class="iconified-button brand-button"
              @click="$refs.manageTwoFactorModal.hide()"
            >
              <CheckIcon />
              Complete setup
            </button>
          </div>
        </template>
      </div>
    </Modal>
    <Modal ref="manageProvidersModal" header="Authentication providers">
      <div class="universal-modal">
        <div class="table">
          <div class="table-head table-row">
            <div class="table-text table-cell">Provider</div>
            <div class="table-text table-cell">Actions</div>
          </div>
          <div v-for="provider in authProviders" :key="provider.id" class="table-row">
            <div class="table-text table-cell">
              <span><component :is="provider.icon" /> {{ provider.display }}</span>
            </div>
            <div class="table-text manage table-cell">
              <button
                v-if="auth.user.auth_providers.includes(provider.id)"
                class="btn"
                @click="removeAuthProvider(provider.id)"
              >
                <TrashIcon /> Remove
              </button>
              <a
                v-else
                class="btn"
                :href="`${getAuthUrl(provider.id, '/settings/account')}&token=${auth.token}`"
              >
                <ExternalIcon /> Add
              </a>
            </div>
          </div>
        </div>
        <p></p>
        <div class="input-group push-right">
          <button class="iconified-button" @click="$refs.manageProvidersModal.hide()">
            <XIcon />
            Close
          </button>
        </div>
      </div>
    </Modal>
    <section class="universal-card">
      <h2 class="text-2xl">账号安全</h2>

      <div class="adjacent-input">
        <label for="theme-selector">
          <span class="label__title">邮箱</span>
          <span class="label__description">更改与您的帐户关联的电子邮件</span>
        </label>
        <div>
          <button class="iconified-button" @click="$refs.changeEmailModal.show()">
            <template v-if="auth.user.email">
              <EditIcon />
              更改电子邮箱
            </template>
            <template v-else>
              <PlusIcon />
              设置邮箱
            </template>
          </button>
        </div>
      </div>
      <div class="adjacent-input">
        <label for="theme-selector">
          <span class="label__title">密码</span>
          <span v-if="auth.user.has_password" class="label__description">
            更改<template v-if="auth.user.auth_providers.length > 0">或删除</template
            >您账户的登录密码
          </span>
          <span v-else class="label__description"> 设置密码来登录您的帐户。 </span>
        </label>
        <div>
          <button
            class="iconified-button"
            @click="
              () => {
                oldPassword = '';
                newPassword = '';
                confirmNewPassword = '';
                removePasswordMode = false;
                $refs.managePasswordModal.show();
              }
            "
          >
            <KeyIcon />
            <template v-if="auth.user.has_password"> 修改密码 </template>
            <template v-else> 设置密码 </template>
          </button>
        </div>
      </div>
      <!--      <div class="adjacent-input">-->
      <!--        <label for="theme-selector">-->
      <!--          <span class="label__title">Two-factor authentication</span>-->
      <!--          <span class="label__description">-->
      <!--            Add an additional layer of security to your account during login.-->
      <!--          </span>-->
      <!--        </label>-->
      <!--        <div>-->
      <!--          <button class="iconified-button" @click="showTwoFactorModal">-->
      <!--            <template v-if="auth.user.has_totp"> <TrashIcon /> Remove 2FA </template>-->
      <!--            <template v-else> <PlusIcon /> Setup 2FA </template>-->
      <!--          </button>-->
      <!--        </div>-->
      <!--      </div>-->
      <!--      <div class="adjacent-input">-->
      <!--        <label for="theme-selector">-->
      <!--          <span class="label__title">Manage authentication providers</span>-->
      <!--          <span class="label__description">-->
      <!--            Add or remove sign-on methods from your account, including GitHub, GitLab, Microsoft,-->
      <!--            Discord, Steam, and Google.-->
      <!--          </span>-->
      <!--        </label>-->
      <!--        <div>-->
      <!--          <button class="iconified-button" @click="$refs.manageProvidersModal.show()">-->
      <!--            <SettingsIcon /> Manage providers-->
      <!--          </button>-->
      <!--        </div>-->
      <!--      </div>-->
    </section>

    <!--    <section id="data-export" class="universal-card">-->
    <!--      <h2>Data export</h2>-->
    <!--      <p>-->
    <!--        Request a copy of all your personal data you have uploaded to Modrinth. This may take-->
    <!--        several minutes to complete.-->
    <!--      </p>-->
    <!--      <a v-if="generated" class="iconified-button" :href="generated" download="export.json">-->
    <!--        <DownloadIcon />-->
    <!--        Download export-->
    <!--      </a>-->
    <!--      <button v-else class="iconified-button" :disabled="generatingExport" @click="exportData">-->
    <!--        <template v-if="generatingExport"> <UpdatedIcon /> Generating export... </template>-->
    <!--        <template v-else> <UpdatedIcon /> Generate export </template>-->
    <!--      </button>-->
    <!--    </section>-->

    <section id="delete-account" class="universal-card">
      <h2>注销账户</h2>
      <p>
        一旦注销帐户，将无法恢复。注销帐户将从我们的服务器中删除所有附加数据（已发布的资源除外）。
      </p>
      <button
        type="button"
        class="iconified-button danger-button"
        @click="$refs.modal_confirm.show()"
      >
        <TrashIcon />
        注销账户
      </button>
    </section>
  </div>
</template>

<script setup>
import {
  CheckIcon,
  EditIcon,
  ExternalIcon,
  LeftArrowIcon,
  PlusIcon,
  RightArrowIcon,
  SaveIcon,
  TrashIcon,
  XIcon,
} from "@modrinth/assets";
import QrcodeVue from "qrcode.vue";
import GitHubIcon from "assets/icons/auth/sso-github.svg";
import MicrosoftIcon from "assets/icons/auth/sso-microsoft.svg";
import GoogleIcon from "assets/icons/auth/sso-google.svg";
import SteamIcon from "assets/icons/auth/sso-steam.svg";
import DiscordIcon from "assets/icons/auth/sso-discord.svg";
import KeyIcon from "assets/icons/auth/key.svg";
import GitLabIcon from "assets/icons/auth/sso-gitlab.svg";
import ModalConfirm from "~/components/ui/ModalConfirm.vue";
import Modal from "~/components/ui/Modal.vue";

useHead({
  title: "Account settings - Modrinth",
});

definePageMeta({
  middleware: "auth",
});

const data = useNuxtApp();
const auth = await useAuth();

const changeEmailModal = ref();
const email = ref(auth.value.user.email);
async function saveEmail() {
  if (!email.value) {
    return;
  }

  startLoading();
  try {
    await useBaseFetch(`auth/email`, {
      method: "PATCH",
      body: {
        email: email.value,
      },
    });
    changeEmailModal.value.hide();
    await useAuth(auth.value.token);
  } catch (err) {
    data.$notify({
      group: "main",
      title: "发生错误",
      text: err.data.description,
      type: "error",
    });
  }
  stopLoading();
}

const managePasswordModal = ref();
const removePasswordMode = ref(false);
const oldPassword = ref("");
const newPassword = ref("");
const confirmNewPassword = ref("");
async function savePassword() {
  if (newPassword.value !== confirmNewPassword.value) {
    return;
  }

  startLoading();
  try {
    await useBaseFetch(`auth/password`, {
      method: "PATCH",
      body: {
        old_password: auth.value.user.has_password ? oldPassword.value : null,
        new_password: removePasswordMode.value ? null : newPassword.value,
      },
    });
    managePasswordModal.value.hide();
    await useAuth(auth.value.token);
  } catch (err) {
    data.$notify({
      group: "main",
      title: "发生错误",
      text: err.data.description,
      type: "error",
    });
  }
  stopLoading();
}

const manageTwoFactorModal = ref();
const twoFactorSecret = ref(null);
const twoFactorFlow = ref(null);
const twoFactorStep = ref(0);
const twoFactorIncorrect = ref(false);
const twoFactorCode = ref(null);
const backupCodes = ref([]);
async function verifyTwoFactorCode() {
  startLoading();
  try {
    const res = await useBaseFetch("auth/2fa", {
      method: "POST",
      body: {
        code: twoFactorCode.value ? twoFactorCode.value : "",
        flow: twoFactorFlow.value,
      },
    });

    backupCodes.value = res.backup_codes;
    twoFactorStep.value = 2;
    await useAuth(auth.value.token);
  } catch {
    twoFactorIncorrect.value = true;
  }
  stopLoading();
}

async function removeTwoFactor() {
  startLoading();
  try {
    await useBaseFetch("auth/2fa", {
      method: "DELETE",
      body: {
        code: twoFactorCode.value ? twoFactorCode.value.toString() : "",
      },
    });
    manageTwoFactorModal.value.hide();
    await useAuth(auth.value.token);
  } catch {
    twoFactorIncorrect.value = true;
  }
  stopLoading();
}

const authProviders = [
  {
    id: "github",
    display: "GitHub",
    icon: GitHubIcon,
  },
  {
    id: "gitlab",
    display: "GitLab",
    icon: GitLabIcon,
  },
  {
    id: "steam",
    display: "Steam",
    icon: SteamIcon,
  },
  {
    id: "discord",
    display: "Discord",
    icon: DiscordIcon,
  },
  {
    id: "microsoft",
    display: "Microsoft",
    icon: MicrosoftIcon,
  },
  {
    id: "google",
    display: "Google",
    icon: GoogleIcon,
  },
];

async function deleteAccount() {
  startLoading();
  try {
    await useBaseFetch(`user/${auth.value.user.id}`, {
      method: "DELETE",
    });
  } catch (err) {
    data.$notify({
      group: "main",
      title: "发生错误",
      text: err.data.description,
      type: "error",
    });
  }

  useCookie("auth-token").value = null;
  window.location.href = "/";

  stopLoading();
}
</script>
<style lang="scss" scoped>
canvas {
  margin: 0 auto;
  border-radius: var(--size-rounded-card);
}

.table-row {
  grid-template-columns: 1fr 10rem;

  span {
    display: flex;
    align-items: center;
    margin: auto 0;

    svg {
      width: 1.25rem;
      height: 1.25rem;
      margin-right: 0.35rem;
    }
  }
}
</style>
