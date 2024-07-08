// SPDX-FileCopyrightText: 2024 Felix Hilgers <contact@fhilgers.com>
//
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

package com.github.fhilgers.qrcloak.ui.screens.shared

import androidx.compose.animation.AnimatedContent
import androidx.compose.animation.SizeTransform
import androidx.compose.animation.scaleIn
import androidx.compose.animation.scaleOut
import androidx.compose.animation.togetherWith
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.compositionLocalOf
import androidx.compose.runtime.getValue
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import cafe.adriel.voyager.core.stack.mutableStateStackOf

val LocalFabProvider = compositionLocalOf { mutableStateStackOf<@Composable () -> Unit>() }

@Composable
fun SetFab(
    key: Any? = null,
    content: @Composable () -> Unit,
) {
    val fabProvider = LocalFabProvider.current

    DisposableEffect(key, content) {
        val previous = fabProvider.lastItemOrNull
        fabProvider.push(content)

        onDispose { fabProvider.popUntil { it == previous } }
    }
}

@Composable
fun CurrentFab(modifier: Modifier = Modifier) {
    AnimatedContent(
        targetState = LocalFabProvider.current.lastItemOrNull,
        contentAlignment = Alignment.Center,
        transitionSpec = { scaleIn().togetherWith(scaleOut()).using(SizeTransform(clip = false)) },
        label = "CurrentFab",
        modifier = modifier,
    ) {
        when (it) {
            null -> {}
            else -> it()
        }
    }
}
