# Collections

A collection is a set of pages that know about each other, such as blog
posts. Each page in a collection gets prev/next links, and other pages can
list the collection (an archive, a category page).

> new in v0.5. Each collection is read on every build;
> the whole site is rebuilt each time
> (677 posts build in about 0.25s; 0.37s with a collection and an archive page)

## Defining a collection

Put a `collection.yaml` in the source folder (or any folder under it):

```yaml
name: posts
include: "[0-9][0-9][0-9][0-9]/**/*.md"
sort: date
group: category
defaults:
  layout: post
  category: general
pretty_urls: true
```

| key | required | meaning |
|---|---|---|
| `name` | yes | how listing pages refer to the collection; must be unique |
| `include` | yes | [glob](https://docs.rs/globset/latest/globset/#syntax) for member files, relative to the folder `collection.yaml` is in |
| `sort` | no | frontmatter key to sort by; only `date` is supported (the default) |
| `group` | no | frontmatter key whose value splits prev/next into separate chains, e.g. one per category |
| `defaults` | no | frontmatter values a member gets when its own frontmatter doesn't set them |
| `pretty_urls` | no | `true` writes `2009/04/slug.md` to `2009/04/slug/index.html`, linked as `/2009/04/slug/`; default `false` (`2009/04/slug.html`) |

* Members are markdown files (`.md`, `.md.hbs`) matching `include`
* A file can belong to only one collection
* `collection.yaml` (like `series.yaml` and `_site.yaml`) is not copied to
  the output

## Members

* Variables for a member page, in order (later wins):
  `_site.yaml`, then the collection's `defaults`, then the page's own
  frontmatter, then prev/next
* **Drafts:** a member with a `status` other than `publish` is left out of
  the collection and not built; a member with no `status` is published
* **Sort order:** by date, oldest first, following the
  [date sorting rules](frontmatter.md#sorting-by-date)
* **Groups:** with `group: category`, each member's prev/next stays among
  members with the same `category` value
  * the value must be a string; a member whose value isn't a string gets a
    warning and no prev/next

### Prev/next

A member page gets:

| variable | value |
|---|---|
| `prev` | URL of the previous (older) member, missing on the first |
| `prev-title` | its title |
| `next` | URL of the next (newer) member, missing on the last |
| `next-title` | its title |

* URLs start at the site root (`/2009/04/slug/`), with the `--prefix`
  option if one is set
* The names match [series](../samples/series) navigation, so one nav
  partial can serve both; if a page is in a series too, the series wins

## Listing pages

Any page can list a collection by naming it under `list` in frontmatter:

```yaml
---
title: Code
list:
  collection: posts
  filter:
    category: code
  order: newest
---
```

* `list: posts` is short for `list: {collection: posts}`
* `filter`: only members whose frontmatter has these exact values
* `order`: `oldest` (default) or `newest`
* `limit`: keep only the first this many, after ordering
* `as`: name the variables something other than the collection name

The page gets two variables, named after the collection (or `as`):

* `posts`: the members, each with its frontmatter (after defaults) and
  `url`
* `posts_by_month`: the same members grouped by month, each with `month`
  (`YYYY-MM`) and `posts` (that month's members); members with no date, or
  a date that is only a year, are left out

```handlebars
{{#each posts_by_month}}
<h2>{{date_format month}}</h2>
<ul>
  {{#each posts}}
  <li><a href="{{url}}">{{title}}</a></li>
  {{/each}}
</ul>
{{/each}}
```

To show several lists on one page, give `list` a sequence of them, each
with its own `as`:

```yaml
---
title: Home
list:
  - collection: posts
    filter: {category: general}
    order: newest
    limit: 5
    as: recent
  - collection: posts
    filter: {category: code}
    order: newest
    limit: 5
    as: notebook
---
```

The page gets `recent`, `recent_by_month`, `notebook` and
`notebook_by_month`; inside `recent_by_month`, each month's members are
under `recent`.

* A listing page doesn't need to be in the collection
* Naming a collection that doesn't exist is a warning, and the page gets
  no list
