package com.github.fhilgers.qrcloak.ui.composables

import androidx.compose.foundation.layout.Column
import androidx.compose.runtime.Composable

@Composable
fun QrCodeDetails(qrCodes: List<String>, type: String, tags: Set<TagData>) {

    Column { QrCodePager(qrCodes = qrCodes) }
}
