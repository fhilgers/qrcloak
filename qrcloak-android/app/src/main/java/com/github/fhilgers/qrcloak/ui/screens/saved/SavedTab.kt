package com.github.fhilgers.qrcloak.ui.screens.saved

import android.os.Parcelable
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import cafe.adriel.voyager.navigator.CurrentScreen
import cafe.adriel.voyager.navigator.Navigator
import cafe.adriel.voyager.navigator.tab.Tab
import cafe.adriel.voyager.navigator.tab.TabOptions
import com.github.fhilgers.qrcloak.R
import com.github.fhilgers.qrcloak.utils.id
import kotlinx.parcelize.Parcelize

@Parcelize
object SavedTab : Tab, Parcelable {
    private fun readResolve(): Any = SavedTab

    override val options: TabOptions
        @Composable
        get() {
            val title = stringResource(id = R.string.saved_tab_name)
            val icon = painterResource(id = R.drawable.inventory_2)

            return remember { TabOptions(index = 1u, title = title, icon = icon) }
        }

    @Composable
    override fun Content() {

        Navigator(screen = HistoryScreen(qrCodes = listOf())) { navigator ->
            LaunchedEffect(this) { navigator.replaceAll(HistoryScreen(qrCodes = makeDummyList())) }

            CurrentScreen()
        }
    }
}
