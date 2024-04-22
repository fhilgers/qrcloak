package com.github.fhilgers.qrcloak.ui.composables

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.ExperimentalFoundationApi
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.pager.HorizontalPager
import androidx.compose.foundation.pager.rememberPagerState
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Card
import androidx.compose.material3.Icon
import androidx.compose.material3.LocalTextStyle
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextFieldDefaults
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import com.github.fhilgers.qrcloak.ui.theme.QrcloakTheme
import io.github.alexzhirkevich.qrose.rememberQrCodePainter
import kotlin.io.encoding.Base64
import kotlin.io.encoding.ExperimentalEncodingApi
import kotlin.random.Random

@OptIn(ExperimentalEncodingApi::class)
private fun makeRandomData(amount: Int = 5, length: Int = 100): List<String> =
    (0..amount).map { Random.nextBytes(length) }.map(Base64::encode).toList()

@Preview
@Composable
fun PreviewQrCodePager() {
    QrcloakTheme {
        val data = remember { makeRandomData() }

        QrCodePager(qrCodes = data, modifier = Modifier.padding(16.dp))
    }
}

@OptIn(ExperimentalFoundationApi::class)
@Composable
fun QrCodePager(qrCodes: List<String>, modifier: Modifier = Modifier) {
    val pagerState = rememberPagerState { qrCodes.size }

    Card(modifier = modifier) {
        HorizontalPager(
            state = pagerState,
            contentPadding = PaddingValues(48.dp),
            pageSpacing = 24.dp
        ) { page ->
            Box(modifier = Modifier.fillMaxWidth(), contentAlignment = Alignment.Center) {
                Icon(
                    painter = rememberQrCodePainter(data = qrCodes[page]),
                    contentDescription = qrCodes[page],
                    modifier = Modifier.fillMaxWidth().aspectRatio(1f)
                )
            }
        }
    }
}

@Preview
@Composable
fun PreviewQrCodeDataPager() {
    QrcloakTheme {
        val data = remember { makeRandomData(3, 100) + makeRandomData(4, 4000) }

        Column {
            QrCodePager(qrCodes = data, modifier = Modifier.padding(16.dp).weight(1f))
            QrCodeDataPager(qrCodes = data, modifier = Modifier.padding(16.dp).weight(1f))
        }
    }
}

@OptIn(ExperimentalFoundationApi::class)
@Composable
fun QrCodeDataPager(qrCodes: List<String>, modifier: Modifier = Modifier) {
    val pagerState = rememberPagerState { qrCodes.size }

    HorizontalPager(
        state = pagerState,
        contentPadding = PaddingValues(48.dp),
        pageSpacing = 24.dp,
        beyondBoundsPageCount = pagerState.pageCount,
        modifier = modifier
    ) { page ->
        ScrollableOutlinedBase64Text(text = qrCodes[page], modifier = Modifier.fillMaxSize())
    }
}

@Composable
fun ScrollableOutlinedBase64Text(
    text: String,
    modifier: Modifier = Modifier,
    maxLines: Int = Int.MAX_VALUE,
) {
    Box(
        modifier =
            modifier
                .border(
                    border =
                        BorderStroke(
                            width = OutlinedTextFieldDefaults.FocusedBorderThickness,
                            color = MaterialTheme.colorScheme.outline,
                        ),
                    shape = OutlinedTextFieldDefaults.shape,
                )
                .padding(OutlinedTextFieldDefaults.contentPadding()),
    ) {
        Box(
            modifier = Modifier.verticalScroll(state = rememberScrollState()),
        ) {
            Text(
                text = text.map { it }.joinToString("\u200D"),
                style =
                    LocalTextStyle.current.copy(
                        fontFamily = FontFamily.Monospace,
                        textAlign = TextAlign.Justify
                    ),
                maxLines = maxLines,
            )
        }
    }
}
