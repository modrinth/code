<template>
  <div>
    <section class="universal-card">
      <div class="adjacent-input">
        <label for="license-multiselect">
          <span class="label__title size-card-header">许可证</span>
          <span class="label__description">
            为您的项目选择合适的许可证非常重要。您可以从我们的列表中选择一个许可证，也可以提供自定义许可证。您还可以为所选许可证提供自定义 URL；否则，将显示许可证文本。
            <span v-if="license && license.friendly === 'Custom'" class="label__subdescription">
              请输入有效的
              <a href="https://spdx.org/licenses/" target="_blank" rel="noopener" class="text-link">
                SPDX 许可证</a
              >
              如果您的许可证没有 SPDX 标识符（例如，如果您自己创建了许可证，或者许可证是 Minecraft 专用的），只需选中该框并输入许可证的名称即可。
            </span>
<!--            <span class="label__subdescription">-->
<!--              Confused? See our-->
<!--              <a-->
<!--                href="https://blog.modrinth.com/licensing-guide/"-->
<!--                target="_blank"-->
<!--                rel="noopener"-->
<!--                class="text-link"-->
<!--              >-->
<!--                licensing guide</a-->
<!--              >-->
<!--              for more information.-->
<!--            </span>-->
          </span>
        </label>
        <div class="input-stack">
          <Multiselect
            id="license-multiselect"
            v-model="license"
            placeholder="选择一个许可..."
            track-by="short"
            label="friendly"
            :options="defaultLicenses"
            :searchable="true"
            :close-on-select="true"
            :show-labels="false"
            :class="{
              'known-error': license?.short === '' && showKnownErrors,
            }"
            :disabled="!hasPermission"
          />
          <Checkbox
            v-if="license?.requiresOnlyOrLater"
            v-model="allowOrLater"
            :disabled="!hasPermission"
            description="Allow later editions of this license"
          >
            Allow later editions of this license
          </Checkbox>
          <Checkbox
            v-if="license?.friendly === 'Custom'"
            v-model="nonSpdxLicense"
            :disabled="!hasPermission"
            description="License does not have a SPDX identifier"
          >
            许可证没有 SPDX 标识符
          </Checkbox>
          <input
            v-if="license?.friendly === 'Custom'"
            v-model="license.short"
            type="text"
            maxlength="2048"
            :placeholder="nonSpdxLicense ? 'License name' : 'SPDX identifier'"
            :class="{
              'known-error': license.short === '' && showKnownErrors,
            }"
            :disabled="!hasPermission"
          />
          <input
            v-model="licenseUrl"
            type="url"
            maxlength="2048"
            placeholder="许可证 URL（可选）"
            :disabled="!hasPermission || licenseId === 'LicenseRef-Unknown'"
          />
        </div>
      </div>
      <div class="input-stack">
        <button
          type="button"
          class="iconified-button brand-button"
          :disabled="!hasChanges || license === null"
          @click="saveChanges()"
        >
          <SaveIcon />
          保存
        </button>
      </div>
    </section>
  </div>
</template>

<script>
import Multiselect from "vue-multiselect";
import Checkbox from "~/components/ui/Checkbox";
import SaveIcon from "~/assets/images/utils/save.svg?component";

export default defineNuxtComponent({
  components: {
    Multiselect,
    Checkbox,
    SaveIcon,
  },
  props: {
    project: {
      type: Object,
      default() {
        return {};
      },
    },
    currentMember: {
      type: Object,
      default() {
        return null;
      },
    },
    patchProject: {
      type: Function,
      default() {
        return () => {
          this.$notify({
            group: "main",
            title: "发生错误",
            text: "Patch project function not found",
            type: "error",
          });
        };
      },
    },
  },
  data() {
    return {
      licenseUrl: "",
      license: { friendly: "", short: "", requiresOnlyOrLater: false },
      allowOrLater: this.project.license.id.includes("-or-later"),
      nonSpdxLicense: this.project.license.id.includes("LicenseRef-"),
      showKnownErrors: false,
    };
  },
  async setup(props) {
    const defaultLicenses = shallowRef([
      { friendly: "自定义", short: "" },
      {
        friendly: "保留所有权利/无许可",
        short: "All-Rights-Reserved",
      },
      { friendly: "Apache 许可证 2.0", short: "Apache-2.0" },
      {
        friendly: 'BSD 2-Clause "简化" 许可证',
        short: "BSD-2-Clause",
      },
      {
        friendly: 'BSD 3-Clause "新" 或 "修订" 许可证',
        short: "BSD-3-Clause",
      },
      {
        friendly: "CC Zero （相当于公共领域）",
        short: "CC0-1.0",
      },
      { friendly: "CC-BY 4.0", short: "CC-BY-4.0" },
      {
        friendly: "CC-BY-SA 4.0",
        short: "CC-BY-SA-4.0",
      },
      {
        friendly: "CC-BY-NC 4.0",
        short: "CC-BY-NC-4.0",
      },
      {
        friendly: "CC-BY-NC-SA 4.0",
        short: "CC-BY-NC-SA-4.0",
      },
      {
        friendly: "CC-BY-ND 4.0",
        short: "CC-BY-ND-4.0",
      },
      {
        friendly: "CC-BY-NC-ND 4.0",
        short: "CC-BY-NC-ND-4.0",
      },
      {
        friendly: "GNU Affero 通用公共许可证 v3",
        short: "AGPL-3.0",
        requiresOnlyOrLater: true,
      },
      {
        friendly: "GNU 较宽松通用公共许可证 v2.1",
        short: "LGPL-2.1",
        requiresOnlyOrLater: true,
      },
      {
        friendly: "GNU 较宽松通用公共许可证 v3",
        short: "LGPL-3.0",
        requiresOnlyOrLater: true,
      },
      {
        friendly: "GNU 通用公共许可证 v2",
        short: "GPL-2.0",
        requiresOnlyOrLater: true,
      },
      {
        friendly: "GNU 通用公共许可证 v3",
        short: "GPL-3.0",
        requiresOnlyOrLater: true,
      },
      { friendly: "ISC 许可证", short: "ISC" },
      { friendly: "MIT 许可证", short: "MIT" },
      { friendly: "Mozilla 公共许可证 2.0", short: "MPL-2.0" },
      { friendly: "Zlib许可证", short: "Zlib" },
    ]);

    const licenseUrl = ref(props.project.license.url);

    const licenseId = props.project.license.id;
    const trimmedLicenseId = licenseId
      .replaceAll("-only", "")
      .replaceAll("-or-later", "")
      .replaceAll("LicenseRef-", "");

    const license = ref(
      defaultLicenses.value.find((x) => x.short === trimmedLicenseId) ?? {
        friendly: "Custom",
        short: licenseId.replaceAll("LicenseRef-", ""),
      },
    );

    if (licenseId === "LicenseRef-Unknown") {
      license.value = {
        friendly: "Unknown",
        short: licenseId.replaceAll("LicenseRef-", ""),
      };
    }

    return {
      defaultLicenses,
      licenseUrl,
      license,
    };
  },
  computed: {
    hasPermission() {
      const EDIT_DETAILS = 1 << 2;
      return (this.currentMember.permissions & EDIT_DETAILS) === EDIT_DETAILS;
    },
    licenseId() {
      let id = "";
      if (this.license === null) return id;
      if (
        (this.nonSpdxLicense && this.license.friendly === "Custom") ||
        this.license.short === "All-Rights-Reserved" ||
        this.license.short === "Unknown"
      ) {
        id += "LicenseRef-";
      }
      id += this.license.short;
      if (this.license.requiresOnlyOrLater) {
        id += this.allowOrLater ? "-or-later" : "-only";
      }
      if (this.nonSpdxLicense && this.license.friendly === "Custom") {
        id = id.replaceAll(" ", "-");
      }
      return id;
    },
    patchData() {
      const data = {};

      if (this.licenseId !== this.project.license.id) {
        data.license_id = this.licenseId;
        data.license_url = this.licenseUrl ? this.licenseUrl : null;
      } else if (this.licenseUrl !== this.project.license.url) {
        data.license_url = this.licenseUrl ? this.licenseUrl : null;
      }

      return data;
    },
    hasChanges() {
      return Object.keys(this.patchData).length > 0;
    },
  },
  methods: {
    saveChanges() {
      if (this.hasChanges) {
        this.patchProject(this.patchData);
      }
    },
  },
});
</script>
