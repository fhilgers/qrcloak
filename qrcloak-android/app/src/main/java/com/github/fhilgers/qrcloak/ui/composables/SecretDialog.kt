package com.github.fhilgers.qrcloak.ui.composables

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Visibility
import androidx.compose.material.icons.filled.VisibilityOff
import androidx.compose.material3.Card
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.text.input.VisualTransformation
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.Dialog
import uniffi.qrcloak_bindings.AgeIdentity

@Composable
fun SecretDialog(
    secret: String,
    headlineText: String,
    onSecretChange: (String) -> Unit,
    onDismiss: () -> Unit,
    onSuccess: () -> Unit,
    modifier: Modifier = Modifier,
    label: (@Composable () -> Unit)? = null,
    isError: Boolean = false,
    supportingText: (@Composable () -> Unit)? = null,
) {
    var hide by remember { mutableStateOf(true) }
    Dialog(
        onDismissRequest = onDismiss,
    ) {
        Card(modifier = modifier, shape = MaterialTheme.shapes.extraLarge) {
            Column(
                modifier = Modifier.padding(24.dp),
                verticalArrangement = Arrangement.Center,
                horizontalAlignment = Alignment.CenterHorizontally,
            ) {
                Text(
                    text = headlineText,
                    style =
                        MaterialTheme.typography.headlineSmall.copy(
                            color = MaterialTheme.colorScheme.onSurface
                        )
                )

                Spacer(modifier = Modifier.height(16.dp))

                OutlinedTextField(
                    value = secret,
                    onValueChange = onSecretChange,
                    label = label,
                    isError = isError,
                    supportingText = supportingText,
                    singleLine = true,
                    trailingIcon = {
                        IconButton(onClick = { hide = !hide }) {
                            when (hide) {
                                true ->
                                    Icon(
                                        imageVector = Icons.Default.Visibility,
                                        contentDescription = "Show"
                                    )
                                false ->
                                    Icon(
                                        imageVector = Icons.Default.VisibilityOff,
                                        contentDescription = "Hide"
                                    )
                            }
                        }
                    },
                    visualTransformation =
                        if (hide) {
                            PasswordVisualTransformation()
                        } else {
                            VisualTransformation.None
                        }
                )

                Spacer(modifier = Modifier.height(24.dp))

                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.spacedBy(8.dp, alignment = Alignment.End),
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    TextButton(onClick = onDismiss) { Text(text = "Cancel") }
                    TextButton(onClick = onSuccess, enabled = !isError && secret.isNotEmpty()) {
                        Text(text = "Decrypt")
                    }
                }
            }
        }
    }
}

sealed interface ParsedOrError<T> {
    data class Parsed<T>(val inner: T) : ParsedOrError<T>

    data class Error<T>(val inner: String) : ParsedOrError<T>
}

@Preview
@Composable
fun SecretDialogPreview() {

    var rawSecret by remember { mutableStateOf("") }

    var parsed by remember { mutableStateOf<ParsedOrError<AgeIdentity>?>(null) }

    SecretDialog(
        secret = rawSecret,
        headlineText = "Enter Secret Key",
        onSecretChange = { it ->
            rawSecret = it
            try {
                parsed = ParsedOrError.Parsed(AgeIdentity.tryFromString(it))
            } catch (e: Exception) {
                e.message?.also { message -> parsed = ParsedOrError.Error(message) }
            }
        },
        onDismiss = {},
        onSuccess = {},
        isError = parsed is ParsedOrError.Error,
        label = { Text("Age Private Key") },
        supportingText = {
            when (val p = parsed) {
                is ParsedOrError.Error -> {
                    Text(text = p.inner)
                }
                else -> {
                    Text(text = "*required")
                }
            }
        }
    )
}
