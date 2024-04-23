package com.github.fhilgers.qrcloak.ui.screens.saved.detail

import android.os.Parcelable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.ArrowBack
import androidx.compose.material.icons.filled.Compress
import androidx.compose.material.icons.filled.EnhancedEncryption
import androidx.compose.material.icons.filled.Visibility
import androidx.compose.material.icons.filled.VisibilityOff
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.FloatingActionButton
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TextField
import androidx.compose.runtime.Composable
import androidx.compose.runtime.MutableState
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalView
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.Dialog
import androidx.compose.ui.window.DialogWindowProvider
import cafe.adriel.voyager.core.model.StateScreenModel
import cafe.adriel.voyager.core.model.rememberScreenModel
import cafe.adriel.voyager.core.model.screenModelScope
import cafe.adriel.voyager.core.screen.Screen
import cafe.adriel.voyager.navigator.CurrentScreen
import cafe.adriel.voyager.navigator.LocalNavigator
import cafe.adriel.voyager.navigator.Navigator
import cafe.adriel.voyager.navigator.currentOrThrow
import com.github.fhilgers.qrcloak.ui.composables.ScrollableOutlinedBase64Text
import com.github.fhilgers.qrcloak.ui.composables.SecretDialogContent
import com.github.fhilgers.qrcloak.ui.composables.Tag
import com.github.fhilgers.qrcloak.ui.composables.TagData
import com.github.fhilgers.qrcloak.ui.composables.TagRow
import com.github.fhilgers.qrcloak.ui.screens.CurrentFab
import com.github.fhilgers.qrcloak.ui.screens.SetAppBar
import com.github.fhilgers.qrcloak.ui.screens.SetFab
import com.github.fhilgers.qrcloak.utils.CompletePayloadParceler
import com.github.fhilgers.qrcloak.utils.dataString
import com.github.fhilgers.qrcloak.utils.tag
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.parcelize.Parcelize
import kotlinx.parcelize.TypeParceler
import uniffi.qrcloak_bindings.AgeIdentity
import uniffi.qrcloak_bindings.Decompression
import uniffi.qrcloak_bindings.Decryption
import uniffi.qrcloak_bindings.Encryption
import uniffi.qrcloak_bindings.GzipCompression
import uniffi.qrcloak_bindings.Passphrase
import uniffi.qrcloak_bindings.PayloadExtractor
import uniffi.qrcloak_bindings.PayloadGenerator
import uniffi.qrcloak_core.CompletePayload
import uniffi.qrcloak_core.CompressionSpec
import uniffi.qrcloak_core.EncryptionSpec

@OptIn(ExperimentalLayoutApi::class)
@Composable
fun CompleteDetails(
    text: String,
    encryption: EncryptionSpec,
    compression: CompressionSpec,
    modifier: Modifier = Modifier
) {
    Column(modifier = modifier.fillMaxWidth().padding(16.dp)) {
        TagRow(modifier = Modifier.fillMaxWidth()) {
            Tag(TagData(text = encryption.tag, icon = Icons.Default.EnhancedEncryption))

            Tag(TagData(text = compression.tag, icon = Icons.Default.Compress))
        }

        Spacer(modifier = Modifier.height(16.dp))

        ScrollableOutlinedBase64Text(text = text, modifier = Modifier.fillMaxWidth())
    }
}

@Preview
@Composable
fun CompleteDetailsPreview() {

    val completePayload =
        PayloadGenerator()
            .withEncryption(Encryption.AgeKey(listOf(AgeIdentity.generate().toPublic())))
            .generate("hello")

    Surface {
        CompleteDetails(
            text = completePayload.dataString,
            encryption = completePayload.encryption,
            compression = completePayload.compression
        )
    }
}

@Preview
@Composable
fun CompletePayloadDetailScreenPreview() {

    val idStr = "AGE-SECRET-KEY-1AJ3YAE7ZZU4N9NHU33NG2QJHRNUXVKYWMS97H28CEG6ETNCQWGJS3AFW6P"
    val id = AgeIdentity.tryFromString(idStr)

    val payload = remember {
        PayloadGenerator()
            .withEncryption(Encryption.AgeKey(listOf(id.toPublic())))
            .generate("hello world")
    }

    Scaffold(floatingActionButton = { CurrentFab() }) {
        Box(modifier = Modifier.padding(it)) {
            Navigator(screen = CompleteDetailScreen(payload = payload)) {
                Column {
                    CurrentScreen()
                    TextField(value = idStr, onValueChange = {})
                }
            }
        }
    }
}

data class CompleteDetailScreenModel(private val payload: CompletePayload) :
    StateScreenModel<CompleteDetailScreenModel.State>(State.Encoded) {

    sealed interface DialogState {}

    sealed interface State {
        data object Encoded : State

        data class KeyEntering(
            private val isError: ((String) -> String?)? = null,
            val dialogHeadline: String,
        ) : State, DialogState {
            private val _secret: MutableState<String> = mutableStateOf("")

            val error: MutableState<String?> = mutableStateOf(null)

            var secret: String
                get() = _secret.value
                set(value) {
                    error.value = isError?.invoke(value)
                    _secret.value = value
                }
        }

        data object Decoding : State, DialogState

        data object Plain : State

        data object Hidden : State
    }

    private var cachedPlain: String? = null

    private fun dialogHeadline(): String {
        return when (payload.encryption) {
            EncryptionSpec.NO_ENCRYPTION -> throw IllegalStateException("not encrypted")
            EncryptionSpec.AGE_PASSPHRASE -> "Enter Age Passphrase"
            EncryptionSpec.AGE_KEY -> "Enter Age Secret Key"
        }
    }

    private fun getDecompression(): Decompression {
        return when (payload.compression) {
            CompressionSpec.NO_COMPRESSION -> Decompression.NoCompression
            CompressionSpec.GZIP -> Decompression.Gzip(GzipCompression())
        }
    }

    private fun startDecodingAgeKey() {
        val previous = state.value as State.KeyEntering
        val current = State.Decoding

        mutableState.value = current

        screenModelScope.launch(Dispatchers.Default) {
            try {
                val identity = AgeIdentity.tryFromString(previous.secret)
                val decompression = getDecompression()

                val plain =
                    PayloadExtractor()
                        .withDecryption(Decryption.AgeKey(listOf(identity)))
                        .withDecompression(decompression)
                        .extract(payload)

                // TODO utf8
                cachedPlain = String(plain)

                mutableState.value = State.Plain
            } catch (e: Exception) {
                previous.error.value = e.message
                mutableState.value = previous
            }
        }
    }

    fun currentText(): String {
        return when (state.value) {
            State.Plain -> cachedPlain!!
            else -> payload.dataString
        }
    }

    fun encryption(): EncryptionSpec = payload.encryption

    fun compression(): CompressionSpec = payload.compression

    private fun startDecodingAgePassphrase() {
        val previous = state.value as State.KeyEntering
        val current = State.Decoding

        mutableState.value = current

        screenModelScope.launch(Dispatchers.Default) {
            try {
                val decompression = getDecompression()

                val plain =
                    PayloadExtractor()
                        .withDecryption(
                            Decryption.AgePassphrase(passphrase = Passphrase(previous.secret))
                        )
                        .withDecompression(decompression)
                        .extract(payload)

                // TODO utf8
                cachedPlain = String(plain)

                mutableState.value = State.Plain
            } catch (e: Exception) {
                previous.error.value = e.message
                mutableState.value = previous
            }
        }
    }

    fun startKeyEntry() {
        when (payload.encryption) {
            EncryptionSpec.NO_ENCRYPTION -> {
                cachedPlain = String(payload.data)
                mutableState.value = State.Plain
            }
            EncryptionSpec.AGE_PASSPHRASE ->
                mutableState.value = State.KeyEntering(dialogHeadline = dialogHeadline())
            EncryptionSpec.AGE_KEY ->
                mutableState.value =
                    State.KeyEntering(
                        dialogHeadline = dialogHeadline(),
                        isError = {
                            try {
                                AgeIdentity.tryFromString(it)
                                null
                            } catch (e: Exception) {
                                e.message
                            }
                        }
                    )
        }
    }

    fun startDecoding() {
        when (payload.encryption) {
            EncryptionSpec.NO_ENCRYPTION -> throw IllegalStateException("not encrypted")
            EncryptionSpec.AGE_PASSPHRASE -> startDecodingAgePassphrase()
            EncryptionSpec.AGE_KEY -> startDecodingAgeKey()
        }
    }

    fun dismissDecoding() {
        mutableState.value = State.Encoded
    }

    fun hideDecoded() {
        mutableState.value = State.Hidden
    }

    fun showDecoded() {
        mutableState.value = State.Plain
    }
}

@Composable
fun CompletePayloadDetailFab(
    state: CompleteDetailScreenModel.State,
    onHide: () -> Unit,
    onStartKeyEntry: () -> Unit,
    onShow: () -> Unit,
) {

    when (state) {
        CompleteDetailScreenModel.State.Plain -> {
            SetFab {
                FloatingActionButton(onClick = onHide) {
                    Icon(imageVector = Icons.Default.VisibilityOff, contentDescription = "Hide")
                }
            }
        }
        CompleteDetailScreenModel.State.Encoded -> {
            SetFab {
                FloatingActionButton(onClick = onStartKeyEntry) {
                    Icon(imageVector = Icons.Default.Visibility, contentDescription = "Show")
                }
            }
        }
        CompleteDetailScreenModel.State.Hidden -> {
            SetFab {
                FloatingActionButton(onClick = onShow) {
                    Icon(imageVector = Icons.Default.Visibility, contentDescription = "Show")
                }
            }
        }
        else -> {}
    }
}

@Composable
fun EntryDialog(
    state: CompleteDetailScreenModel.DialogState,
    onDismissDecoding: () -> Unit,
    onStartDecoding: () -> Unit,
) {
    Dialog(onDismissRequest = onDismissDecoding) {
        when (state) {
            is CompleteDetailScreenModel.State.KeyEntering -> {
                SecretDialogContent(
                    secret = state.secret,
                    headlineText = state.dialogHeadline,
                    onSecretChange = { state.secret = it },
                    isError = state.error.value != null,
                    supportingText = state.error.value?.let { { Text(text = it) } },
                    onDismiss = onDismissDecoding,
                    onSuccess = onStartDecoding
                )
            }
            is CompleteDetailScreenModel.State.Decoding -> {
                (LocalView.current.parent as DialogWindowProvider).window.setDimAmount(0.3f)
                CircularProgressIndicator()
            }
        }
    }
}

@Composable
fun CompletePayloadDetail(
    text: String,
    encryption: EncryptionSpec,
    compression: CompressionSpec,
    state: CompleteDetailScreenModel.State,
    onHide: () -> Unit,
    onStartKeyEntry: () -> Unit,
    onShow: () -> Unit,
    onDismissDecoding: () -> Unit,
    onStartDecoding: () -> Unit,
    modifier: Modifier = Modifier
) {
    CompletePayloadDetailFab(
        state = state,
        onHide = onHide,
        onShow = onShow,
        onStartKeyEntry = onStartKeyEntry,
    )

    Box(modifier = modifier) {
        if (state is CompleteDetailScreenModel.DialogState) {
            EntryDialog(
                state = state,
                onDismissDecoding = onDismissDecoding,
                onStartDecoding = onStartDecoding
            )
        }
        CompleteDetails(
            text = text,
            encryption = encryption,
            compression = compression,
        )
    }
}

@Parcelize
@TypeParceler<CompletePayload, CompletePayloadParceler>
data class CompleteDetailScreen(val payload: CompletePayload) : Screen, Parcelable {
    @Composable
    override fun Content() {

        val screenModel =
            rememberScreenModel(tag = payload.toString()) { CompleteDetailScreenModel(payload) }

        val state by screenModel.state.collectAsState()

        val navigator = LocalNavigator.currentOrThrow

        SetAppBar(
            title = { Text(text = "Complete Payload") },
            navigationIcon = {
                IconButton(onClick = { navigator.pop() }) {
                    Icon(
                        imageVector = Icons.Default.ArrowBack,
                        contentDescription = "Navigate Back"
                    )
                }
            },
            actions = {}
        )

        CompletePayloadDetail(
            text = screenModel.currentText(),
            encryption = screenModel.encryption(),
            compression = screenModel.compression(),
            state = state,
            onHide = { screenModel.hideDecoded() },
            onStartKeyEntry = { screenModel.startKeyEntry() },
            onShow = { screenModel.showDecoded() },
            onDismissDecoding = { screenModel.dismissDecoding() },
            onStartDecoding = { screenModel.startDecoding() }
        )
    }
}
