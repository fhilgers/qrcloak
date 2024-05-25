package com.github.fhilgers.qrcloak.ui.screens

import androidx.camera.core.CameraSelector
import androidx.camera.core.ImageAnalysis
import androidx.camera.core.resolutionselector.ResolutionSelector
import androidx.camera.core.resolutionselector.ResolutionStrategy
import androidx.camera.mlkit.vision.MlKitAnalyzer
import androidx.camera.view.LifecycleCameraController
import androidx.camera.view.PreviewView
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.shape.ZeroCornerSize
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.NavigationBarDefaults
import androidx.compose.material3.SnackbarDuration
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.TopAppBarDefaults
import androidx.compose.material3.surfaceColorAtElevation
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.geometry.CornerRadius
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Rect
import androidx.compose.ui.geometry.RoundRect
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.ClipOp
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.Path
import androidx.compose.ui.graphics.drawscope.clipPath
import androidx.compose.ui.hapticfeedback.HapticFeedback
import androidx.compose.ui.hapticfeedback.HapticFeedbackType
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.platform.LocalHapticFeedback
import androidx.compose.ui.platform.LocalLifecycleOwner
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import androidx.compose.ui.viewinterop.AndroidView
import androidx.core.util.Consumer
import androidx.datastore.core.DataStore
import androidx.datastore.preferences.core.Preferences
import androidx.datastore.preferences.core.edit
import cafe.adriel.voyager.core.model.ScreenModel
import cafe.adriel.voyager.core.model.rememberScreenModel
import cafe.adriel.voyager.core.model.screenModelScope
import cafe.adriel.voyager.navigator.tab.Tab
import cafe.adriel.voyager.navigator.tab.TabOptions
import com.github.fhilgers.qrcloak.R
import com.github.fhilgers.qrcloak.utils.RAW_KEY
import com.github.fhilgers.qrcloak.utils.dataStore
import com.google.accompanist.permissions.ExperimentalPermissionsApi
import com.google.mlkit.vision.barcode.BarcodeScannerOptions
import com.google.mlkit.vision.barcode.BarcodeScanning
import com.google.mlkit.vision.barcode.common.Barcode
import java.util.concurrent.Executors
import kotlin.math.min
import kotlin.time.Duration.Companion.seconds
import kotlin.time.TimeSource
import kotlinx.coroutines.launch

val options = BarcodeScannerOptions.Builder().setBarcodeFormats(Barcode.FORMAT_QR_CODE).build()
val barcodeScanner = BarcodeScanning.getClient(options)

@Composable
fun CameraPreview(
    controller: LifecycleCameraController,
    modifier: Modifier = Modifier,
) {
    val lifecycleOwner = LocalLifecycleOwner.current

    AndroidView(
        factory = {
            PreviewView(it).apply {
                controller.bindToLifecycle(lifecycleOwner)
                this.controller = controller
            }
        },
        modifier = modifier,
    )
}

object CameraWithOverlayDefaults {
    @OptIn(ExperimentalMaterial3Api::class)
    val brush: Brush
        @Composable
        get() {

            val topAppBarColors = TopAppBarDefaults.topAppBarColors().containerColor
            val bottomNavBarColors =
                MaterialTheme.colorScheme.surfaceColorAtElevation(NavigationBarDefaults.Elevation)

            return Brush.verticalGradient(
                colors =
                    listOf(
                        topAppBarColors,
                        bottomNavBarColors,
                    ),
            )
        }
}

fun Rect.toAndroidRect(): android.graphics.Rect {
    return android.graphics.Rect(left.toInt(), top.toInt(), right.toInt(), bottom.toInt())
}

data class ScannerScreenModel(
    var region: Rect?,
    var dataStore: DataStore<Preferences>,
    var hapticFeedback: HapticFeedback,
    var snackbarHostState: SnackbarHostState,
) : Consumer<MlKitAnalyzer.Result>, ScreenModel {
    val timedMap: MutableMap<String, TimeSource.Monotonic.ValueTimeMark> = mutableMapOf()

    override fun accept(value: MlKitAnalyzer.Result) {
        val now = TimeSource.Monotonic.markNow()

        val region = region?.toAndroidRect() ?: return

        value
            .getValue(barcodeScanner)
            ?.filter { it.boundingBox != null }
            ?.filter { region.contains(it.boundingBox!!) }
            ?.filter { it.rawValue != null }
            ?.forEach {
                val rawData = it.rawValue!!

                val oldTime = timedMap[rawData]

                if (oldTime == null || now - oldTime > 1.seconds) {
                    screenModelScope.launch {
                        snackbarHostState.currentSnackbarData?.dismiss()

                        snackbarHostState.showSnackbar(
                            message = rawData,
                            duration = SnackbarDuration.Short,
                            withDismissAction = true,
                        )

                        dataStore.edit {
                            val old = it[RAW_KEY] ?: emptySet()

                            it[RAW_KEY] = old + rawData
                        }
                    }
                    hapticFeedback.performHapticFeedback(HapticFeedbackType.LongPress)
                }

                timedMap[rawData] = now
            }
    }
}

@Composable
fun CameraWithOverlay(
    onRegionChanged: (Rect) -> Unit,
    consumer: Consumer<MlKitAnalyzer.Result>,
    modifier: Modifier = Modifier,
) {
    var init by remember { mutableStateOf(false) }

    val context = LocalContext.current
    val executor = remember { Executors.newSingleThreadExecutor() }

    BoxWithConstraints(modifier = modifier) {
        val cameraController = remember {
            LifecycleCameraController(context).apply {
                cameraSelector = CameraSelector.DEFAULT_BACK_CAMERA
                imageAnalysisResolutionSelector =
                    ResolutionSelector.Builder()
                        .setResolutionStrategy(ResolutionStrategy.HIGHEST_AVAILABLE_STRATEGY)
                        .build()
                imageAnalysisBackpressureStrategy = ImageAnalysis.STRATEGY_KEEP_ONLY_LATEST
                previewResolutionSelector =
                    ResolutionSelector.Builder()
                        .setResolutionStrategy(ResolutionStrategy.HIGHEST_AVAILABLE_STRATEGY)
                        .build()

                val inner =
                    MlKitAnalyzer(
                        listOf(barcodeScanner),
                        ImageAnalysis.COORDINATE_SYSTEM_VIEW_REFERENCED,
                        executor,
                        consumer,
                    )

                setImageAnalysisAnalyzer(executor, inner)
            }
        }

        CameraPreview(
            controller = cameraController,
            modifier = Modifier.matchParentSize(),
        )

        if (init) {
            val density = LocalDensity.current

            val width = constraints.maxWidth / 1.5f
            val height = min(width, constraints.maxHeight.toFloat())
            val widthOffset = (constraints.maxWidth - width) / 2f
            val heightOffset = (constraints.maxHeight - height) / 2f

            val rectOffset = Offset(widthOffset, heightOffset)
            val rectSize = Size(width, height)
            val rect = Rect(rectOffset, rectSize)
            val roundRect =
                RoundRect(
                    rect = rect,
                    cornerRadius = with(density) { CornerRadius(16.dp.toPx(), 16.dp.toPx()) },
                )

            val path =
                Path().apply {
                    addRoundRect(
                        roundRect = roundRect,
                    )
                }

            LaunchedEffect(rect) { onRegionChanged(rect) }

            Canvas(modifier = Modifier.matchParentSize()) {
                clipPath(path, clipOp = ClipOp.Difference) {
                    drawRect(
                        color = Color.Black.copy(alpha = 0.3f),
                        size = size,
                    )
                }
            }
        }

        init = true
    }
}

object ScannerTab : Tab {
    private fun readResolve(): Any = ScannerTab

    override val options: TabOptions
        @Composable
        get() {
            val title = stringResource(id = R.string.scanner_tab_name)
            val icon = painterResource(id = R.drawable.qr_code_scanner)

            return remember { TabOptions(index = 0u, title = title, icon = icon) }
        }

    @OptIn(ExperimentalPermissionsApi::class)
    @Composable
    override fun Content() {
        val hapticFeedback = LocalHapticFeedback.current
        val dataStore = LocalContext.current.dataStore
        val snackbarHostState = LocalSnackbarHostState.current

        val model = rememberScreenModel {
            ScannerScreenModel(null, dataStore, hapticFeedback, snackbarHostState)
        }

        CameraWithOverlay(
            onRegionChanged = { model.region = it },
            consumer = model,
            modifier =
                Modifier.fillMaxSize()
                    .background(CameraWithOverlayDefaults.brush)
                    .clip(
                        MaterialTheme.shapes.medium.copy(
                            topStart = ZeroCornerSize,
                            topEnd = ZeroCornerSize,
                        ),
                    ),
        )
    }
}
