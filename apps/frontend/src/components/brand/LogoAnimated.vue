<template>
  <div>
    <svg
      class="rotate outer"
      width="100%"
      height="100%"
      viewBox="0 0 590 591"
      xmlns="http://www.w3.org/2000/svg"
      xml:space="preserve"
      style="fill-rule: evenodd; clip-rule: evenodd; stroke-linejoin: round; stroke-miterlimit: 2"
    >
      <g transform="matrix(1,0,0,1,652.392,-0.400578)">
        <g transform="matrix(4.16667,0,0,4.16667,-735.553,0)">
          <g transform="matrix(0.24,0,0,0.24,0,0)">
            <path
              d="M134.44,316.535C145.027,441.531 249.98,539.829 377.711,539.829C474.219,539.829 557.724,483.712 597.342,402.371L645.949,419.197C599.165,520.543 496.595,590.954 377.711,590.954C221.751,590.954 93.869,469.779 83.161,316.535L134.44,316.535ZM83.946,265.645C99.012,116.762 224.88,0.401 377.711,0.401C540.678,0.401 672.987,132.71 672.987,295.677C672.987,321.817 669.583,347.168 663.194,371.313L614.709,354.529C619.381,335.689 621.862,315.971 621.862,295.677C621.862,160.926 512.461,51.526 377.711,51.526C253.133,51.526 150.223,145.03 135.392,265.645L83.946,265.645Z"
              style="fill: var(--color-brand)"
            />
          </g>
        </g>
      </g>
    </svg>
    <svg
      class="rotate inner"
      width="100%"
      height="100%"
      viewBox="0 0 590 591"
      xmlns="http://www.w3.org/2000/svg"
      xml:space="preserve"
      style="fill-rule: evenodd; clip-rule: evenodd; stroke-linejoin: round; stroke-miterlimit: 2"
    >
      <g transform="matrix(1,0,0,1,652.392,-0.400578)">
        <g transform="matrix(4.16667,0,0,4.16667,-735.553,0)">
          <g transform="matrix(0.24,0,0,0.24,0,0)">
            <path
              d="M376.933,153.568C298.44,153.644 234.735,217.396 234.735,295.909C234.735,374.47 298.516,438.251 377.077,438.251C381.06,438.251 385.005,438.087 388.914,437.764L403.128,487.517C394.611,488.667 385.912,489.261 377.077,489.261C270.363,489.261 183.725,402.623 183.725,295.909C183.725,189.195 270.363,102.557 377.077,102.557C379.723,102.557 382.357,102.611 384.983,102.717L376.933,153.568ZM435.127,111.438C513.515,136.114 570.428,209.418 570.428,295.909C570.428,375.976 521.655,444.742 452.22,474.093L438.063,424.541C486.142,401.687 519.418,352.653 519.418,295.909C519.418,234.923 480.981,182.843 427.029,162.593L435.127,111.438Z"
              style="fill: var(--color-brand)"
            />
          </g>
        </g>
      </g>
    </svg>
    <svg
      width="100%"
      height="100%"
      viewBox="0 0 590 591"
      xmlns="http://www.w3.org/2000/svg"
      xml:space="preserve"
      style="fill-rule: evenodd; clip-rule: evenodd; stroke-linejoin: round; stroke-miterlimit: 2"
    >
      <g transform="matrix(1,0,0,1,652.392,-0.400578)">
        <g transform="matrix(4.16667,0,0,4.16667,-735.553,0)">
          <g transform="matrix(0.24,0,0,0.24,0,0)">
            <path
              d="M300.366,311.86L283.216,266.381L336.966,211.169L404.9,196.531L424.57,220.74L393.254,252.46L365.941,261.052L346.425,281.11L355.987,307.719L375.387,328.306L402.745,321.031L422.216,299.648L464.729,286.185L477.395,314.677L433.529,368.46L360.02,391.735L327.058,355.031L138.217,468.344C129.245,456.811 118.829,440.485 112.15,424.792L300.366,311.86Z"
              style="fill: var(--color-brand)"
            />
          </g>
        </g>
        <g transform="matrix(4.16667,0,0,4.16667,-735.553,0)">
          <g transform="matrix(0.24,0,0,0.24,0,0)">
            <path
              d="M655.189,194.555L505.695,234.873C513.927,256.795 516.638,269.674 518.915,283.863L668.152,243.609C665.764,227.675 661.5,211.444 655.189,194.555Z"
              style="fill: var(--color-brand)"
            />
          </g>
        </g>
      </g>
    </svg>
  </div>
</template>

<style lang="scss" scoped>
div {
  display: flex;
  justify-content: center;
  align-items: center;
  height: 5rem;
  margin-top: 1rem;

  svg {
    width: 5rem;
    height: 5rem;
    position: absolute;
    &.rotate {
      animation: rotate 4s infinite linear;
      &.inner {
        animation: rotate 6s infinite linear reverse;
      }
    }
    @keyframes rotate {
      0% {
        transform: rotate(0);
      }
      100% {
        transform: rotate(360deg);
      }
    }
  }
}
</style>
