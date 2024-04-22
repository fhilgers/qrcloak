package com.github.fhilgers.qrcloak.ui.screens

import android.os.Parcelable
import android.util.Log
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
import androidx.compose.foundation.layout.size
import androidx.compose.material3.Icon
import androidx.compose.material3.NavigationBar
import androidx.compose.material3.NavigationBarItem
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.compositionLocalOf
import androidx.compose.runtime.getValue
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import cafe.adriel.voyager.core.screen.Screen
import cafe.adriel.voyager.core.stack.mutableStateStackOf
import cafe.adriel.voyager.navigator.CurrentScreen
import cafe.adriel.voyager.navigator.tab.LocalTabNavigator
import cafe.adriel.voyager.navigator.tab.Tab
import cafe.adriel.voyager.navigator.tab.TabNavigator
import com.github.fhilgers.qrcloak.ui.screens.saved.SavedTab
import kotlinx.parcelize.Parcelize

@Parcelize
object RootScreen : Screen, Parcelable {

    private fun readResolve(): Any = RootScreen

    @Composable
    override fun Content() {
        TabNavigator(ScannerTab) { navigator ->
            Scaffold(
                bottomBar = {
                    BottomNavigation {
                        BottomNavigationItem(tab = ScannerTab)
                        BottomNavigationItem(tab = SavedTab)
                    }
                },
                floatingActionButton = {
                    Box(
                        modifier =
                            Modifier.consumeWindowInsets(WindowInsets.navigationBars)
                                .consumeWindowInsets(PaddingValues(vertical = 80.dp))
                                .imePadding()
                    ) {
                        CurrentFab()
                    }
                }
            ) { contentPadding ->
                Box(modifier = Modifier.padding(contentPadding)) { CurrentScreen() }
            }
        }
    }
}

val LocalFabProvider = compositionLocalOf { mutableStateStackOf<@Composable () -> Unit>() }

@Composable
fun SetFab(content: @Composable () -> Unit) {
    val fabProvider = LocalFabProvider.current

    DisposableEffect(content) {
        val previous = fabProvider.lastItemOrNull
        fabProvider.push(content)

        onDispose { fabProvider.popUntil { it == previous } }
    }
}

@Composable
fun CurrentFab() {

    AnimatedContent(
        targetState = LocalFabProvider.current.lastItemOrNull,
        contentAlignment = Alignment.Center,
        transitionSpec = { scaleIn().togetherWith(scaleOut()).using(SizeTransform(clip = false)) }
    ) {
        Log.d("arsoten", "$it")

        when (it) {
            null ->
                Box(
                    modifier = Modifier.size(40.dp)
                ) // STUPID BUG IN COMPOSE, otherwise it does not render
            else -> it()
        }
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
