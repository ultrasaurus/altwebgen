# Frontmatter

Frontmatter is an optional YAML block at the top of a markdown or handlebars
source file, between `---` lines:

```yaml
---
title: A Room of One's Own
author: Woolf, Virginia
date: 1929
layout: booklet
---
```

## Status
<= `v0.4.4` every value must be a string, site values override page values, ref frontmatter was ignored

new in `v0.5`: Typed attributes, page values override site values, and refs read `reader`

collections: sorting by date, `status` and `list`

data files: `<slug>.<name>.yaml` next to a page

TBD: sorting by author

## Where frontmatter is read

* **Pages** in `source/` (`.md`, `.md.hbs`, `.html.hbs`): every key becomes a
  template variable for the page and its layout
* **Refs** in `ref/` (`.md`): only the keys listed under [Refs](#refs) are
  used; any other key is ignored with a warning
* **Site-wide values** come from `source/_site.yaml` (optional) and are
  merged into every page's variables
  * a page's own value wins over a site value with the same key
  * `site_url` (e.g. `https://example.com`): the build also writes
    `sitemap.xml`, see [Sitemap](#sitemap)
* **Data files** next to a page, see [Data files](#data-files)
* **`baseurl`** is set by altwebgen, not frontmatter: the `--baseurl`
  option without its final `/`, so `""` for a site at the root and
  `/docs` for one served under `/docs/`. Use it for links in templates,
  e.g. `<a href="{{baseurl}}/about.html">`

## Data files

For data too long to keep in frontmatter, a page can have data files: a
file `<slug>.<name>.yaml` next to the page gives it the template variable
`<name>`, holding the whole file.

* e.g. `hello-world.reviews.yaml` next to `hello-world.md` (or
  `.md.hbs`, `.html.hbs`) gives that page `reviews`
* any YAML value: a list, a mapping, a string; templates use it as usual,
  `{{#each reviews}}`, `{{stats.views}}`
* a page can have several data files, each under its own name
* the page's frontmatter wins over a data file of the same name (with a
  warning); a data file wins over site values and collection defaults
* data files aren't published; a `.yaml` file with no page of that slug
  next to it is copied to the output as usual
* a file that isn't valid YAML is ignored with a warning
* only `.yaml`, not `.yml` or `.json`

`hello-world.reviews.yaml`:

```yaml
- author: First Reader
  url: https://example.com/first
  stars: 5
  text: <p>Clear and useful.</p>
```

and in the page's layout:

```handlebars
{{#each reviews}}
<p><a href="{{url}}">{{author}}</a>: {{stars}} stars</p>
{{{text}}}
{{/each}}
```

## Sitemap

With `site_url` in `_site.yaml`, each build writes `sitemap.xml` to the
output folder, listing every HTML page it builds:

```yaml
site_url: https://example.com
```

* each page at the URL it's served from: `site_url`, `--baseurl`,
  then the page's path, e.g. `https://example.com/2009/04/slug/` for a
  collection with `pretty_urls`
* `<lastmod>` is the page's `updated` frontmatter, or else its `date`,
  as precise as written down to the day (`2009-04-17`, `2009-04`); none if
  it has neither
* left out: pages with `sitemap: false`, drafts, and plain copied files
  (images, PDFs)
* a `sitemap.xml` in the source folder is copied as is instead (with a
  warning)
* `robots.txt` is up to the site: add one to the source folder with
  `Sitemap: https://example.com/sitemap.xml`

## Value types

* Any YAML value is allowed: string, number, boolean, list, map
* Values are passed to templates as is, so templates can use
  `{{#each tags}}`, `{{#if draft}}`, `{{photo.width}}` and so on
* Strings render exactly as before, including multi-line strings
  (`|` and `>` blocks; the last line has no trailing newline)
* `layout` must be a string; if it is missing (or isn't a string, with a
  warning), the `default` layout is used
* YAML has no date type, so dates are written as strings or numbers and
  interpreted by altwebgen (see [Dates](#dates))

## Reserved keys

These keys have meaning to altwebgen. Any other key is just a template
variable.

| key | used for | value |
|---|---|---|
| `layout` | which template wraps the page | template name, e.g. `booklet` |
| `title` | page title; sorting by title | string |
| `date` | display; sorting by date | see [Dates](#dates) |
| `author` | display; sorting by author | see [People](#people) |
| `reader` | refs: "Read by ..." next to the audio | see [People](#people) |
| `status` | anything other than `publish` is a draft, built only by `dev --drafts` (which sets `draft: true`); see [collections](collections.md#members) | string |
| `list` | a page that lists a collection | see [collections](collections.md#listing-pages) |
| `sitemap` | `false` leaves the page out of `sitemap.xml` | boolean |
| `updated` | `sitemap.xml` lastmod (else `date`) | see [Dates](#dates) |

A collection can also name a key to group by (e.g. `category`); see
[collections](collections.md).

## Dates

A date can be as precise as you know it to be:

| precision | example | default display |
|---|---|---|
| year | `date: 1984` | 1984 |
| month | `date: 1984-10` | October 1984 |
| day | `date: 2025-11-08` | November 8, 2025 |
| date and time | `date: '2003-03-03T15:17:38-08:00'` | March 3, 2003 |

* Quote a date with a time (`'...'`), since it contains `:`
* The UTC offset (`-08:00`) is optional; a time without an offset is
  read as-is, with no timezone conversion
* Use ISO order (year-month-day), not free text; `Nov. 8, 2025` is not a
  valid date
* An invalid date is still available to templates as the original string,
  but it sorts after all valid dates and gets a warning at build time

### Displaying dates

* `{{date}}` renders the value exactly as written in the frontmatter
* `{{date_format date}}` renders the default display for its precision
  (table above)
* `{{date_format date "%b %-d, %Y"}}` uses a custom
  [strftime](https://docs.rs/chrono/latest/chrono/format/strftime/index.html)
  format
  * if the format asks for more precision than the date has (e.g. a day
    for `1984-10`), the default display for that precision is used instead
  * likewise for a timezone (`%z`) on a time written without an offset
  * an invalid format string is a build error
* `{{date_format x}}` on a value that isn't a valid date renders it as
  written; on a missing value, renders nothing

### Sorting by date

Used by [collections](collections.md).

* Oldest first
* A less precise date sorts at the start of its period, before more
  precise dates in that period: `1984` < `1984-10` < `1984-10-01`
* Times sort by clock time as written; the UTC offset is only used for
  display
* Ties (e.g. two blog posts on the same day with no time, or the same
  timestamp) are broken by title, then by file path, so the order is
  always the same from build to build
* A missing or invalid date sorts after all valid dates

## People

Applies to `author` and `reader`.

* One person: a string
* More than one: a list
* Write a person's name as `Last, First`; write an organization's name
  as is (no comma)

```yaml
author: Vasquez, Maria
```

```yaml
author:
  - Smith, Mary Ann
  - Farmer, Jane
```

```yaml
author: The Daystrom Institute
```

* Only the first comma splits the name: `Last, First` is displayed as
  `First Last`
* Known limitation: a suffix like `King, Martin Luther, Jr.` can't be
  displayed correctly yet

### Displaying people

* `{{people author}}` renders display names joined in a sentence:

  | names | display |
  |---|---|
  | one | Mary Ann Smith |
  | two | Mary Ann Smith and Jane Farmer |
  | three or more | Mary Ann Smith, Jane Farmer, and Ada Lovelace |

* `{{person name}}` links one name to its page:
  `<a href='{baseurl}/people/{slug}'>First Last</a>`
  * the slug is made from the display name (`First Last`), so
    `Farmer, Jane` still links to `/people/jane-farmer`

### Sorting by author *(planned)*

* By the first author's name as written (`Last, First`), ignoring case
* Then by title, then by file path

## Refs

A ref is a markdown file in `ref/`, optionally with a matching audio file
and transcript, rendered into a partial that pages include with
`{{> ref/<name>.html }}`.

* `reader`: if the ref has audio, renders inside the audio block, after
  the player:

  ```html
  <div class='reader'>Read by Maria Vasquez</div>
  ```

  * uses the same name rules as `author` (string or list,
    `Last, First`)
  * ignored with a warning if the ref has no audio
* Any other key is ignored with a warning (`title`, `author` and `layout`
  belong on the page that includes the ref)
