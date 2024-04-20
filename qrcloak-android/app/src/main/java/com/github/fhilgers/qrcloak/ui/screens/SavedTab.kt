package com.github.fhilgers.qrcloak.ui.screens

import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.material3.FloatingActionButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import cafe.adriel.voyager.navigator.tab.TabOptions
import com.github.fhilgers.qrcloak.R

object SavedTab : ExtendedTab {

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

        LazyColumn(modifier = Modifier.fillMaxSize()) {}
    }

    @Composable
    override fun Fab() {
        FloatingActionButton(onClick = { /*TODO*/}) {}
    }
}
