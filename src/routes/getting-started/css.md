---
title: Writing CSS
---

## Conventions

### Avoid inconsistent CSS units

Prefer using `rem` units, using only whole and half units, eg. `2rem` or `1.5rem`. If you need a specific pixel (`px`) measurement, use `px` and add comment explaining why you used it. The one exception is that `0.25` is allowed.

> Using `rem` units lets you change the scale of the UI by simply changing the body font size.

### Always use `HSL` colors

### All colors should be theme variables

## Abilities

Omorphia leverages PostCSS to allow you to write in future-standards-compliant CSS. Browse [the CSSWG drafts](https://cssdb.org/) to see what is possible (not including stage 0).

Notable features:

- [Nesting](https://www.w3.org/TR/css-nesting-1/#example-aecb8796)
- [Gap](https://developer.mozilla.org/en-US/docs/Web/CSS/gap)
- [`clamp` function](<https://developer.mozilla.org/en-US/docs/Web/CSS/clamp()>)
- [Custom Media Queries](https://www.w3.org/TR/mediaqueries-5/#example-532b0adb)
- [`:has()`](https://developer.mozilla.org/en-US/docs/Web/CSS/:has)
- [place-content](https://developer.mozilla.org/en-US/docs/Web/CSS/place-content)

## Styles

Conform to [BEM styling](http://getbem.com/introduction/) wherever possible. When working in components, you may want to leverage [Svelte's conditional class shorthand](https://svelte.dev/tutorial/class-shorthand) instead of BEM's modifier class name format.
