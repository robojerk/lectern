# Media Management Template (Naming)

The **Media Management Template** controls how converted audiobooks are organized when you use a **Local Library** path (Settings → Local Library). You type a path pattern using placeholders; the app fills them from the current book metadata and saves the file there.

---

## Placeholders (variables)

| Placeholder       | Source              | If empty |
|-------------------|---------------------|----------|
| `{Author}`        | Author(s)           | Replaced with `Unknown Author` |
| `{Title}`         | Book title          | Replaced with `Unknown Title` |
| `{Series}`         | Series name         | Empty string; segment is dropped (see rules below) |
| `{SeriesNumber}`  | Series number/order | Empty string; segment is dropped |
| `{Year}`          | Publish year        | Empty string; segment is dropped |
| `{Genre}`         | Genre(s)            | Empty string; segment is dropped |
| `{ASIN}`          | Audible ASIN        | Empty string; segment is dropped |
| `{Language}`      | Language            | Empty string; segment is dropped |
| `{Tags}`          | Tags (comma-separated) | Empty string; segment is dropped |

---

## Rules

### 1. Sanitization

Values are sanitized for use in paths:

- Any `/` or `\` in a value is replaced with `-`.
- Leading and trailing whitespace is trimmed.

So `Author/Name` becomes `Author-Name` in the path.

### 2. Empty optional fields (Series, SeriesNumber, Year, Genre, ASIN, Language, Tags)

If a placeholder is empty, it is replaced with an empty string. The path is then **normalized**:

- Repeated slashes are collapsed to a single separator.
- Leading/trailing slashes in the expanded part are trimmed.

So a template like `{Author}/{Series}/{Title}.m4b` with no series does **not** produce `Author//Title.m4b`; it becomes `Author/Title.m4b`. Empty segments are effectively omitted.

### 3. SeriesNumber optional suffix

You can add **one** character immediately after `SeriesNumber` inside the braces. That character is used as a suffix and is **only included when SeriesNumber has a value**.

| Template            | SeriesNumber = `4`     | SeriesNumber empty   |
|---------------------|------------------------|----------------------|
| `{SeriesNumber-}`    | `4-`                   | (nothing)            |
| `{SeriesNumber.}`    | `4.`                   | (nothing)            |
| `{SeriesNumber }`    | `4 ` (space)           | (nothing)            |
| `{SeriesNumber}`     | `4`                    | (nothing)            |

Any single character is allowed (e.g. `{SeriesNumber -}` uses a space as the suffix). The suffix character is the one right before the closing `}`.

### 4. Tags

`{Tags}` is filled from the metadata **Tags** field, which is typically comma-separated (e.g. `Science Fiction, Adventure`). The value is sanitized the same way as other fields (`/` and `\` → `-`); commas are left as-is, so a path segment may look like `Science Fiction, Adventure`. Long or many tags can produce long path segments. If you prefer a single category, use `{Genre}` instead, or keep tags short. (Alternative behaviour for Tags—e.g. replacing commas with a hyphen or using only the first tag—could be added later if needed.)

### 5. Path assembly

- The **Local Library** path is trimmed of trailing slashes/backslashes.
- The expanded template (after placeholder replacement and slash normalization) is joined to it using the platform path separator (`/` on Unix, `\` on Windows).
- The output directory (including any `Author`/`Series` subdirs) is created automatically when you convert.

---

## Examples

**Author and title only**

- Template: `{Author}/{Title}.m4b`
- Library: `/home/you/Music/Audiobooks/`
- Result: `/home/you/Music/Audiobooks/Sir Arthur Conan Doyle/When the World Screamed.m4b`

**With series (no number)**

- Template: `{Author}/{Series}/{Title}.m4b`
- Result: `.../Sir Arthur Conan Doyle/Professor Challenger/When the World Screamed.m4b`
- If the book has no series, result: `.../Sir Arthur Conan Doyle/When the World Screamed.m4b` (no double slash).

**With series and number + hyphen**

- Template: `{Author}/{Series}/{SeriesNumber-}{Title}.m4b`
- SeriesNumber = `4` → `.../Professor Challenger/4-When the World Screamed.m4b`
- SeriesNumber empty → `.../Professor Challenger/When the World Screamed.m4b`

**With series and number + dot**

- Template: `{Author}/{Series}/{SeriesNumber.}{Title}.m4b`
- SeriesNumber = `4` → `.../Professor Challenger/4.When the World Screamed.m4b`

**With year**

- Template: `{Author}/{Year}/{Title}.m4b`
- Result: `.../Sir Arthur Conan Doyle/2022/When the World Screamed.m4b`
- If year is empty, result: `.../Sir Arthur Conan Doyle/When the World Screamed.m4b`

**With genre**

- Template: `{Author}/{Genre}/{Title}.m4b`
- Result: `.../Sir Arthur Conan Doyle/Science Fiction/When the World Screamed.m4b`
- If genre is empty, result: `.../Sir Arthur Conan Doyle/When the World Screamed.m4b`

**With ASIN**

- Template: `{Author}/{ASIN}-{Title}.m4b`
- Result: `.../Sir Arthur Conan Doyle/B09XNZ18M7-When the World Screamed.m4b`
- Useful for keeping a stable ID in the filename.

**With language**

- Template: `{Author}/{Language}/{Title}.m4b`
- Result: `.../Sir Arthur Conan Doyle/English/When the World Screamed.m4b`

---

## Where it’s used

- **Settings** tab: “Media Management Template” — edit the template and see the placeholder hint.
- **Convert** tab: “Output Location” shows the resolved path when Local Library is set (same template + current metadata).
