#!/usr/bin/env nextflow

nextflow.enable.dsl = 2

include { CHIPSEQ } from './workflows/chipseq.nf'

workflow {
    CHIPSEQ()
}
