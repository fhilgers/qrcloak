package com.github.fhilgers.qrcloak.ui.screens

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.RowScope
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Icon
import androidx.compose.material3.NavigationBar
import androidx.compose.material3.NavigationBarItem
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import cafe.adriel.voyager.core.screen.Screen
import cafe.adriel.voyager.navigator.CurrentScreen
import cafe.adriel.voyager.navigator.tab.LocalTabNavigator
import cafe.adriel.voyager.navigator.tab.Tab
import cafe.adriel.voyager.navigator.tab.TabNavigator

object RootScreen : Screen {

    private fun readResolve(): Any = RootScreen

    @Composable
    override fun Content() {

        TabNavigator(ScannerTab) { navigator ->
            val tab = navigator.current as ExtendedTab

            Scaffold(
                bottomBar = {
                    BottomNavigation {
                        BottomNavigationItem(tab = ScannerTab)
                        BottomNavigationItem(tab = SavedTab)
                    }
                },
                floatingActionButton = { tab.Fab() }
            ) { contentPadding ->
                Box(modifier = Modifier.padding(contentPadding)) { CurrentScreen() }
            }
        }
    }
}

interface ExtendedTab : Tab {

    @Composable fun Fab() {}
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
