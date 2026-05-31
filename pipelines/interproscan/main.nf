#!/usr/bin/env nextflow

nextflow.enable.dsl = 2

include { INTERPROSCAN_ANNOTATION } from './workflows/interproscan.nf'

workflow {
    INTERPROSCAN_ANNOTATION()
}
