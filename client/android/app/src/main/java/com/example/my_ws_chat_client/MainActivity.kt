package com.example.my_ws_chat_client

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Button
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TextField
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.input.TextFieldValue
import androidx.compose.ui.unit.dp
import com.example.my_ws_chat_client.ui.theme.MywschatclientTheme

class MainActivity : ComponentActivity() {

    @OptIn(ExperimentalMaterial3Api::class)
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        setContent {
            MywschatclientTheme {
                Surface(
                    modifier = Modifier.fillMaxSize().padding(10.dp),
                    color = MaterialTheme.colorScheme.background,
                ) {
                    Column(
                        verticalArrangement = Arrangement.Center,
                        horizontalAlignment = Alignment.CenterHorizontally
                    ) {
                        var senderValue by remember { mutableStateOf(TextFieldValue("")) }
                        var addresseeValue by remember { mutableStateOf(TextFieldValue("")) }
                        TextField(
                            value = addresseeValue,
                            onValueChange = { addresseeValue = it },
                            modifier = Modifier.fillMaxWidth().padding(5.dp),
                            label = { Text(text = "Addressee")},
                        )
                        TextField(
                            value = senderValue,
                            onValueChange = { senderValue = it },
                            modifier = Modifier.fillMaxWidth().padding(5.dp),
                            label = { Text(text = "Sender")},
                        )
                        Button(
                            onClick = { startChatActivity(senderValue.text, addresseeValue.text)},
                            modifier = Modifier.fillMaxWidth().padding(5.dp),
                        ) {
                            Text(text = "Start Chat")
                        }
                    }
                }
            }
        }
    }

    private fun startChatActivity(sender: String, addressee: String) {
        val intent = ChatActivity.intent(this@MainActivity, sender, addressee)
        startActivity(intent)
    }
}