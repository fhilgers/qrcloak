package com.github.fhilgers.qrcloak

import android.content.Context
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.datastore.core.DataStore
import androidx.datastore.preferences.core.Preferences
import androidx.datastore.preferences.core.stringSetPreferencesKey
import androidx.datastore.preferences.preferencesDataStore
import cafe.adriel.voyager.navigator.Navigator
import com.github.fhilgers.qrcloak.ui.screens.RootScreen
import com.github.fhilgers.qrcloak.ui.theme.QrcloakTheme

val Context.dataStore: DataStore<Preferences> by preferencesDataStore(name = "qrCodes")
val NORMAL_KEY = stringSetPreferencesKey("normal")
val GROUP_KEY = stringSetPreferencesKey("group")
val COMPLETE_KEY = stringSetPreferencesKey("complete")

class MainActivity : ComponentActivity() {

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()
        setContent { QrcloakTheme { Navigator(screen = RootScreen) } }
    }
}
