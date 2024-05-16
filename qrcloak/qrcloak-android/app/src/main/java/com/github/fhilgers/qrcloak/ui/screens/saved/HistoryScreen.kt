package com.github.fhilgers.qrcloak.ui.screens.saved

import android.os.Parcelable
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.NavigateNext
import androidx.compose.material.icons.filled.Compress
import androidx.compose.material.icons.filled.Create
import androidx.compose.material.icons.filled.EnhancedEncryption
import androidx.compose.material.icons.filled.Inventory2
import androidx.compose.material.icons.filled.NavigateNext
import androidx.compose.material.icons.filled.Numbers
import androidx.compose.material3.FloatingActionButton
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.ListItem
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.MutableState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.datastore.core.DataStore
import androidx.datastore.preferences.core.Preferences
import androidx.datastore.preferences.core.edit
import cafe.adriel.voyager.core.model.ScreenModel
import cafe.adriel.voyager.core.model.rememberScreenModel
import cafe.adriel.voyager.core.model.screenModelScope
import cafe.adriel.voyager.core.screen.Screen
import cafe.adriel.voyager.navigator.LocalNavigator
import cafe.adriel.voyager.navigator.currentOrThrow
import com.github.fhilgers.qrcloak.COMPLETE_KEY
import com.github.fhilgers.qrcloak.GROUP_KEY
import com.github.fhilgers.qrcloak.NORMAL_KEY
import com.github.fhilgers.qrcloak.R
import com.github.fhilgers.qrcloak.RAW_KEY
import com.github.fhilgers.qrcloak.dataStore
import com.github.fhilgers.qrcloak.ui.composables.Tag
import com.github.fhilgers.qrcloak.ui.composables.TagData
import com.github.fhilgers.qrcloak.ui.composables.TagRow
import com.github.fhilgers.qrcloak.ui.screens.SetAppBar
import com.github.fhilgers.qrcloak.ui.screens.SetFab
import com.github.fhilgers.qrcloak.ui.screens.saved.detail.CompleteDetailScreen
import com.github.fhilgers.qrcloak.ui.screens.saved.detail.GroupDetailScreen
import com.github.fhilgers.qrcloak.ui.screens.saved.detail.NormalDetailScreen
import com.github.fhilgers.qrcloak.utils.CompletePayloadParceler
import com.github.fhilgers.qrcloak.utils.OptionalPartialPayloadParceler
import com.github.fhilgers.qrcloak.utils.compressionTag
import com.github.fhilgers.qrcloak.utils.dataString
import com.github.fhilgers.qrcloak.utils.encryptionTag
import com.github.fhilgers.qrcloak.utils.id
import com.github.fhilgers.qrcloak.utils.index
import com.github.fhilgers.qrcloak.utils.tag
import com.github.fhilgers.qrcloak.utils.toPayload
import kotlinx.coroutines.DelicateCoroutinesApi
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.async
import kotlinx.coroutines.coroutineScope
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.forEach
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.merge
import kotlinx.coroutines.launch
import kotlinx.coroutines.newSingleThreadContext
import kotlinx.parcelize.Parcelize
import kotlinx.parcelize.TypeParceler
import uniffi.qrcloak_bindings.AgeRecipient
import uniffi.qrcloak_bindings.Compression
import uniffi.qrcloak_bindings.Encryption
import uniffi.qrcloak_bindings.GzipCompression
import uniffi.qrcloak_bindings.Passphrase
import uniffi.qrcloak_bindings.PayloadDecoder
import uniffi.qrcloak_bindings.PayloadEncoder
import uniffi.qrcloak_bindings.PayloadGenerator
import uniffi.qrcloak_bindings.PayloadMerger
import uniffi.qrcloak_bindings.PayloadSplitter
import uniffi.qrcloak_core.CompletePayload
import uniffi.qrcloak_core.DecodingOpts
import uniffi.qrcloak_core.EncodingOpts
import uniffi.qrcloak_core.PartialPayload
import uniffi.qrcloak_core.Payload

class HistoryScreenModel(dataStore: DataStore<Preferences>) : ScreenModel {

    val qrCodes: MutableState<List<QrCode>> = mutableStateOf(listOf())

    init {
        screenModelScope.launch { QrCode.fromStore(dataStore).collect { qrCodes.value = it } }
    }
}

@Parcelize
data object HistoryScreen : Screen, Parcelable {

    @Composable
    override fun Content() {

        val dataStore = LocalContext.current.dataStore
        val model =
            rememberScreenModel<HistoryScreenModel> { HistoryScreenModel(dataStore = dataStore) }

        val navigator = LocalNavigator.currentOrThrow

        SetAppBar(title = { Text(text = "Saved QRCodes") }, navigationIcon = {}, actions = {})

        SetFab {
            FloatingActionButton(onClick = { navigator.push(CreateScreen) }) {
                Icon(imageVector = Icons.Default.Create, contentDescription = "Create")
            }
        }

        QrCodeList(
            qrCodes = model.qrCodes.value,
            onClick = {
                when (it) {
                    is QrCode.Complete -> navigator.push(CompleteDetailScreen(it.payload))
                    is QrCode.Group -> navigator.push(GroupDetailScreen(it.id, it.payloads))
                    is QrCode.Normal -> navigator.push(NormalDetailScreen(data = it.data))
                }
            },
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
    data class Group(val id: UInt, val payloads: List<PartialPayload?>) : QrCode

    companion object {}
}

fun QrCode.Companion.fromStore(dataStore: DataStore<Preferences>): Flow<List<QrCode>> {
    return dataStore.data.map {
        val decoder = PayloadDecoder().withDecoding(DecodingOpts.JSON)

        val encodedNormals = it[NORMAL_KEY] ?: emptySet()
        val encodedGroups = it[GROUP_KEY] ?: emptySet()
        val encodedCompletes = it[COMPLETE_KEY] ?: emptySet()
        val raw = it[RAW_KEY] ?: emptySet()

        val leftOver = mutableSetOf<String>()
        val rawPayloads = mutableSetOf<Payload>()

        raw.forEach() {
            try {
                val payload = decoder.decode(it)[0]

                rawPayloads.add(payload)
            } catch (_: Exception) {
                leftOver.add(it)
            }
        }

        val mergeResult = PayloadMerger().merge(rawPayloads.toList())

        val rawNormals = leftOver.map { QrCode.Normal(it) }
        val rawCompletes = mergeResult.complete.map { QrCode.Complete(it) }
        val rawGroups =
            mergeResult.incomplete.partials.map {
                QrCode.Group(id = it.key.id, payloads = it.value)
            }

        (rawNormals + rawGroups + rawCompletes).save(dataStore)
        dataStore.edit { it[RAW_KEY] = emptySet() }

        val normals = encodedNormals.map { it }.map { QrCode.Normal(it) }
        val completes =
            encodedCompletes
                .map { (decoder.decode(it)[0] as Payload.Complete).v1 }
                .map { QrCode.Complete(it) }
        val groups =
            encodedGroups
                .map { decoder.decode(it) }
                .map {
                    val merged = PayloadMerger().merge(it)

                    if (merged.complete.size == 1) {
                        it.map { (it as Payload.Partial).v1 }
                    } else {
                        merged.incomplete.partials.values.first()
                    }
                }
                .map { QrCode.Group(id = it.id, payloads = it) }

        normals + completes + groups
    }
}

suspend fun QrCode.Normal.save(dataStore: DataStore<Preferences>) {
    dataStore.edit { qrCodes ->
        val previous = qrCodes[NORMAL_KEY] ?: emptySet()
        qrCodes[NORMAL_KEY] = previous + data
    }
}

suspend fun QrCode.Complete.save(dataStore: DataStore<Preferences>) {
    val encoder = PayloadEncoder().withEncoding(EncodingOpts.Json(pretty = false, merge = true))

    dataStore.edit { qrCodes ->
        val previous = qrCodes[COMPLETE_KEY] ?: emptySet()
        qrCodes[COMPLETE_KEY] = previous + encoder.encode(listOf(payload.toPayload()))[0]
    }
}

suspend fun QrCode.Group.save(dataStore: DataStore<Preferences>) {
    val encoder = PayloadEncoder().withEncoding(EncodingOpts.Json(pretty = false, merge = true))

    dataStore.edit { qrCodes ->
        val previous = qrCodes[GROUP_KEY] ?: emptySet()

        qrCodes[GROUP_KEY] = previous + encoder.encode(payloads.mapNotNull { it?.toPayload() })[0]
    }
}

suspend fun QrCode.save(dataStore: DataStore<Preferences>) {
    when (this) {
        is QrCode.Complete -> this.save(dataStore)
        is QrCode.Group -> this.save(dataStore)
        is QrCode.Normal -> this.save(dataStore)
    }
}

suspend fun List<QrCode>.save(dataStore: DataStore<Preferences>) {
    forEach { it.save(dataStore) }
}

@Composable
fun NoPayloadListItem(data: String, onClick: () -> Unit, modifier: Modifier = Modifier) {
    ListItem(
        overlineContent = { Text(text = stringResource(id = R.string.no_payload_tag)) },
        headlineContent = { Text(text = data, maxLines = 1, overflow = TextOverflow.Ellipsis) },
        trailingContent = {
            Icon(
                imageVector = Icons.AutoMirrored.Default.NavigateNext,
                contentDescription = "NavigateNext"
            )
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
            Icon(
                imageVector = Icons.AutoMirrored.Default.NavigateNext,
                contentDescription = "NavigateNext"
            )
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
            Icon(
                imageVector = Icons.AutoMirrored.Default.NavigateNext,
                contentDescription = "NavigateNext"
            )
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
        verticalArrangement = Arrangement.Top,
        modifier = modifier,
    ) {
        itemsIndexed(qrCodes) { index, qrCode ->
            QrCodeListItem(qrCode = qrCode, onClick = { onClick(qrCode) })

            Spacer(modifier = Modifier.height(8.dp))

            if (index < qrCodes.lastIndex) {
                HorizontalDivider()

                Spacer(modifier = Modifier.height(8.dp))
            }
        }
    }
}

@OptIn(ExperimentalCoroutinesApi::class, DelicateCoroutinesApi::class)
suspend fun makeDummyList(): List<QrCode> = coroutineScope {
    val ageRecipient =
        AgeRecipient.tryFromString("age1jkrld9cvkwlrtxqzf4ymlv6vfpaqnkephks0t2t4gz4lkx2e0vaq6z7yc4")

    val encryptedPayload =
        PayloadGenerator()
            .withEncryption(Encryption.AgeKey(listOf(ageRecipient)))
            .withCompression(Compression.Gzip(GzipCompression()))
            .generate("hello")

    val pwEncrypted =
        async(newSingleThreadContext("Key Derivation")) {
                PayloadGenerator()
                    .withEncryption(
                        Encryption.AgePassphrase(passphrase = Passphrase("hello world"))
                    )
                    .generate("arosetin")
            }
            .await()

    val payload = PayloadGenerator().generate("hello")
    val normal = "hello"

    val partials = PayloadSplitter().withSplits(4u).split(encryptedPayload)

    val partialsMapped = partials.map { (it as Payload.Partial).v1 }

    val someMerged = PayloadMerger().merge(partials.subList(0, 3))

    val p = someMerged.incomplete.partials.map { QrCode.Group(id = it.key.id, payloads = it.value) }

    return@coroutineScope listOf(QrCode.Normal(data = normal), QrCode.Complete(payload = payload)) +
        p +
        QrCode.Complete(payload = pwEncrypted) +
        QrCode.Complete(payload = encryptedPayload) +
        QrCode.Group(id = partialsMapped[0].id!!, payloads = partialsMapped)
}

@Preview
@Composable
fun PreviewQrCodeList() {

    var qrCodes by remember { mutableStateOf(listOf<QrCode>()) }

    LaunchedEffect(Unit) { qrCodes = makeDummyList() }

    QrCodeList(qrCodes = qrCodes, onClick = {})
}
