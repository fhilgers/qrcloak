package com.github.fhilgers.qrcloak.ui.screens.shared

import androidx.compose.foundation.layout.RowScope
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.MediumTopAppBar
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.compositionLocalOf
import androidx.compose.runtime.getValue
import androidx.compose.runtime.setValue
import cafe.adriel.voyager.core.stack.SnapshotStateStack
import cafe.adriel.voyager.core.stack.mutableStateStackOf

data class TopAppBarData(
    val title: SnapshotStateStack<@Composable () -> Unit> = mutableStateStackOf(),
    val navigationIcon: SnapshotStateStack<@Composable () -> Unit> = mutableStateStackOf(),
    val actions:
        SnapshotStateStack<
            @Composable()
            (RowScope.() -> Unit),
        > =
        mutableStateStackOf(),
)

val LocalTopAppBarProvider = compositionLocalOf { TopAppBarData() }

@Composable
fun SetAppBar(
    title: @Composable () -> Unit,
    navigationIcon: @Composable () -> Unit,
    actions: @Composable RowScope.() -> Unit,
) {
    val provider = LocalTopAppBarProvider.current

    DisposableEffect(title, navigationIcon, actions) {
        val previousTitle = provider.title.lastItemOrNull
        val previousNavigationIcon = provider.navigationIcon.lastItemOrNull
        val previousActions = provider.navigationIcon.lastItemOrNull

        provider.title.push(title)
        provider.navigationIcon.push(navigationIcon)
        provider.actions.push(actions)

        onDispose {
            provider.title.popUntil { it == previousTitle }
            provider.navigationIcon.popUntil { it == previousNavigationIcon }
            provider.navigationIcon.popUntil { it == previousActions }
        }
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun CurrentAppBar() {
    val provider = LocalTopAppBarProvider.current

    val title = provider.title.lastItemOrNull
    val navigation = provider.navigationIcon.lastItemOrNull
    val actions = provider.actions.lastItemOrNull

    if (title != null && navigation != null && actions != null) {
        MediumTopAppBar(
            title = { title() },
            navigationIcon = { navigation() },
            actions = { actions() },
        )
    }
}
