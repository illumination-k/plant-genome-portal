include { SRA_FETCH                     } from '../modules/local/sra_fetch.nf'
include { FASTP                         } from '../modules/local/fastp.nf'
include { SAMTOOLS_FAIDX                } from '../modules/local/samtools_faidx.nf'
include { BOWTIE2_BUILD                 } from '../modules/local/bowtie2_build.nf'
include { BOWTIE2_ALIGN                 } from '../modules/local/bowtie2_align.nf'
include { SAMTOOLS_SORT                 } from '../modules/local/samtools_sort.nf'
include { PICARD_MARKDUPLICATES         } from '../modules/local/picard_markduplicates.nf'
include { SAMTOOLS_FILTER               } from '../modules/local/samtools_filter.nf'
include { DEEPTOOLS_ALIGNMENTSIEVE      } from '../modules/local/deeptools_alignmentsieve.nf'
include { SAMTOOLS_RESORT               } from '../modules/local/samtools_resort.nf'
include { PICARD_COLLECTMULTIPLEMETRICS } from '../modules/local/picard_collectmultiplemetrics.nf'
include { DEEPTOOLS_BAMCOVERAGE         } from '../modules/local/deeptools_bamcoverage.nf'
include { MACS3_CALLPEAK                } from '../modules/local/macs3_callpeak.nf'
include { FRIP                          } from '../modules/local/frip.nf'
include { HOMER_FINDMOTIFS              } from '../modules/local/homer_findmotifs.nf'
include { MULTIQC                       } from '../modules/local/multiqc.nf'

workflow ATACSEQ {

    // ------------------------------------------------------------------
    // Param validation
    // ------------------------------------------------------------------
    if (!params.input)        error "Missing required parameter --input (CSV samplesheet)"
    if (!params.genome_fasta) error "Missing required parameter --genome_fasta"

    ch_genome  = file(params.genome_fasta, checkIfExists: true)
    ch_no_file = file("${projectDir}/assets/NO_FILE")

    def parse_row = { row ->
        def has_sra = row.sra && row.sra.trim()
        def meta = [
            id        : row.sample,
            group     : (row.group ?: row.sample).trim(),
            replicate : (row.replicate ?: '1').trim(),
            sra       : has_sra ? row.sra.trim() : null,
            single_end: has_sra ? null : !(row.fastq_2 && row.fastq_2.trim()),
        ]
        return [meta, row]
    }

    // ------------------------------------------------------------------
    // Parse samplesheet → branch SRA vs local FASTQ
    // ------------------------------------------------------------------
    ch_samples = Channel.fromPath(params.input, checkIfExists: true)
        .splitCsv(header: true, strip: true)
        .map(parse_row)

    ch_samples.branch { meta, _row ->
        sra:   meta.sra != null
        local: true
    }.set { ch_branched }

    // ---- SRA branch ---------------------------------------------------
    SRA_FETCH(ch_branched.sra.map { meta, _row -> [meta, meta.sra] })

    ch_sra_reads = SRA_FETCH.out.reads.map { meta, fqs ->
        def list = (fqs instanceof List) ? fqs : [fqs]
        def sorted = list.sort { it.name }
        def m = meta + [single_end: sorted.size() == 1]
        return [m, sorted.size() == 1 ? sorted[0] : sorted]
    }

    // ---- Local FASTQ branch ------------------------------------------
    ch_local_reads = ch_branched.local.map { meta, row ->
        def r1 = file(row.fastq_1, checkIfExists: true)
        def reads = (row.fastq_2 && row.fastq_2.trim())
            ? [r1, file(row.fastq_2, checkIfExists: true)]
            : r1
        return [meta, reads]
    }

    ch_reads = ch_local_reads.mix(ch_sra_reads)

    // ------------------------------------------------------------------
    // QC + trim
    // ------------------------------------------------------------------
    FASTP(ch_reads)

    // ------------------------------------------------------------------
    // Reference: faidx + Bowtie2 index
    // ------------------------------------------------------------------
    SAMTOOLS_FAIDX(ch_genome)

    if (params.bowtie2_index) {
        ch_bt2_index = Channel.value(file(params.bowtie2_index, checkIfExists: true))
    } else {
        BOWTIE2_BUILD(ch_genome)
        ch_bt2_index = BOWTIE2_BUILD.out.index
    }

    // ------------------------------------------------------------------
    // Align → sort → mark duplicates → filter (incl. mito/plastid drop)
    // ------------------------------------------------------------------
    BOWTIE2_ALIGN(FASTP.out.reads, ch_bt2_index)
    SAMTOOLS_SORT(BOWTIE2_ALIGN.out.sam)
    PICARD_MARKDUPLICATES(SAMTOOLS_SORT.out.bam)
    SAMTOOLS_FILTER(PICARD_MARKDUPLICATES.out.bam)

    // ------------------------------------------------------------------
    // Tn5 shift (+4 / -5) — canonical ATAC correction so reads represent
    // the actual Tn5 cut site. Optional via --atac_shift.
    // ------------------------------------------------------------------
    if (params.atac_shift) {
        DEEPTOOLS_ALIGNMENTSIEVE(SAMTOOLS_FILTER.out.bam)
        SAMTOOLS_RESORT(DEEPTOOLS_ALIGNMENTSIEVE.out.bam)
        ch_final_bam = SAMTOOLS_RESORT.out.bam
    } else {
        ch_final_bam = SAMTOOLS_FILTER.out.bam
    }

    PICARD_COLLECTMULTIPLEMETRICS(ch_final_bam, SAMTOOLS_FAIDX.out.fasta)

    // ------------------------------------------------------------------
    // BigWig coverage tracks
    // ------------------------------------------------------------------
    if (!params.skip_bigwig) {
        DEEPTOOLS_BAMCOVERAGE(ch_final_bam)
    }

    // ------------------------------------------------------------------
    // Peak calling (MACS3, ATAC mode)
    // ------------------------------------------------------------------
    ch_peaks = Channel.empty()
    if (!params.skip_peaks) {
        MACS3_CALLPEAK(ch_final_bam)
        ch_peaks = MACS3_CALLPEAK.out.peaks

        // ---- FRiP (Fraction of Reads in Peaks) -----------------------
        if (!params.skip_frip) {
            ch_peak_bed = ch_peaks.map { meta, files ->
                def file_list = (files instanceof List) ? files : [files]
                def bed = file_list.find { it.name.endsWith('.narrowPeak') || it.name.endsWith('.broadPeak') }
                return bed ? [meta, bed] : null
            }.filter { it != null }

            ch_bam_for_frip = ch_final_bam.map { meta, bam, bai -> [meta.id, meta, bam, bai] }
            ch_bed_for_frip = ch_peak_bed.map { meta, bed -> [meta.id, bed] }

            ch_frip_input = ch_bam_for_frip
                .combine(ch_bed_for_frip, by: 0)
                .map { _id, meta, bam, bai, bed -> [meta, bam, bai, bed] }

            FRIP(ch_frip_input)
        }

        // ---- HOMER motif analysis -----------------------------------
        if (!params.skip_motifs) {
            ch_motif_bed = ch_peaks.map { meta, files ->
                def file_list = (files instanceof List) ? files : [files]
                def bed = file_list.find { it.name.endsWith('.narrowPeak') || it.name.endsWith('.broadPeak') }
                return bed ? [meta, bed] : null
            }.filter { it != null }

            HOMER_FINDMOTIFS(ch_motif_bed, ch_genome)
        }
    }

    // ------------------------------------------------------------------
    // Aggregate QC with MultiQC
    // ------------------------------------------------------------------
    if (!params.skip_multiqc) {
        ch_multiqc_files = Channel.empty()
            .mix(FASTP.out.json)
            .mix(BOWTIE2_ALIGN.out.log)
            .mix(SAMTOOLS_SORT.out.flagstat)
            .mix(SAMTOOLS_SORT.out.stats)
            .mix(SAMTOOLS_SORT.out.idxstats)
            .mix(PICARD_MARKDUPLICATES.out.metrics)
            .mix(SAMTOOLS_FILTER.out.flagstat)
            .mix(PICARD_COLLECTMULTIPLEMETRICS.out.metrics.map { _meta, files -> files })
            .collect()

        ch_multiqc_config = params.multiqc_config
            ? Channel.value(file(params.multiqc_config, checkIfExists: true))
            : Channel.value(ch_no_file)

        MULTIQC(ch_multiqc_files, ch_multiqc_config)
    }
}
