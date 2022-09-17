<template>
  <div class="page-container">
    <div class="page-contents">
      <header class="card">
        <div class="columns">
          <h3 class="column-grow-1">Create a project</h3>
          <button
            title="Save draft"
            class="iconified-button column"
            :disabled="!$nuxt.$loading"
            @click="createDraft"
          >
            <SaveIcon />
            Save as a draft
          </button>
          <button
            title="Submit for review"
            class="iconified-button brand-button-colors column"
            :disabled="!$nuxt.$loading"
            @click="createProjectForReview"
          >
            <CheckIcon />
            Submit for review
          </button>
        </div>
        <div v-if="showKnownErrors" class="known-errors">
          <ul>
            <li v-if="name === ''">Your project must have a name.</li>
            <li v-if="description === ''">Your project must have a summary.</li>
            <li v-if="slug === ''">Your project must have a vanity URL.</li>
            <li v-if="!savingAsDraft && body === ''">
              Your project must have a body.
            </li>
            <li v-if="!savingAsDraft && versions.length < 1">
              Your project must have at least one version.
            </li>
            <li
              v-if="
                license === null || license_url === null || license_url === ''
              "
            >
              Your project must have a license.
            </li>
            <li v-if="currentVersionIndex !== -1">
              You must finish editing your versions
            </li>
            <li v-if="currentGalleryIndex !== -1">
              You must finish editing your gallery images
            </li>
          </ul>
        </div>
      </header>
      <section class="card essentials">
        <label>
          <span>
            <h3>Type<span class="required">*</span></h3>
            <span class="no-padding">The type of project your project is.</span>
          </span>
          <Multiselect
            v-model="projectType"
            placeholder="Select one"
            label="display"
            :options="projectTypes"
            :searchable="false"
            :close-on-select="true"
            :show-labels="false"
            :allow-empty="false"
            @input="setCategories(true)"
          />
        </label>
        <label>
          <span>
            <h3>Name<span class="required">*</span></h3>
            <span>
              Be creative! Generic project names will be harder to search for.
            </span>
          </span>
          <input
            v-model="name"
            :class="{ 'known-error': name === '' && showKnownErrors }"
            type="text"
            placeholder="Enter the name"
          />
        </label>
        <label>
          <span>
            <h3>Summary<span class="required">*</span></h3>
            <span>
              Give a short description of your project that will appear on
              search pages.
            </span>
          </span>
          <input
            v-model="description"
            :class="{ 'known-error': description === '' && showKnownErrors }"
            type="text"
            placeholder="Enter the summary"
          />
        </label>
        <label>
          <span>
            <h3>Categories</h3>
            <span class="no-padding">
              Select up to 3 categories that will help others <br />
              find your project.
            </span>
          </span>
          <multiselect
            id="categories"
            v-model="categories"
            :options="selectableCategories"
            :custom-label="
              (value) => value.charAt(0).toUpperCase() + value.slice(1)
            "
            :loading="$tag.categories.length === 0"
            :multiple="true"
            :searchable="true"
            :show-no-results="false"
            :close-on-select="false"
            :clear-on-select="true"
            :show-labels="false"
            :max="3"
            :limit="6"
            :hide-selected="true"
            placeholder="Choose categories"
            @input="setCategories(false)"
          />
        </label>
        <label>
          <span>
            <h3>Additional Categories</h3>
            <span class="no-padding">
              Select more categories that will help others <br />
              find your project. These are searchable, but not <br />
              displayed in search.
            </span>
          </span>
          <multiselect
            id="additional_categories"
            v-model="additional_categories"
            :show-no-results="false"
            :options="selectableAdditionalCategories"
            :custom-label="
              (value) => value.charAt(0).toUpperCase() + value.slice(1)
            "
            :loading="$tag.categories.length === 0"
            :multiple="true"
            :searchable="true"
            :close-on-select="false"
            :clear-on-select="true"
            :show-labels="false"
            :max="255"
            :limit="6"
            :hide-selected="true"
            placeholder="Choose additional categories"
            @input="setCategories(false)"
          />
        </label>
        <label>
          <span>
            <h3>Vanity URL (slug)<span class="required">*</span></h3>
            <span class="slug-description"
              >https://modrinth.com/{{ projectType.id }}/{{
                slug ? slug : 'your-slug'
              }}
            </span>
          </span>
          <input
            id="name"
            v-model="slug"
            :class="{ 'known-error': slug === '' && showKnownErrors }"
            type="text"
            placeholder="Enter the vanity URL"
          />
        </label>
      </section>
      <section class="card project-icon">
        <h3>Icon</h3>
        <img
          :src="
            previewImage
              ? previewImage
              : 'https://cdn.modrinth.com/placeholder.svg'
          "
          alt="preview-image"
        />
        <SmartFileInput
          :show-icon="false"
          :max-size="262144"
          accept="image/png,image/jpeg,image/gif,image/webp"
          class="choose-image"
          prompt="Choose image or drag it here"
          @change="showPreviewImage"
        />
        <button
          class="iconified-button"
          @click="
            icon = null
            previewImage = null
          "
        >
          <TrashIcon />
          Reset
        </button>
      </section>
      <section
        v-if="
          projectType.realId !== 'resourcepack' &&
          projectType.realId !== 'plugin'
        "
        class="card game-sides"
      >
        <div class="columns">
          <div class="column">
            <h3>Supported environments</h3>
            <span>
              Let others know what environments your project supports.
            </span>
          </div>
          <div class="labeled-control">
            <h3>Client<span class="required">*</span></h3>
            <Multiselect
              v-model="clientSideType"
              placeholder="Select one"
              :options="sideTypes"
              :searchable="false"
              :close-on-select="true"
              :show-labels="false"
              :allow-empty="false"
            />
          </div>
          <div class="labeled-control">
            <h3>Server<span class="required">*</span></h3>
            <Multiselect
              v-model="serverSideType"
              placeholder="Select one"
              :options="sideTypes"
              :searchable="false"
              :close-on-select="true"
              :show-labels="false"
              :allow-empty="false"
            />
          </div>
        </div>
      </section>
      <section class="card description">
        <h3>
          <label
            for="body"
            title="You can type an extended description of your project here."
          >
            Description<span class="required">*</span>
          </label>
        </h3>
        <span>
          You can type an extended description of your mod here. This editor
          supports
          <a
            class="text-link"
            href="https://guides.github.com/features/mastering-markdown/"
            target="_blank"
            rel="noopener noreferrer"
            >Markdown</a
          >. HTML can also be used inside your description, not including
          styles, scripts, and iframes (though YouTube iframes are allowed).
        </span>
        <ThisOrThat
          v-model="bodyViewMode"
          class="separator"
          :items="['source', 'preview']"
        />
        <div class="edit-wrapper">
          <div v-if="bodyViewMode === 'source'" class="textarea-wrapper">
            <textarea
              id="body"
              v-model="body"
              :class="{ 'known-error': body === '' && showKnownErrors }"
            />
          </div>
          <div
            v-if="bodyViewMode === 'preview'"
            v-highlightjs
            class="markdown-body"
            v-html="body ? $xss($md.render(body)) : 'No body specified.'"
          ></div>
        </div>
      </section>
      <section class="card versions">
        <div class="title">
          <h3>Versions<span class="required">*</span></h3>
          <button
            title="Add a version"
            class="iconified-button"
            :disabled="currentVersionIndex !== -1"
            @click="createVersion"
          >
            <PlusIcon />
            Add a version
          </button>
        </div>
        <div v-if="versions.length === 0" class="no-versions-warning">
          At least 1 version is required to submit for review.
        </div>
        <div
          v-if="currentVersionIndex !== -1"
          class="new-version outlined-area"
        >
          <div class="header">
            <h3 v-if="currentVersionNew" class="title">Add a version</h3>
            <h3 v-else class="title">Edit version</h3>
            <div class="controls">
              <button
                v-if="currentVersionNew"
                class="iconified-button"
                title="Cancel adding version"
                @click="deleteVersion()"
              >
                <CrossIcon />
                Cancel
              </button>
              <button
                v-else
                class="iconified-button"
                title="Remove version"
                @click="deleteVersion()"
              >
                <TrashIcon />
                Remove
              </button>
              <button
                v-if="currentVersionNew"
                class="brand-button-colors iconified-button"
                title="Save version"
                @click="saveVersion()"
              >
                <CheckIcon />
                Add
              </button>
              <button
                v-else
                class="brand-button-colors iconified-button"
                title="Save version"
                @click="saveVersion()"
              >
                <SaveIcon />
                Save
              </button>
            </div>
          </div>
          <div v-if="showKnownVersionErrors" class="known-errors">
            <ul>
              <li v-if="versions[currentVersionIndex].version_number === ''">
                Your version must have a unique version number.
              </li>
              <li v-if="versions[currentVersionIndex].loaders.length < 1">
                Your version must have the supported loaders selected.
              </li>
              <li v-if="versions[currentVersionIndex].game_versions.length < 1">
                Your version must have the supported Minecraft versions
                selected.
              </li>
              <li v-if="versions[currentVersionIndex].files.length < 1">
                Your version must have a file uploaded.
              </li>
            </ul>
          </div>
          <div class="main">
            <label>
              <span>
                <h3>Title</h3>
                <span>
                  An optional nicely formatted name for this version.
                </span>
              </span>
              <input
                v-model="versions[currentVersionIndex].version_title"
                type="text"
                placeholder="Enter the title of this version"
              />
            </label>
            <label>
              <span>
                <h3>Version number<span class="required">*</span></h3>
                <span>
                  A unique identifier for versions of your project, used for
                  display and in URLs.
                </span>
              </span>
              <input
                v-model="versions[currentVersionIndex].version_number"
                :class="{
                  'known-error':
                    versions[currentVersionIndex].version_number === '' &&
                    showKnownVersionErrors,
                }"
                type="text"
                placeholder="Enter the version number"
              />
            </label>
            <label>
              <span>
                <h3>Release channel<span class="required">*</span></h3>
                <span>
                  Used to indicate to users the stability of this release.
                </span>
              </span>
              <multiselect
                v-model="versions[currentVersionIndex].release_channel"
                :class="{
                  'known-error':
                    versions[currentVersionIndex].release_channel === null &&
                    showKnownVersionErrors,
                }"
                placeholder="Select one"
                :options="['release', 'beta', 'alpha']"
                :custom-label="
                  (value) => value.charAt(0).toUpperCase() + value.slice(1)
                "
                :searchable="false"
                :close-on-select="true"
                :show-labels="false"
                :allow-empty="false"
              />
            </label>
            <label v-if="projectType.realId !== 'resourcepack'">
              <span>
                <h3>Loaders<span class="required">*</span></h3>
                <span> Select all loaders this version supports. </span>
              </span>
              <multiselect
                v-model="versions[currentVersionIndex].loaders"
                :class="{
                  'known-error':
                    versions[currentVersionIndex].loaders.length < 1 &&
                    showKnownVersionErrors,
                }"
                :options="
                  $tag.loaders
                    .filter((x) => {
                      if (projectType.realId === 'plugin') {
                        return $tag.loaderData.allPluginLoaders.includes(x.name)
                      } else if (projectType.realId === 'mod') {
                        return $tag.loaderData.modLoaders.includes(x.name)
                      }

                      return x.supported_project_types.includes(projectType.id)
                    })
                    .map((it) => it.name)
                "
                :custom-label="(value) => $formatCategory(value)"
                :loading="$tag.loaders.length === 0"
                :multiple="true"
                :searchable="false"
                :show-no-results="false"
                :close-on-select="false"
                :clear-on-select="false"
                :show-labels="false"
                :limit="6"
                :hide-selected="true"
                placeholder="Choose loaders..."
              />
            </label>
            <label>
              <span>
                <h3>Minecraft versions<span class="required">*</span></h3>
                <span>
                  Select all versions of Minecraft this version supports.
                </span>
              </span>
              <multiselect
                v-model="versions[currentVersionIndex].game_versions"
                :class="{
                  'known-error':
                    versions[currentVersionIndex].game_versions.length < 1 &&
                    showKnownVersionErrors,
                }"
                :options="$tag.gameVersions.map((it) => it.version)"
                :loading="$tag.gameVersions.length === 0"
                :multiple="true"
                :searchable="true"
                :show-no-results="false"
                :close-on-select="false"
                :clear-on-select="false"
                :show-labels="false"
                :limit="6"
                :hide-selected="true"
                placeholder="Choose versions..."
              />
            </label>
          </div>
          <div
            v-if="projectType.id.toLowerCase() !== 'modpack'"
            class="dependencies"
          >
            <h3>Dependencies</h3>
            <div class="dependency-selector">
              <ThisOrThat
                v-model="dependencyAddMode"
                class="separator"
                :items="['project', 'version']"
              />
              <div class="edit-info">
                <input
                  v-model="newDependencyId"
                  type="text"
                  :placeholder="`Enter the ${dependencyAddMode} ID${
                    dependencyAddMode === 'project' ? '/slug' : ''
                  }`"
                  @keyup.enter="addDependency"
                />
                <Multiselect
                  v-model="newDependencyType"
                  class="input"
                  placeholder="Select one"
                  :options="[
                    'required',
                    'optional',
                    'incompatible',
                    'embedded',
                  ]"
                  :custom-label="
                    (value) => value.charAt(0).toUpperCase() + value.slice(1)
                  "
                  :searchable="false"
                  :close-on-select="true"
                  :show-labels="false"
                  :allow-empty="false"
                />
                <button class="iconified-button" @click="addDependency">
                  <PlusIcon />
                  Add
                </button>
              </div>
            </div>
            <div class="new-dependencies">
              <div
                v-for="(dependency, index) in versions[currentVersionIndex]
                  .dependencies"
                :key="index"
                class="dependency"
              >
                <img
                  class="icon"
                  :src="
                    dependency.project
                      ? dependency.project.icon_url
                        ? dependency.project.icon_url
                        : 'https://cdn.modrinth.com/placeholder.svg?inline'
                      : 'https://cdn.modrinth.com/placeholder.svg?inline'
                  "
                  alt="dependency-icon"
                />
                <div class="info">
                  <h4 class="title">
                    {{
                      dependency.project
                        ? dependency.project.title
                        : 'Unknown Project'
                    }}
                  </h4>
                  <p v-if="dependency.version" class="version-number">
                    Version {{ dependency.version.version_number }} is
                    {{ dependency.dependency_type }}
                  </p>
                  <div class="bottom">
                    <button
                      class="iconified-button"
                      @click="
                        versions[currentVersionIndex].dependencies.splice(
                          index,
                          1
                        )
                      "
                    >
                      <TrashIcon />
                      Remove
                    </button>
                  </div>
                </div>
              </div>
            </div>
          </div>
          <div class="files">
            <h3>Files<span class="required">*</span></h3>
            <span>
              You may upload multiple files, but this should only be used for
              cases like sources or Javadocs for mods/plugins.
            </span>
            <p v-if="projectType.id === 'modpack'" aria-label="Warning">
              Modpack support is currently in alpha, and you may encounter
              issues. Our documentation includes instructions on
              <a
                href="https://docs.modrinth.com/docs/modpacks/creating_modpacks/"
                target="_blank"
                class="text-link"
                >creating modpacks</a
              >. Join us on
              <a
                href="https://discord.gg/EUHuJHt"
                target="_blank"
                class="text-link"
                >Discord</a
              >
              for support.
            </p>
            <StatelessFileInput
              :class="{
                'known-error':
                  versions[currentVersionIndex].files.length < 1 &&
                  showKnownVersionErrors,
              }"
              class="file-input"
              multiple
              :accept="
                projectType.id === 'modpack'
                  ? '.mrpack,application/x-modrinth-modpack+zip'
                  : projectType.id === 'mod'
                  ? '.jar,application/java-archive'
                  : '*'
              "
              prompt="Choose files or drag them here"
              :max-size="524288000"
              @change="
                (x) =>
                  x.forEach((y) => versions[currentVersionIndex].files.push(y))
              "
            />
            <div class="uploaded-files">
              <div
                v-for="(file, index) in versions[currentVersionIndex].files"
                :key="index + 'new'"
                class="file"
              >
                <p class="filename">{{ file.name }}</p>
                <button
                  class="action iconified-button"
                  @click="versions[currentVersionIndex].files.splice(index, 1)"
                >
                  <TrashIcon aria-hidden="true" />
                  Remove
                </button>
              </div>
            </div>
          </div>
          <div class="changelog">
            <h3>Changes</h3>
            <span>
              Let everyone know what's changed since the last version. This
              editor supports
              <a
                class="text-link"
                href="https://guides.github.com/features/mastering-markdown/"
                target="_blank"
                rel="noopener noreferrer"
                >Markdown</a
              >.
            </span>
            <ThisOrThat
              v-model="changelogViewMode"
              class="separator"
              :items="['source', 'preview']"
            />
            <div class="edit-wrapper">
              <div
                v-if="changelogViewMode === 'source'"
                class="textarea-wrapper"
              >
                <textarea
                  id="body"
                  v-model="versions[currentVersionIndex].version_body"
                />
              </div>
              <div
                v-if="changelogViewMode === 'preview'"
                v-highlightjs
                class="markdown-body"
                v-html="
                  versions[currentVersionIndex].version_body
                    ? $xss(
                        $md.render(versions[currentVersionIndex].version_body)
                      )
                    : 'No changelog specified.'
                "
              ></div>
            </div>
          </div>
        </div>
        <table>
          <thead>
            <tr
              v-if="
                (versions.length > 0 && currentVersionIndex === -1) ||
                versions.length > 1
              "
            >
              <th>Name</th>
              <th>Version</th>
              <th>Project loaders</th>
              <th>Minecraft versions</th>
              <th>Release channel</th>
              <th>Actions</th>
            </tr>
          </thead>
          <tbody>
            <tr
              v-for="(version, index) in versions.filter((it) =>
                currentVersionIndex !== -1
                  ? it !== versions[currentVersionIndex]
                  : true
              )"
              :key="version.id"
            >
              <td>{{ version.version_title }}</td>
              <td>{{ version.version_number }}</td>
              <td>
                {{
                  version.loaders
                    .map((x) => x.charAt(0).toUpperCase() + x.slice(1))
                    .join(', ')
                }}
              </td>
              <td>
                {{
                  version.game_versions
                    .map((x) => x.charAt(0).toUpperCase() + x.slice(1))
                    .join(', ')
                }}
              </td>
              <td>
                <VersionBadge
                  v-if="version.release_channel === 'release'"
                  type="release"
                  color="green"
                />
                <VersionBadge
                  v-else-if="version.release_channel === 'beta'"
                  type="beta"
                  color="yellow"
                />
                <VersionBadge
                  v-else-if="version.release_channel === 'alpha'"
                  type="alpha"
                  color="red"
                />
              </td>
              <td>
                <button
                  class="iconified-button"
                  title="Remove version"
                  @click="versions.splice(index, 1)"
                >
                  <TrashIcon />
                  Remove
                </button>
                <button
                  class="iconified-button"
                  title="Edit version"
                  @click="editVersion(index)"
                >
                  <EditIcon />
                  Edit
                </button>
              </td>
            </tr>
          </tbody>
        </table>
      </section>
      <section class="card gallery">
        <div class="title">
          <div class="text">
            <h3>Gallery images</h3>
            <i>— this section is optional</i>
          </div>
          <button
            title="Add an image"
            class="iconified-button"
            :disabled="currentGalleryIndex !== -1"
            @click="createGalleryImage()"
          >
            <PlusIcon />
            Add an image
          </button>
        </div>
        <div
          v-if="currentGalleryIndex !== -1"
          class="new-gallery-item outlined-area"
        >
          <div class="header">
            <h3 class="title">Add an image</h3>
            <div class="controls">
              <button
                v-if="currentGalleryNew"
                class="iconified-button"
                title="Cancel adding image"
                @click="deleteGalleryImage()"
              >
                <CrossIcon />
                Cancel
              </button>
              <button
                v-else
                class="iconified-button"
                title="Remove image"
                @click="deleteGalleryImage()"
              >
                <TrashIcon />
                Remove
              </button>
              <button
                v-if="currentGalleryNew"
                class="brand-button-colors iconified-button"
                title="Add image"
                @click="saveGalleryImage()"
              >
                <CheckIcon />
                Add
              </button>
              <button
                v-else
                class="brand-button-colors iconified-button"
                title="Save image"
                @click="saveGalleryImage()"
              >
                <SaveIcon />
                Save changes
              </button>
            </div>
          </div>
          <div v-if="showKnownGalleryErrors" class="known-errors">
            <ul>
              <li v-if="gallery[currentGalleryIndex].title === ''">
                Your image must have a title.
              </li>
              <li v-if="gallery[currentGalleryIndex].preview === null">
                Your image must have a file uploaded.
              </li>
            </ul>
          </div>
          <div class="info">
            <label title="The title of the gallery item">
              <span>Title</span>
              <input
                v-model="gallery[currentGalleryIndex].title"
                :class="{
                  'known-error':
                    gallery[currentGalleryIndex].title === '' &&
                    showKnownGalleryErrors,
                }"
                type="text"
                placeholder="Enter a title"
              />
            </label>
            <label
              title="The description of what is being featured in the link."
            >
              <span>Description (optional)</span>
              <input
                v-model="gallery[currentGalleryIndex].description"
                type="text"
                placeholder="Enter a description"
              />
            </label>
          </div>
          <div class="image">
            <img
              :src="
                gallery[currentGalleryIndex].preview
                  ? gallery[currentGalleryIndex].preview
                  : 'https://cdn.modrinth.com/placeholder-banner.svg'
              "
              alt="preview-image"
            />
            <div class="bottom">
              <SmartFileInput
                :max-size="5242880"
                accept="image/png,image/jpeg,image/gif,image/webp,.png,.jpeg,.gif,.webp"
                prompt="Upload"
                :class="{
                  'known-error':
                    !gallery[currentGalleryIndex].preview &&
                    showKnownGalleryErrors,
                }"
                @change="
                  (files) => showGalleryPreviewImage(files, currentGalleryIndex)
                "
              />
              <div>
                <button
                  class="iconified-button"
                  @click="
                    gallery[currentGalleryIndex].icon = null
                    gallery[currentGalleryIndex].preview = null
                    $forceUpdate()
                  "
                >
                  <TrashIcon />
                  Reset
                </button>
              </div>
            </div>
          </div>
        </div>
        <div class="gallery-items">
          <div
            v-for="(item, index) in gallery.filter((it) =>
              currentGalleryIndex === -1 || it !== gallery[currentGalleryIndex]
                ? it !== gallery[currentGalleryIndex]
                : true
            )"
            :key="index"
            class="gallery-item"
          >
            <div class="info">
              <h3>{{ item.title }}</h3>
              <h4 v-if="item.description">{{ item.description }}</h4>
            </div>
            <div class="image">
              <img
                :src="
                  item.preview
                    ? item.preview
                    : 'https://cdn.modrinth.com/placeholder-banner.svg'
                "
                alt="preview-image"
              />
            </div>
            <div class="controls">
              <button
                class="iconified-button"
                title="Edit gallery image"
                @click="editGalleryImage(index)"
              >
                <EditIcon />
                Edit
              </button>
              <button
                class="iconified-button"
                title="Remove gallery image"
                @click="gallery.splice(index, 1)"
              >
                <TrashIcon />
                Remove
              </button>
            </div>
          </div>
        </div>
      </section>
      <section class="card extra-links">
        <div class="title">
          <div class="text">
            <h3>External links</h3>
            <i>— this section is optional</i>
          </div>
        </div>
        <label
          title="A place for users to report bugs, issues, and concerns about your project."
        >
          <span>Issue tracker</span>
          <input
            v-model="issues_url"
            type="url"
            placeholder="Enter a valid URL"
          />
        </label>
        <label
          title="A page/repository containing the source code for your project"
        >
          <span>Source code</span>
          <input
            v-model="source_url"
            type="url"
            placeholder="Enter a valid URL"
          />
        </label>
        <label
          title="A page containing information, documentation, and help for the project."
        >
          <span>Wiki page</span>
          <input
            v-model="wiki_url"
            type="url"
            placeholder="Enter a valid URL"
          />
        </label>
        <label title="An invitation link to your Discord server.">
          <span>Discord invite</span>
          <input
            v-model="discord_url"
            type="url"
            placeholder="Enter a valid URL"
          />
        </label>
      </section>
      <section class="card license">
        <div class="title">
          <div class="text">
            <h3>License<span class="required">*</span></h3>
          </div>
        </div>
        <div class="row">
          <label for="license">
            <span>
              It is very important to choose a proper license for your mod. You
              may choose one from our list or provide a URL to a custom license.
              <br />
              Confused? See our
              <a
                class="text-link"
                href="https://blog.modrinth.com/licensing-guide/"
                target="_blank"
                rel="noopener noreferrer"
              >
                licensing guide</a
              >
              for more information.
            </span>
          </label>
          <div class="input-group">
            <Multiselect
              id="license"
              v-model="license"
              :class="{ 'known-error': license === null && showKnownErrors }"
              :custom-label="(value) => value.short.toUpperCase()"
              placeholder="Choose license..."
              :loading="$tag.licenses.length === 0"
              :options="$tag.licenses"
              track-by="short"
              :multiple="false"
              :searchable="true"
              :close-on-select="true"
              :show-labels="false"
              :allow-empty="false"
            />
            <input
              v-model="license_url"
              aria-label="License URL"
              type="url"
              placeholder="License URL"
              :class="{
                'known-error':
                  (license_url === null || license_url === '') &&
                  showKnownErrors,
              }"
            />
          </div>
        </div>
      </section>
      <section class="card donations">
        <div class="title">
          <div class="text">
            <h3>Donation links</h3>
            <i>— this section is optional</i>
          </div>
          <button
            title="Add a link"
            class="iconified-button"
            :disabled="false"
            @click="
              donationPlatforms.push({})
              donationLinks.push('')
            "
          >
            <PlusIcon />
            Add a link
          </button>
        </div>
        <div
          v-for="(item, index) in donationPlatforms"
          :key="index"
          class="outlined-area"
        >
          <div class="header">
            <h3 class="title">Link</h3>
            <div class="controls">
              <button
                class="iconified-button"
                @click="
                  donationPlatforms.splice(index, 1)
                  donationLinks.splice(index, 1)
                "
              >
                <TrashIcon />
                Remove link
              </button>
            </div>
          </div>
          <label>
            <span class="no-padding"
              >Donation link<span class="required">*</span></span
            >
            <input
              v-model="donationLinks[index]"
              type="url"
              placeholder="Enter a valid URL"
            />
          </label>
          <label>
            <span class="no-padding"
              >Donation platform<span class="required">*</span></span
            >
            <Multiselect
              id="donationPlatform"
              v-model="donationPlatforms[index]"
              placeholder="Choose a platform..."
              :loading="$tag.donationPlatforms.length === 0"
              :options="$tag.donationPlatforms"
              track-by="short"
              label="name"
              :multiple="false"
              :close-on-select="true"
              :show-labels="false"
              :allow-empty="false"
            />
          </label>
        </div>
      </section>
    </div>
  </div>
</template>

<script>
import Multiselect from 'vue-multiselect'
import SmartFileInput from '~/components/ui/SmartFileInput'
import StatelessFileInput from '~/components/ui/StatelessFileInput'
import ThisOrThat from '~/components/ui/ThisOrThat'
import VersionBadge from '~/components/ui/Badge'

import CheckIcon from '~/assets/images/utils/check.svg?inline'
import PlusIcon from '~/assets/images/utils/plus.svg?inline'
import SaveIcon from '~/assets/images/utils/save.svg?inline'
import TrashIcon from '~/assets/images/utils/trash.svg?inline'
import EditIcon from '~/assets/images/utils/edit.svg?inline'
import CrossIcon from '~/assets/images/utils/x.svg?inline'

export default {
  components: {
    SmartFileInput,
    StatelessFileInput,
    Multiselect,
    CheckIcon,
    PlusIcon,
    SaveIcon,
    TrashIcon,
    CrossIcon,
    EditIcon,
    ThisOrThat,
    VersionBadge,
  },
  beforeRouteLeave(to, from, next) {
    if (
      this.isEditing &&
      !window.confirm('Are you sure that you want to leave without saving?')
    ) {
      return
    }
    next()
  },
  data() {
    return {
      previewImage: null,
      compiledBody: '',
      releaseChannels: ['beta', 'alpha', 'release'],
      currentVersionIndex: -1,
      currentVersionNew: true,
      currentGalleryIndex: -1,
      currentGalleryNew: true,
      name: '',
      slug: '',
      draft: false,
      description: '',
      body: '',
      versions: [],
      categories: [],
      additional_categories: [],
      issues_url: null,
      source_url: null,
      wiki_url: null,
      discord_url: null,
      icon: null,
      license: null,
      license_url: null,

      selectableCategories: [],
      selectableAdditionalCategories: [],

      projectTypes: [
        {
          display: 'Mod',
          id: 'mod',
          realId: 'mod',
        },
        {
          display: 'Plugin',
          id: 'mod',
          realId: 'plugin',
        },
        {
          display: 'Mod and Plugin',
          id: 'mod',
          realId: 'mod+plugin',
        },
        {
          display: 'Modpack',
          id: 'modpack',
          realId: 'modpack',
        },
        {
          display: 'Resource Pack',
          id: 'resourcepack',
          realId: 'resourcepack',
        },
      ],
      projectType: {
        display: 'Mod',
        id: 'mod',
        realId: 'mod',
      },

      sideTypes: ['Required', 'Optional', 'Unsupported'],
      clientSideType: 'Required',
      serverSideType: 'Required',

      donationLinks: [],
      donationPlatforms: [],

      gallery: [],

      isEditing: true,
      bodyViewMode: 'source',
      changelogViewMode: 'source',

      dependencyAddMode: 'project',
      newDependencyType: 'required',
      newDependencyId: '',

      showKnownErrors: false,
      showKnownVersionErrors: false,
      showKnownGalleryErrors: false,
      savingAsDraft: false,
    }
  },
  fetch() {
    this.setCategories()
  },
  watch: {
    license(newValue, oldValue) {
      if (newValue == null) {
        this.license_url = ''
        return
      }

      switch (newValue.short) {
        case 'custom':
          this.license_url = ''
          break
        default:
          this.license_url = `https://cdn.modrinth.com/licenses/${newValue.short}.txt`
      }
    },
  },
  mounted() {
    function preventLeave(e) {
      e.preventDefault()
      e.returnValue = ''
    }

    window.addEventListener('beforeunload', preventLeave)
    this.$once('hook:beforeDestroy', () => {
      window.removeEventListener('beforeunload', preventLeave)
    })
  },
  methods: {
    setCategories(reset) {
      this.selectableCategories = this.$tag.categories
        .filter(
          (x) =>
            x.project_type === this.projectType.id &&
            !this.additional_categories.includes(x.name)
        )
        .map((it) => it.name)

      this.selectableAdditionalCategories = this.$tag.categories
        .filter(
          (x) =>
            x.project_type === this.projectType.id &&
            !this.categories.includes(x.name)
        )
        .map((it) => it.name)

      if (reset) {
        this.categories = []
        this.additional_categories = []
      }
    },
    checkFields() {
      const reviewConditions = this.body !== '' && this.versions.length > 0
      if (
        this.name !== '' &&
        this.description !== '' &&
        this.slug !== '' &&
        this.license !== null &&
        this.license_url !== null &&
        this.license_url !== ''
      ) {
        if (this.savingAsDraft) {
          return true
        } else if (reviewConditions) {
          return true
        }
      }
      this.showKnownErrors = true
      this.showKnownVersionErrors = true
      this.showKnownGalleryErrors = true
      return false
    },
    async createDraft() {
      this.setValues()
      this.savingAsDraft = true
      if (this.checkFields()) {
        this.draft = true
        await this.createProject()
      }
    },
    async createProjectForReview() {
      this.setValues()
      this.savingAsDraft = false
      if (this.checkFields()) {
        await this.createProject()
      }
    },
    setValues() {
      if (this.projectType.realId === 'resourcepack') {
        this.clientSideType = 'required'
        this.serverSideType = 'optional'
      } else if (this.projectType.realId === 'plugin') {
        this.clientSideType = 'unsupported'
        this.serverSideType = 'required'
      }
    },
    async createProject() {
      this.$nuxt.$loading.start()

      for (let i = 0; i < this.versions.length; i++) {
        const version = this.versions[i]
        if (!version.version_title) {
          version.version_title = version.version_number
        }

        const newFileParts = []
        for (let j = 0; j < version.files.length; j++) {
          newFileParts.push(`version-${i}-${j}`)
        }

        version.file_parts = newFileParts
      }

      const formData = new FormData()

      formData.append(
        'data',
        JSON.stringify({
          title: this.name,
          project_type: this.projectType.id,
          slug: this.slug,
          description: this.description,
          body: this.body,
          initial_versions: this.versions,
          team_members: [
            {
              user_id: this.$auth.user.id,
              name: this.$auth.user.username,
              role: 'Owner',
            },
          ],
          categories: this.categories,
          additional_categories: this.additional_categories,
          issues_url: this.issues_url ? this.issues_url : null,
          source_url: this.source_url ? this.source_url : null,
          wiki_url: this.wiki_url ? this.wiki_url : null,
          discord_url: this.discord_url,
          client_side: this.clientSideType.toLowerCase(),
          server_side: this.serverSideType.toLowerCase(),
          license_id: this.license ? this.license.short : 'arr',
          license_url: this.license_url ? this.license_url : null,
          is_draft: this.draft,
          donation_urls: this.donationPlatforms.map((it, index) => {
            return {
              id: it.short,
              platform: it.name,
              url: this.donationLinks[index],
            }
          }),
          gallery_items: this.gallery.map((x, index) => {
            return {
              item: `gallery-${index}`,
              featured: false,
              title: x.title,
              description: x.description || null,
            }
          }),
        })
      )

      if (this.icon) {
        formData.append('icon', new Blob([this.icon]), this.icon.name)
      }

      for (let i = 0; i < this.gallery.length; i++) {
        formData.append(
          `gallery-${i}`,
          new Blob([this.gallery[i].icon]),
          this.gallery[i].icon.name
        )
      }

      for (let i = 0; i < this.versions.length; i++) {
        const version = this.versions[i]
        for (let j = 0; j < version.files.length; j++) {
          formData.append(
            `version-${i}-${j}`,
            new Blob([version.files[j]]),
            version.files[j].name
          )
        }
      }

      try {
        await this.$axios({
          url: 'project',
          method: 'POST',
          data: formData,
          headers: {
            'Content-Type': 'multipart/form-data',
            Authorization: this.$auth.token,
          },
        })

        this.isEditing = false
        await this.$store.dispatch('user/fetchProjects')

        await this.$router.replace(`/user/${this.$auth.user.username}`)
      } catch (err) {
        let description = !err.response.data
          ? 'Data is null?'
          : err.response.data.description

        if (description.includes('JSON')) {
          description = 'Please fill in missing required fields.'
        }

        this.$notify({
          group: 'main',
          title: 'An error occurred',
          text: description,
          type: 'error',
          duration: 10000,
        })

        window.scrollTo({ top: 0, behavior: 'smooth' })
      }

      this.$nuxt.$loading.finish()
    },

    showPreviewImage(files) {
      const reader = new FileReader()
      this.icon = files[0]

      if (this.icon instanceof Blob) {
        reader.readAsDataURL(this.icon)

        reader.onload = (event) => {
          this.previewImage = event.target.result
        }
      }
    },

    showGalleryPreviewImage(files, index) {
      const reader = new FileReader()
      this.gallery[index].icon = files[0]

      if (this.gallery[index].icon instanceof Blob) {
        reader.readAsDataURL(this.gallery[index].icon)

        reader.onload = (event) => {
          this.gallery[index].preview = event.target.result

          // TODO: Find an alternative for this!
          this.$forceUpdate()
        }
      }
    },

    createVersion() {
      this.versions.push({
        files: [],
        version_number: '',
        version_title: '',
        version_body: '',
        dependencies: [],
        game_versions: [],
        release_channel: 'release',
        loaders: [],
        featured: false,
      })

      this.currentVersionIndex = this.versions.length - 1
      this.currentVersionNew = true
    },

    saveVersion() {
      const version = this.versions[this.currentVersionIndex]
      if (this.projectType.realId === 'resourcepack') {
        version.loaders = ['minecraft']
      }
      if (
        version.version_number !== '' &&
        version.releaseChannels !== null &&
        version.loaders.length > 0 &&
        version.game_versions.length > 0 &&
        version.files.length > 0
      ) {
        this.currentVersionIndex = -1
      } else {
        this.showKnownVersionErrors = true
      }
    },

    editVersion(index) {
      this.currentVersionIndex = index
      this.currentVersionNew = false
    },

    deleteVersion() {
      this.versions.splice(this.currentVersionIndex, 1)
      this.currentVersionIndex = -1
      this.showKnownVersionErrors = false
    },

    createGalleryImage() {
      this.gallery.push({
        title: '',
        description: '',
        preview: null,
      })
      this.currentGalleryIndex = this.gallery.length - 1
      this.currentGalleryNew = true
    },

    saveGalleryImage() {
      const image = this.gallery[this.currentGalleryIndex]
      if (image.title !== '' && image.preview !== null) {
        this.currentGalleryIndex = -1
        this.showKnownGalleryErrors = false
      } else {
        this.showKnownGalleryErrors = true
      }
    },

    editGalleryImage(index) {
      this.currentGalleryIndex = index
      this.currentGalleryNew = false
    },

    deleteGalleryImage() {
      this.gallery.splice(this.currentGalleryIndex, 1)
      this.currentGalleryIndex = -1
      this.showKnownGalleryErrors = false
    },

    async addDependency() {
      try {
        if (this.dependencyAddMode === 'project') {
          const project = (
            await this.$axios.get(`project/${this.newDependencyId}`)
          ).data

          this.versions[this.currentVersionIndex].dependencies.push({
            project,
            project_id: project.id,
            dependency_type: this.newDependencyType,
          })
        } else if (this.dependencyAddMode === 'version') {
          const version = (
            await this.$axios.get(`version/${this.newDependencyId}`)
          ).data
          const project = (
            await this.$axios.get(`project/${version.project_id}`)
          ).data

          this.versions[this.currentVersionIndex].dependencies.push({
            version,
            project,
            version_id: version.id,
            project_id: project.id,
            dependency_type: this.newDependencyType,
          })
        }
      } catch {
        this.$notify({
          group: 'main',
          title: 'Invalid Dependency',
          text: 'The specified dependency does not exist',
          type: 'error',
        })
      }

      this.newDependencyId = ''
    },
  },
}
</script>

<style lang="scss" scoped>
.title {
  .text * {
    display: inline;
  }

  display: flex;
  justify-content: space-between;
  align-items: center;
}

.page-contents {
  display: grid;
  grid-template:
    'header       header       header' auto
    'essentials   essentials   essentials' auto
    'project-icon project-icon project-icon' auto
    'game-sides   game-sides   game-sides' auto
    'description  description  description' auto
    'versions     versions     versions' auto
    'gallery      gallery      gallery' auto
    'extra-links  extra-links  extra-links' auto
    'license      license      license' auto
    'donations    donations    donations' auto
    'footer       footer       footer' auto
    / 4fr 1fr 4fr;

  @media screen and (min-width: 1024px) {
    grid-template:
      'header       header      header' auto
      'essentials   essentials  project-icon' auto
      'game-sides   game-sides  game-sides' auto
      'description  description description' auto
      'versions     versions    versions' auto
      'gallery      gallery      gallery' auto
      'extra-links  license     license' auto
      'donations    donations   donations' auto
      'footer       footer      footer' auto
      / 4fr 2fr 1.5fr;
  }

  column-gap: var(--spacing-card-md);
  row-gap: var(--spacing-card-md);
}

header {
  grid-area: header;

  h3 {
    margin: auto 0;
    color: var(--color-text-dark);
    font-weight: var(--font-weight-extrabold);
  }

  button {
    margin-left: 0.5rem;
  }
}

section.essentials {
  grid-area: essentials;
}

section.project-icon {
  grid-area: project-icon;

  img {
    max-width: 100%;
    margin-bottom: 0.25rem;
    border-radius: var(--size-rounded-lg);

    @media screen and (min-width: 420px) {
      max-width: 20rem;
    }

    @media screen and (min-width: 1024px) {
      max-width: 100%;
    }
  }

  .iconified-button {
    margin-top: 0.5rem;
  }
}

section.game-sides {
  grid-area: game-sides;

  .columns {
    flex-wrap: wrap;

    div {
      flex: 2;
    }

    .labeled-control {
      margin-left: var(--spacing-card-lg);
      margin-bottom: 0.5rem;

      h3 {
        margin-bottom: var(--spacing-card-sm);
      }
    }
  }
}

section.description {
  grid-area: description;

  .separator {
    margin: var(--spacing-card-sm) 0;
  }

  .edit-wrapper * {
    min-height: 10rem;
    max-height: 40rem;
  }

  .markdown-body {
    overflow-y: auto;
    padding: 0 var(--spacing-card-sm);
  }

  label {
    flex-direction: row;
  }
}

.outlined-area {
  margin: var(--spacing-card-md) 0;
  border: 1px solid var(--color-divider);
  border-radius: var(--size-rounded-card);
  padding: 1rem;

  .header {
    display: flex;
    align-items: center;
    padding-bottom: 1rem;
    grid-area: header;

    .title {
      flex-grow: 1;
    }
  }
}

.controls {
  display: flex;
  flex-direction: row;
  gap: 0.5rem;
}

section.versions {
  grid-area: versions;

  table {
    border-collapse: collapse;
    margin-bottom: var(--spacing-card-md);
    background: var(--color-raised-bg);
    border-radius: var(--size-rounded-card);
    table-layout: fixed;
    width: 100%;

    * {
      text-align: left;
    }

    th,
    td {
      &:first-child {
        text-align: center;
        width: 7%;

        svg {
          color: var(--color-text);

          &:hover,
          &:focus {
            color: var(--color-text-hover);
          }
        }
      }

      &:nth-child(2),
      &:nth-child(5) {
        padding-left: 0;
        width: 12%;
      }

      &:last-child {
        display: flex;
      }

      @media screen and (max-width: 800px) {
        + &:nth-child(4),
        &:nth-child(3) {
          display: none;
        }
        &:first-child,
        &:nth-child(5) {
          width: unset;
        }
      }

      @media screen and (max-width: 1024px) {
        &:nth-child(2) {
          display: none;
        }
      }
    }

    th {
      color: var(--color-heading);
      letter-spacing: 0.02rem;
      padding: 0.75rem 1rem;
    }

    td {
      overflow: hidden;
      padding: 0.75rem 1rem;

      img {
        height: 3rem;
        width: 3rem;
      }
    }
  }

  hr {
    background-color: var(--color-divider);
    border: none;
    color: var(--color-divider);
    height: 1px;
    margin: 0.5rem 0;
  }

  .no-versions-warning {
    @extend .required;
    text-align: center;
    padding: 1rem;
  }

  .new-version {
    display: grid;
    grid-template:
      'header header' auto
      'known-errors known-errors' auto
      'main main' auto
      'dependencies dependencies' auto
      'files files'
      'changelog changelog'
      / 5fr 4fr;
    column-gap: var(--spacing-card-md);

    @media screen and (min-width: 1024px) {
      grid-template:
        'header header' auto
        'known-errors known-errors' auto
        'main dependencies' auto
        'main files' 1fr
        'changelog changelog'
        / 5fr 4fr;
    }

    .main {
      grid-area: main;
    }

    .dependencies {
      grid-area: dependencies;

      .dependency-selector {
        .separator {
          margin: var(--spacing-card-sm) 0;
        }

        h4 {
          margin: 0 0 0.25rem 0;
        }

        .edit-info {
          display: flex;
          flex-wrap: wrap;
          gap: 0.5rem;

          .multiselect {
            max-width: 10rem;
          }

          .iconified-button {
            min-height: 2.5rem;
          }

          input {
            flex-grow: 1;
            max-width: fit-content;
          }

          input,
          .multiselect,
          .iconified-button {
            margin: 0;
          }
        }
      }

      .new-dependencies {
        margin-top: var(--spacing-card-sm);
        display: flex;
        flex-wrap: wrap;

        .dependency {
          align-items: center;
          display: flex;
          flex-basis: 50%;
          margin-bottom: 0.5rem;

          .icon {
            width: 3rem;
            height: 3rem;
            margin-right: 0.5rem;
            border-radius: var(--size-rounded-xs);
            object-fit: contain;
          }

          .info {
            display: flex;
            flex-direction: column;
            justify-content: space-between;
            padding: 0.25rem;

            p {
              margin: 0;
            }

            .title {
              margin: 0 0.25rem 0 0;
            }
          }
        }
      }
    }

    .files {
      grid-area: files;

      .file-input {
        margin-top: 1rem;
      }

      .uploaded-files {
        .file {
          display: flex;
          align-items: center;
          margin-bottom: 0.25rem;
          flex-wrap: wrap;
          row-gap: 0.25rem;

          * {
            margin-left: 0.25rem;
          }

          .filename {
            margin: 0;
            font-weight: bold;
          }
        }
      }
    }

    .changelog {
      grid-area: changelog;
      display: flex;
      flex-direction: column;

      .separator {
        margin: var(--spacing-card-sm) 0;
      }

      .edit-wrapper {
        flex: 1;

        .textarea-wrapper {
          min-height: 10rem;
          height: 100%;
        }
      }
    }
  }
}

section.gallery {
  grid-area: gallery;

  .gallery-items {
    display: flex;
    flex-wrap: wrap;
    gap: var(--spacing-card-lg);

    .gallery-item {
      display: flex;
      flex-direction: column;
      gap: 0.5rem;

      h2 {
        margin: 0;
      }
      h4 {
        margin: 0;
      }
      width: calc((100% - 2 * var(--spacing-card-lg)) / 3);

      img {
        width: 100%;
        border-radius: var(--size-rounded-card);
      }
    }
  }

  .new-gallery-item {
    margin-top: 1rem;
    display: grid;
    grid-template:
      'header' auto
      'known-errors' auto
      'info' auto
      'image' auto;

    @media screen and (min-width: 1024px) {
      grid-template:
        'header header' auto
        'known-errors known-errors' auto
        'info image' auto;
    }

    column-gap: var(--spacing-card-md);
    row-gap: var(--spacing-card-sm);

    .info {
      grid-area: info;

      label {
        align-items: center;
        margin-top: var(--spacing-card-sm);

        span {
          flex: 1;
        }
      }
    }

    .image {
      grid-area: image;

      img {
        width: 100%;
        margin-top: 0.5rem;
        margin-bottom: 0.5rem;

        height: 14rem;
        object-fit: cover;

        border-radius: var(--size-rounded-card);
      }

      .bottom {
        display: flex;
        gap: 0.5rem;
        align-items: center;
      }
    }

    .buttons {
      grid-area: buttons;

      hr {
        margin-top: 0.5rem;
      }
    }
  }
}

section.extra-links {
  grid-area: extra-links;

  .title {
    margin-top: 0.5rem;
  }

  label {
    align-items: center;
    margin-top: var(--spacing-card-sm);

    span {
      flex: 1;
    }
  }
}

section.license {
  grid-area: license;

  .title {
    margin-top: 0.5rem;
  }

  label {
    margin-top: var(--spacing-card-sm);
  }

  .row {
    display: flex;
    flex-direction: row;
    align-items: center;
  }
}

section.donations {
  grid-area: donations;
}

.footer {
  grid-area: footer;
}

.choose-image {
  cursor: pointer;
}

.card {
  margin-bottom: 0;
}

.required {
  color: var(--color-badge-red-bg);
}

.essentials,
.new-version,
.donations {
  label {
    margin-bottom: 1rem;
  }
  span {
    margin-bottom: 0.5rem;
  }

  @media screen and (min-width: 1024px) {
    label {
      margin-bottom: 0.5rem;
    }
    input {
      margin-left: 1.5rem;
    }
  }
}

.slug-description {
  overflow-wrap: break-word;
}
</style>
