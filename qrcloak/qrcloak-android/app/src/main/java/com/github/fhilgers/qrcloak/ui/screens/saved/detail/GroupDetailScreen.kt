package com.github.fhilgers.qrcloak.ui.screens.saved.detail

import android.os.Parcelable
import androidx.compose.foundation.ExperimentalFoundationApi
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.pager.HorizontalPager
import androidx.compose.foundation.pager.rememberPagerState
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.filled.ArrowBack
import androidx.compose.material.icons.filled.Compress
import androidx.compose.material.icons.filled.EnhancedEncryption
import androidx.compose.material.icons.filled.Inventory2
import androidx.compose.material.icons.filled.Merge
import androidx.compose.material.icons.filled.Numbers
import androidx.compose.material3.FloatingActionButton
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import cafe.adriel.voyager.core.screen.Screen
import cafe.adriel.voyager.navigator.LocalNavigator
import cafe.adriel.voyager.navigator.currentOrThrow
import com.github.fhilgers.qrcloak.ui.composables.ScrollableOutlinedBase64Text
import com.github.fhilgers.qrcloak.ui.composables.Tag
import com.github.fhilgers.qrcloak.ui.composables.TagData
import com.github.fhilgers.qrcloak.ui.composables.TagRow
import com.github.fhilgers.qrcloak.ui.screens.shared.SetAppBar
import com.github.fhilgers.qrcloak.ui.screens.shared.SetFab
import com.github.fhilgers.qrcloak.utils.OptionalPartialPayloadParceler
import com.github.fhilgers.qrcloak.utils.compressionTag
import com.github.fhilgers.qrcloak.utils.dataString
import com.github.fhilgers.qrcloak.utils.encryptionTag
import com.github.fhilgers.qrcloak.utils.id
import kotlinx.parcelize.Parcelize
import kotlinx.parcelize.TypeParceler
import uniffi.qrcloak_bindings.PayloadMerger
import uniffi.qrcloak_core.PartialPayload
import uniffi.qrcloak_core.Payload

@OptIn(ExperimentalLayoutApi::class, ExperimentalFoundationApi::class)
@Composable
fun GroupDetails(
    id: UInt,
    texts: List<String?>,
    encryptionTag: String,
    compressionTag: String,
    onMerge: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val pagerState = rememberPagerState { texts.size }

    val total = texts.count()
    val there = texts.mapNotNull { it }.count()

    if (total == there) {
        SetFab {
            FloatingActionButton(onClick = onMerge) {
                Icon(imageVector = Icons.Default.Merge, contentDescription = "Merged Payloads")
            }
        }
    }

    Column(modifier = modifier.padding(16.dp).fillMaxSize()) {
        TagRow(modifier = Modifier.fillMaxWidth()) {
            Tag(TagData(text = "$there/$total", icon = Icons.Default.Inventory2))

            Tag(TagData(text = "$id", icon = Icons.Default.Numbers))

            Tag(TagData(text = encryptionTag, icon = Icons.Default.EnhancedEncryption))

            Tag(TagData(text = compressionTag, icon = Icons.Default.Compress))
        }

        Spacer(modifier = Modifier.height(8.dp))

        HorizontalPager(
            state = pagerState,
            contentPadding = PaddingValues(16.dp),
            pageSpacing = 8.dp,
            beyondBoundsPageCount = pagerState.pageCount,
        ) { page ->
            when (val text = texts[page]) {
                null -> {
                    ScrollableOutlinedBase64Text(
                        text = "",
                        isError = true,
                        modifier = Modifier.fillMaxSize(),
                    )
                }
                else -> {
                    ScrollableOutlinedBase64Text(text = text, modifier = Modifier.fillMaxSize())
                }
            }
        }
    }
}

@Parcelize
@TypeParceler<PartialPayload?, OptionalPartialPayloadParceler>
data class GroupDetailScreen(val id: UInt, val payloads: List<PartialPayload?>) :
    Screen, Parcelable {
    @Composable
    override fun Content() {
        val navigator = LocalNavigator.currentOrThrow

        val isIncomplete = payloads.contains(null)

        val prefix =
            if (isIncomplete) {
                "Incomplete"
            } else {
                "Complete"
            }

        SetAppBar(
            title = { Text(text = "$prefix Payload Group") },
            navigationIcon = {
                IconButton(onClick = { navigator.pop() }) {
                    Icon(
                        imageVector = Icons.AutoMirrored.Default.ArrowBack,
                        contentDescription = "Navigate Back",
                    )
                }
            },
            actions = {},
        )

        GroupDetails(
            id = id,
            texts = payloads.map { it?.dataString },
            encryptionTag = payloads.encryptionTag,
            compressionTag = payloads.compressionTag,
            onMerge = {
                val payloads = payloads.map { it!! }.map { Payload.Partial(it) }

                val merged = PayloadMerger().merge(payloads)

                navigator.push(CompleteDetailScreen(payload = merged.complete[0]))
            },
        )
    }
}
