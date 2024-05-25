package com.github.fhilgers.qrcloak.ui.screens.saved

import android.os.Parcelable
import android.widget.Switch
import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.expandVertically
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.scaleIn
import androidx.compose.animation.scaleOut
import androidx.compose.animation.shrinkVertically
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.consumeWindowInsets
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.imePadding
import androidx.compose.foundation.layout.navigationBars
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.filled.ArrowBack
import androidx.compose.material.icons.filled.Check
import androidx.compose.material.icons.filled.QrCode
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.ExposedDropdownMenuBox
import androidx.compose.material3.ExposedDropdownMenuDefaults
import androidx.compose.material3.ExtendedFloatingActionButton
import androidx.compose.material3.FloatingActionButton
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.NavigationBar
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Slider
import androidx.compose.material3.Switch
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.derivedStateOf
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableFloatStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalView
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.Dialog
import androidx.compose.ui.window.DialogWindowProvider
import cafe.adriel.voyager.core.screen.Screen
import cafe.adriel.voyager.navigator.LocalNavigator
import cafe.adriel.voyager.navigator.currentOrThrow
import com.github.fhilgers.qrcloak.ui.composables.SecretTextField
import com.github.fhilgers.qrcloak.ui.screens.saved.detail.CompleteDetailScreen
import com.github.fhilgers.qrcloak.ui.screens.saved.detail.GroupDetailScreen
import com.github.fhilgers.qrcloak.ui.screens.saved.detail.NormalDetailScreen
import com.github.fhilgers.qrcloak.ui.screens.shared.SetAppBar
import com.github.fhilgers.qrcloak.ui.screens.shared.SetFab
import com.github.fhilgers.qrcloak.utils.dataStore
import com.github.fhilgers.qrcloak.utils.id
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.async
import kotlinx.coroutines.coroutineScope
import kotlinx.coroutines.launch
import kotlinx.parcelize.Parcelize
import uniffi.qrcloak_bindings.AgeRecipient
import uniffi.qrcloak_bindings.Compression
import uniffi.qrcloak_bindings.Encryption
import uniffi.qrcloak_bindings.GzipCompression
import uniffi.qrcloak_bindings.Passphrase
import uniffi.qrcloak_bindings.PayloadGenerator
import uniffi.qrcloak_bindings.PayloadSplitter
import uniffi.qrcloak_core.Payload

interface SelectionName {
    fun displayName(): String
}

sealed interface EncryptionType : SelectionName {
    data object NoEncryption : EncryptionType {
        override fun displayName(): String = "No Encryption"
    }

    data object AgePassphrase : EncryptionType {
        override fun displayName(): String = "Age Passphrase"
    }

    data object AgeKey : EncryptionType {
        override fun displayName(): String = "Age Recipient"
    }
}

sealed interface CompressionType : SelectionName {
    data object NoCompression : CompressionType {
        override fun displayName(): String = "No Compression"
    }

    data object Gzip : CompressionType {
        override fun displayName(): String = "Gzip"
    }
}

@Composable
fun QRCodeTextField(
    text: String,
    onTextChange: (String) -> Unit,
    modifier: Modifier = Modifier,
) {
    OutlinedTextField(
        value = text,
        onValueChange = onTextChange,
        maxLines = 5,
        label = { Text(text = "Content") },
        modifier = modifier,
    )
}

@Composable
fun QRCodeSecretField(
    password: String,
    passwordError: String? = null,
    onPasswordChange: (String) -> Unit,
    modifier: Modifier = Modifier,
) {
    var hide by remember { mutableStateOf(true) }

    SecretTextField(
        hide = hide,
        secret = password,
        onSecretChange = onPasswordChange,
        onHide = { hide = !hide },
        modifier = modifier,
        label = { Text(text = "Secret") },
        isError = passwordError != null,
        supportingText =
            if (passwordError != null) {
                { Text(text = passwordError) }
            } else {
                null
            },
    )
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun <T : SelectionName> DropDown(
    options: List<T>,
    selectedOption: T,
    label: @Composable () -> Unit,
    onSelectionChange: (T) -> Unit,
    modifier: Modifier = Modifier,
) {
    var expanded by remember { mutableStateOf(false) }

    ExposedDropdownMenuBox(
        expanded = expanded,
        onExpandedChange = { expanded = it },
        modifier = modifier,
    ) {
        OutlinedTextField(
            // The `menuAnchor` modifier must be passed to the text field for correctness.
            modifier = Modifier.menuAnchor().fillMaxWidth(),
            readOnly = true,
            value = selectedOption.displayName(),
            onValueChange = {},
            label = label,
            trailingIcon = { ExposedDropdownMenuDefaults.TrailingIcon(expanded = expanded) },
            colors = ExposedDropdownMenuDefaults.textFieldColors(),
        )

        DropdownMenu(
            expanded = expanded,
            onDismissRequest = { expanded = false },
            modifier = Modifier.exposedDropdownSize(true),
        ) {
            options.forEach { selectionOption ->
                DropdownMenuItem(
                    text = { Text(selectionOption.displayName()) },
                    onClick = {
                        onSelectionChange(selectionOption)
                        expanded = false
                    },
                    contentPadding = ExposedDropdownMenuDefaults.ItemContentPadding,
                )
            }
        }
    }
}

class CreateQRCodeData {
    var selectedEncryption by mutableStateOf<EncryptionType>(EncryptionType.NoEncryption)
    var selectedCompression by mutableStateOf<CompressionType>(CompressionType.NoCompression)
    var text by mutableStateOf("")
    var password by mutableStateOf("")
    var passwordError by mutableStateOf<String?>(null)

    var sliderPosition by mutableFloatStateOf(0f)

    val createEnabled by derivedStateOf {
        when (selectedEncryption) {
            EncryptionType.AgeKey -> text != "" && password != "" && passwordError == null
            EncryptionType.AgePassphrase -> text != "" && password != ""
            EncryptionType.NoEncryption -> text != ""
        } && !decoding
    }

    var sliderEnabled by mutableStateOf(false)

    fun toggleSlider() {
        sliderEnabled = !sliderEnabled
    }

    fun changeSlider(value: Float) {
        sliderPosition = value
    }

    fun changeEncryption(selection: EncryptionType) {
        password = ""
        passwordError = null
        selectedEncryption = selection
    }

    fun changeCompression(selection: CompressionType) {
        selectedCompression = selection
    }

    fun changePassword(password: String) {
        this.password = password

        if (selectedEncryption == EncryptionType.AgeKey) {
            try {
                AgeRecipient.tryFromString(password)
            } catch (e: Exception) {
                passwordError = e.message
            }
        }
    }

    var decoding by mutableStateOf(false)

    suspend fun createQRCode(): QrCode = coroutineScope {
        decoding = true

        val res =
            async(Dispatchers.Default) {
                    val encryption =
                        when (selectedEncryption) {
                            EncryptionType.AgeKey ->
                                Encryption.AgeKey(listOf(AgeRecipient.tryFromString(password)))
                            EncryptionType.AgePassphrase ->
                                Encryption.AgePassphrase(Passphrase(password))
                            EncryptionType.NoEncryption -> Encryption.NoEncryption
                        }

                    val compression =
                        when (selectedCompression) {
                            CompressionType.Gzip -> Compression.Gzip(GzipCompression())
                            CompressionType.NoCompression -> Compression.NoCompression
                        }

                    val payload =
                        PayloadGenerator()
                            .withCompression(compression)
                            .withEncryption(encryption)
                            .generate(text)

                    if (sliderEnabled) {
                        val split =
                            PayloadSplitter()
                                .withSplits(sliderPosition.toUInt())
                                .split(payload)
                                .map { (it as Payload.Partial).v1 }

                        QrCode.Group(id = split.id, payloads = split)
                    } else {
                        QrCode.Complete(payload = payload)
                    }
                }
                .await()

        decoding = false

        res
    }
}

@OptIn(ExperimentalLayoutApi::class)
@Preview
@Composable
fun CreateQRCodeScreenPreview() {
    MaterialTheme {
        val createQRCodeData = remember { CreateQRCodeData() }

        Scaffold(
            floatingActionButton = {
                AnimatedVisibility(
                    visible = createQRCodeData.createEnabled,
                    enter = scaleIn(),
                    exit = scaleOut(),
                ) {
                    CreateQRCodeDefaults.FloatingActionButton(
                        onClick = { /*TODO*/},
                        modifier =
                            Modifier.consumeWindowInsets(WindowInsets.navigationBars)
                                .consumeWindowInsets(PaddingValues(vertical = 80.dp))
                                .imePadding(),
                    )
                }
            },
            bottomBar = { NavigationBar {} },
        ) { paddingValues ->
            CreateQRCode(
                data = createQRCodeData,
                modifier =
                    Modifier.padding(paddingValues)
                        .consumeWindowInsets(paddingValues)
                        .imePadding()
                        .padding(12.dp),
            )
        }
    }
}

object CreateQRCodeDefaults {
    @Composable
    fun FloatingActionButton(
        onClick: () -> Unit,
        modifier: Modifier = Modifier,
    ) {
        ExtendedFloatingActionButton(
            text = { Text(text = "Create") },
            icon = {
                Icon(imageVector = Icons.Default.QrCode, contentDescription = "Create QRCode")
            },
            onClick = onClick,
            modifier = modifier,
        )
    }
}

@Composable
fun CreateQRCode(
    data: CreateQRCodeData,
    modifier: Modifier = Modifier,
) {
    LazyColumn(
        verticalArrangement = Arrangement.spacedBy(16.dp),
        contentPadding = PaddingValues(16.dp),
        modifier = modifier,
    ) {
        item {
            QRCodeTextField(
                text = data.text,
                onTextChange = { data.text = it },
                modifier = Modifier.fillMaxWidth(),
            )
        }

        item {
            DropDown(
                options = listOf(CompressionType.NoCompression, CompressionType.Gzip),
                selectedOption = data.selectedCompression,
                onSelectionChange = { data.changeCompression(it) },
                label = { Text(text = "Compression") },
            )
        }

        item {
            DropDown(
                options =
                    listOf(
                        EncryptionType.NoEncryption,
                        EncryptionType.AgeKey,
                        EncryptionType.AgePassphrase,
                    ),
                selectedOption = data.selectedEncryption,
                onSelectionChange = { data.changeEncryption(it) },
                label = { Text(text = "Encryption") },
            )
        }

        item {
            Row(
                modifier = Modifier.fillMaxWidth(),
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.SpaceBetween,
            ) {
                Column {
                    Text(text = "Split", style = MaterialTheme.typography.bodyLarge)
                    Text(
                        text = "Split the QR code into multiple parts",
                        style = MaterialTheme.typography.bodyMedium,
                    )
                }

                Switch(checked = data.sliderEnabled, onCheckedChange = { data.toggleSlider() })
            }
        }

        item {
            Slider(
                enabled = data.sliderEnabled,
                value = data.sliderPosition,
                onValueChange = { data.changeSlider(it) },
                steps = 7,
                valueRange = 1f..8f,
            )
        }

        item {
            AnimatedVisibility(
                visible = data.selectedEncryption !is EncryptionType.NoEncryption,
                enter = fadeIn() + expandVertically(),
                exit = fadeOut() + shrinkVertically(),
            ) {
                QRCodeSecretField(
                    password = data.password,
                    onPasswordChange = { data.changePassword(it) },
                    passwordError = data.passwordError,
                    modifier = Modifier.fillMaxWidth(),
                )
            }
        }
    }
}

@Parcelize
data object CreateScreen : Screen, Parcelable {
    @Composable
    override fun Content() {
        val navigator = LocalNavigator.currentOrThrow

        SetAppBar(
            title = { Text(text = "Compose") },
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

        val createQRCodeData = remember { CreateQRCodeData() }

        val dataStore = LocalContext.current.dataStore

        val scope = rememberCoroutineScope()

        if (createQRCodeData.createEnabled) {
            SetFab {
                FloatingActionButton(
                    onClick = {
                        scope.launch {
                            val qrCode = createQRCodeData.createQRCode()

                            qrCode.save(dataStore)

                            navigator.pop()

                            when (qrCode) {
                                is QrCode.Group -> {
                                    navigator.push(
                                        GroupDetailScreen(
                                            id = qrCode.id,
                                            payloads = qrCode.payloads,
                                        ),
                                    )
                                }
                                is QrCode.Complete -> {
                                    navigator.push(CompleteDetailScreen(payload = qrCode.payload))
                                }
                                is QrCode.Normal -> {
                                    navigator.push(NormalDetailScreen(qrCode.data))
                                }
                            }
                        }
                    },
                ) {
                    Icon(imageVector = Icons.Default.Check, contentDescription = "Create QRCode")
                }
            }
        }

        if (createQRCodeData.decoding) {
            Dialog(onDismissRequest = {}) {
                (LocalView.current.parent as DialogWindowProvider).window.setDimAmount(0.3f)
                CircularProgressIndicator()
            }
        }

        CreateQRCode(
            data = createQRCodeData,
        )
    }
}
