# Frontend Design

植物ゲノムポータルの UI/UX 設計指針。`web/` 配下の React + Tailwind v4 + @base-ui/react 実装の指針となる。

> Status: **adopted**。P0〜P6 のスケルトンが入り、`web/src/design/` のトークン・`web/src/ui/` の primitive・`web/src/bio/` の biology primitive・サイドレールシェル・search-first landing・gene detail のタブ・⌘K command palette・`/browser` ルートが稼働中。残りは P7 (motion・i18n・a11y audit・印刷スタイル) と、後述する未実装項目 (BLAST / 領域検索 / `/downloads` / `/api` 等)。

---

## 1. デザイン哲学

研究者のための database portal。「植物科学誌の組版」と「優れた開発者ツール」の交差点に置く。

| 原則                        | 含意                                                                                  |
| --------------------------- | ------------------------------------------------------------------------------------- |
| **Scientific seriousness**  | 装飾ではなくデータが主役。illustration やストック写真を入れない。                     |
| **Density over whitespace** | 研究者は時間で勝負する。Linear / GitHub のようにスキャンしやすい密度を取る。          |
| **Biology-aware**           | 配列・座標・strand・遺伝子構造はテキスト以上の情報を持つ。専用 visualization で表す。 |
| **Keyboard-first**          | `⌘K` で accession ジャンプ、テーブル `j/k` ナビゲーション、`/` でフォーカス。         |
| **Accession-first**         | URL も UI も accession (`GCA_037833805.1`, `Mp1g00010`) で語る。表示名は副。          |
| **Dual-mode**               | Light / Dark を first-class。プリファレンス保存、システム追従。                       |
| **No surprise**             | アニメーション控えめ、ページ遷移は瞬時、フォーカス常に可視。                          |

非目標: マーケティングサイト的な見た目、植物の写真、グラデーション多用、英雄的なコピー。

---

## 2. Visual identity

### 2.1 トーン

「落ち着いた森」+「論文の組版」+「ダッシュボードの整列感」。彩度は抑え、緑をブランドカラーに据えるが、面積は小さく保つ — 緑が UI を支配するのではなく、データに対して指差し役として機能する。

### 2.2 Color tokens

CSS variables で定義し、Tailwind v4 の `@theme` でユーティリティに展開する (`web/src/styles.css`)。Tailwind の `zinc-*` / `emerald-*` を直接書かない。

#### Foundation (semantic, theme-aware)

| Token                    | Light                      | Dark                  | 用途                      |
| ------------------------ | -------------------------- | --------------------- | ------------------------- |
| `--color-canvas`         | `#FAF9F6` (warm off-white) | `#0E1410`             | アプリ全体の背景          |
| `--color-surface`        | `#FFFFFF`                  | `#141A16`             | カード / パネル           |
| `--color-surface-raised` | `#FFFFFF` (with shadow)    | `#1A211C`             | ポップオーバー / モーダル |
| `--color-surface-muted`  | `#F4F2EE`                  | `#1A211C`             | テーブル行交互 / disabled |
| `--color-overlay`        | `rgba(15, 26, 20, 0.32)`   | `rgba(0, 0, 0, 0.56)` | モーダル背景              |

#### Text

| Token                   | Light     | Dark      |
| ----------------------- | --------- | --------- |
| `--color-text`          | `#0F1A14` | `#E8EDE9` |
| `--color-text-muted`    | `#4A5650` | `#9CA8A1` |
| `--color-text-subtle`   | `#7A857E` | `#6B7670` |
| `--color-text-disabled` | `#B7BDB8` | `#4A5650` |
| `--color-text-inverse`  | `#FFFFFF` | `#0F1A14` |
| `--color-text-link`     | `#1E5631` | `#76B889` |

#### Border

| Token                   | Light     | Dark      | 用途                          |
| ----------------------- | --------- | --------- | ----------------------------- |
| `--color-border-subtle` | `#ECEAE4` | `#222A24` | テーブル罫線、divider         |
| `--color-border`        | `#D9D6CE` | `#2C352E` | カード / 入力                 |
| `--color-border-strong` | `#9BA298` | `#4A5650` | active input, focus container |

#### Brand — Chlorophyll Green

11 stops で定義。`primary-700` が brand。`emerald` よりやや沈んだ青緑。

```
--color-primary-50:  #EEF5EF
--color-primary-100: #D7E7DA
--color-primary-200: #B0CFB5
--color-primary-300: #87B68F
--color-primary-400: #5E9D6A
--color-primary-500: #3E8350
--color-primary-600: #2D6A3F
--color-primary-700: #1E5631   /* brand */
--color-primary-800: #154225
--color-primary-900: #0D2F1A
--color-primary-950: #061A0D
```

#### Accent — Sunlit Amber

控えめ、ハイライト / inline highlight にのみ使う。

```
--color-accent-50 .. 950   /* base: #C77800 */
```

#### Semantic state

| Token             | 用途                          |
| ----------------- | ----------------------------- |
| `--color-success` | `#1E5631` (brand と同じで OK) |
| `--color-warning` | `#B45309`                     |
| `--color-danger`  | `#B42318`                     |
| `--color-info`    | `#175CD3`                     |

#### Biology semantic

UI のあちこちで使う。色覚特性 (8% の男性が赤緑色覚異常) を考慮し、形状や記号 (`+` / `−`) と必ず併用する。色だけに意味を載せない。

| Token                       | Light                     | Dark                      | 意味               |
| --------------------------- | ------------------------- | ------------------------- | ------------------ |
| `--color-strand-forward`    | `#175CD3`                 | `#7BA9F0`                 | `+` strand         |
| `--color-strand-reverse`    | `#B42318`                 | `#F08A7C`                 | `−` strand         |
| `--color-feature-cds`       | `--color-primary-700`     | `--color-primary-400`     | CDS exon           |
| `--color-feature-utr`       | `--color-primary-200`     | `--color-primary-800`     | UTR exon           |
| `--color-feature-intron`    | `--color-border-strong`   | `--color-border-strong`   | intron線           |
| `--color-feature-noncoding` | `#7C6BAB`                 | `#B0A4D8`                 | ncRNA / pseudogene |
| `--color-track-highlight`   | `rgba(199, 120, 0, 0.18)` | `rgba(199, 120, 0, 0.28)` | 選択中の領域       |

### 2.3 Elevation

shadow は最大 3 段階。dark mode では border を強める方向に倒し、shadow は弱める。

```
--shadow-1: 0 1px 2px rgba(15,26,20,0.06), 0 1px 1px rgba(15,26,20,0.04);
--shadow-2: 0 4px 12px rgba(15,26,20,0.08), 0 2px 4px rgba(15,26,20,0.04);
--shadow-3: 0 16px 32px rgba(15,26,20,0.12), 0 4px 8px rgba(15,26,20,0.06);
```

### 2.4 Radius

| Token           | px                           |
| --------------- | ---------------------------- |
| `--radius-xs`   | 4                            |
| `--radius-sm`   | 6                            |
| `--radius-md`   | 8 — default for card / input |
| `--radius-lg`   | 12 — modal, sheet            |
| `--radius-full` | 9999 — chips, pills          |

`rounded-2xl` 以上の大きすぎる radius は使わない (科学ツールには軟らかすぎる)。

---

## 3. Typography

### 3.1 Font families

| Role            | Family                                                                                          |
| --------------- | ----------------------------------------------------------------------------------------------- |
| UI / sans       | **Inter Variable** (latin) + system-ui fallback. Japanese は `"Hiragino Sans", "Noto Sans JP"`. |
| Mono            | **JetBrains Mono Variable**. 配列、accession、coordinates、code。                               |
| Scientific name | Inter Italic。`<Sci>` コンポーネントで強制。                                                    |

Serif は採用しない (組版感は spacing と階層で出す)。

### 3.2 Type scale

15px base。データテーブルは 14px、caption は 13px。

| Token             | size / line | weight                          | 用途                |
| ----------------- | ----------- | ------------------------------- | ------------------- |
| `text-display-xl` | 32 / 40     | 700                             | ランディング hero   |
| `text-display-lg` | 24 / 32     | 700                             | ページタイトル      |
| `text-heading`    | 18 / 26     | 600                             | セクション見出し    |
| `text-subheading` | 15 / 22     | 600                             | カード内見出し      |
| `text-body`       | 15 / 22     | 400                             | 本文 default        |
| `text-body-sm`    | 14 / 20     | 400                             | テーブル / dense UI |
| `text-caption`    | 13 / 18     | 500                             | ラベル / メタ       |
| `text-overline`   | 11 / 14     | 600, tracking 0.08em, uppercase | カードラベル        |
| `text-mono`       | 14 / 20     | 400                             | 配列 / accession    |
| `text-mono-sm`    | 12 / 18     | 400                             | 座標 inline         |

### 3.3 ルール

- 数字 (counts, coordinates, lengths) は `font-variant-numeric: tabular-nums` を必ず指定。
- accession / gene ID は **常に mono**。コピーボタンを併設する。
- 学名 (`Marchantia polymorpha`) は **常に italic**。`<Sci>` コンポーネントを通すこと。属の頭文字省略 (`M. polymorpha`) も同様。
- 体裁: 行長は 72ch まで、見出しは 56ch まで。

---

## 4. Spacing & layout

### 4.1 Spacing scale

4px ベース: `0, 1 (4), 2 (8), 3 (12), 4 (16), 5 (20), 6 (24), 8 (32), 10 (40), 12 (48), 16 (64), 24 (96)`。

`p-7`, `p-9` のような中途半端な値を使わない。

### 4.2 Breakpoints

| Token | min-width | 想定                              |
| ----- | --------- | --------------------------------- |
| `sm`  | 640px     | tablet portrait (補助)            |
| `md`  | 960px     | tablet landscape / 小ラップトップ |
| `lg`  | 1280px    | 標準ラボ環境                      |
| `xl`  | 1600px    | 大型ディスプレイ                  |

スマホ最適化は副。ラップトップ以上 (`lg` 以上) を主戦場とする。`md` 未満では sidebar が drawer に畳まれ、3-pane が 1-pane に潰れる。

### 4.3 12-column grid

すべてのページレイアウトは **12-column grid** の上に組む。Tailwind v4 の `grid grid-cols-12 gap-6` を基本クラスとし、子要素は `col-span-*` で領域を取る。

#### 仕様

| 項目                | 値                                                      |
| ------------------- | ------------------------------------------------------- |
| Columns             | **12** (固定)                                           |
| Gutter              | `gap-6` = 24px (`md` 以上) / `gap-4` = 16px (`md` 未満) |
| Container max-width | 1440px (`max-w-[1440px]`)                               |
| Outer padding       | `px-6` (24px, `md` 以下) / `px-8` (32px, `md` 以上)     |

#### Breakpoint ごとの挙動

| Breakpoint       | Columns             | Gutter         | 備考                                                                                                                                        |
| ---------------- | ------------------- | -------------- | ------------------------------------------------------------------------------------------------------------------------------------------- |
| `< md` (< 960px) | 4 (mobile sub-grid) | 16px           | 12-col 維持は破綻するため 4-col に縮退、`col-span-12` → `col-span-4`、`col-span-6` → `col-span-4`、`col-span-4` → `col-span-2` のマッピング |
| `md` (≥ 960px)   | 12                  | 24px           | 標準                                                                                                                                        |
| `lg` (≥ 1280px)  | 12                  | 24px           | 標準                                                                                                                                        |
| `xl` (≥ 1600px)  | 12                  | 32px (`gap-8`) | 視覚的に締まる                                                                                                                              |

#### 代表的な span パターン

| 用途                      | span                                          |
| ------------------------- | --------------------------------------------- |
| Full-bleed (hero / table) | `col-span-12`                                 |
| Main + Sidebar            | `col-span-8` + `col-span-4`                   |
| Half / Half               | `col-span-6` + `col-span-6`                   |
| Triptych (metric grid)    | `col-span-4` × 3                              |
| Quartet                   | `col-span-3` × 4                              |
| Main + Aside (狭)         | `col-span-9` + `col-span-3`                   |
| Centered form             | `col-start-3 col-span-8` (centered, 余白 2/2) |

`col-span-5`, `col-span-7`, `col-span-11` のような割り切れない span は原則禁止 (やむを得ない場合のみ、コメントで理由を残す)。

### 4.4 Application shell

App shell は grid の外側にある独立した layer。**TopBar** と **SideRail** はフレーム、**Main** だけが 12-col grid のキャンバスを提供する。Inspector は Main の grid を侵食しない overlay として右から滑り込む。

```
┌────────────────────────────────────────────────────────────────────────────────┐
│  Top bar (48px, sticky)                                                        │
│  ─ Logo ─ Assembly switcher ─ ⌘K ──────────── ─ theme ─ docs ─ account ─       │
├──────────┬─────────────────────────────────────────────────────┬───────────────┤
│          │  Main content (max-w 1440, px-6/px-8)               │               │
│  Side    │  ┌─────────────────────────────────────────────┐    │  Inspector    │
│  rail    │  │   12-column grid · gap-6                    │    │  (overlay,    │
│  240px   │  │ ┌──┬──┬──┬──┬──┬──┬──┬──┬──┬──┬──┬──┐       │    │   collapsible,│
│          │  │ │ 1│ 2│ 3│ 4│ 5│ 6│ 7│ 8│ 9│10│11│12│       │    │   360px)      │
│          │  │ └──┴──┴──┴──┴──┴──┴──┴──┴──┴──┴──┴──┘       │    │               │
│          │  └─────────────────────────────────────────────┘    │               │
└──────────┴─────────────────────────────────────────────────────┴───────────────┘
```

- **Top bar**: 48px, sticky, border-bottom 1px。Logo (text-only) + assembly switcher + ⌘K trigger (中央寄りに大きく) + 右端に theme toggle / docs / account。
- **Side rail**: 240px、collapse 時 56px (icon のみ)。grid の外、`grid-cols-12` の参照基準には入らない。
- **Main**: `max-w-[1440px]` (xl 未満では 100%)、`mx-auto`、`px-6/px-8`、内側に `grid grid-cols-12 gap-6`。すべてのページは `col-span-*` の合計が 12 になるよう構成する。
- **Inspector**: 任意。grid の上に乗る overlay (もしくは main の幅を `col-span-8` に絞って右に `col-span-4` で並列表示する push mode、ユーザー選択可)。`Esc` で閉じる。

### 4.5 Nested grids (sub-grid)

Card / panel 内部でさらに分割する場合は **同じ 12-col grid を入れ子** にし、`grid grid-cols-12 gap-4` を使う。例: gene detail header 内で `<Title col-span-8>` + `<Actions col-span-4>`。CSS Grid の `subgrid` (Tailwind v4 で `grid-cols-subgrid`) を活用して、子の column line が親と揃うことを保証する。

```tsx
<section className="col-span-12 grid grid-cols-subgrid gap-6">
  <h2 className="col-span-8">Functional annotation</h2>
  <div className="col-span-4 text-right">…actions</div>
</section>
```

これで「外側 grid の 7 列目」と「内側 grid の 7 列目」が常に揃う — 表組のような視覚整列が無料で手に入る。

### 4.6 Navigation 構造

Side rail は **目的別** に切る。`Gene` `Genome` `Tools` `Data` の 4 区分:

```
EXPLORE
  ├─ Search             /            (search-first landing)
  ├─ Genes              /genes
  └─ Genome browser     /browser

GENOMES
  ├─ Species            /species
  └─ Assemblies         /assemblies

TOOLS                  (MVP では disabled w/ "Coming soon")
  ├─ BLAST              /tools/blast
  └─ Region lookup      /tools/region

DATA
  ├─ Downloads          /downloads
  ├─ API reference      /api
  └─ Status             /status
```

旧 `DashboardPage` の metric grid は `/portal` (フッターリンク) に追いやる。トップは検索ファースト。

---

## 5. Page redesigns

### 5.1 `/` — Search-first landing

12-col 上での組み方:

```
 1  2  3 │ 4  5  6  7  8  9 10 │11 12
─────────┼─────────────────────┼───────
         │  hero title         │
         │  (col-start-3 col-span-8, text-center)
         │  search input       │
         │  (col-start-3 col-span-8)
         │  hint row           │
─────────┼──────────┬──────────┼───────
         │ recent   │ popular  │
         │ col-     │ col-     │
         │ span-4   │ span-4   │
         │ (start-3)│          │
```

具体的には:

- Hero + 検索入力: `col-start-3 col-span-8` で中央 8 列に集約。
- Suggestion: `col-start-3 col-span-4` (recent) + `col-span-4` (popular)。
- metrics は表示しない (ノイズ)。

### 5.2 `/genes` — Search & results

上部 sticky な検索バー (`col-span-12`)、下に結果テーブル (`col-span-12`)。Inspector 展開時は table を `col-span-8` に絞り、右に `col-span-4` で preview を並べる。

テーブルカラム:

| Gene                     | Symbol | Location                    | Strand          | Length     | Biotype        | GO terms       |
| ------------------------ | ------ | --------------------------- | --------------- | ---------- | -------------- | -------------- |
| `Mp1g00010` (mono, link) | MpARF1 | `Chr1:12,345–14,567` (mono) | `+` (blue chip) | `2,223 bp` | protein_coding | 3 chips + `+5` |

- 行 hover で右の Inspector に preview (gene model + 1-paragraph functional annotation)。
- 行クリックで `/genes/:id` 遷移。
- `j/k` で行移動、`Enter` で開く、`Space` で Inspector トグル。
- ページネーションは cursor-based、`Load more` ボタンと無限スクロール両対応 (キーボードユーザーのため Load more は残す)。
- 結果 0 件のとき: 「該当なし」+ 検索ヒント (free-text → symbol prefix 検索の suggest 等)。

### 5.3 `/genes/:geneId` — Gene detail

タブ構成。タブはルーティング (`?tab=annotation`) で保存。

```
┌─────────────────────────────────────────────────────────────┐
│  Mp1g00010   ●+strand   Chr1:12,345–14,567   2,223 bp      │
│  MpARF1 · auxin response factor 1                          │
│  Marchantia polymorpha · MpTak1 v7.1                       │
│  [Copy] [Open in browser] [Download FASTA] [Download GFF]  │
├─────────────────────────────────────────────────────────────┤
│  Overview · Annotation · Sequence · Transcripts · Browser  │
└─────────────────────────────────────────────────────────────┘
```

#### Overview タブ (12-col grid)

```
 1  2  3  4  5  6  7  8 │ 9 10 11 12
────────────────────────┼───────────
  GeneStructure         │  Key attributes
  col-span-8            │  col-span-4
  (gene model viz)      │  (definition list)
────────────────────────┴───────────
  Functional annotation summary           col-span-12
  (GO / Pfam / InterPro / KEGG chips, grouped)
```

- 左 `col-span-8`: **GeneStructure** (proper exon/intron 図、strand-aware、UTR は薄い fill、CDS は濃い fill、transcript ごとに行を分けて重畳表示)。
- 右 `col-span-4`: 主要属性 (definition list — `dt`/`dd` で組む、Tailwind `grid-cols-[auto_1fr]`)。
- 下段 `col-span-12`: Functional annotation のサマリ (GO / Pfam / InterPro / KEGG の chip cluster、grouped、各 chip クリックで対象 DB の該当 term/family にリンク)。

#### Annotation タブ

- GO terms をテーブルで (`GO:0008150` mono · namespace · term name · evidence · source)。
- Pfam / InterPro / KEGG / NCBIfam / KOG を同様に。
- Nomenclature (symbol synonyms) を別カードに。

#### Sequence タブ

- **SequenceBlock** コンポーネント: 60-char block、行頭に 1-based 座標、scroll で長い配列も滑らかに。
- 上部に表示モード切替: `genomic / mRNA / CDS / protein` (将来)。
- 右上に Copy + Download FASTA + refget checksum (truncated, copy 可能)。
- 下に「外部リンク」セクション: NCBI sequence viewer、refget URI 表示。

#### Transcripts タブ

- Transcripts → Exons の ネストしたテーブル。各行はアコーディオン展開で exon list を出す。
- 各 exon に座標 + 長さ + frame。

#### Browser タブ

- JBrowse 2 embed (full width, height 480-640px、`/jbrowse/config/{accession}` を読む)。
- 現在の遺伝子に locate された状態で開く。

### 5.4 `/browser` — JBrowse genome browser

- フルブリード (chrome なし) で JBrowse 2 embed。
- 上部に細い control bar: assembly switcher / region 入力 (`Chr1:1-100000`) / track 設定。
- URL は `/browser?loc=Chr1:1-100000&tracks=genes,...` で deep-link 可能。

### 5.5 `/species` — Species index

カード grid。12-col 上で各カードは `col-span-4` (xl: 3 列) / `md:col-span-6` (md: 2 列) / `col-span-12` (sm: 1 列)。カード内容は種学名 (italic) + TaxID + assembly 数 + 代表 thumbnail (chromosomes の小さな karyotype mini)。MVP は Marchantia 1 種のみだが、layout は複数前提で作る。

### 5.6 `/downloads`

カテゴリ別テーブル (`col-span-12`、Assembly / Annotation / Functional annotation / Snapshot)。各行: file name (mono) · size · sha256 · download。

---

## 6. Component system

### 6.1 構造

```
web/src/
  design/
    tokens.css            # CSS variables (themes)
    typography.css        # font-face, body classes
  ui/                     # 汎用 primitives (biology を知らない)
    Button.tsx
    IconButton.tsx
    Input.tsx
    Select.tsx
    Combobox.tsx          # async, used by ⌘K
    Checkbox.tsx
    Switch.tsx
    Card.tsx              # single Surface primitive
    Table.tsx
    Tabs.tsx
    Tooltip.tsx
    Popover.tsx
    Dialog.tsx
    Sheet.tsx
    Toast.tsx
    Badge.tsx             # static label
    Chip.tsx              # interactive / removable
    Code.tsx              # inline + block
    KbdKey.tsx
    Skeleton.tsx
    EmptyState.tsx
    ErrorState.tsx
  bio/                    # biology-aware primitives
    Accession.tsx
    GeneIdLink.tsx
    ScientificName.tsx    # <Sci>{name}</Sci>
    StrandBadge.tsx       # + / − with color + symbol
    CoordinateRange.tsx   # chr1:1,000–2,000
    SequenceBlock.tsx
    SequenceInline.tsx    # short inline like ATGCC...
    GeneStructure.tsx     # exon/intron diagram
    KaryotypeBar.tsx
    FunctionalAnnotationGroup.tsx
    RefgetChecksum.tsx
  components/             # 既存 — 徐々に ui/ + bio/ に移行
  layouts/
  pages/
```

`@base-ui/react` を採用済み。Dialog / Popover / Tabs / Tooltip / Select / Switch / Combobox の primitives はそのまま使い、`ui/` で見た目を付ける。

### 6.2 Button

variants × sizes、Tailwind v4 の `@variant` で組む。

| variant     | 用途                                                              |
| ----------- | ----------------------------------------------------------------- |
| `primary`   | 主アクション (search submit, save) — bg primary-700, text-inverse |
| `secondary` | 補助 — surface, border                                            |
| `ghost`     | tertiary、tableの行内アクション                                   |
| `danger`    | 破壊的 (将来)                                                     |
| `link`      | テキストリンク的                                                  |

sizes: `sm` (28px), `md` (32px), `lg` (40px)。Icon 専用は `IconButton` (square)。Loading 時は spinner と disabled。Focus ring は `outline: 2px solid var(--color-primary-500); outline-offset: 2px`。

### 6.3 Table

- sticky header、`th` はやや薄い muted bg。
- 行高は `dense` (32px) / `comfortable` (44px) — ユーザー切替 (将来)、default は dense。
- 行 hover で `bg-surface-muted`。
- 列 sort: `aria-sort` を必ず付ける。
- 数値列 / mono 列は `font-variant-numeric: tabular-nums`、右寄せ (counts) または左寄せ (IDs)。
- 空セルは `—` (em dash, muted) — never empty string。

### 6.4 Inputs

- 32px 標準 / 40px (検索 hero)。
- Label は常に上に。placeholder は label の代替にしない。
- Focus: border-primary-700 + ring 3px primary-100。
- Invalid: border-danger + helper-text。
- ⌘K trigger は `Input` の見た目を借りた button (実態は `Combobox` を modal で開く)。

### 6.5 Biology-aware primitives — 仕様

#### `<Accession value="GCA_037833805.1" />`

- Mono、copy ボタン inline (hover で出現)。
- 任意で外部 (NCBI / MarpolBase) リンク icon を `external` prop で。

#### `<StrandBadge strand="+" />`

- 26×18 の pill。`+` は forward 色 + 記号、`−` は reverse 色 + 記号。文字とアイコン両方で識別可能に。

#### `<CoordinateRange chr="Chr1" start={1} end={100000} />`

- 1-based closed (API 境界と一致)。`Chr1:1–100,000` (en dash、`tabular-nums`、桁区切り)。
- Click で copy、Shift+Click で `/browser?loc=...` 遷移。

#### `<GeneStructure transcripts={...} />`

- SVG。strand に応じて矢印方向。
- CDS は塗り、UTR は半透明、intron は線 (山型 ↗↘)。
- Hover で exon ごとに座標と長さを tooltip。
- viewBox はゲノム座標 (1:1)、超長 intron は collapse する optional mode (`mode="introns-collapsed"`)。

#### `<SequenceBlock sequence={…} start={1} />`

- 60 chars per line、10 chars ごとに微かな gap。
- 行頭に座標 (right-aligned mono)。
- 上下に `Copy FASTA` `Download .fa` ボタン。
- 長い配列 (> 10 kb) は仮想スクロール。
- Optional: アミノ酸翻訳 (3-frame) を下に重ねる (将来)。

#### `<FunctionalAnnotationGroup annotation={...} />`

- GO, Pfam, InterPro, KEGG, NCBIfam, KOG をグループ化したカード。
- 各 term は chip。max 6 表示 + `+N more` で展開。

---

## 7. Interaction

### 7.1 Keyboard shortcuts

| Keys            | Action                           |
| --------------- | -------------------------------- |
| `⌘K` / `Ctrl+K` | Command palette (検索 + jump-to) |
| `/`             | フォーカス検索                   |
| `g s`           | Go to search                     |
| `g g`           | Go to genes                      |
| `g b`           | Go to browser                    |
| `j` / `k`       | next / prev row (table)          |
| `Enter`         | open selected row                |
| `Space`         | toggle Inspector preview         |
| `Esc`           | close Inspector / Dialog / Sheet |
| `?`             | show shortcut help               |

`?` の help dialog にすべての shortcut を載せる。

### 7.2 Command palette (⌘K)

- 即時起動。fuzzy search で gene / accession / region / page。
- グループ: **Genes**, **Assemblies**, **Pages**, **Recent**。
- 結果は最大 8 件、`↑↓` で移動、`Enter` で確定。
- Empty query 時は recent + suggested。
- API は `geneSearchOptions` を再利用、debounce 120ms。

### 7.3 Motion

- すべての遷移 ≤ 200ms。easing は `cubic-bezier(0.2, 0, 0, 1)` (ease-out)。
- Page transitions: なし (instant)。
- Drawer / Sheet: 200ms slide。
- `prefers-reduced-motion: reduce` で全遷移を 0ms。

### 7.4 Feedback

- 同期アクション (copy, toggle): inline confirmation (`Copied!` を 1.5s)、Toast 不要。
- 非同期 (download, future mutations): Toast。
- Error: 永続 toast + 再試行ボタン。

---

## 8. Async / state UX

### 8.1 Loading

- 初回ロード: **Skeleton** (コンテンツ形状を予告する灰色プレースホルダ)。Spinner は使わない (位置情報を失うため)。
- ボタンクリック後の loading: button 内 spinner + disabled。
- バックグラウンド refetch (react-query): top bar に細い progress bar (1px、primary-400)。

### 8.2 Empty

- アイコン (line icon、`stroke-width: 1.5`) + heading + 1 文の説明 + 1 つの CTA (or 検索ヒント)。
- "No results" だけで終わらせない。

### 8.3 Error

- inline (フィールドエラー) と global (ネットワーク等) を分ける。
- 必ず `retry` ボタンを出す。`expand` で技術詳細 (stack / request id) を見られるようにする。
- 404 (`GeneNotFound` 等) は「該当遺伝子が見つかりません」+ 検索に戻る CTA。

### 8.4 External boundary validation

外界から入ってくる値 (URL 検索パラメータ・URL パスパラメータ・`localStorage`・API
レスポンス・ユーザー入力) は **必ず valibot で正しい型に変換** してから内側のコードに
渡す。`string | null | undefined` のような曖昧な型をページ／コンポーネント内で扱わない。

- URL search-param は `useValidatedSearchParam(key, schema, fallback)` (`web/src/lib/useValidatedSearchParam.ts`) を経由する。
- URL path-param は `useValidatedParam(key, schema, fallback)` (`web/src/lib/useValidatedParam.ts`) を経由する。
- `localStorage` から読む値はその場で `v.safeParse` を通す (例: `web/src/lib/theme.ts`)。
- API レスポンスは `web/src/api/client/valibot.gen.ts` 由来のスキーマで既に validate されている (hey-api 経由)。

これにより JSX 内で `undefined` リテラルを書かなくて済み、`unicorn/no-null` / `no-undefined` といった lint ルールとも自然に整合する。

---

## 9. Accessibility

- **WCAG 2.2 AA** を目標。Contrast はテキスト 4.5:1、UI コンポーネント 3:1。
- すべての interactive 要素にフォーカスリング (`outline: 2px`)、`outline: none` 禁止。
- Icon-only ボタンには `aria-label` 必須。
- Table ヘッダは `<th scope="col">`、ソート可能は `aria-sort`。
- Live region (`aria-live="polite"`) でロード完了 / 検索結果数を読み上げ。
- `prefers-reduced-motion` 尊重。
- カラー単独で意味を伝えない (strand は色 + 記号、status は色 + アイコン)。

---

## 10. Internationalization

- 主要言語: **English** + **Japanese** (プロジェクトの研究者層に合わせる)。
- ライブラリ: `@lingui/react` または `react-i18next` (どちらでも可、決定は実装時)。
- Top bar に言語切替 (footer ではなく目立つ位置に置く)。
- 学名 (`Marchantia polymorpha`) はラテン語のまま、常に italic。`<Sci>` でラップし翻訳対象外。
- 数値・日付は `Intl` API でロケール対応。

---

## 11. Theming

- light / dark / system の 3 択、`localStorage` に保存。
- `data-theme="light|dark"` を `<html>` に。CSS variables で切り替え。
- メタタグ `theme-color` も同期 (モバイル address bar)。
- 画像 / アイコン / chart は両方で見えることを保証 (テスト追加)。

---

## 12. Data visualization

ゲノムポータルの心臓部。共通の design principle:

- **正確さ第一**: 座標を歪めない、proportional に描く (introns-collapsed mode は明示)。
- **タッチターゲット 24px 以上** (hover-only に頼らない)。
- **凡例必須**: 凡例なしの色付きは存在させない。
- **印刷 / SVG export 対応**: PNG ではなく SVG で描画、`<title>` `<desc>` 付与。

優先実装:

1. `GeneStructure` — gene detail で最重要。
2. `KaryotypeBar` — chromosome 全長 + 遺伝子密度。`/species/:taxId/assemblies/:acc` で表示。
3. `RegionMiniMap` — JBrowse 連動の overview。
4. (v2) Expression heatmap, Coexpression network。

---

## 13. Implementation roadmap

| Phase                         | Scope                                                                                                                                              | Status    | 現状                                                                                                                                                                                                                              |
| ----------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------- | --------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **P0 Tokens**                 | `design/tokens.css`, `typography.css`, theme switcher                                                                                              | ✅ done   | `web/src/design/{tokens,typography}.css` + `@theme inline` で Tailwind に公開、`web/src/lib/theme.ts` + `web/src/ui/ThemeToggle.tsx` で light / dark / system 切替。`pgp:theme` を valibot で validate。                          |
| **P1 Primitives**             | `ui/` 一式 (Tabs, Dialog wrapper, Skeleton, EmptyState, ErrorState, CopyButton, KbdKey, CommandPalette\* …)                                        | ◐ partial | 現在の gene detail / palette / 状態表示に必要な分だけ実装。Button / Input / Tooltip / Toast / Sheet / Code / Chip 等の汎用 primitive と `/_dev/ui` playground は未着手。                                                          |
| **P2 Shell**                  | TopBar, SideRail, Inspector slot を持つ RootLayout                                                                                                 | ◐ partial | TopBar (48px sticky / brand / assembly chip / ⌘K trigger / theme toggle) と SideRail (240px, 目的別 4 グループ) は稼働。`md` 未満では sidebar が drawer に畳まれる挙動、Inspector overlay (gene detail の preview 用) は未実装。  |
| **P3 Bio primitives**         | `bio/` 一式 (Accession, StrandBadge, CoordinateRange, Sci, GeneIdLink, RefgetChecksum, FunctionalAnnotationGroup/Chip, GeneHeader, GeneSymbolLine) | ◐ partial | gene detail / table で使用中。`SequenceBlock`、`KaryotypeBar`、`RegionMiniMap` は未実装 (refget proxy 入りで対応予定)。                                                                                                           |
| **P4 Search-first home + ⌘K** | `/` を search-first に置き換え、Command palette 稼働                                                                                               | ✅ done   | `DashboardPage` を `LandingHero` + `LandingSearchForm` + Recent/Popular に置き換え。⌘K / Ctrl+K / `/` で `CommandPalette` (base-ui Dialog + Pages/Genes section) が開く。グローバルショートカットは `GlobalShortcuts` が listen。 |
| **P5 Gene detail v2**         | タブ構成 (Overview / Annotation / Sequence / Transcripts / Browser)、新 GeneStructure                                                              | ✅ done   | `GeneDetailTabs` (`useValidatedSearchParam` + valibot で `?tab=` を validate)。各タブは `GeneOverviewTab` / `GeneAnnotationTab` / `GeneSequenceTab` / `GeneTranscriptsTab` / `GeneBrowserTab`。SequenceBlock の本体は未着手。     |
| **P6 Browser route**          | `/browser` に JBrowse 2 embed、deep-link 対応                                                                                                      | ✅ done   | `BrowserPage` が `?loc=Chr1:1-100000` を valibot で validate、`GenomeBrowser` を full-bleed で表示。                                                                                                                              |
| **P7 Polish**                 | Motion, a11y audit, i18n 切替、印刷スタイル                                                                                                        | ⬜ todo   | `prefers-reduced-motion` ガードと focus-visible ring は入っているが、Motion 全体 / axe / lighthouse / 印刷 / `@lingui/react` 等は未着手。                                                                                         |

各 phase ごとに既存 `components/` から該当機能を `ui/` + `bio/` へ徐々に移送する。big-bang ではなく漸進。

---

## 14. Don'ts

- 🚫 装飾的なグラデーション、glass-morphism、neumorphism。
- 🚫 アイコンを意味の唯一の伝達手段にする (必ずテキストか aria-label を伴う)。
- 🚫 `emerald-*` / `zinc-*` を直接書く (semantic token を使う)。
- 🚫 `unwrap` 相当の UX (loading なし、empty なし、error なし)。
- 🚫 100ch を超える本文行長、72ch を超える見出し。
- 🚫 全画面 spinner で画面を覆う (skeleton にする)。
- 🚫 hover でしか出ない情報を critical path に置く (タッチ環境で死ぬ)。
- 🚫 学名のローマン体 (italic を忘れない)。
- 🚫 12-col grid からの逸脱 (`col-span-5` `col-span-7` 等の割り切れない span、固定 px width で grid を無視した layout)。やむを得ない場合はコメントで理由を残す。

---

## 15. Open questions

- 学名以外で italic を使う場面 (遺伝子記号は species によって italic が慣習。Marchantia ではどうか — 要確認)。
- ⌘K の global 検索 backend: 現在の `/v2/gene/search` で十分か、専用 search index (tantivy 等) を別途立てるか。
- 印刷スタイル: gene detail のクリーンな PDF export を提供すべきか。
- 認証: MVP は public read-only だが、ヘッダーに account slot を置いておくか (将来 v2 で復活させやすい)。

これらは実装フェーズで判断する。
