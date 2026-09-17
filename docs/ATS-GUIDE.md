# ATS Guide — How Applicant Tracking Systems Actually Read a Resume

A working reference for building resumes with RVM. Written from the parser's
point of view: what the software receives, what breaks it, and what genuinely
does not matter. Every claim marked as tested was verified by extracting the
text layer of a real PDF with `pdftotext`.

**The one-line summary:** an ATS never sees your resume. It receives the *text
layer* of your file, in the *order* the file hands it over, and fills a database
from that. Design decisions that look tidy to a human can scramble that order.
Almost everything that matters is invisible in a PDF viewer.

---

## 1. What actually happens when you submit

```
your file  ->  text extraction  ->  field mapping  ->  keyword index  ->  ranking
             (the text layer)     (name, email,    (searchable)        (recruiter
                                   jobs, dates,                         searches)
                                   skills)
```

Each stage can lose you, but they fail differently:

| Stage | Typical failure |
|---|---|
| Extraction | Text in the wrong **order**; icon glyphs become garbage; a scanned page yields nothing |
| Field mapping | Dates detach from their job title; a sidebar's skills land inside your work history |
| Keyword index | A word is present but not *matchable* (ligatures, exotic characters) |
| Ranking | Genuine mismatch, or keyword stuffing that reads as noise |

**Reading order is the thing most people underestimate.** A PDF has no concept
of columns or sections — it is text placed at coordinates. The parser rebuilds
order from position, and for side-by-side content it tends to sweep left to
right across the whole page. The words are all there; the *structure* is gone.
That is why a two-column resume is not "rejected" but *quietly downgraded*
([ATS Verification benchmark](https://atsverification.com/blog/two-column-resume-ats-friendly/)).

---

## 2. What actually breaks parsing (ranked by real impact)

### 2.1 Two-column layouts — the single biggest structural risk

In a controlled six-layout test, the two-column version was the **only** layout
to draw a critical flag and the only one to lose points, falling from a clean
100 to 85. Single-column parsed at 100. The mechanism is reading order, not lost
text: your sidebar gets interleaved into your work history, and dates separate
from their jobs.

Two-column LaTeX templates (AltaCV, Deedy, Awesome-CV sidebars) scramble exactly
the same way — LaTeX does not rescue you.

**Fix:** single column, full width, top to bottom.

### 2.2 Icon fonts — they extract as garbage

FontAwesome envelope/phone/GitHub glyphs are characters from a symbol font with
no meaningful Unicode mapping. They extract as stray characters glued to your
contact details, and some parsers then fail to detect those details at all.

**Tested on a real resume.** The header compiled fine and looked polished, but
the extracted text was:

```
a github.com/manohar-nalluri | ] manohar-nalluri | c manoharnalluri47@gmail.com | x +91 93900 93641
```

`a`, `]`, `ć`, `×` are the icon glyphs. Note the LinkedIn entry had also lost its
domain, so a parser saw a bare username rather than a profile URL.

**Fix:** delete the icons and write the values, or label them. After the fix:

```
github.com/manohar-nalluri | linkedin.com/in/manohar-nalluri | manoharnalluri47@gmail.com | +91 93900 93641
```

Always write full URLs, including the domain.

### 2.3 Ligatures — a real trap, but engine-dependent

TeX typesets `fi`, `fl`, `ff`, `ffi`, `ffl` as single glyphs. If the PDF lacks a
proper glyph-to-character map, "Financial" extracts as `F-ﬁ-nancial` and a
keyword search for "financial" misses it ([source](https://atsverification.com/blog/latex-resume-ats-friendly/)).

**This must be tested, not assumed.** On this machine's XeTeX-based pipeline
(tectonic) ligatures extracted correctly — `offline`, `conflict`, `field`,
`Flutter` all came through intact, with no ligature codepoints present.

**Fixes if you do see breakage:** pdflatex users add `\usepackage[T1]{fontenc}`,
`\usepackage{lmodern}`, `\usepackage{cmap}`; or disable ligatures outright with
`\usepackage{microtype}` + `\DisableLigatures{encoding = *, family = *}`; or on
fontspec engines, `\setmainfont{...}[Ligatures=NoCommon]`.

### 2.4 Text in headers, footers, and text boxes

Content placed in a page header/footer or a floating box is often skipped or
attached to the wrong page. `fullpage` + `fancyhdr` with the contact line in the
**body** is fine; putting it in the `\fancyhead` machinery is not.

### 2.5 Images and text-in-graphics

Any text rendered as an image is invisible to extraction. A scanned or
image-exported resume can yield an empty text layer — the file is not "rejected",
it is simply unreadable.

**Five-second test:** if you cannot select the text in a PDF viewer, a parser
cannot read it either.

---

## 3. What does **not** matter (common myths)

The same benchmark found these did **not** break parsing:

- **Em dashes and curly quotes** — parsed cleanly. LaTeX turns `--` into an en
  dash and `'` into a right single quote by default; that is acceptable.
- **Skills grids / comma-separated skill lines** — fine, as long as they are
  plain text in a single column, not a table or a sidebar.
- **Bold and italics** for emphasis — ATS reads text, not styling. Heavy bolding
  does not "break" anything, but see §6 on why keyword-stuffing still hurts.
- **A single `tabular*` used as a tab stop** for right-aligned dates — this is
  the *recommended* pattern, not a table problem. Dates on the same line as the
  job title, pushed right by a fill glue, is exactly what parsers want.
- **A clean, well-formed LaTeX PDF** — these generally parse *better* than Word
  exports: real text layer, embedded fonts, single column, plain headings.

---

## 4. Section headings

Parsers classify sections by matching heading text. Use boring, standard names:

| Use | Avoid |
|---|---|
| `Summary` / `Professional Summary` | `About Me`, `Profile Snapshot` |
| `Experience` / `Work Experience` | `Where I've Been`, `Career Journey` |
| `Skills` / `Technical Skills` | `My Toolbox`, `Superpowers` |
| `Projects` | `Things I Built` |
| `Education` | `Academics` |
| `Certifications` | `Badges` |

Keep headings as plain text — not small caps in a custom font, not a graphic,
not an image. Recognisable headings are what allow the parser to map the right
block to the right field.

---

## 5. Writing bullets that work for both machine and human

### The XYZ formula

Google's recruiters popularised it ([Laszlo Bock](https://law.utexas.edu/wp-content/uploads/sites/44/2020/09/Google-Recruiters-Say-Using-the-X-Y-Z-Formula-on-Your-Resume-Will-Improve-Your-Odds-of-Getting-Hired-at-Google-_-Inc.com_.pdf)):

> Accomplished **[X]** as measured by **[Y]**, by doing **[Z]**.

Example: *"Cut p95 latency 40% (Y) by replacing per-client polling with a shared
subscription room (Z), serving 3 tenants from one feed (X)."*

### Rules

1. **Start with a strong verb**, past tense for past work: Built, Designed,
   Implemented, Reduced, Migrated, Automated, Owned. Avoid "Responsible for",
   "Worked on", "Helped with".
2. **Never invent a number.** A bullet with a fabricated metric is a liability in
   the interview and a falsehood on the page. If you lack the figure, describe
   the *specific mechanism* instead — "grouped subscribers into shared rooms so
   clients reuse one upstream subscription" is strong without any number.
3. **Prefer mechanism over adjectives.** "Optimised performance" says nothing;
   "grouped subscribers into shared rooms" says exactly what you did.
4. **Length: 15–25 words per bullet.** Past ~30 words the reader skims and the
   parser adds nothing.
5. **One idea per bullet.** Two claims in one bullet get read as neither.
6. **Name the technology where it is real and load-bearing**, not as padding.
7. **Keep it truthful and specific.** Depth on two real systems beats a list of
   ten tools you have touched once.

### A weak-vs-strong example

> Weak: *"Worked on a full-stack platform using React, Node.js, and databases,
> improving performance by 40%."*

> Strong (no invented number): *"Streamed live gold prices to clients over
> WebSockets and grouped subscribers into shared rooms, so every client watching
> the same instrument reuses one upstream subscription instead of holding its
> own."*

---

## 6. Keywords: match the JD, do not stuff

- **Tailoring beats volume.** Mirror the *job description's* vocabulary for
  things you genuinely did. If the JD says "REST APIs" and you wrote "web
  services", use their phrasing.
- **Exact string matching still matters.** Some systems do literal matching, so
  a word that fails to extract (see ligatures §2.3) is invisible. This is also
  why a skills section is worth having as plain comma-separated text.
- **Keyword stuffing backfires.** Repeating a term or bolding every third word
  does not raise a match score; it makes the resume read as noise to the human
  who eventually opens it, and modern systems weight relevance, not raw counts
  ([Jobscan on false "missing skill" results](https://support.jobscan.co/hc/en-us/articles/41334900375571-Why-do-my-results-say-I-m-missing-skills-when-they-re-already-on-my-resume)).
- **Acronyms:** give both forms once — "Continuous Integration (CI)",
  "Application Programming Interface (API)" — so either search hits.
- **Do not claim what you cannot defend.** An inflated keyword that gets you the
  screen will fail you in the interview.

---

## 7. File format

- **PDF from a real text layer** is the safe default, and what RVM compiles.
- **DOCX** parses reliably in many systems when they demand Word.
- **Never** submit a scan or an image-only export.
- Keep the file name professional: `ManoharNalluri.pdf`, not `resume_final_v3.pdf`.

---

## 8. Exactly one page, with no dead space

Two failure modes: spilling a few lines onto page 2, or leaving a visible gap at
the bottom. Both are fixable, and the second one has a trick worth knowing.

### The `\raggedbottom` trap

LaTeX's `\raggedbottom` lets a page be *shorter* than the text block, which
leaves a dead gap at the foot. `\flushbottom` instead stretches the existing
vertical glue so the body ends flush with the bottom margin.

### `\flushbottom` needs stretchable glue

`\flushbottom` can only distribute space that is *stretchable*. If you zero out
list spacing with rigid glue, there is nothing to stretch and the gap remains:

```latex
% Rigid: \flushbottom has nothing to work with
\setlength{\itemsep}{0pt}

% Stretchable: \flushbottom spreads the slack evenly between bullets
\setlength{\itemsep}{0pt plus 6pt}
\setlength{\parsep}{0pt plus 1pt}
```

Set these with `\setlength` **after** `\begin{itemize}`, not through enumitem's
`itemsep=` key — the key drops the rubber component and the stretch silently
does nothing.

### Fitting the content

Order of operations that works:

1. Write the content with real substance. Do not pad, do not cut truth.
2. Set print-safe margins first — see below.
3. If it spills, tighten spacing (per-item `\vspace`, section title `\vspace`)
   before deleting content.
4. Let `\flushbottom` close the final gap.

### Print-safe margins

Text closer than about **0.25 in (18 pt)** to any edge can be clipped by printers.
Measure it, do not eyeball it. A resume whose name sits 0.11 in from the top edge
looks fine on screen and clips in print.

---

## 9. Verification: run this before every application

The only reliable check is to read the text layer. `pdftotext` (poppler) gives
exactly what a parser sees.

```bash
# 1. What does the parser receive? Read the first screen.
pdftotext -layout resume.pdf - | head -40
```

Check: name on line 1 · contact line clean with full URLs · job title and its
dates on the same line · section headings as plain words.

```bash
# 2. Any garbage glyphs left over? (icon fonts, exotic symbols)
pdftotext resume.pdf - | python3 -c "
import sys,unicodedata
t=sys.stdin.read()
for ch in sorted({c for c in t if ord(c)>127}):
    print(f'U+{ord(ch):04X} x{t.count(ch):<3} {unicodedata.name(ch,\"<private use>\")}')"
```

Expect only benign output: en/em dashes, curly quotes and apostrophes. Anything
labelled *private use* is an icon font and must go.

```bash
# 3. Page count and how much of the last page is empty
pdftotext -bbox resume.pdf - | python3 -c "
import sys,re
x=sys.stdin.read()
for i,(w,h,b) in enumerate(re.findall(r'<page width=\"([\d.]+)\" height=\"([\d.]+)\">(.*?)</page>',x,re.S),1):
    ys=[float(m) for m in re.findall(r'yMax=\"([\d.]+)\"',b)]
    H=float(h)
    print(f'page {i}: bottom gap {H-max(ys):.1f}pt ({(H-max(ys))/H*100:.1f}%)')"
```

`RVM` ships a version of this as `measure.sh` next to the resume workspace.

---

## 10. Per-job-description workflow

1. **Fork a branch**: `rvm branch <company>-<role>` from a clean, neutral `main`.
   Never tailor on `main` — that is what protection is for.
2. **Extract the JD's vocabulary.** Pull the required and preferred skills, and
   the exact phrasing used for them.
3. **Rewrite the Summary** in that JD's language, for what you actually did. The
   summary is the highest-leverage block for keyword matching.
4. **Reorder or re-emphasise bullets** so the most relevant real work is first.
   Do not invent work to match a requirement you lack.
5. **Mirror terminology** you can defend in the interview.
6. **Add genuine gaps only if true.** A missing requirement is better than a
   fabricated one.
7. **Compile, then run all three checks in §9.** Fix the text layer before you
   fix the design.
8. **Re-check the one-page constraint** after every content change — a single
   extra line can push you over.
9. **Keep `main` clean.** Your JD-tailored versions live on branches; `main`
   stays the honest base you fork from.

---

## 11. Pre-submission checklist

- [ ] Single column, no sidebar, no multi-column block
- [ ] Contact line is plain text, full URLs, **no icon fonts**
- [ ] Text layer has no private-use/garbage characters
- [ ] Name is the first thing extracted
- [ ] Job title and dates extract on the same line
- [ ] Standard section headings: Summary, Experience, Skills, Projects, Education
- [ ] No images, no text baked into graphics
- [ ] No text in page headers/footers
- [ ] Exactly one page; margins ≥ 0.25 in on all sides
- [ ] No visible dead space at the foot
- [ ] Every bullet starts with a strong verb
- [ ] No invented metrics; every number defensible in an interview
- [ ] Bullets are 15–25 words and carry one idea each
- [ ] JD vocabulary mirrored for work genuinely done
- [ ] PDF with a selectable text layer

---

## 12. Sources

- [Are LaTeX Resumes ATS-Friendly? The Two Traps and the Fix](https://atsverification.com/blog/latex-resume-ats-friendly/) — ligatures, icon fonts, the `pdftotext` check
- [Are Two-Column Resumes ATS-Friendly? (2026 Test)](https://atsverification.com/blog/two-column-resume-ats-friendly/) — the six-layout benchmark
- [What Is Resume Parsing?](https://recruiterflow.com/glossary/resume-parsing/) — extraction-to-database pipeline
- [Job Seeker's Guide to an Applicant Tracking System](https://www.coursera.org/gb/articles/applicant-tracking-system) — ATS fundamentals
- [Why do my results say I'm missing skills when they're already on my resume?](https://support.jobscan.co/hc/en-us/articles/41334900375571-Why-do-my-results-say-I-m-missing-skills-when-they-re-already-on-my-resume) — keyword matching pitfalls
- [Crafting Bullet Points for Your Resume](https://careers.augsburg.edu/resources/crafting-bullet-points-for-your-resume/) — bullet construction
- [Google's XYZ formula](https://law.utexas.edu/wp-content/uploads/sites/44/2020/09/Google-Recruiters-Say-Using-the-X-Y-Z-Formula-on-Your-Resume-Will-Improve-Your-Odds-of-Getting-Hired-at-Google-_-Inc.com_.pdf) — Laszlo Bock
- [Do ATS read tables in a resume?](https://www.jobscan.co/blog/resume-tables-columns-ats/) — tables and columns

---

## Appendix: why RVM's own validator is not an ATS check

`rvm commit` runs `rvm-validator`, which at the time of writing checks only:
page count, literal presence of configured section names, bullets shorter than
five words, multi-column commands, `\begin{tabular`, images, and the presence of
any standard heading. It does **not** inspect the text layer, reading order,
icon fonts, ligatures, keyword relevance, bullet quality, or verbosity.

It also has three known defects worth remembering before you trust a green run:

1. `W011` matches the substring `\begin{tabular`, which also matches
   `\begin{tabular*}` — the recommended tab-stop pattern for right-aligned dates.
   With `strict_mode = true` this warning is promoted to an error, so *every*
   resume using that idiom reports a validation failure.
2. `E010` looks for the literal string `contact`, so contact details that are
   present but unlabelled are reported as a missing section.
3. `I020` only matches lines beginning with `\item`. Templates that define their
   own bullet macro (`\resumeItem{...}`, as Jake's Resume does) are never checked
   at all, so the bullet-quality check silently does nothing.

Treat `rvm` validation as a compile-time sanity check, and use §9 as the real ATS
verification.
