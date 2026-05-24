# Frontend Design

植物ゲノムポータルの UI/UX 設計指針。`web/` 配下の React + Tailwind v4 + @base-ui/react 実装の指針となる。

> Status: **proposal**。現在の `web/src/` の実装はこの設計書より前のもので、当面は混在する。新規コンポーネントとリファクタはこの設計書に従う。

---

## 1. デザイン哲学

研究者のための database portal。「植物科学誌の組版」と「優れた開発者ツール」の交差点に置く。

| 原則 | 含意 |
| --- | --- |
| **Scientific seriousness** | 装飾ではなくデータが主役。illustration やストック写真を入れない。 |
| **Density over whitespace** | 研究者は時間で勝負する。Linear / GitHub のようにスキャンしやすい密度を取る。 |
| **Biology-aware** | 配列・座標・strand・遺伝子構造はテキスト以上の情報を持つ。専用 visualization で表す。 |
| **Keyboard-first** | `⌘K` で accession ジャンプ、テーブル `j/k` ナビゲーション、`/` でフォーカス。 |
| **Accession-first** | URL も UI も accession (`GCA_037833805.1`, `Mp1g00010`) で語る。表示名は副。 |
| **Dual-mode** | Light / Dark を first-class。プリファレンス保存、システム追従。 |
| **No surprise** | アニメーション控えめ、ページ遷移は瞬時、フォーカス常に可視。 |

非目標: マーケティングサイト的な見た目、植物の写真、グラデーション多用、英雄的なコピー。

---

## 2. Visual identity

### 2.1 トーン

「落ち着いた森」+「論文の組版」+「ダッシュボードの整列感」。彩度は抑え、緑をブランドカラーに据えるが、面積は小さく保つ — 緑が UI を支配するのではなく、データに対して指差し役として機能する。

### 2.2 Color tokens

CSS variables で定義し、Tailwind v4 の `@theme` でユーティリティに展開する (`web/src/styles.css`)。Tailwind の `zinc-*` / `emerald-*` を直接書かない。

#### Foundation (semantic, theme-aware)

| Token | Light | Dark | 用途 |
| --- | --- | --- | --- |
| `--color-canvas` | `#FAF9F6` (warm off-white) | `#0E1410` | アプリ全体の背景 |
| `--color-surface` | `#FFFFFF` | `#141A16` | カード / パネル |
| `--color-surface-raised` | `#FFFFFF` (with shadow) | `#1A211C` | ポップオーバー / モーダル |
| `--color-surface-muted` | `#F4F2EE` | `#1A211C` | テーブル行交互 / disabled |
| `--color-overlay` | `rgba(15, 26, 20, 0.32)` | `rgba(0, 0, 0, 0.56)` | モーダル背景 |

#### Text

| Token | Light | Dark |
| --- | --- | --- |
| `--color-text` | `#0F1A14` | `#E8EDE9` |
| `--color-text-muted` | `#4A5650` | `#9CA8A1` |
| `--color-text-subtle` | `#7A857E` | `#6B7670` |
| `--color-text-disabled` | `#B7BDB8` | `#4A5650` |
| `--color-text-inverse` | `#FFFFFF` | `#0F1A14` |
| `--color-text-link` | `#1E5631` | `#76B889` |

#### Border

| Token | Light | Dark | 用途 |
| --- | --- | --- | --- |
| `--color-border-subtle` | `#ECEAE4` | `#222A24` | テーブル罫線、divider |
| `--color-border` | `#D9D6CE` | `#2C352E` | カード / 入力 |
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

| Token | 用途 |
| --- | --- |
| `--color-success` | `#1E5631` (brand と同じで OK) |
| `--color-warning` | `#B45309` |
| `--color-danger` | `#B42318` |
| `--color-info` | `#175CD3` |

#### Biology semantic

UI のあちこちで使う。色覚特性 (8% の男性が赤緑色覚異常) を考慮し、形状や記号 (`+` / `−`) と必ず併用する。色だけに意味を載せない。

| Token | Light | Dark | 意味 |
| --- | --- | --- | --- |
| `--color-strand-forward` | `#175CD3` | `#7BA9F0` | `+` strand |
| `--color-strand-reverse` | `#B42318` | `#F08A7C` | `−` strand |
| `--color-feature-cds` | `--color-primary-700` | `--color-primary-400` | CDS exon |
| `--color-feature-utr` | `--color-primary-200` | `--color-primary-800` | UTR exon |
| `--color-feature-intron` | `--color-border-strong` | `--color-border-strong` | intron線 |
| `--color-feature-noncoding` | `#7C6BAB` | `#B0A4D8` | ncRNA / pseudogene |
| `--color-track-highlight` | `rgba(199, 120, 0, 0.18)` | `rgba(199, 120, 0, 0.28)` | 選択中の領域 |

### 2.3 Elevation

shadow は最大 3 段階。dark mode では border を強める方向に倒し、shadow は弱める。

```
--shadow-1: 0 1px 2px rgba(15,26,20,0.06), 0 1px 1px rgba(15,26,20,0.04);
--shadow-2: 0 4px 12px rgba(15,26,20,0.08), 0 2px 4px rgba(15,26,20,0.04);
--shadow-3: 0 16px 32px rgba(15,26,20,0.12), 0 4px 8px rgba(15,26,20,0.06);
```

### 2.4 Radius

| Token | px |
| --- | --- |
| `--radius-xs` | 4 |
| `--radius-sm` | 6 |
| `--radius-md` | 8 — default for card / input |
| `--radius-lg` | 12 — modal, sheet |
| `--radius-full` | 9999 — chips, pills |

`rounded-2xl` 以上の大きすぎる radius は使わない (科学ツールには軟らかすぎる)。

---

## 3. Typography

### 3.1 Font families

| Role | Family |
| --- | --- |
| UI / sans | **Inter Variable** (latin) + system-ui fallback. Japanese は `"Hiragino Sans", "Noto Sans JP"`. |
| Mono | **JetBrains Mono Variable**. 配列、accession、coordinates、code。 |
| Scientific name | Inter Italic。`<Sci>` コンポーネントで強制。 |

Serif は採用しない (組版感は spacing と階層で出す)。

### 3.2 Type scale

15px base。データテーブルは 14px、caption は 13px。

| Token | size / line | weight | 用途 |
| --- | --- | --- | --- |
| `text-display-xl` | 32 / 40 | 700 | ランディング hero |
| `text-display-lg` | 24 / 32 | 700 | ページタイトル |
| `text-heading` | 18 / 26 | 600 | セクション見出し |
| `text-subheading` | 15 / 22 | 600 | カード内見出し |
| `text-body` | 15 / 22 | 400 | 本文 default |
| `text-body-sm` | 14 / 20 | 400 | テーブル / dense UI |
| `text-caption` | 13 / 18 | 500 | ラベル / メタ |
| `text-overline` | 11 / 14 | 600, tracking 0.08em, uppercase | カードラベル |
| `text-mono` | 14 / 20 | 400 | 配列 / accession |
| `text-mono-sm` | 12 / 18 | 400 | 座標 inline |

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

| Token | min-width | 想定 |
| --- | --- | --- |
| `sm` | 640px | tablet portrait (補助) |
| `md` | 960px | tablet landscape / 小ラップトップ |
| `lg` | 1280px | 標準ラボ環境 |
| `xl` | 1600px | 大型ディスプレイ |

スマホ最適化は副。ラップトップ以上 (`lg` 以上) を主戦場とする。`md` 未満では sidebar が drawer に畳まれ、3-pane が 1-pane に潰れる。

### 4.3 Application shell

```
┌─────────────────────────────────────────────────────────────────┐
│  Top bar (48px)                                                 │
│  ─ Logo ─ Assembly switcher ─ ⌘K ──── ─ theme ─ docs ─ account  │
├──────────┬──────────────────────────────────┬───────────────────┤
│          │                                  │                   │
│  Side    │       Main content               │  Inspector        │
│  rail    │       (routed)                   │  (contextual,     │
│  240px   │       max-w 1280, padded         │   collapsible,    │
│          │                                  │   360px)          │
│          │                                  │                   │
└──────────┴──────────────────────────────────┴───────────────────┘
```

- **Top bar**: 48px, sticky, border-bottom 1px。Logo (text-only) + assembly switcher (現在の MpTak1_v7.1 表示) + ⌘K trigger (中央寄りに大きく) + 右端に theme toggle / docs / API キー (将来)。
- **Side rail**: 240px、collapse 時 56px (icon のみ)。`g+s`, `g+g` でセクションジャンプ。
- **Main**: 最大幅 1280px、`px-6` (24px)、`py-8` (32px)。
- **Inspector**: 任意。遺伝子検索結果で行を hover/select すると右に preview。閉じられる、`Esc` で閉じる。

### 4.4 Navigation 構造

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

```
                  Plant Genome Portal
        Marchantia polymorpha MpTak1 v7.1 · 19,138 genes

   ┌──────────────────────────────────────────────────┐
   │  🔍  Search genes, accessions, or regions…  ⌘K  │
   └──────────────────────────────────────────────────┘
        e.g.  Mp1g00010   ·   MpARF1   ·   Chr1:1-100000

   Recent          Popular entry points
   • Mp1g00010     • Browse Chr1
   • MpARF1        • Download GFF3
                   • API reference
```

中央寄せの大きな検索入力。下にサジェスチョン (recent + popular)。metrics は表示しない (ノイズ)。

### 5.2 `/genes` — Search & results

二段構成: 上部 sticky な検索バー (symbol / locus_tag / free-text / chromosome filter)、下に結果テーブル。

テーブルカラム:

| Gene | Symbol | Location | Strand | Length | Biotype | GO terms |
|---|---|---|---|---|---|---|
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

#### Overview タブ
- 左 2/3: **GeneStructure** (proper exon/intron 図、strand-aware、UTR は薄い fill、CDS は濃い fill、transcript ごとに行を分けて重畳表示)。
- 右 1/3: 主要属性 (definition list — `dt`/`dd` で組む、Tailwind grid-cols-[auto_1fr])。
- 下段: Functional annotation のサマリ (GO / Pfam / InterPro / KEGG の chip cluster、grouped、各 chip クリックで対象 DB の該当 term/family にリンク)。

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

カード grid (1 列 / md:2 列 / xl:3 列)、各カードに種学名 (italic) + TaxID + assembly 数 + 代表 thumbnail (chromosomes の小さな karyotype mini)。MVP は Marchantia 1 種のみだが、layout は複数前提で作る。

### 5.6 `/downloads`

カテゴリ別テーブル (Assembly / Annotation / Functional annotation / Snapshot)。各行: file name (mono) · size · sha256 · download。

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

| variant | 用途 |
| --- | --- |
| `primary` | 主アクション (search submit, save) — bg primary-700, text-inverse |
| `secondary` | 補助 — surface, border |
| `ghost` | tertiary、tableの行内アクション |
| `danger` | 破壊的 (将来) |
| `link` | テキストリンク的 |

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

| Keys | Action |
| --- | --- |
| `⌘K` / `Ctrl+K` | Command palette (検索 + jump-to) |
| `/` | フォーカス検索 |
| `g s` | Go to search |
| `g g` | Go to genes |
| `g b` | Go to browser |
| `j` / `k` | next / prev row (table) |
| `Enter` | open selected row |
| `Space` | toggle Inspector preview |
| `Esc` | close Inspector / Dialog / Sheet |
| `?` | show shortcut help |

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

| Phase | Scope | Definition of done |
| --- | --- | --- |
| **P0 Tokens** | `design/tokens.css`, `typography.css`, theme switcher | light/dark で既存ページが破綻せず動く |
| **P1 Primitives** | `ui/` 一式 (Button, Input, Card, Table, Tabs, Dialog, Tooltip, Chip, Code, Skeleton, EmptyState) | 単体テスト + Storybook 的な playground page (`/_dev/ui`) |
| **P2 Shell** | TopBar, SideRail, Inspector slot を持つ RootLayout | `lg` 以上で 3-pane、`md` 以下で 1-pane に潰れる |
| **P3 Bio primitives** | `bio/` 一式 (Accession, StrandBadge, CoordinateRange, ScientificName, GeneStructure, SequenceBlock, FunctionalAnnotationGroup) | gene detail で使用、unit + visual test |
| **P4 Search-first home + ⌘K** | `/` を search-first に置き換え、Command palette 稼働 | キーボードのみで gene 詳細まで到達できる |
| **P5 Gene detail v2** | タブ構成、新 GeneStructure、SequenceBlock 採用 | API は既存のまま、表示のみ刷新 |
| **P6 Browser route** | `/browser` に JBrowse 2 embed、deep-link 対応 | URL `?loc=...` で領域指定が可能 |
| **P7 Polish** | Motion, a11y audit, i18n 切替、印刷スタイル | axe / lighthouse a11y 95+ |

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

---

## 15. Open questions

- 学名以外で italic を使う場面 (遺伝子記号は species によって italic が慣習。Marchantia ではどうか — 要確認)。
- ⌘K の global 検索 backend: 現在の `/v2/gene/search` で十分か、専用 search index (tantivy 等) を別途立てるか。
- 印刷スタイル: gene detail のクリーンな PDF export を提供すべきか。
- 認証: MVP は public read-only だが、ヘッダーに account slot を置いておくか (将来 v2 で復活させやすい)。

これらは実装フェーズで判断する。
