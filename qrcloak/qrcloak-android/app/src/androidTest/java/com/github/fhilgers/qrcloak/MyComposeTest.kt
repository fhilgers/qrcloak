package com.github.fhilgers.qrcloak

import androidx.compose.foundation.layout.Column
import androidx.compose.material3.Button
import androidx.compose.material3.Text
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.test.assertIsDisplayed
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import androidx.test.ext.junit.runners.AndroidJUnit4
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class MyComposeTest {

    @get:Rule val composeTestRule = createComposeRule()

    @Test
    fun myTest() {
        composeTestRule.setContent {
            var showWelcome by remember { mutableStateOf(false) }
            Column {
                Button(onClick = { showWelcome = !showWelcome }) { Text("Continue") }
                if (showWelcome) {
                    Text("Welcome")
                }
            }
        }

        composeTestRule.onNodeWithText("Continue").performClick()

        composeTestRule.onNodeWithText("Welcome").assertIsDisplayed()
    }
}
