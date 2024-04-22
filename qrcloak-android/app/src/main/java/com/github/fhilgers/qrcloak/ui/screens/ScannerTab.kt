package com.github.fhilgers.qrcloak.ui.screens

import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import cafe.adriel.voyager.navigator.tab.Tab
import cafe.adriel.voyager.navigator.tab.TabOptions
import com.github.fhilgers.qrcloak.R

object ScannerTab : Tab {

    private fun readResolve(): Any = ScannerTab

    override val options: TabOptions
        @Composable
        get() {
            val title = stringResource(id = R.string.scanner_tab_name)
            val icon = painterResource(id = R.drawable.qr_code_scanner)

            return remember { TabOptions(index = 0u, title = title, icon = icon) }
        }

    @Composable override fun Content() {}
}
