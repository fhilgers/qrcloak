package com.github.fhilgers.qrcloak.ui.screens.saved.detail

import android.os.Parcelable
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import cafe.adriel.voyager.core.screen.Screen
import com.github.fhilgers.qrcloak.ui.composables.ScrollableOutlinedBase64Text
import kotlinx.parcelize.Parcelize

@Composable
fun NormalDetail(text: String, modifier: Modifier = Modifier) {
    Column(modifier = modifier.fillMaxWidth().padding(16.dp)) {
        Spacer(modifier = Modifier.height(16.dp))

        ScrollableOutlinedBase64Text(text = text, modifier = Modifier.fillMaxWidth())
    }
}

@Parcelize
data class NormalDetailScreen(val data: String) : Screen, Parcelable {
    @Composable
    override fun Content() {
        NormalDetail(text = data)
    }
}
