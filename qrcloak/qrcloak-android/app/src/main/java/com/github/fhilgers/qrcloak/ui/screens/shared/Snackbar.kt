package com.github.fhilgers.qrcloak.ui.screens.shared

import androidx.compose.material3.SnackbarHostState
import androidx.compose.runtime.compositionLocalOf
import androidx.compose.runtime.getValue
import androidx.compose.runtime.setValue

val LocalSnackbarHostState = compositionLocalOf { SnackbarHostState() }
