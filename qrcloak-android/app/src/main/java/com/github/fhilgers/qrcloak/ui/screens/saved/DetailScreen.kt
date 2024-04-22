package com.github.fhilgers.qrcloak.ui.screens.saved

import android.os.Parcelable
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Compress
import androidx.compose.material.icons.filled.EnhancedEncryption
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import cafe.adriel.voyager.core.screen.Screen
import com.github.fhilgers.qrcloak.ui.composables.ScrollableOutlinedBase64Text
import com.github.fhilgers.qrcloak.ui.composables.Tag
import com.github.fhilgers.qrcloak.ui.composables.TagData
import com.github.fhilgers.qrcloak.ui.composables.TagRow
import com.github.fhilgers.qrcloak.utils.compressionTag
import com.github.fhilgers.qrcloak.utils.dataString
import com.github.fhilgers.qrcloak.utils.encryptionTag
import kotlinx.parcelize.Parcelize
import uniffi.qrcloak_core.CompletePayload
import uniffi.qrcloak_core.PartialPayload

@Composable fun NormalDetails(data: String, modifier: Modifier = Modifier) {}

@Composable
fun GroupDetails(data: List<PartialPayload?>, id: UInt, modifier: Modifier = Modifier) {}

@OptIn(ExperimentalLayoutApi::class)
@Composable
fun CompleteDetails(data: CompletePayload, modifier: Modifier = Modifier) {
    Column(modifier = modifier.fillMaxSize().padding(16.dp)) {
        TagRow(modifier = Modifier.fillMaxWidth()) {
            Tag(TagData(text = data.encryptionTag, icon = Icons.Default.EnhancedEncryption))

            Tag(TagData(text = data.compressionTag, icon = Icons.Default.Compress))
        }

        ScrollableOutlinedBase64Text(text = data.dataString, modifier = Modifier.weight(1f))

        // Size of fab
        Spacer(modifier = Modifier.height(40.dp).padding(16.dp))
    }
}

@Parcelize
data class DetailScreen(val qrCode: QrCode) : Screen, Parcelable {

    @Composable
    override fun Content() {

        when (qrCode) {
            is QrCode.Complete -> CompleteDetails(data = qrCode.payload)
            is QrCode.Group -> GroupDetails(data = qrCode.payloads, id = qrCode.id)
            is QrCode.Normal -> NormalDetails(data = qrCode.data)
        }
    }
}
