<script lang="ts">
    export let as: 'button' | 'a' | 'summary' | 'input'  = 'button'
    export let href: string
    if (href) as = 'a'

    // Use `value` if the button is an `<input`>
    export let value: string;

    export let size: 'sm' | 'md' | 'lg' = 'md'
    export let color: 'outline' | 'primary' | 'danger';

    let className = `btn btn--${size}`;
    className += color && (` btn--${color}`)
</script>

{#if as === 'button'}
    <button class={className}>
        <slot />
    </button>
{:else if as === 'a'}
    <a class={className} {href}>
        <slot />
    </a>
{:else if as === 'summary'}
    <summary class={className}>
        <slot />
    </summary>
{:else if as === 'input'}
    <input class={className} {value} />
{/if}

<style lang="postcss">
  /* Base button styles */
  .btn {
      position: relative;
      display: inline-block;
      padding: 5px var(--spacer-3);
      font-size: var(--body-font-size);
      font-weight: var(--font-weight-semibold);
      line-height: 20px; /* Specifically not inherit our `<body>` default */
      white-space: nowrap;
      vertical-align: middle;
      cursor: pointer;
      user-select: none;
      border: var(--border-width) var(--border-style);
      border-radius: var(--radii-2);
      appearance: none; /* Corrects inability to style clickable `input` types in iOS. */

      &:hover {
          text-decoration: none;
      }

      &:disabled,
      &.disabled,
      &[aria-disabled='true'] {
          cursor: default;
      }

      i {
          font-style: normal;
          font-weight: var(--font-weight-semibold);
          opacity: 0.75;
      }

      /*
    .icon {
      margin-right: $spacer-1;
      color: var(--color-fg-muted);
      vertical-align: text-bottom;

      &:only-child {
        margin-right: 0;
      }
    }

    .Counter {
      margin-left: 2px;
      color: inherit;
      text-shadow: none;
      vertical-align: top;
      background-color: var(--color-btn-counter-bg);
    }

    .dropdown-caret {
      margin-left: $spacer-1;
      opacity: 0.8;
    }
  */
  }

  /* Default button */
  .btn {
    color: var(--color-btn-text);
    background-color: var(--color-btn-bg);
    border-color: var(--color-btn-border);
    box-shadow: var(--color-btn-shadow), var(--color-btn-inset-shadow);
    transition: 0.2s cubic-bezier(0.3, 0, 0.5, 1);
    transition-property: color, background-color, border-color;

    &:hover,
    &.hover,
    [open] > & {
      background-color: var(--color-btn-hover-bg);
      border-color: var(--color-btn-hover-border);
      transition-duration: 0.1s;
    }

    &:active {
      background-color: var(--color-btn-active-bg);
      border-color: var(--color-btn-active-border);
      transition: none;
    }

    &.selected,
    &[aria-selected='true'] {
      background-color: var(--color-btn-selected-bg);
      box-shadow: var(--color-primer-shadow-inset);
    }

    &:disabled,
    &.disabled,
    &[aria-disabled='true'] {
      color: var(--color-primer-fg-disabled);
      background-color: var(--color-btn-bg);
      border-color: var(--color-btn-border);

      .octicon {
        color: var(--color-primer-fg-disabled);
      }
    }

    /* Keep :focus after :disabled. Allows to see the focus ring even on disabled buttons */
    &:focus,
    &.focus {
      border-color: var(--color-btn-focus-border);
      outline: none;
      box-shadow: var(--color-btn-focus-shadow);
    }
  }

  /* Primary button */
  .btn--primary {
    color: var(--color-btn-primary-text);
    background-color: var(--color-btn-primary-bg);
    border-color: var(--color-btn-primary-border);
    box-shadow: var(--color-btn-primary-shadow), var(--color-btn-primary-inset-shadow);

    &:hover,
    &.hover,
    [open] > & {
      background-color: var(--color-btn-primary-hover-bg);
      border-color: var(--color-btn-primary-hover-border);
    }

    &:active,
    &.selected,
    &[aria-selected='true'] {
      background-color: var(--color-btn-primary-selected-bg);
      box-shadow: var(--color-btn-primary-selected-shadow);
    }

    &:disabled,
    &.disabled,
    &[aria-disabled='true'] {
      color: var(--color-btn-primary-disabled-text);
      background-color: var(--color-btn-primary-disabled-bg);
      border-color: var(--color-btn-primary-disabled-border);

      .octicon {
        color: var(--color-btn-primary-disabled-text);
      }
    }

    &:focus,
    &.focus {
      background-color: var(--color-btn-primary-focus-bg);
      border-color: var(--color-btn-primary-focus-border);
      box-shadow: var(--color-btn-primary-focus-shadow);
    }

    .Counter {
      color: inherit;
      background-color: var(--color-btn-primary-counter-bg);
    }

    .octicon {
      color: var(--color-btn-primary-icon);
    }
  }

  /* Outline button */
  .btn--outline {
    color: var(--color-btn-outline-text);

    &:hover,
    [open] > & {
      color: var(--color-btn-outline-hover-text);
      background-color: var(--color-btn-outline-hover-bg);
      border-color: var(--color-btn-outline-hover-border);
      box-shadow: var(--color-btn-outline-hover-shadow), var(--color-btn-outline-hover-inset-shadow);

      .Counter {
        background-color: var(--color-btn-outline-hover-counter-bg);
      }

      .octicon {
        color: inherit;
      }
    }

    &:active,
    &.selected,
    &[aria-selected='true'] {
      color: var(--color-btn-outline-selected-text);
      background-color: var(--color-btn-outline-selected-bg);
      border-color: var(--color-btn-outline-selected-border);
      box-shadow: var(--color-btn-outline-selected-shadow);
    }

    &:disabled,
    &.disabled,
    &[aria-disabled='true'] {
      color: var(--color-btn-outline-disabled-text);
      background-color: var(--color-btn-outline-disabled-bg);
      border-color: var(--color-btn-border);
      box-shadow: none;

      .Counter {
        background-color: var(--color-btn-outline-disabled-counter-bg);
      }
    }

    &:focus {
      border-color: var(--color-btn-outline-focus-border);
      box-shadow: var(--color-btn-outline-focus-shadow);
    }

    .Counter {
      color: inherit;
      background-color: var(--color-btn-outline-counter-bg);
    }
  }

  /* Danger button */
  .btn--danger {
    color: var(--color-btn-danger-text);

    .octicon {
      color: var(--color-btn-danger-icon);
    }

    &:hover,
    [open] > & {
      color: var(--color-btn-danger-hover-text);
      background-color: var(--color-btn-danger-hover-bg);
      border-color: var(--color-btn-danger-hover-border);
      box-shadow: var(--color-btn-danger-hover-shadow), var(--color-btn-danger-hover-inset-shadow);

      .Counter {
        background-color: var(--color-btn-danger-hover-counter-bg);
      }

      .octicon {
        color: var(--color-btn-danger-hover-icon);
      }
    }

    &:active,
    &.selected,
    &[aria-selected='true'] {
      color: var(--color-btn-danger-selected-text);
      background-color: var(--color-btn-danger-selected-bg);
      border-color: var(--color-btn-danger-selected-border);
      box-shadow: var(--color-btn-danger-selected-shadow);
    }

    &:disabled,
    &.disabled,
    &[aria-disabled='true'] {
      color: var(--color-btn-danger-disabled-text);
      background-color: var(--color-btn-danger-disabled-bg);
      border-color: var(--color-btn-border);
      box-shadow: none;

      .Counter {
        background-color: var(--color-btn-danger-disabled-counter-bg);
      }

      .octicon {
        color: var(--color-btn-danger-disabled-text);
      }
    }

    &:focus {
      border-color: var(--color-btn-danger-focus-border);
      box-shadow: var(--color-btn-danger-focus-shadow);
    }

    .Counter {
      color: inherit;
      background-color: var(--color-btn-danger-counter-bg);
    }
  }

  /* Sizes */
  .btn--sm {
    padding: 3px 12px;
    font-size: var(--font-0);
    line-height: 20px;

      /*
    .octicon {
      vertical-align: text-top;
    }
    */
  }

  .btn--lg {
    /* padding: $em-spacer-6 1.5em; */
    /* line-height: $lh-default; */

    font-size: var(--font-2);
    border-radius: 0.5em;
  }

  /* Full-width button */
  /* These buttons expand to the full width of their parent container */
  .btn-block {
    display: block;
    width: 100%;
    text-align: center;
  }
</style>
