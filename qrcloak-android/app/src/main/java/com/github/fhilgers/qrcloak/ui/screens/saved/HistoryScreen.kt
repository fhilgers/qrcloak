package com.github.fhilgers.qrcloak.ui.screens.saved

import android.os.Parcelable
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Compress
import androidx.compose.material.icons.filled.EnhancedEncryption
import androidx.compose.material.icons.filled.Inventory2
import androidx.compose.material.icons.filled.NavigateNext
import androidx.compose.material.icons.filled.Numbers
import androidx.compose.material3.Divider
import androidx.compose.material3.Icon
import androidx.compose.material3.ListItem
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import cafe.adriel.voyager.core.screen.Screen
import cafe.adriel.voyager.navigator.LocalNavigator
import cafe.adriel.voyager.navigator.currentOrThrow
import com.github.fhilgers.qrcloak.R
import com.github.fhilgers.qrcloak.ui.composables.Tag
import com.github.fhilgers.qrcloak.ui.composables.TagData
import com.github.fhilgers.qrcloak.ui.composables.TagRow
import com.github.fhilgers.qrcloak.utils.CompletePayloadParceler
import com.github.fhilgers.qrcloak.utils.OptionalPartialPayloadParceler
import com.github.fhilgers.qrcloak.utils.compressionTag
import com.github.fhilgers.qrcloak.utils.dataString
import com.github.fhilgers.qrcloak.utils.encryptionTag
import com.github.fhilgers.qrcloak.utils.id
import com.github.fhilgers.qrcloak.utils.tag
import kotlinx.parcelize.Parcelize
import kotlinx.parcelize.TypeParceler
import uniffi.qrcloak_bindings.AgeIdentity
import uniffi.qrcloak_bindings.Compression
import uniffi.qrcloak_bindings.Encryption
import uniffi.qrcloak_bindings.GzipCompression
import uniffi.qrcloak_bindings.PayloadGenerator
import uniffi.qrcloak_bindings.PayloadMerger
import uniffi.qrcloak_bindings.PayloadSplitter
import uniffi.qrcloak_core.CompletePayload
import uniffi.qrcloak_core.PartialPayload

@Parcelize
data class HistoryScreen(val qrCodes: List<QrCode>) : Screen, Parcelable {

    @Composable
    override fun Content() {

        val navigator = LocalNavigator.currentOrThrow

        QrCodeList(
            qrCodes = qrCodes,
            onClick = { navigator.push(DetailScreen(qrCode = it)) },
            modifier = Modifier.fillMaxSize()
        )
    }
}

@Parcelize
sealed interface QrCode : Parcelable {

    @Parcelize data class Normal(val data: String) : QrCode

    @Parcelize
    @TypeParceler<CompletePayload, CompletePayloadParceler>
    data class Complete(val payload: CompletePayload) : QrCode

    @Parcelize
    @TypeParceler<PartialPayload?, OptionalPartialPayloadParceler>
    data class Group(val size: UInt, val id: UInt, val payloads: List<PartialPayload?>) : QrCode
}

@Composable
fun NoPayloadListItem(data: String, onClick: () -> Unit, modifier: Modifier = Modifier) {
    ListItem(
        overlineContent = { Text(text = stringResource(id = R.string.no_payload_tag)) },
        headlineContent = { Text(text = data, maxLines = 1, overflow = TextOverflow.Ellipsis) },
        trailingContent = {
            Icon(imageVector = Icons.Default.NavigateNext, contentDescription = "NavigateNext")
        },
        modifier = modifier.clickable(onClick = onClick)
    )
}

@OptIn(ExperimentalLayoutApi::class)
@Composable
fun CompletePayloadListItem(
    payload: CompletePayload,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
) {
    ListItem(
        overlineContent = { Text(text = payload.tag) },
        headlineContent = {
            Text(text = payload.dataString, maxLines = 1, overflow = TextOverflow.Ellipsis)
        },
        supportingContent = {
            TagRow {
                Tag(TagData(text = payload.encryptionTag, icon = Icons.Default.EnhancedEncryption))

                Tag(TagData(text = payload.compressionTag, icon = Icons.Default.Compress))
            }
        },
        trailingContent = {
            Icon(imageVector = Icons.Default.NavigateNext, contentDescription = "NavigateNext")
        },
        modifier = modifier.clickable(onClick = onClick)
    )
}

@OptIn(ExperimentalLayoutApi::class)
@Composable
fun GroupPayloadListItem(
    payload: List<PartialPayload?>, // TODO: add class for this
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
) {
    ListItem(
        overlineContent = { Text(text = payload.tag) },
        headlineContent = {
            Text(text = payload.dataString, maxLines = 1, overflow = TextOverflow.Ellipsis)
        },
        supportingContent = {
            TagRow {
                val total = payload.count()
                val there = payload.mapNotNull { it }.count()

                Tag(TagData(text = "$there/$total", icon = Icons.Default.Inventory2))

                Tag(TagData(text = "${payload.id}", icon = Icons.Default.Numbers))

                Tag(TagData(text = payload.encryptionTag, icon = Icons.Default.EnhancedEncryption))

                Tag(TagData(text = payload.compressionTag, icon = Icons.Default.Compress))
            }
        },
        trailingContent = {
            Icon(imageVector = Icons.Default.NavigateNext, contentDescription = "NavigateNext")
        },
        modifier = modifier.clickable(onClick = onClick)
    )
}

@Composable
fun QrCodeListItem(qrCode: QrCode, onClick: () -> Unit, modifier: Modifier = Modifier) {
    when (qrCode) {
        is QrCode.Complete ->
            CompletePayloadListItem(
                payload = qrCode.payload,
                onClick = onClick,
                modifier = modifier
            )
        is QrCode.Normal ->
            NoPayloadListItem(data = qrCode.data, onClick = onClick, modifier = modifier)
        is QrCode.Group ->
            GroupPayloadListItem(payload = qrCode.payloads, onClick = onClick, modifier = modifier)
    }
}

@Composable
fun QrCodeList(qrCodes: List<QrCode>, onClick: (QrCode) -> Unit, modifier: Modifier = Modifier) {
    LazyColumn(
        contentPadding = PaddingValues(12.dp),
        verticalArrangement = Arrangement.spacedBy(8.dp),
        modifier = modifier,
    ) {
        items(qrCodes) { qrCode ->
            QrCodeListItem(qrCode = qrCode, onClick = { onClick(qrCode) })
            Divider()
        }
    }
}

fun makeDummyList(): List<QrCode> {
    val encryptedPayload =
        PayloadGenerator()
            .withEncryption(Encryption.AgeKey(listOf(AgeIdentity.generate().toPublic())))
            .withCompression(Compression.Gzip(GzipCompression()))
            .generate("hello")

    val payload = PayloadGenerator().generate("hello")
    val normal = "hello"

    val partials = PayloadSplitter().withSplits(4u).split(encryptedPayload)

    val someMerged = PayloadMerger().merge(partials.subList(0, 3))

    val p =
        someMerged.incomplete.partials.map {
            QrCode.Group(size = it.key.size, id = it.key.id, payloads = it.value)
        }

    return listOf(QrCode.Normal(normal), QrCode.Complete(payload)) +
        p +
        QrCode.Complete(encryptedPayload)
}

@Preview
@Composable
fun PreviewQrCodeList() {

    QrCodeList(qrCodes = makeDummyList(), onClick = {})
}
