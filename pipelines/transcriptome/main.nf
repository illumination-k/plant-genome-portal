#!/usr/bin/env nextflow

nextflow.enable.dsl = 2

include { TRANSCRIPTOME } from './workflows/transcriptome.nf'

workflow {
    TRANSCRIPTOME()
}
