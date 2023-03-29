<template>
  <Card class="mod-card">
    <span class="manage">
      <DropdownSelect id="filterLoader" :options="versions.flatMap(value => value.loaders).filter((value, index, self) => self.indexOf(value) === index)"/>
      <DropdownSelect id="filterVersions" :options="versions.flatMap(value => value.game_versions).filter((value, index, self) => self.indexOf(value) === index)"/>
      <Button>
        <CheckCircleIcon />
        Clear Filters
      </Button>
    </span>
    <div class="table-container">
      <div class="table-row table-head">
        <div class="table-cell table-text name-cell">Name</div>
        <div class="table-cell table-text">Supports</div>
        <div class="table-cell table-text">Stats</div>
      </div>
      <div class="table-row" v-for="version in versions" :key="version.id">
        <div class="table-cell table-text name-cell">
          {{ version.name.charAt(0).toUpperCase() + version.name.slice(1).toLowerCase() }}
        </div>
        <div class="table-cell table-text">
          {{ version.loaders.join(', ') }}<br />
          {{ version.game_versions.join(', ') }}
        </div>
        <div class="table-cell table-text">
          Published: {{ new Date(version.date_published).toLocaleDateString() }}<br />
          Downloads: {{ version.downloads }}
        </div>
      </div>
      <Modal v-for="version in versions" ref="version_modals" :id="version.id">
        pOKEMON
      </Modal>

      <Button v-for="modal in this.$refs.version_modals" :action="() => modal.show()">
        Test
      </Button>
      {{this.$refs.versions}}
    </div>
  </Card>
</template>

<script setup>
import { Card, Button, PlusIcon, SearchIcon, DropdownSelect, Modal, CheckCircleIcon } from 'omorphia'
const versions = [
  {
    "id": "tykm7yBk",
    "project_id": "b1LdOZlE",
    "author_id": "6ZoP9xY8",
    "featured": false,
    "name": "2.2.4",
    "version_number": "2.2.4",
    "changelog": "> Fix level being null occassionally in the soul crystal item\n",
    "changelog_url": null,
    "date_published": "2023-03-07T15:32:30.682062Z",
    "downloads": 220,
    "version_type": "release",
    "status": "listed",
    "requested_status": null,
    "files": [
      {
        "hashes": {
          "sha512": "bb917d7e48a0e79bff06d1db68b23cc2290700735c4c1951e21bd496d2ec6d968ecba3798337b184e32afe1273c5c39f99862bdfe887e01a72e01acb18d8671f",
          "sha1": "d85921b45832983167c74abe87b0030118753564"
        },
        "url": "https://cdn.modrinth.com/data/b1LdOZlE/versions/tykm7yBk/spirit-forge-1.19.2-2.2.4.jar",
        "filename": "spirit-forge-1.19.2-2.2.4.jar",
        "primary": true,
        "size": 658707,
        "file_type": null
      }
    ],
    "dependencies": [],
    "game_versions": [
      "1.19.2"
    ],
    "loaders": [
      "forge"
    ]
  },
  {
    "id": "17lKW7bl",
    "project_id": "b1LdOZlE",
    "author_id": "6ZoP9xY8",
    "featured": false,
    "name": "2.2.4",
    "version_number": "2.2.4",
    "changelog": "> Fix level being null occassionally in the soul crystal item\n",
    "changelog_url": null,
    "date_published": "2023-03-07T15:32:23.611916Z",
    "downloads": 208,
    "version_type": "release",
    "status": "listed",
    "requested_status": null,
    "files": [
      {
        "hashes": {
          "sha1": "19be757d03c5d106a87b043b068f741f20674bb6",
          "sha512": "f14016cbcd1f33b0a3e9830fa3d5e58d05ce87e20eab360b756d01b1b94f57b5fc56be035981ead13400a08395d6825d7d50b1d044af4a8f76071757a1b99096"
        },
        "url": "https://cdn.modrinth.com/data/b1LdOZlE/versions/17lKW7bl/spirit-fabric-1.19.2-2.2.4.jar",
        "filename": "spirit-fabric-1.19.2-2.2.4.jar",
        "primary": true,
        "size": 646055,
        "file_type": null
      }
    ],
    "dependencies": [
      {
        "version_id": "hfsU4hXq",
        "project_id": "P7dR8mSH",
        "file_name": null,
        "dependency_type": "required"
      }
    ],
    "game_versions": [
      "1.19.2"
    ],
    "loaders": [
      "fabric"
    ]
  },
  {
    "id": "X7uRgrEW",
    "project_id": "b1LdOZlE",
    "author_id": "6ZoP9xY8",
    "featured": false,
    "name": "2.2.3",
    "version_number": "2.2.3",
    "changelog": "> Fix keybind issue\n> Fixed broken loot tables",
    "changelog_url": null,
    "date_published": "2022-12-03T18:20:50.106333Z",
    "downloads": 1045,
    "version_type": "release",
    "status": "listed",
    "requested_status": null,
    "files": [
      {
        "hashes": {
          "sha512": "e605ae60cca1778de05102c7c9f99b1895aeadc14a5b3110aed27839ac3209bc5178f2fa10eac577a95ef14a4510ed19b5603caaa2043c995fc6609c308e65aa",
          "sha1": "91da7a1e1d456e3e07bc664ebcb19cc9f59cfd25"
        },
        "url": "https://cdn.modrinth.com/data/b1LdOZlE/versions/X7uRgrEW/spirit-forge-1.19.2-2.2.3.jar",
        "filename": "spirit-forge-1.19.2-2.2.3.jar",
        "primary": true,
        "size": 662205,
        "file_type": null
      }
    ],
    "dependencies": [],
    "game_versions": [
      "1.19.2"
    ],
    "loaders": [
      "forge"
    ]
  },
  {
    "id": "FtFcNoht",
    "project_id": "b1LdOZlE",
    "author_id": "6ZoP9xY8",
    "featured": false,
    "name": "2.2.3",
    "version_number": "2.2.3",
    "changelog": "> Fix keybind issue\n> Fixed broken loot tables",
    "changelog_url": null,
    "date_published": "2022-12-03T18:20:44.803041Z",
    "downloads": 1157,
    "version_type": "release",
    "status": "listed",
    "requested_status": null,
    "files": [
      {
        "hashes": {
          "sha512": "a77e8fa0d15d6d5aed15888f3f6a45651e96221eff963a2bb20f19281b17adacd59ab41302985fd81634dac08407a4c7d7f024ae9fe68e359d546c5378711573",
          "sha1": "b99520db5ef73938b9fa460b544b1d0f33b539ab"
        },
        "url": "https://cdn.modrinth.com/data/b1LdOZlE/versions/FtFcNoht/spirit-fabric-1.19.2-2.2.3.jar",
        "filename": "spirit-fabric-1.19.2-2.2.3.jar",
        "primary": true,
        "size": 649550,
        "file_type": null
      }
    ],
    "dependencies": [
      {
        "version_id": "hfsU4hXq",
        "project_id": "P7dR8mSH",
        "file_name": null,
        "dependency_type": "required"
      }
    ],
    "game_versions": [
      "1.19.2"
    ],
    "loaders": [
      "fabric"
    ]
  },
  {
    "id": "4K2VzMbE",
    "project_id": "b1LdOZlE",
    "author_id": "mL6r9Xal",
    "featured": false,
    "name": "2.2.2",
    "version_number": "2.2.2",
    "changelog": "Spirit 2.2.2\nAdded zooming pinch and drag to REI/JEI",
    "changelog_url": null,
    "date_published": "2022-11-13T21:54:30.340961Z",
    "downloads": 220,
    "version_type": "release",
    "status": "listed",
    "requested_status": null,
    "files": [
      {
        "hashes": {
          "sha512": "e5e57b4612c7e0169ed42bf4d1b58af882f956b10124f26cd854ba109314053f50cf3ad63e6e8da1f700e4f0aa8d9d8e6ae2d6e97f515efa8ae5f9ea9d25303d",
          "sha1": "ed7381e119903910137a0cc40a38aef263908135"
        },
        "url": "https://cdn.modrinth.com/data/b1LdOZlE/versions/4K2VzMbE/Spirit-forge-1.19.2-2.2.2.jar",
        "filename": "Spirit-forge-1.19.2-2.2.2.jar",
        "primary": true,
        "size": 660543,
        "file_type": null
      }
    ],
    "dependencies": [],
    "game_versions": [
      "1.19.2"
    ],
    "loaders": [
      "forge"
    ]
  },
  {
    "id": "btBFr95k",
    "project_id": "b1LdOZlE",
    "author_id": "mL6r9Xal",
    "featured": false,
    "name": "2.2.2",
    "version_number": "2.2.2",
    "changelog": "Spirit 2.2.2\nAdded zooming pinch and drag to REI/JEI",
    "changelog_url": null,
    "date_published": "2022-11-13T21:54:25.535618Z",
    "downloads": 188,
    "version_type": "release",
    "status": "listed",
    "requested_status": null,
    "files": [
      {
        "hashes": {
          "sha1": "01277a940aba63e748d361fa4a3432ebbe895ff9",
          "sha512": "c7efa48586df28ff8da9eef3404716b6d181a2dde70049d64b8f3bb1276f1bbcdf9650a6c7f02cf1a492be52de78622723e288bf44911989bb38eb37ae86f15a"
        },
        "url": "https://cdn.modrinth.com/data/b1LdOZlE/versions/btBFr95k/Spirit-fabric-1.19.2-2.2.2.jar",
        "filename": "Spirit-fabric-1.19.2-2.2.2.jar",
        "primary": true,
        "size": 648780,
        "file_type": null
      }
    ],
    "dependencies": [
      {
        "version_id": "hfsU4hXq",
        "project_id": "P7dR8mSH",
        "file_name": null,
        "dependency_type": "required"
      }
    ],
    "game_versions": [
      "1.19.2"
    ],
    "loaders": [
      "fabric"
    ]
  },
  {
    "id": "ZkB9qAZW",
    "project_id": "b1LdOZlE",
    "author_id": "mL6r9Xal",
    "featured": false,
    "name": "2.1.12",
    "version_number": "2.1.12",
    "changelog": "Initial modrinth info",
    "changelog_url": null,
    "date_published": "2022-09-17T20:02:44.266258Z",
    "downloads": 95,
    "version_type": "release",
    "status": "listed",
    "requested_status": null,
    "files": [
      {
        "hashes": {
          "sha1": "45872029d79fd77b42781e9b8712cfdf29ef473b",
          "sha512": "bccaf826dcb939f2317d6f4da4af17eab7c224d58a9c85e5b97010a1c3b5838a31a4d31d2f9c05ae2881d136443711cdbaff81fb6460bc1996b9cd409e90148d"
        },
        "url": "https://cdn.modrinth.com/data/b1LdOZlE/versions/ZkB9qAZW/spirit-fabric-1.19.2-2.1.12.jar",
        "filename": "spirit-fabric-1.19.2-2.1.12.jar",
        "primary": true,
        "size": 513816,
        "file_type": null
      }
    ],
    "dependencies": [
      {
        "version_id": "hfsU4hXq",
        "project_id": "P7dR8mSH",
        "file_name": null,
        "dependency_type": "required"
      }
    ],
    "game_versions": [
      "1.19.2"
    ],
    "loaders": [
      "fabric"
    ]
  },
  {
    "id": "X9sqJmJD",
    "project_id": "b1LdOZlE",
    "author_id": "mL6r9Xal",
    "featured": false,
    "name": "2.1.12",
    "version_number": "2.1.12",
    "changelog": "Initial Modrinth upload",
    "changelog_url": null,
    "date_published": "2022-09-17T20:01:23.751244Z",
    "downloads": 335,
    "version_type": "release",
    "status": "listed",
    "requested_status": null,
    "files": [
      {
        "hashes": {
          "sha1": "567ed1d85ffaa092050ca7aac473e5079c74ce45",
          "sha512": "efc76cc7f5f33d8c76edc020d90141db91aeaa64b696b78603ab41a4575cc90a4b8366cae538b9b8240f571e76657e030662cdf348ee6efd2020e593e1d18d0f"
        },
        "url": "https://cdn.modrinth.com/data/b1LdOZlE/versions/X9sqJmJD/spirit-forge-1.19.2-2.1.12.jar",
        "filename": "spirit-forge-1.19.2-2.1.12.jar",
        "primary": true,
        "size": 522558,
        "file_type": null
      }
    ],
    "dependencies": [],
    "game_versions": [
      "1.19.2"
    ],
    "loaders": [
      "forge"
    ]
  },
  {
    "id": "guuekcl2",
    "project_id": "b1LdOZlE",
    "author_id": "mL6r9Xal",
    "featured": false,
    "name": "2.1.8",
    "version_number": "2.1.8",
    "changelog": "Initial modrinth upload",
    "changelog_url": null,
    "date_published": "2022-09-17T20:00:48.276807Z",
    "downloads": 115,
    "version_type": "release",
    "status": "listed",
    "requested_status": null,
    "files": [
      {
        "hashes": {
          "sha1": "abb87e3b15cd38a30859ef5d1d2816c4ebcbe015",
          "sha512": "12df097d340664fb5738173817da507decf11530d9f91ba49984ad377800b03cbba0f55b52e26a8474f94ec1c0e3d64c5c8463494f90775d6a0e672f503e7907"
        },
        "url": "https://cdn.modrinth.com/data/b1LdOZlE/versions/guuekcl2/spirit-fabric-1.18.2-2.1.8.jar",
        "filename": "spirit-fabric-1.18.2-2.1.8.jar",
        "primary": true,
        "size": 486103,
        "file_type": null
      }
    ],
    "dependencies": [
      {
        "version_id": "95QMsRyb",
        "project_id": "P7dR8mSH",
        "file_name": null,
        "dependency_type": "required"
      }
    ],
    "game_versions": [
      "1.18.2"
    ],
    "loaders": [
      "fabric"
    ]
  },
  {
    "id": "PEhhc4aW",
    "project_id": "b1LdOZlE",
    "author_id": "mL6r9Xal",
    "featured": false,
    "name": "2.1.8",
    "version_number": "2.1.8",
    "changelog": "Initial modrinth upload",
    "changelog_url": null,
    "date_published": "2022-09-17T19:59:54.318844Z",
    "downloads": 47,
    "version_type": "release",
    "status": "listed",
    "requested_status": null,
    "files": [
      {
        "hashes": {
          "sha512": "1389f4a06e4599de5110ab0b5f32481bfe783041f9431f0feabd045bffc11cd45de3bf92e6cc9404fac7b0aad51d650aa6dad087e9f9aef777672ba3f1e6cbfb",
          "sha1": "6f31804390b1ea001df090899f55a7a74a1a3fa7"
        },
        "url": "https://cdn.modrinth.com/data/b1LdOZlE/versions/PEhhc4aW/spirit-forge-1.18.2-2.1.8.jar",
        "filename": "spirit-forge-1.18.2-2.1.8.jar",
        "primary": true,
        "size": 495583,
        "file_type": null
      }
    ],
    "dependencies": [],
    "game_versions": [
      "1.18.2"
    ],
    "loaders": [
      "forge"
    ]
  }
]
</script>

<script>
export default {
  name: "Versions"
}
</script>

<style scoped lang="scss">
.table-container {
  display: grid;
  grid-template-rows: repeat(auto-fill, auto);
  width: 100%;
  border-radius: var(--radius-md);
  overflow: hidden;
}

.table-row {
  display: grid;
  grid-template-columns: 1fr 1fr 1fr;
}

.table-head {
  .table-cell {
    background-color: var(--color-accent-contrast);
  }
}

.table-cell {
  padding: 1rem;
  height: 100%;
  align-items: center;
  vertical-align: center;
  display: flex;
}

.table-text {
  overflow: hidden;
  white-space: nowrap;
  text-overflow: fade;
}

.manage {
  display: flex;
  gap: 0.5rem;
}

.mod-text {
  display: flex;
  align-items: center;
  gap: 1rem;
  color: var(--color-contrast);
}

.table-row:nth-child(even) .table-cell {
  background-color: var(--color-bg);
}

.card-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  background-color: var(--color-raised-bg);
}

.mod-card {
  display: flex;
  flex-direction: column;
  gap: 1rem;
  overflow: hidden;
}

.text-combo {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}
</style>