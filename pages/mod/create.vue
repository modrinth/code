<template>
  <div class="content">
    <h2>Create Mod</h2>
    <section>
      <h3>Initial Data</h3>
      <div class="initial">
        <div class="image-data">
          <img
            :src="
              previewImage
                ? previewImage
                : 'https://i0.kym-cdn.com/entries/icons/facebook/000/013/564/aP2dv.gif'
            "
          />
          <input
            id="file"
            class="file-input"
            type="file"
            accept="image/x-png,image/gif,image/jpeg"
            @change="showPreviewImage"
          />
          <label for="file">Upload Icon</label>
        </div>
        <div class="mod-data">
          <label class="required" title="The name of your mod"> Name </label>
          <input v-model="name" type="text" placeholder="Example Mod" />
          <label
            class="required"
            title="The namespace of your mod, an example is com.example.mod"
          >
            Namespace
          </label>
          <input
            v-model="namespace"
            type="text"
            placeholder="com.example.examplemod"
          />
          <label
            class="required"
            title="The short-form description of your mod. This shows up in searches"
          >
            Short Description
          </label>
          <input
            v-model="description"
            type="text"
            placeholder="An example mod which does example stuff!"
          />
        </div>
      </div>
    </section>
    <section class="editor">
      <h3>Mod Body</h3>
      <p>
        You can type the of the long form of your description here. This editor
        supports markdown. You can find the syntax
        <a href="https://github.com/dimerapp/markdown/blob/develop/syntax.md"
          >here</a
        >.
      </p>
      <textarea v-model="body"></textarea>
      <div v-html="$md.render(body)"></div>
    </section>
    <section>
      <h3>Versions</h3>
    </section>
    <section>
      <h3>Extras</h3>
      <div class="extras">
        <label
          title="A link where users can go to report bugs, issues, and concerns about your mod."
        >
          Issues URL
          <input v-model="issues_url" type="text" placeholder="Optional" />
        </label>
        <label title="A link to a page/repository containing the source code ">
          Source Code Link
          <input v-model="source_url" type="text" placeholder="Optional" />
        </label>
        <label
          title="A link to a page containing information, documentation, and help for the mod. (Optional)"
        >
          Wiki Link
          <input v-model="wiki_url" type="text" placeholder="Optional" />
        </label>
      </div>
    </section>
    <button @click="createMod">Create mod</button>
  </div>
</template>

<script>
import axios from 'axios'

export default {
  data() {
    return {
      previewImage: null,
      name: '',
      namespace: '',
      description: '',
      body: '',
      compiledBody: '',
      versions: [],
      categories: [],
      issues_url: null,
      source_url: null,
      wiki_url: null,
      icon: null,
    }
  },
  methods: {
    async createMod() {
      const formData = new FormData()

      formData.append(
        'data',
        JSON.stringify({
          mod_name: this.name,
          mod_namespace: this.namespace,
          mod_description: this.description,
          mod_body: this.body,
          initial_versions: this.versions,
          team_members: [
            {
              user_id: this.$auth.user.id,
              name: this.$auth.user.username,
              role: 'Owner',
            },
          ],
          categories: this.categories,
          issues_url: this.issues_url,
          source_url: this.source_url,
          wiki_url: this.wiki_url,
        })
      )

      if (this.icon)
        formData.append('icon', new Blob([this.icon]), this.icon.name)

      try {
        const result = await axios({
          url: 'https://api.modrinth.com/api/v1/mod',
          method: 'POST',
          data: formData,
          headers: {
            'Content-Type': 'multipart/form-data',
            Authorization: this.$auth.getToken('local'),
          },
        })

        // eslint-disable-next-line no-console
        console.log(result)
      } catch (err) {
        // eslint-disable-next-line no-console
        console.error(err)
      }
    },
    showPreviewImage(e) {
      const reader = new FileReader()
      this.icon = e.target.files[0]
      reader.readAsDataURL(this.icon)

      reader.onload = (event) => {
        this.previewImage = event.target.result
      }
    },
  },
}
</script>

<style lang="scss" scoped>
section {
  box-shadow: 0 2px 3px 1px var(--color-grey-2);
  margin: 50px 25px;
  padding: 5px 20px 20px 20px;
  border-radius: 10px;
  background-color: var(--color-bg);

  h3 {
    margin-bottom: 15px;
  }
}

input {
  width: 100%;
  padding: 0.5rem 5px;
  margin-bottom: 20px;
}

.initial {
  display: flex;
  .image-data {
    flex: 1;

    img {
      object-fit: cover;
      width: 100%;
      height: 85%;
      margin-bottom: 20px;
    }
    [type='file'] {
      border: 0;
      clip: rect(0, 0, 0, 0);
      height: 1px;
      overflow: hidden;
      padding: 0;
      position: absolute !important;
      white-space: nowrap;
      width: 1px;

      + label {
        border-radius: 5px;
        color: var(--color-grey-5);
        background-color: var(--color-grey-1);
        padding: 10px 20px;
      }

      &:focus + label,
      + label:hover,
      &:focus + label {
        background-color: var(--color-grey-2);
        color: var(--color-text);
      }
    }
  }

  .mod-data {
    flex: 4;
    margin: 20px;
  }
}

.editor {
  width: calc(100% - 90px);
  min-height: 500px;

  h3 {
    margin-bottom: 5px;
  }
  p {
    margin-top: 0;
    margin-bottom: 15px;
    a {
      text-decoration: underline;
    }
  }

  textarea {
    padding: 20px;
    width: calc(50% - 50px);
    min-height: 500px;
    resize: none;
    outline: none;
    border: none;
    margin: 0;
    background-color: var(--color-grey-1);
    border-right: 1px solid var(--color-text);
    color: var(--color-text);
    font-family: monospace;
  }
  div {
    padding: 20px;
    margin: 0;
    min-height: 540px;
    display: inline-block;
    width: calc(50%);
    box-sizing: border-box;
    vertical-align: top;
    background-color: var(--color-grey-2);
  }
}

button {
  float: right;
  margin: -10px 25px 20px 0;
  cursor: pointer;
  padding: 10px;
  outline: none;
  color: var(--color-grey-5);
  background-color: var(--color-grey-1);
  border: none;
  border-radius: 5px;

  &:hover {
    background-color: var(--color-grey-2);
    color: var(--color-text);
  }
}

.extras {
  padding: 10px;
}

.required:after {
  content: ' *';
  color: red;
}
</style>
