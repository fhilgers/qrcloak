// SPDX-FileCopyrightText: 2024 Felix Hilgers <contact@fhilgers.com>
//
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

package com.github.fhilgers.qrcloak.ui.screens

import android.os.Parcelable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.RowScope
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.consumeWindowInsets
import androidx.compose.foundation.layout.imePadding
import androidx.compose.foundation.layout.navigationBars
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Icon
import androidx.compose.material3.NavigationBar
import androidx.compose.material3.NavigationBarItem
import androidx.compose.material3.Scaffold
import androidx.compose.material3.SnackbarHost
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import cafe.adriel.voyager.core.screen.Screen
import cafe.adriel.voyager.navigator.CurrentScreen
import cafe.adriel.voyager.navigator.tab.LocalTabNavigator
import cafe.adriel.voyager.navigator.tab.Tab
import cafe.adriel.voyager.navigator.tab.TabNavigator
import com.github.fhilgers.qrcloak.ui.composables.SingleLineSnackbar
import com.github.fhilgers.qrcloak.ui.screens.saved.SavedTab
import com.github.fhilgers.qrcloak.ui.screens.shared.CurrentAppBar
import com.github.fhilgers.qrcloak.ui.screens.shared.CurrentFab
import com.github.fhilgers.qrcloak.ui.screens.shared.LocalSnackbarHostState
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
                                .imePadding(),
                    )
                },
                contentWindowInsets = WindowInsets.navigationBars,
            ) { contentPadding ->
                Box(
                    modifier =
                        Modifier.padding(contentPadding)
                            .consumeWindowInsets(contentPadding)
                            .imePadding(),
                ) {
                    CurrentScreen()
                }
            }
        }
    }
}

@Composable
private fun BottomNavigation(content: @Composable RowScope.() -> Unit) {
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
