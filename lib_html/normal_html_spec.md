# Attributes

All HTML element could have `class` and `id`.

- `class` attribute value must be set of space-separated tokens, representing the various classes that the element belongs to
- `id` attribute value must be unique, must contain at least one character and not contain any ASCII whitespace

# Content models

Each element defined in this specification has a content model: a description of the element's expected contents. An HTML element must have contents that match the requirements described in the element's content model. The contents of an element are its children in the DOM.

Authors must not use HTML elements anywhere except where they are explicitly allowed.

## "nothing" content model

When an element's content model is nothing, the element must contain no `Text` nodes (other than inter-element whitespace) and no element nodes.

## Flow

TODO

## Heading (Flow)

h1, h2, h3, h4, h5, h6, hgroup

## Sectioning (Flow)

article, aside, nav, section

## Embedded (Phrasing)

audio, canvas, embed, iframe, img, math, object, picture, svg, video

## Phrasing (Flow)

TODO

## Interactive (Flow)

always: button, details, embed, iframe, keygen, label, select, textarea
under certain circumstances: a, audio, img, input, object, video

## Metadata

base, link, meta, style, title

## Palpable content

As a general rule, elements whose content model allows any `flow content` or `phrasing content` should have at least one node in its contents that is palpable content and that does not have the `hidden` attribute specified.

## Select element inner content

option, optgroup, hr, div

## Optgroup element inner content

option, div

## Option element inner content

div, `phrasing content` without datalist, object, `interactive content`

# Elements
