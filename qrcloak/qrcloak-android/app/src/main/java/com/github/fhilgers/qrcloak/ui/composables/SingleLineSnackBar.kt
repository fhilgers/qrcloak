package com.github.fhilgers.qrcloak.ui.composables

import androidx.compose.foundation.layout.padding
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Close
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.IconButtonDefaults
import androidx.compose.material3.Snackbar
import androidx.compose.material3.SnackbarData
import androidx.compose.material3.SnackbarDefaults
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp

@Composable
fun SingleLineSnackbar(snackbarData: SnackbarData) {
    val actionLabel = snackbarData.visuals.actionLabel
    val actionComposable: (@Composable () -> Unit)? =
        if (actionLabel != null) {
            @Composable {
                TextButton(
                    colors =
                        ButtonDefaults.textButtonColors(
                            contentColor = SnackbarDefaults.actionColor
                        ),
                    onClick = { snackbarData.performAction() },
                    content = { Text(actionLabel) },
                )
            }
        } else {
            null
        }
    val dismissComposable: (@Composable () -> Unit)? =
        if (snackbarData.visuals.withDismissAction) {
            @Composable {
                IconButton(
                    colors = IconButtonDefaults.iconButtonColors(),
                    onClick = { snackbarData.dismiss() },
                    content = {
                        Icon(imageVector = Icons.Filled.Close, contentDescription = "Dismiss")
                    },
                )
            }
        } else {
            null
        }

    Snackbar(
        action = actionComposable,
        dismissAction = dismissComposable,
        modifier = Modifier.padding(12.dp),
    ) {
        Text(snackbarData.visuals.message, maxLines = 1, overflow = TextOverflow.Ellipsis)
    }
}
