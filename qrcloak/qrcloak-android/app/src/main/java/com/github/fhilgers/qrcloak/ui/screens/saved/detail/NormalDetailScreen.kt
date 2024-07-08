// SPDX-FileCopyrightText: 2024 Felix Hilgers <contact@fhilgers.com>
//
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

package com.github.fhilgers.qrcloak.ui.screens.saved.detail

import android.os.Parcelable
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.filled.ArrowBack
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import cafe.adriel.voyager.core.screen.Screen
import cafe.adriel.voyager.navigator.LocalNavigator
import cafe.adriel.voyager.navigator.currentOrThrow
import com.github.fhilgers.qrcloak.ui.composables.ScrollableOutlinedBase64Text
import com.github.fhilgers.qrcloak.ui.screens.shared.SetAppBar
import kotlinx.parcelize.Parcelize

@Composable
fun NormalDetail(
    text: String,
    modifier: Modifier = Modifier,
) {
    Column(modifier = modifier.fillMaxWidth().padding(16.dp)) {
        Spacer(modifier = Modifier.height(16.dp))

        ScrollableOutlinedBase64Text(text = text, modifier = Modifier.fillMaxWidth())
    }
}

@Parcelize
data class NormalDetailScreen(val data: String) : Screen, Parcelable {
    @Composable
    override fun Content() {
        val navigator = LocalNavigator.currentOrThrow

        SetAppBar(
            title = { Text(text = "Normal") },
            navigationIcon = {
                IconButton(onClick = { navigator.pop() }) {
                    Icon(
                        imageVector = Icons.AutoMirrored.Default.ArrowBack,
                        contentDescription = "Navigate Back",
                    )
                }
            },
            actions = {},
        )

        NormalDetail(text = data)
    }
}
