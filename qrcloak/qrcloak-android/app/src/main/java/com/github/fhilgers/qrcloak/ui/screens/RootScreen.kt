package com.github.fhilgers.qrcloak.ui.screens

import android.os.Parcelable
import androidx.compose.animation.AnimatedContent
import androidx.compose.animation.SizeTransform
import androidx.compose.animation.scaleIn
import androidx.compose.animation.scaleOut
import androidx.compose.animation.togetherWith
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.RowScope
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.consumeWindowInsets
import androidx.compose.foundation.layout.imePadding
import androidx.compose.foundation.layout.navigationBars
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.MediumTopAppBar
import androidx.compose.material3.NavigationBar
import androidx.compose.material3.NavigationBarItem
import androidx.compose.material3.Scaffold
import androidx.compose.material3.SnackbarHost
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.compositionLocalOf
import androidx.compose.runtime.getValue
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import cafe.adriel.voyager.core.screen.Screen
import cafe.adriel.voyager.core.stack.SnapshotStateStack
import cafe.adriel.voyager.core.stack.mutableStateStackOf
import cafe.adriel.voyager.navigator.CurrentScreen
import cafe.adriel.voyager.navigator.tab.LocalTabNavigator
import cafe.adriel.voyager.navigator.tab.Tab
import cafe.adriel.voyager.navigator.tab.TabNavigator
import com.github.fhilgers.qrcloak.ui.composables.SingleLineSnackbar
import com.github.fhilgers.qrcloak.ui.screens.saved.SavedTab
import com.google.accompanist.permissions.ExperimentalPermissionsApi
import com.google.accompanist.permissions.isGranted
import com.google.accompanist.permissions.rememberPermissionState
import kotlinx.parcelize.Parcelize

@Parcelize
object RootScreen : Screen, Parcelable {

    private fun readResolve(): Any = RootScreen

    @OptIn(ExperimentalPermissionsApi::class)
    @Composable
    override fun Content() {

        val cameraPermissionState = rememberPermissionState(android.Manifest.permission.CAMERA)

        LaunchedEffect(cameraPermissionState.status) {
            if (!cameraPermissionState.status.isGranted) {
                cameraPermissionState.launchPermissionRequest()
            }
        }

        val snackbarHostState = LocalSnackbarHostState.current

        TabNavigator(ScannerTab) { _ ->
            Scaffold(
                snackbarHost = {
                    SnackbarHost(
                        hostState = snackbarHostState,
                    ) {
                        SingleLineSnackbar(snackbarData = it)
                    }
                },
                bottomBar = {
                    BottomNavigation {
                        BottomNavigationItem(tab = ScannerTab)
                        BottomNavigationItem(tab = SavedTab)
                    }
                },
                topBar = { CurrentAppBar() },
                floatingActionButton = {
                    CurrentFab(
                        modifier =
                            Modifier.consumeWindowInsets(WindowInsets.navigationBars)
                                .consumeWindowInsets(PaddingValues(vertical = 80.dp))
                                .imePadding()
                    )
                },
                contentWindowInsets = WindowInsets.navigationBars
            ) { contentPadding ->
                Box(
                    modifier =
                        Modifier.padding(contentPadding)
                            .consumeWindowInsets(contentPadding)
                            .imePadding()
                ) {
                    CurrentScreen()
                }
            }
        }
    }
}

data class TopAppBarData(
    val title: SnapshotStateStack<@Composable () -> Unit> = mutableStateStackOf(),
    val navigationIcon: SnapshotStateStack<@Composable () -> Unit> = mutableStateStackOf(),
    val actions: SnapshotStateStack<@Composable() (RowScope.() -> Unit)> = mutableStateStackOf()
)

val LocalSnackbarHostState = compositionLocalOf { SnackbarHostState() }
val LocalTopAppBarProvider = compositionLocalOf { TopAppBarData() }
val LocalFabProvider = compositionLocalOf { mutableStateStackOf<@Composable () -> Unit>() }

@Composable
fun SetFab(key: Any? = null, content: @Composable () -> Unit) {
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
        modifier = modifier
    ) {
        when (it) {
            null -> {}
            else -> it()
        }
    }
}

@Composable
fun SetAppBar(
    title: @Composable () -> Unit,
    navigationIcon: @Composable () -> Unit,
    actions: @Composable RowScope.() -> Unit
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
            actions = { actions() }
        )
    }
}

@Composable
private fun BottomNavigation(
    content: @Composable RowScope.() -> Unit,
) {
    NavigationBar(content = content)
}

@Composable
private fun RowScope.BottomNavigationItem(tab: Tab) {
    val tabNavigator = LocalTabNavigator.current

    NavigationBarItem(
        selected = tabNavigator.current == tab,
        onClick = { tabNavigator.current = tab },
        label = { Text(text = tab.options.title) },
        icon = {
            tab.options.icon?.also { Icon(painter = it, contentDescription = tab.options.title) }
        },
    )
}