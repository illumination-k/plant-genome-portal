#!/usr/bin/env python3
"""Deterministic synthetic RNA-seq fixture for transcriptome pipeline tests.

Builds:
- genome.fa     — 2 chromosomes × 2 kb random DNA
- annotation.gtf — 2 genes / 2 transcripts / 4 exons on chr1
- transcripts.fa — spliced transcript sequences (t1, t2)
- sampleA_1.fastq.gz / sampleA_2.fastq.gz — 220 paired-end reads
                   (120 from t1, 100 from t2)
- sampleB_se.fastq.gz — 140 single-end reads (80 from t1, 60 from t2)

Reads are sampled directly from transcript sequences with seed=42 / 43 so the
exact NumReads in Salmon quant.sf are reproducible across runs.
"""

from __future__ import annotations

import gzip
import os
import random
import sys
from pathlib import Path

ALPH = "ACGT"


def rand_seq(n: int) -> str:
    return "".join(random.choices(ALPH, k=n))


def revcomp(s: str) -> str:
    return s.translate(str.maketrans("ACGTN", "TGCAN"))[::-1]


def write_fa(path: Path, recs: list[tuple[str, str]], line: int = 60) -> None:
    with path.open("w") as f:
        for name, seq in recs:
            f.write(f">{name}\n")
            for i in range(0, len(seq), line):
                f.write(seq[i : i + line] + "\n")


def write_gtf(path: Path) -> None:
    def line(chrom, ftype, start, end, strand, attrs):
        a = " ".join(f'{k} "{v}";' for k, v in attrs)
        return "\t".join(
            [chrom, "test", ftype, str(start), str(end), ".", strand, ".", a]
        )

    rows = [
        # gene1 on chr1, + strand, exons 101-500 and 701-1200
        line("chr1", "gene", 101, 1200, "+", [("gene_id", "gene1"), ("gene_name", "gene1")]),
        line("chr1", "transcript", 101, 1200, "+", [("gene_id", "gene1"), ("transcript_id", "t1")]),
        line("chr1", "exon", 101, 500, "+", [("gene_id", "gene1"), ("transcript_id", "t1"), ("exon_number", "1")]),
        line("chr1", "exon", 701, 1200, "+", [("gene_id", "gene1"), ("transcript_id", "t1"), ("exon_number", "2")]),
        # gene2 on chr1, + strand, exons 1301-1600 and 1750-2000
        line("chr1", "gene", 1301, 2000, "+", [("gene_id", "gene2"), ("gene_name", "gene2")]),
        line("chr1", "transcript", 1301, 2000, "+", [("gene_id", "gene2"), ("transcript_id", "t2")]),
        line("chr1", "exon", 1301, 1600, "+", [("gene_id", "gene2"), ("transcript_id", "t2"), ("exon_number", "1")]),
        line("chr1", "exon", 1750, 2000, "+", [("gene_id", "gene2"), ("transcript_id", "t2"), ("exon_number", "2")]),
    ]
    path.write_text("\n".join(rows) + "\n")


def sample_paired(tx: str, n_pairs: int, prefix: str, read_len: int = 75, insert: int = 200):
    L = len(tx)
    pairs = []
    for i in range(n_pairs):
        s = random.randint(0, L - insert)
        frag = tx[s : s + insert]
        r1 = frag[:read_len]
        r2 = revcomp(frag[-read_len:])
        q = "I" * read_len
        pairs.append((f"@{prefix}_{i}/1\n{r1}\n+\n{q}\n", f"@{prefix}_{i}/2\n{r2}\n+\n{q}\n"))
    return pairs


def sample_single(tx: str, n: int, prefix: str, read_len: int = 75):
    L = len(tx)
    out = []
    for i in range(n):
        s = random.randint(0, L - read_len)
        frag = tx[s : s + read_len]
        if random.random() < 0.5:
            frag = revcomp(frag)
        q = "I" * read_len
        out.append(f"@{prefix}_{i}\n{frag}\n+\n{q}\n")
    return out


def main(out_dir: Path) -> None:
    out_dir.mkdir(parents=True, exist_ok=True)

    random.seed(42)
    chr1 = rand_seq(2000)
    chr2 = rand_seq(2000)
    write_fa(out_dir / "genome.fa", [("chr1", chr1), ("chr2", chr2)])

    def slice_g(chrom: str, start: int, end: int) -> str:
        return chrom[start - 1 : end]

    t1 = slice_g(chr1, 101, 500) + slice_g(chr1, 701, 1200)
    t2 = slice_g(chr1, 1301, 1600) + slice_g(chr1, 1750, 2000)
    write_fa(out_dir / "transcripts.fa", [("t1", t1), ("t2", t2)])
    write_gtf(out_dir / "annotation.gtf")

    # Paired-end sample
    pe = sample_paired(t1, 120, "tx1") + sample_paired(t2, 100, "tx2")
    random.shuffle(pe)
    with gzip.open(out_dir / "sampleA_1.fastq.gz", "wt") as f1, \
         gzip.open(out_dir / "sampleA_2.fastq.gz", "wt") as f2:
        for r1, r2 in pe:
            f1.write(r1)
            f2.write(r2)

    # Single-end sample
    random.seed(43)
    se = sample_single(t1, 80, "se_t1") + sample_single(t2, 60, "se_t2")
    random.shuffle(se)
    with gzip.open(out_dir / "sampleB_se.fastq.gz", "wt") as f:
        f.writelines(se)

    print(f"Fixture written to {out_dir}:")
    for name in sorted(os.listdir(out_dir)):
        p = out_dir / name
        print(f"  {name:30s} {p.stat().st_size:>8d} bytes")


if __name__ == "__main__":
    target = Path(sys.argv[1]) if len(sys.argv) > 1 else Path("fixtures")
    main(target)
