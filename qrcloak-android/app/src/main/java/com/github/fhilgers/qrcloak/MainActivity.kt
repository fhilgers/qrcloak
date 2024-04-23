package com.github.fhilgers.qrcloak

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import cafe.adriel.voyager.navigator.Navigator
import com.github.fhilgers.qrcloak.ui.screens.RootScreen
import com.github.fhilgers.qrcloak.ui.theme.QrcloakTheme

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()
        setContent { QrcloakTheme { Navigator(screen = RootScreen) } }
    }
}