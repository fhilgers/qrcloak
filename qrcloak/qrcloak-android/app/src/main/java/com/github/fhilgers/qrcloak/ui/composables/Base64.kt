// SPDX-FileCopyrightText: 2024 Felix Hilgers <contact@fhilgers.com>
//
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

package com.github.fhilgers.qrcloak.ui.composables

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.LocalTextStyle
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextFieldDefaults
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.style.TextAlign

@Composable
fun ScrollableOutlinedBase64Text(
    text: String,
    modifier: Modifier = Modifier,
    maxLines: Int = Int.MAX_VALUE,
    isError: Boolean = false,
) {
    Box(
        modifier =
            modifier
                .border(
                    border =
                        BorderStroke(
                            width = OutlinedTextFieldDefaults.FocusedBorderThickness,
                            color =
                                if (isError) {
                                    MaterialTheme.colorScheme.error
                                } else {
                                    MaterialTheme.colorScheme.outline
                                },
                        ),
                    shape = OutlinedTextFieldDefaults.shape,
                )
                .padding(OutlinedTextFieldDefaults.contentPadding()),
    ) {
        Box(
            modifier = Modifier.verticalScroll(state = rememberScrollState()),
        ) {
            Text(
                text = text.map { it }.joinToString("\u200D"),
                style =
                    LocalTextStyle.current.copy(
                        fontFamily = FontFamily.Monospace,
                        textAlign = TextAlign.Justify,
                    ),
                maxLines = maxLines,
            )
        }
    }
}
