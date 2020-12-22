<template>
  <div>
    <div class="main-hero columns">
      <div class="left">
        <h1 class="typewriter">
          {{ currentText }}<span aria-hidden="true"></span>
        </h1>
        <h1>modding platform</h1>
      </div>
      <div class="right columns">
        <img class="char" src="~/assets/images/logo.svg" alt="logo" />
      </div>
    </div>
    <div class="slanted-hero columns">
      <div class="left hero">
        <h3>Conveniently modern</h3>
        <h1>A redefined search interface</h1>
        <p>
          We implemented <span>fast and adaptable</span> search algorithms so
          you don't have to wait, while creating a responsive interface that
          makes sense. Modrinth is full of elegant mod discovery and a platform
          platform which just works.
        </p>
      </div>
      <div class="right hero-image">
        <video
          src="~/assets/images/search.mp4"
          alt="search"
          loading="lazy"
          loop
          muted
          autoplay
        />
      </div>
    </div>
    <div class="default-hero columns">
      <div class="left hero">
        <h3>Built for developers</h3>
        <h1>The world's most modder-friendly platform</h1>
        <p>
          Modrinth intends to give back to the community, not leech from it.
          That's why we are giving
          <span>one hundred percent of ad revenue back</span> to modders, while
          creating easy to use tools for every modder to publish their mods on
          the Modrinth platform.
        </p>
      </div>
      <div class="right columns workflow">
        <div>
          <h3>Code</h3>
          <svg
            style="background-color: #1a202c"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
          >
            <polyline points="16 18 22 12 16 6"></polyline>
            <polyline points="8 6 2 12 8 18"></polyline>
          </svg>
        </div>
        <img class="char" src="~/assets/images/right-arrow.svg" alt="logo" />
        <div>
          <h3>Build</h3>
          <svg
            style="background-color: #127183"
            viewBox="0 0 256 188"
            preserveAspectRatio="xMidYMid"
            fill="currentColor"
            stroke="currentColor"
          >
            <g>
              <path
                d="M242.08678,11.8947553 C226.620445,-3.57845046 201.673638,-4.00992041 185.681477,10.9191919 C184.914067,11.6431863 184.467166,12.6433945 184.439851,13.6980695 C184.445493,14.7482934 184.858879,15.7552589 185.592789,16.5065097 L190.647982,21.739077 C192.011075,23.0964262 194.163868,23.2474994 195.703174,22.0938274 C199.713048,19.0675791 204.603476,17.4374363 209.627124,17.4525105 C219.052991,17.4112768 227.56987,23.0684934 231.186492,31.7730153 C234.803114,40.4775372 232.802822,50.5045093 226.123015,57.1549851 C193.89986,89.3781401 150.886382,-0.905819089 53.270916,45.5369117 C49.9136349,47.1062801 47.3653193,50.0077326 46.2423493,53.5394732 C45.1193793,57.0712139 45.5240245,60.9116046 47.3584105,64.1317416 L64.0908011,93.0734561 C67.7236263,99.3391772 75.708598,101.536031 82.0352554,98.0103982 L82.4491308,97.7738979 L82.1239429,98.0103982 L89.5441374,93.8420818 C97.8137349,88.7071757 105.627223,82.8717856 112.898534,76.4001905 C114.43778,75.0798243 116.710106,75.0798243 118.249352,76.4001905 L118.249352,76.4001905 C119.158022,77.1129332 119.691473,78.2016086 119.697915,79.3564433 C119.719135,80.4788344 119.255955,81.5559976 118.426727,82.312696 C110.76814,89.1108483 102.528036,95.2241548 93.8011413,100.582338 L93.5646411,100.582338 L86.1444467,104.721092 C83.0080654,106.498631 79.4617275,107.425978 75.8566871,107.411282 C68.2794178,107.411872 61.2698464,103.395151 57.4392324,96.8574596 L41.6232802,69.5121216 C11.2330018,90.9745166 -7.21401537,132.332493 2.68943136,184.805979 C3.05238462,186.613115 4.630317,187.91984 6.47343489,187.939608 L24.5065767,187.939608 C26.4460384,187.939608 28.0814398,186.494195 28.3201428,184.569479 C29.9827135,171.36939 41.2081637,161.469113 54.5125422,161.469113 C67.8169207,161.469113 79.0423709,171.36939 80.7049416,184.569479 C80.9574458,186.498798 82.6022985,187.939608 84.5480702,187.939608 L102.108212,187.939608 C104.047673,187.939608 105.683075,186.494195 105.921778,184.569479 C107.637793,171.405162 118.853268,161.556778 132.128958,161.556778 C145.404648,161.556778 156.620124,171.405162 158.336139,184.569479 C158.574842,186.494195 160.210243,187.939608 162.149705,187.939608 L179.502909,187.939608 C181.602533,187.939608 183.313735,186.254979 183.346037,184.155604 C183.759913,159.707393 190.352356,131.622992 209.154124,117.551229 C274.280372,68.8321835 257.163669,27.060332 242.08678,11.8947553 Z M175.65978,85.5941366 L163.243464,79.3564433 L163.243464,79.3564433 C163.229874,75.6716754 165.795264,72.4793993 169.39663,71.69974 C172.997995,70.9200808 176.653993,72.7654893 178.165574,76.1259689 C179.677154,79.4864485 178.632425,83.4462961 175.65978,85.6236991 L175.65978,85.5941366 Z"
              ></path>
            </g>
          </svg>
          <svg
            style="background-color: #6a4fba"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
          >
            <path
              d="M22.65 14.39L12 22.13 1.35 14.39a.84.84 0 0 1-.3-.94l1.22-3.78 2.44-7.51A.42.42 0 0 1 4.82 2a.43.43 0 0 1 .58 0 .42.42 0 0 1 .11.18l2.44 7.49h8.1l2.44-7.51A.42.42 0 0 1 18.6 2a.43.43 0 0 1 .58 0 .42.42 0 0 1 .11.18l2.44 7.51L23 13.45a.84.84 0 0 1-.35.94z"
            ></path>
          </svg>
          <svg
            style="background-color: #e14329"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
          >
            <path
              d="M9 19c-5 1.5-5-2.5-7-3m14 6v-3.87a3.37 3.37 0 0 0-.94-2.61c3.14-.35 6.44-1.54 6.44-7A5.44 5.44 0 0 0 20 4.77 5.07 5.07 0 0 0 19.91 1S18.73.65 16 2.48a13.38 13.38 0 0 0-7 0C6.27.65 5.09 1 5.09 1A5.07 5.07 0 0 0 5 4.77a5.44 5.44 0 0 0-1.5 3.78c0 5.42 3.3 6.61 6.44 7A3.37 3.37 0 0 0 9 18.13V22"
            ></path>
          </svg>
        </div>
        <img class="char" src="~/assets/images/right-arrow.svg" alt="logo" />
        <div>
          <h3>Publish</h3>
          <svg
            style="background-color: #4d9227"
            viewBox="0 0 512 514"
            fill="none"
          >
            <path
              fill-rule="evenodd"
              clip-rule="evenodd"
              d="M503.16 323.56C514.55 281.47 515.32 235.91 503.2 190.76C466.57 54.2299 326.04 -26.8001 189.33 9.77991C83.8101 38.0199 11.3899 128.07 0.689941 230.47H43.99C54.29 147.33 113.74 74.7298 199.75 51.7098C306.05 23.2598 415.13 80.6699 453.17 181.38L411.03 192.65C391.64 145.8 352.57 111.45 306.3 96.8198L298.56 140.66C335.09 154.13 364.72 184.5 375.56 224.91C391.36 283.8 361.94 344.14 308.56 369.17L320.09 412.16C390.25 383.21 432.4 310.3 422.43 235.14L464.41 223.91C468.91 252.62 467.35 281.16 460.55 308.07L503.16 323.56Z"
              fill="#fff"
            />
            <path
              d="M321.99 504.22C185.27 540.8 44.7501 459.77 8.11011 323.24C3.84011 307.31 1.17 291.33 0 275.46H43.27C44.36 287.37 46.4699 299.35 49.6799 311.29C53.0399 323.8 57.45 335.75 62.79 347.07L101.38 323.92C98.1299 316.42 95.39 308.6 93.21 300.47C69.17 210.87 122.41 118.77 212.13 94.7601C229.13 90.2101 246.23 88.4401 262.93 89.1501L255.19 133C244.73 133.05 234.11 134.42 223.53 137.25C157.31 154.98 118.01 222.95 135.75 289.09C136.85 293.16 138.13 297.13 139.59 300.99L188.94 271.38L174.07 231.95L220.67 184.08L279.57 171.39L296.62 192.38L269.47 219.88L245.79 227.33L228.87 244.72L237.16 267.79C237.16 267.79 253.95 285.63 253.98 285.64L277.7 279.33L294.58 260.79L331.44 249.12L342.42 273.82L304.39 320.45L240.66 340.63L212.08 308.81L162.26 338.7C187.8 367.78 226.2 383.93 266.01 380.56L277.54 423.55C218.13 431.41 160.1 406.82 124.05 361.64L85.6399 384.68C136.25 451.17 223.84 484.11 309.61 461.16C371.35 444.64 419.4 402.56 445.42 349.38L488.06 364.88C457.17 431.16 398.22 483.82 321.99 504.22Z"
              fill="#fff"
            />
          </svg>
        </div>
        <img class="char" src="~/assets/images/right-arrow.svg" alt="logo" />
        <div>
          <h3>Earn</h3>
          <svg
            style="background-color: #f3b433"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="4"
            stroke-linecap="round"
            stroke-linejoin="round"
          >
            <line x1="12" y1="1" x2="12" y2="23"></line>
            <path d="M17 5H9.5a3.5 3.5 0 0 0 0 7h5a3.5 3.5 0 0 1 0 7H6"></path>
          </svg>
        </div>
      </div>
    </div>
    <div class="slanted-hero columns">
      <div class="left hero">
        <h3>Easy to implement</h3>
        <h1>Backed by an open source API</h1>
        <p>
          Modrinth's code is fully open source licensed under AGPL. We've
          created a high peformant Rust based backend that is
          <span><nuxt-link to="/developers">fully documented</nuxt-link></span>
          for all kinds of tools to use. Our team is dedicated to maintaining an
          open source ecosystem for all Modrinth applications.
        </p>
      </div>
      <div class="right hero-image less-margin">
        <pre v-highlightjs>
<code class="javascript">const fetch = require('node-fetch');
fetch('https://api.modrinth.com/api/v1/mod').then(res => res.json()).then(data => {
    console.log(data);
    // hits: [Object {author: "mezz", author_url: …, …}, …]
    // limit: 10
    // offset: 0
    /// total_hits: 19440
});</code>
        </pre>
      </div>
    </div>
    <m-footer class="footer" centered />
  </div>
</template>

<script>
import MFooter from '@/components/MFooter'

export default {
  components: {
    MFooter,
  },
  auth: false,
  data() {
    return {
      currentText: 'Open source',
      texts: ['Open source', 'Easy to use', 'Developer focused', 'API Based'],
    }
  },
  beforeMount() {
    document.documentElement.setAttribute('data-theme', 'light')

    this.startNext(0)
  },
  methods: {
    startNext(i) {
      const startIndex = i % this.texts.length

      this.typeWriter(this.texts[startIndex], 0, () => {
        this.startNext(startIndex + 1)
      })
    },
    typeWriter(text, i, callback) {
      if (!text || i >= text.length) {
        setTimeout(callback, 1000 + Math.random() * 500)
        return
      }

      this.currentText = text.substring(0, i + 1)
      setTimeout(
        () => this.typeWriter(text, i + 1, callback),
        150 + Math.random() * 50
      )
    },
  },
}
</script>

<style lang="scss" scoped>
.left,
.right {
  width: 50%;
}

.main-hero {
  margin-top: 100px;
  height: 600px;

  .left {
    padding-top: 75px;
    padding-left: 100px;

    .typewriter {
      display: inline-block;
      color: var(--color-brand);

      span {
        border-right: 0.15em solid var(--color-brand);
        animation: caret 1s steps(1) infinite;

        @keyframes caret {
          50% {
            border-color: transparent;
          }
        }
      }
    }

    h1 {
      margin: 0;
      font-size: 4em;
    }
  }
  .right {
    padding-left: 20%;

    .char {
      height: 400px;
    }
  }
}

.hero {
  margin-left: 15%;
  width: 30%;
  z-index: 1;
  h3 {
    font-size: 18px;
    font-weight: bolder;
    color: var(--color-brand);
  }
  h1 {
    font-size: 46px;
    font-weight: 520;
  }
  p {
    line-height: 25px;
    letter-spacing: 0.2px;
    color: var(--color-text);

    span {
      color: var(--color-brand);
      font-weight: bold;
    }
  }
}

.hero-image {
  padding-left: 10%;
  padding-top: 75px;
  padding-right: 10%;

  img,
  video {
    height: 300px;
  }
}

.slanted-hero {
  background: var(--color-raised-bg);
  height: 500px;
  position: relative;
  z-index: 1;

  &:before,
  &:after {
    background: inherit;
    content: '';
    display: block;
    height: 50%;
    left: 0;
    position: absolute;
    right: 0;
    z-index: -1;
    -webkit-backface-visibility: hidden;
    backface-visibility: hidden;
  }

  &:before {
    top: 0;
    transform: skewY(5deg);
    transform-origin: 100% 0;
  }

  &:after {
    bottom: 0;
    transform: skewY(-5deg);
    transform-origin: 100%;
  }
}

.default-hero {
  margin-top: 200px;
  height: 700px;
}

.workflow {
  padding-top: 75px;
  padding-left: 10%;
  div {
    margin: 0 20px;
    text-align: center;
    width: 100px;
    h3 {
      margin: 0;
    }
    svg {
      color: #fff;
      border-radius: 50%;
      margin-top: 10px;
      padding: 25px;
    }
  }
  img {
    color: var(--color-text);
    align-self: flex-start;
  }
}

.footer {
  margin-top: 150px;
}

@media screen and (max-width: 500px) {
  .hero {
    margin-top: 0 !important;
  }
}

@media screen and (max-width: 1300px) {
  .workflow {
    display: none;
  }

  .right {
    display: none;
  }

  .left {
    padding-left: 0 !important;
    text-align: center;
    width: 100%;
  }

  .hero {
    margin-top: 100px;
    text-align: center;
    margin-left: 0;
    width: 100%;
  }
}

@media screen and (min-width: 1300px) and (max-width: 1500px) {
  .hero {
    margin-left: 10%;
  }
}
</style>
